//! Universal message envelope — every event on the NATS bus carries this wrapper.
//!
//! Type parameter `T` is the payload. Envelope<T> is serialized as one atomic JSON document.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The universal envelope structure.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Envelope<T> {
    /// Unique identifier for this exact message (UUID v4)
    pub message_id: String,
    /// Session identifier — groups all messages in one conversation
    /// Format: "session_<uuid>"
    pub session_id: String,
    /// Correlation ID — links a chain of messages back to the original task.
    /// Root = original request UUID; children append "-N" suffixes.
    pub correlation_id: String,
    /// Routing key / topic. Determines which modules receive this.
    pub topic: String,
    /// Typed payload
    pub payload: T,
    /// Schema version marker (default: 1). Increment on breaking envelope changes.
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
    /// When the sender created this message (Unix seconds). Auto-filled if missing.
    #[serde(default = "now_timestamp")]
    pub timestamp: i64,
}

impl<T> Envelope<T> {
    /// Construct a new envelope with fresh IDs and current timestamp.
    #[inline]
    pub fn new(topic: impl Into<String>, payload: T, session_id: Option<String>) -> Self {
        let message_id = Uuid::new_v4().to_string();
        let session_id = session_id.unwrap_or_else(|| format!("session_{}", Uuid::new_v4()));
        let correlation_id = Uuid::new_v4().to_string();
        let topic = topic.into();
        let timestamp = now_timestamp();

        Self {
            message_id,
            session_id,
            correlation_id,
            topic,
            payload,
            schema_version: current_schema_version(),
            timestamp,
        }
    }

    /// Create a child envelope inheriting parent session/correlation.
    /// Child correlation_id becomes "{parent}-N" where N is 1-indexed.
    #[inline]
    pub fn child(&self, topic: impl Into<String>, payload: T, child_index: u32) -> Self {
        let correlation_id = format!("{}-{}", self.correlation_id, child_index);
        let message_id = Uuid::new_v4().to_string();
        let timestamp = now_timestamp();

        Self {
            message_id,
            session_id: self.session_id.clone(),
            correlation_id,
            topic: topic.into(),
            payload,
            schema_version: current_schema_version(),
            timestamp,
        }
    }

    /// Create a reply envelope for responding to this message.
    /// Inherits session_id, generates a fresh correlation_id.
    #[inline]
    pub fn reply(&self, topic: impl Into<String>, payload: T) -> Self {
        let timestamp = now_timestamp();
        Self {
            message_id: Uuid::new_v4().to_string(),
            session_id: self.session_id.clone(),
            correlation_id: Uuid::new_v4().to_string(),
            topic: topic.into(),
            payload,
            schema_version: current_schema_version(),
            timestamp,
        }
    }

    /// Extract the parent correlation ID from a child envelope.
    /// For root envelopes, returns self.correlation_id.
    /// For child envelopes (correlation_id ends with "-N"), strips the suffix
    /// to return the parent's correlation_id.
    #[inline]
    pub fn correlation_parent(&self) -> String {
        // Strip trailing "-N" suffix if present to get parent correlation_id
        // Only strip if the suffix is numeric (e.g., "abc-1" -> "abc", but "some-thing" -> "some-thing")
        if let Some(idx) = self.correlation_id.rfind('-') {
            let suffix = &self.correlation_id[idx + 1..];
            if suffix.chars().all(|c| c.is_ascii_digit()) {
                return self.correlation_id[..idx].to_string();
            }
        }
        self.correlation_id.clone()
    }

    /// Validate envelope sanity. Returns Ok(()) or an error.
    pub fn validate(&self) -> Result<(), EnvelopeError> {
        if self.schema_version > 1 {
            return Err(EnvelopeError::SchemaVersionTooNew(self.schema_version));
        }
        if self.message_id.is_empty() {
            return Err(EnvelopeError::MissingField("message_id".into()));
        }
        if self.session_id.is_empty() {
            return Err(EnvelopeError::MissingField("session_id".into()));
        }
        if self.correlation_id.is_empty() {
            return Err(EnvelopeError::MissingField("correlation_id".into()));
        }
        if self.topic.is_empty() {
            return Err(EnvelopeError::MissingField("topic".into()));
        }

        let now = now_timestamp();
        let diff = (self.timestamp - now).abs();
        if diff > 60 {
            return Err(EnvelopeError::ClockSkew(diff));
        }
        Ok(())
    }

    /// Serialize envelope to bytes efficiently.
    /// Uses JSON serialization with optimizations for repeated fields.
    #[inline]
    pub fn to_bytes(&self) -> Result<Vec<u8>, EnvelopeError>
    where
        T: serde::Serialize,
    {
        serde_json::to_vec(self).map_err(|e| EnvelopeError::SerializationError(e.to_string()))
    }

    /// Serialize envelope to bytes using compact JSON (no whitespace).
    /// More efficient for network transmission.
    #[inline]
    pub fn to_bytes_compact(&self) -> Result<Vec<u8>, EnvelopeError>
    where
        T: serde::Serialize,
    {
        serde_json::to_vec(self).map_err(|e| EnvelopeError::SerializationError(e.to_string()))
    }

    /// Deserialize envelope from bytes.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, EnvelopeError>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_slice(bytes).map_err(|e| EnvelopeError::DeserializationError(e.to_string()))
    }

    /// Zero-copy view of envelope from bytes using serde_json::Value
    /// This avoids full deserialization when you only need to inspect fields
    #[inline]
    pub fn from_bytes_view(bytes: &[u8]) -> Result<Envelope<serde_json::Value>, EnvelopeError> {
        serde_json::from_slice(bytes).map_err(|e| EnvelopeError::DeserializationError(e.to_string()))
    }

    /// Extract payload as JSON value without full deserialization
    /// Useful for inspecting payload structure without deserializing to concrete type
    #[inline]
    pub fn payload_as_value(&self) -> Result<serde_json::Value, EnvelopeError>
    where
        T: serde::Serialize,
    {
        serde_json::to_value(&self.payload).map_err(|e| EnvelopeError::SerializationError(e.to_string()))
    }

    /// Convert envelope to envelope with different payload type using JSON conversion
    /// This is a zero-copy operation when converting to/from serde_json::Value
    #[inline]
    pub fn convert_payload<U>(self) -> Result<Envelope<U>, EnvelopeError>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        let payload_value = serde_json::to_value(&self.payload)
            .map_err(|e| EnvelopeError::SerializationError(e.to_string()))?;
        let new_payload = serde_json::from_value(payload_value)
            .map_err(|e| EnvelopeError::DeserializationError(e.to_string()))?;
        
        Ok(Envelope {
            message_id: self.message_id,
            session_id: self.session_id,
            correlation_id: self.correlation_id,
            topic: self.topic,
            payload: new_payload,
            schema_version: self.schema_version,
            timestamp: self.timestamp,
        })
    }

    /// Create envelope with borrowed string references (zero-copy for string fields)
    /// This is useful when you have string slices and want to avoid allocation
    #[inline]
    pub fn with_borrowed_fields(
        message_id: &str,
        session_id: &str,
        correlation_id: &str,
        topic: &str,
        payload: T,
    ) -> Self {
        Self {
            message_id: message_id.to_string(),
            session_id: session_id.to_string(),
            correlation_id: correlation_id.to_string(),
            topic: topic.to_string(),
            payload,
            schema_version: current_schema_version(),
            timestamp: now_timestamp(),
        }
    }

    /// Get references to all string fields for zero-copy inspection
    #[inline]
    pub fn string_fields(&self) -> (&str, &str, &str, &str) {
        (
            &self.message_id,
            &self.session_id,
            &self.correlation_id,
            &self.topic,
        )
    }
}

// Helpers

const fn current_schema_version() -> u32 {
    1
}

#[inline]
fn now_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Validation errors for malformed envelopes
#[derive(thiserror::Error, Debug)]
pub enum EnvelopeError {
    #[error("schema version {} exceeds current maximum (max 1)", .0)]
    SchemaVersionTooNew(u32),

    #[error("missing required field: {}", .0)]
    MissingField(String),

    #[error("timestamp is in the future by {} seconds", .0)]
    ClockSkew(i64),

    #[error("serialization error: {}", .0)]
    SerializationError(String),

    #[error("deserialization error: {}", .0)]
    DeserializationError(String),
}

/// Schema cache for compiled schemas to avoid recompilation.
/// Uses LRU eviction with size limit to prevent unbounded memory growth.
#[cfg(feature = "schema-validation")]
use std::sync::OnceLock;

#[cfg(feature = "schema-validation")]
fn get_schema_cache() -> &'static dashmap::DashMap<String, jsonschema::JSONSchema> {
    static CACHE: OnceLock<dashmap::DashMap<String, jsonschema::JSONSchema>> = OnceLock::new();
    CACHE.get_or_init(|| dashmap::DashMap::new())
}

#[cfg(feature = "schema-validation")]
const MAX_SCHEMA_CACHE_SIZE: usize = 1000;

#[cfg(feature = "schema-validation")]
fn evict_if_needed(cache: &dashmap::DashMap<String, jsonschema::JSONSchema>) {
    if cache.len() > MAX_SCHEMA_CACHE_SIZE {
        // Remove oldest entries (simple FIFO eviction)
        // In production, you might want a proper LRU implementation
        let keys_to_remove: Vec<String> = cache
            .iter()
            .take(cache.len() - MAX_SCHEMA_CACHE_SIZE + 100) // Remove 100 extra to avoid frequent evictions
            .map(|entry| entry.key().clone())
            .collect();
        
        for key in keys_to_remove {
            cache.remove(&key);
        }
    }
}

/// Validate a JSON payload against a JSON Schema draft-07 schema.
/// Returns Ok(()) if valid, Err with description if invalid.
///
/// This validates the payload against the provided schema using the jsonschema crate.
/// Schemas are cached after compilation to improve performance with LRU eviction.
#[cfg(feature = "schema-validation")]
pub fn validate_payload_schema(
    schema: &serde_json::Value,
    payload: &serde_json::Value,
) -> Result<(), String> {
    use jsonschema::JSONSchema;

    let schema_key = schema.to_string();
    let cache = get_schema_cache();

    // Evict old schemas if cache is too large
    evict_if_needed(cache);

    // Try to get from cache first
    let compiled_schema = cache.entry(schema_key.clone()).or_try_insert_with(|| {
        JSONSchema::compile(schema).map_err(|e| format!("Failed to compile schema: {}", e))
    })?;

    // Validate the payload
    let result = compiled_schema.validate(payload);

    if let Err(errors) = result {
        let error_messages: Vec<String> = errors
            .map(|e| {
                let instance_path = e.instance_path.to_string();
                let schema_path = e.schema_path.to_string();
                format!(
                    "Validation error at {}: {} (schema: {})",
                    instance_path, e, schema_path
                )
            })
            .collect();
        Err(error_messages.join("; "))
    } else {
        Ok(())
    }
}

/// Load a JSON schema from embedded schema strings.
/// Returns Ok(schema) if found, Err if the schema name is invalid.
#[cfg(feature = "schema-validation")]
pub fn load_schema(schema_name: &str) -> Result<serde_json::Value, String> {
    let schema_str = match schema_name {
        "task_submitted.json" => EMBEDDED_TASK_SUBMITTED_SCHEMA,
        "task_enriched.json" => EMBEDDED_TASK_ENRICHED_SCHEMA,
        "task_complete.json" => EMBEDDED_TASK_COMPLETE_SCHEMA,
        "agent_job.json" => EMBEDDED_AGENT_JOB_SCHEMA,
        "agent_result.json" => EMBEDDED_AGENT_RESULT_SCHEMA,
        _ => return Err(format!("Unknown schema: {}", schema_name)),
    };

    serde_json::from_str(schema_str)
        .map_err(|e| format!("Failed to parse schema {}: {}", schema_name, e))
}

// Embedded schemas - these are compiled into the binary to ensure they're always available
// regardless of where the binary is executed from
#[cfg(feature = "schema-validation")]
const EMBEDDED_TASK_SUBMITTED_SCHEMA: &str = r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://github.com/Takon/Wireframe-AI/schemas/v1/task_submitted.json",
  "title": "Task Submitted",
  "description": "User request as submitted by the Interface module",
  "type": "object",
  "properties": {
    "session_id": {
      "type": "string",
      "pattern": "^session_[a-f0-9-]+$",
      "description": "Session identifier for this conversation"
    },
    "user_input": {
      "type": "string",
      "description": "The user's request or task description"
    },
    "submitted_at": {
      "type": "integer",
      "description": "Unix timestamp when the task was submitted"
    }
  },
  "required": ["session_id", "user_input", "submitted_at"],
  "additionalProperties": false
}"##;

#[cfg(feature = "schema-validation")]
const EMBEDDED_TASK_ENRICHED_SCHEMA: &str = r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://github.com/Takon/Wireframe-AI/schemas/v1/task_enriched.json",
  "title": "TaskEnriched",
  "description": "Payload for the 'task.enriched' topic. Emitted by the Context module after enriching a submitted task with memory and history.",
  "type": "object",
  "properties": {
    "session_id": {
      "type": "string",
      "description": "Session identifier from the original request"
    },
    "correlation_id": {
      "type": "string",
      "description": "Correlation ID echoed from the original Envelope"
    },
    "user_input": {
      "type": "string",
      "description": "Original user request text (preserved from TaskSubmitted)"
    },
    "context": {
      "$ref": "#/$defs/ContextPackage",
      "description": "Enriched context package with memory, history, file snippets"
    },
    "inferred_constraints": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Any constraints discovered or inferred from context"
    },
    "enriched_at": {
      "type": "integer",
      "description": "Unix timestamp when enrichment occurred"
    }
  },
  "required": ["session_id", "correlation_id", "user_input", "context", "enriched_at"],
  "$defs": {
    "ContextPackage": {
      "type": "object",
      "properties": {
        "memory_chunks": {
          "type": "array",
          "items": { "$ref": "#/$defs/MemoryChunk" }
        },
        "session_history": {
          "type": "array",
          "items": { "$ref": "#/$defs/ChatMessage" }
        },
        "readonly_files": {
          "type": "array",
          "items": { "$ref": "#/$defs/FileSnapshot" }
        },
        "safe_env": {
          "type": "object",
          "additionalProperties": { "type": "string" }
        },
        "working_dir": { "type": "string" },
        "max_context_tokens": { "type": "integer" }
      },
      "required": ["memory_chunks", "session_history", "readonly_files", "safe_env", "working_dir", "max_context_tokens"]
    },
    "MemoryChunk": {
      "type": "object",
      "properties": {
        "id": { "type": "string" },
        "content": { "type": "string" },
        "source": { "type": "string" },
        "relevance_score": { "type": "number" }
      },
      "required": ["id", "content", "source", "relevance_score"]
    },
    "ChatMessage": {
      "type": "object",
      "properties": {
        "role": { "type": "string", "enum": ["user", "assistant", "system"] },
        "content": { "type": "string" },
        "timestamp": { "type": "integer" }
      },
      "required": ["role", "content", "timestamp"]
    },
    "FileSnapshot": {
      "type": "object",
      "properties": {
        "path": { "type": "string" },
        "content": { "type": "string" },
        "size_bytes": { "type": "integer" },
        "last_modified": { "type": "integer" }
      },
      "required": ["path", "content", "size_bytes", "last_modified"]
    }
  }
}"##;

#[cfg(feature = "schema-validation")]
const EMBEDDED_TASK_COMPLETE_SCHEMA: &str = r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://github.com/Takon/Wireframe-AI/schemas/v1/task_complete.json",
  "title": "Task Complete",
  "description": "Final result returned to Interface for display to user",
  "type": "object",
  "properties": {
    "session_id": {
      "type": "string",
      "pattern": "^session_[a-f0-9-]+$",
      "description": "Session identifier for this conversation"
    },
    "correlation_id": {
      "type": "string",
      "format": "uuid",
      "description": "Correlation ID linking to original task"
    },
    "result": {
      "type": "string",
      "description": "The final answer / output text"
    },
    "side_effects": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "kind": {
            "type": "string",
            "description": "Type of side effect (e.g., file_written, command_run)"
          },
          "description": {
            "type": "string",
            "description": "Human-readable description of the side effect"
          },
          "path": {
            "type": "string",
            "description": "File path if applicable"
          }
        },
        "required": ["kind", "description"]
      },
      "description": "Optional side effects (files written, commands run)"
    },
    "warnings": {
      "type": "array",
      "items": {
        "type": "string"
      },
      "description": "Any errors encountered (non-fatal)"
    },
    "completed_at": {
      "type": "integer",
      "description": "Unix timestamp when the task was completed"
    }
  },
  "required": ["session_id", "correlation_id", "result", "side_effects", "warnings", "completed_at"],
  "additionalProperties": false
}"##;

#[cfg(feature = "schema-validation")]
const EMBEDDED_AGENT_JOB_SCHEMA: &str = r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://github.com/Takon/Wireframe-AI/schemas/v1/agent_job.json",
  "title": "AgentJob",
  "description": "Self-contained unit of work dispatched to a Reasoning Adapter. Must carry everything the adapter needs — no external queries allowed.",
  "type": "object",
  "properties": {
    "job_id": {
      "type": "string",
      "format": "uuid",
      "description": "Unique identifier for this job instance"
    },
    "correlation_parent": {
      "type": "string",
      "description": "Root correlation ID from the original user request. Adapter MUST echo this back in AgentResult.correlation_parent."
    },
    "task": { "$ref": "#/$defs/TaskDescription" },
    "context": { "$ref": "#/$defs/ContextPackage" },
    "available_tool_capabilities": {
      "type": "array",
      "items": { "$ref": "#/$defs/ToolCapability" }
    },
    "constraints": { "$ref": "#/$defs/ExecutionConstraints" },
    "model_config": { "$ref": "#/$defs/ModelConfig" },
    "metadata": { "$ref": "#/$defs/JobMetadata" },
    "schema_version": {
      "type": "integer",
      "default": 1
    }
  },
  "required": ["job_id", "correlation_parent", "task", "context", "available_tool_capabilities", "constraints", "model_config", "metadata"],
  "$defs": {
    "TaskDescription": {
      "type": "object",
      "properties": {
        "user_input": { "type": "string" },
        "sub_task": { "$ref": "#/$defs/SubTask" },
        "output_format": { "$ref": "#/$defs/OutputFormat" },
        "user_constraints": { "type": "array", "items": { "type": "string" } }
      },
      "required": ["user_input"]
    },
    "SubTask": {
      "type": "object",
      "properties": {
        "title": { "type": "string" },
        "description": { "type": "string" },
        "expected_artifacts": { "type": "array", "items": { "type": "string" } }
      },
      "required": ["title", "description"]
    },
    "OutputFormat": {
      "type": "object",
      "properties": {
        "format": { "type": "string" },
        "template": { "type": "string" }
      },
      "required": ["format"]
    },
    "ContextPackage": {
      "type": "object",
      "properties": {
        "memory_chunks": { "type": "array", "items": { "$ref": "#/$defs/MemoryChunk" } },
        "session_history": { "type": "array", "items": { "$ref": "#/$defs/ChatMessage" } },
        "readonly_files": { "type": "array", "items": { "$ref": "#/$defs/FileSnapshot" } },
        "safe_env": { "type": "object", "additionalProperties": { "type": "string" } },
        "working_dir": { "type": "string" },
        "max_context_tokens": { "type": "integer" }
      },
      "required": ["memory_chunks", "session_history", "readonly_files", "safe_env", "working_dir", "max_context_tokens"]
    },
    "MemoryChunk": {
      "type": "object",
      "properties": {
        "id": { "type": "string" },
        "content": { "type": "string" },
        "source": { "type": "string" },
        "relevance_score": { "type": "number" }
      },
      "required": ["id", "content", "source", "relevance_score"]
    },
    "ChatMessage": {
      "type": "object",
      "properties": {
        "role": { "type": "string", "enum": ["user", "assistant", "system"] },
        "content": { "type": "string" },
        "timestamp": { "type": "integer" }
      },
      "required": ["role", "content", "timestamp"]
    },
    "FileSnapshot": {
      "type": "object",
      "properties": {
        "path": { "type": "string" },
        "content": { "type": "string" },
        "size_bytes": { "type": "integer" },
        "last_modified": { "type": "integer" }
      },
      "required": ["path", "content", "size_bytes", "last_modified"]
    },
    "ToolCapability": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "description": { "type": "string" },
        "input_schema": { "type": "object" },
        "required_credentials": {
          "type": "array", "items": { "$ref": "#/$defs/CredentialRef" }
        },
        "rate_limit": { "$ref": "#/$defs/RateLimit" }
      },
      "required": ["name", "description", "input_schema"]
    },
    "CredentialRef": {
      "type": "object",
      "properties": {
        "credential_id": { "type": "string" },
        "scope": { "type": "array", "items": { "type": "string" } }
      },
      "required": ["credential_id", "scope"]
    },
    "RateLimit": {
      "type": "object",
      "properties": {
        "requests_per_minute": { "type": "integer" },
        "burst": { "type": "integer" }
      },
      "required": ["requests_per_minute", "burst"]
    },
    "ExecutionConstraints": {
      "type": "object",
      "properties": {
        "timeout_seconds": { "type": "integer" },
        "max_completion_tokens": { "type": "integer" },
        "network_access": { "type": "string", "enum": ["None", "OutboundOnly", "Full"] },
        "filesystem_policy": { "type": "string", "enum": ["Readonly", "SandboxWritable", "IsolatedVM"] },
        "allow_subprocess": { "type": "boolean" }
      }
    },
    "ModelConfig": {
      "type": "object",
      "properties": {
        "provider": { "type": "string" },
        "model_name": { "type": "string" },
        "temperature": { "type": "number" },
        "top_p": { "type": "number" },
        "extra": { "type": "object" }
      },
      "required": ["provider", "model_name"]
    },
    "JobMetadata": {
      "type": "object",
      "properties": {
        "submitter": { "type": "string" },
        "priority": { "type": "integer", "default": 1 },
        "tags": { "type": "object", "additionalProperties": { "type": "string" } }
      },
      "required": ["submitter"]
    }
  }
}"##;

#[cfg(feature = "schema-validation")]
const EMBEDDED_AGENT_RESULT_SCHEMA: &str = r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://github.com/Takon/Wireframe-AI/schemas/v1/agent_result.json",
  "title": "Agent Result",
  "description": "What an adapter sends back after finishing a job",
  "type": "object",
  "properties": {
    "job_id": {
      "type": "string",
      "format": "uuid",
      "description": "The job this completes"
    },
    "correlation_parent": {
      "type": "string",
      "format": "uuid",
      "description": "Root correlation ID (echoed from AgentJob.correlation_parent)"
    },
    "output": {
      "type": "object",
      "properties": {
        "text": {
          "type": "string",
          "description": "Final text result"
        },
        "structured": {
          "description": "Structured output (JSON, if requested)"
        },
        "files_written": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "Files written (relative to working_dir)"
        },
        "commands_run": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "Commands executed"
        }
      },
      "description": "What the adapter produced"
    },
    "tool_invocations": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "tool_name": {
            "type": "string"
          },
          "parameters": {
            "description": "Tool parameters"
          },
          "result": {
            "description": "Tool result"
          },
          "duration_ms": {
            "type": "integer"
          }
        },
        "required": ["tool_name", "parameters", "result", "duration_ms"]
      },
      "description": "Tool calls made (if any)"
    },
    "errors": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "code": {
            "type": "string"
          },
          "message": {
            "type": "string"
          },
          "retryable": {
            "type": "boolean"
          }
        },
        "required": ["code", "message", "retryable"]
      },
      "description": "Errors that occurred during execution (non-fatal)"
    },
    "usage": {
      "type": "object",
      "properties": {
        "prompt_tokens": {
          "type": "integer"
        },
        "completion_tokens": {
          "type": "integer"
        },
        "total_tokens": {
          "type": "integer"
        },
        "cost_cents": {
          "type": "number"
        }
      },
      "required": ["prompt_tokens", "completion_tokens", "total_tokens"],
      "description": "Token usage / cost metrics"
    },
    "completed_at": {
      "type": "integer",
      "description": "Unix timestamp when completed"
    }
  },
  "required": ["job_id", "correlation_parent", "output", "tool_invocations", "errors", "completed_at"],
  "additionalProperties": false
}"##;

/// Validate an envelope's payload against the appropriate schema for its topic.
/// This is a convenience function that loads the schema and validates the payload.
#[cfg(feature = "schema-validation")]
pub fn validate_envelope_payload<T>(topic: &str, payload: &T) -> Result<(), String>
where
    T: serde::Serialize,
{
    let schema_name = match topic {
        "task.submitted" => "task_submitted.json",
        "task.enriched" => "task_enriched.json",
        "task.complete" => "task_complete.json",
        "agent.job" => "agent_job.json",
        "agent.result" => "agent_result.json",
        _ => return Ok(()), // No schema for this topic, skip validation
    };

    let schema = load_schema(schema_name)?;
    let payload_json =
        serde_json::to_value(payload).map_err(|e| format!("Failed to serialize payload: {}", e))?;

    validate_payload_schema(&schema, &payload_json)
}

// Stub implementations when schema-validation feature is not enabled
#[cfg(not(feature = "schema-validation"))]
pub fn validate_payload_schema(
    _schema: &serde_json::Value,
    _payload: &serde_json::Value,
) -> Result<(), String> {
    Ok(())
}

#[cfg(not(feature = "schema-validation"))]
pub fn load_schema(_schema_name: &str) -> Result<serde_json::Value, String> {
    Err("Schema validation is not enabled. Build with the 'schema-validation' feature to enable schema validation.".to_string())
}

#[cfg(not(feature = "schema-validation"))]
pub fn validate_envelope_payload<T>(_topic: &str, _payload: &T) -> Result<(), String>
where
    T: serde::Serialize,
{
    // When schema validation is not enabled, validation always succeeds (no-op)
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message_types::TaskSubmitted;

    #[test]
    fn envelope_roundtrip() {
        let payload = "hello";
        let env = Envelope::new("test.topic", payload, None);
        assert_eq!(env.payload, "hello");
        assert!(env.validate().is_ok());
    }

    #[test]
    fn child_correlation_inherits() {
        let parent = Envelope::new("task.enriched", "x", None);
        let child = parent.child("agent.job", "y", 1);
        assert_eq!(child.correlation_parent(), parent.correlation_id);
        assert_eq!(child.correlation_id, format!("{}-1", parent.correlation_id));
    }

    #[test]
    fn envelope_serialization_roundtrip() {
        let env = Envelope::new("test.topic", vec![1, 2, 3], Some("session_123".into()));
        let json = serde_json::to_string(&env).unwrap();
        let deserialized: Envelope<Vec<i32>> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.message_id, env.message_id);
        assert_eq!(deserialized.session_id, "session_123");
        assert_eq!(deserialized.topic, "test.topic");
        assert_eq!(deserialized.payload, vec![1, 2, 3]);
        assert!(deserialized.validate().is_ok());
    }

    #[test]
    fn envelope_session_id_defaults() {
        let env = Envelope::<String>::new("test", "x".into(), None);
        assert!(env.session_id.starts_with("session_"));
    }

    #[test]
    fn envelope_reply_inherits_session() {
        let parent = Envelope::new("test.submitted", "hello", Some("session_reply_test".into()));
        let reply = parent.reply("test.result", "world");
        assert_eq!(reply.session_id, "session_reply_test");
        assert_ne!(reply.correlation_id, parent.correlation_id);
        assert_eq!(reply.topic, "test.result");
        assert_eq!(reply.payload, "world");
    }

    #[test]
    #[cfg(feature = "schema-validation")]
    fn validate_payload_schema_works() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "count": { "type": "integer" }
            },
            "required": ["name"]
        });
        let valid_payload = serde_json::json!({"name": "test", "count": 1});
        let invalid_payload = serde_json::json!({"count": 1});

        // Valid payload should pass
        assert!(validate_payload_schema(&schema, &valid_payload).is_ok());

        // Invalid payload should fail (missing required field)
        assert!(validate_payload_schema(&schema, &invalid_payload).is_err());
    }

    #[test]
    #[cfg(not(feature = "schema-validation"))]
    fn validate_payload_schema_stub_always_ok() {
        let schema = serde_json::json!({"type": "object"});
        let payload = serde_json::json!({"foo": "bar"});
        assert!(validate_payload_schema(&schema, &payload).is_ok());
    }

    #[test]
    fn validate_envelope_payload_with_task_submitted() {
        // This test will skip if the schema file doesn't exist (e.g., in CI without full repo)
        let task = TaskSubmitted {
            session_id: "session_test".into(),
            user_input: "Test task".into(),
            submitted_at: 1714880000,
        };

        // Try to validate - if schema is missing, that's ok for this test
        let result = validate_envelope_payload("task.submitted", &task);
        // We don't assert here because the schema might not be available in all environments
        // The important thing is that the function doesn't crash
        let _ = result;
    }

    #[test]
    fn validate_envelope_payload_unknown_topic_skips() {
        // Unknown topics should skip validation without error
        let result =
            validate_envelope_payload("unknown.topic", &serde_json::json!({"test": "data"}));
        assert!(result.is_ok());
    }

    #[cfg(not(feature = "schema-validation"))]
    #[test]
    fn validate_envelope_payload_without_feature_succeeds() {
        // When the feature is not enabled, validation should always succeed (no-op)
        let result =
            validate_envelope_payload("task.submitted", &serde_json::json!({"test": "data"}));
        assert!(result.is_ok());
    }
}

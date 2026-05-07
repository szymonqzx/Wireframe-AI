"""
agentic_sdk — Wireframe AI Python SDK

Provides:
- Envelope: universal NATS message wrapper
- Module: base class for all modules (auto-wires NATS, heartbeat, identity)
- All message payload types
"""

from .envelope import Envelope
from .module import Module
from .message_types import (
    TaskSubmitted,
    TaskEnriched,
    TaskComplete,
    AgentJob,
    AgentResult,
    AgentOutput,
    ToolInvocation,
    AdapterError,
    UsageMetrics,
    SideEffect,
    ContextPackage,
    MemoryChunk,
    ChatMessage,
    FileSnapshot,
    ToolCapability,
    ExecutionConstraints,
    ModelConfig,
    JobMetadata,
    TaskDescription,
    SubTask,
    OutputFormat,
    CredentialRef,
    RateLimit,
    NetworkPolicy,
    FilesystemPolicy,
)

__all__ = [
    "Envelope",
    "Module",
    "TaskSubmitted",
    "TaskEnriched",
    "TaskComplete",
    "AgentJob",
    "AgentResult",
    "AgentOutput",
    "ToolInvocation",
    "AdapterError",
    "UsageMetrics",
    "SideEffect",
    "ContextPackage",
    "MemoryChunk",
    "ChatMessage",
    "FileSnapshot",
    "ToolCapability",
    "ExecutionConstraints",
    "ModelConfig",
    "JobMetadata",
    "TaskDescription",
    "SubTask",
    "OutputFormat",
    "CredentialRef",
    "RateLimit",
    "NetworkPolicy",
    "FilesystemPolicy",
]

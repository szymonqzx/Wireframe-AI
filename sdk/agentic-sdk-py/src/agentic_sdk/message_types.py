"""
agentic_sdk.message_types — All payload types carried inside Envelope

These mirror the Rust structs from agentic_sdk::message_types.
"""

from __future__ import annotations
from dataclasses import dataclass, asdict, field
from datetime import datetime, timezone
from typing import List, Dict, Optional, Any
from enum import Enum
import uuid


def now_ts() -> int:
    return int(datetime.now(timezone.utc).timestamp())


def gen_uuid() -> str:
    return str(uuid.uuid4())


# --------------------------- Task Flow --------------------------- #

@dataclass
class TaskSubmitted:
    session_id: str
    user_input: str
    submitted_at: int = None

    def __post_init__(self) -> None:
        if self.submitted_at is None:
            self.submitted_at = now_ts()

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> TaskSubmitted:
        return cls(**d)


@dataclass
class TaskEnriched:
    session_id: str
    correlation_id: str
    user_input: str
    context: ContextPackage
    inferred_constraints: List[str] = field(default_factory=list)
    enriched_at: int = None

    def __post_init__(self) -> None:
        if self.enriched_at is None:
            self.enriched_at = now_ts()

    def to_dict(self) -> Dict[str, Any]:
        return {
            'session_id': self.session_id,
            'correlation_id': self.correlation_id,
            'user_input': self.user_input,
            'context': self.context.to_dict(),
            'inferred_constraints': self.inferred_constraints,
            'enriched_at': self.enriched_at,
        }

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> TaskEnriched:
        return TaskEnriched(
            session_id=d['session_id'],
            correlation_id=d['correlation_id'],
            user_input=d['user_input'],
            context=ContextPackage.from_dict(d['context']),
            inferred_constraints=d.get('inferred_constraints', []),
            enriched_at=d.get('enriched_at'),
        )


@dataclass
class TaskComplete:
    session_id: str
    correlation_id: str
    result: str
    side_effects: List[SideEffect] = field(default_factory=list)
    warnings: List[str] = field(default_factory=list)
    completed_at: int = None

    def __post_init__(self) -> None:
        if self.completed_at is None:
            self.completed_at = now_ts()

    def to_dict(self) -> Dict[str, Any]:
        return {
            'session_id': self.session_id,
            'correlation_id': self.correlation_id,
            'result': self.result,
            'side_effects': [s.to_dict() for s in self.side_effects],
            'warnings': self.warnings,
            'completed_at': self.completed_at,
        }

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> TaskComplete:
        return TaskComplete(
            session_id=d['session_id'],
            correlation_id=d['correlation_id'],
            result=d['result'],
            side_effects=[SideEffect.from_dict(s) for s in d.get('side_effects', [])],
            warnings=d.get('warnings', []),
            completed_at=d.get('completed_at'),
        )


# --------------------------- Agent Job & Result --------------------------- #

@dataclass
class TaskDescription:
    user_input: str
    sub_task: Optional[SubTask] = None
    output_format: Optional[OutputFormat] = None
    user_constraints: List[str] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        d = {'user_input': self.user_input}
        if self.sub_task:
            d['sub_task'] = self.sub_task.to_dict()
        if self.output_format:
            d['output_format'] = self.output_format.to_dict()
        if self.user_constraints:
            d['user_constraints'] = self.user_constraints
        return d

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> TaskDescription:
        return TaskDescription(
            user_input=d['user_input'],
            sub_task=SubTask.from_dict(d['sub_task']) if d.get('sub_task') else None,
            output_format=OutputFormat.from_dict(d['output_format']) if d.get('output_format') else None,
            user_constraints=d.get('user_constraints', []),
        )


@dataclass
class SubTask:
    title: str
    description: str
    expected_artifacts: List[str] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> SubTask:
        return cls(**d)


@dataclass
class OutputFormat:
    format: str
    template: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> OutputFormat:
        return cls(**d)


@dataclass
class ContextPackage:
    memory_chunks: List[MemoryChunk]
    session_history: List[ChatMessage]
    readonly_files: List[FileSnapshot]
    safe_env: Dict[str, str]
    working_dir: str
    max_context_tokens: int

    def to_dict(self) -> Dict[str, Any]:
        return {
            'memory_chunks': [m.to_dict() for m in self.memory_chunks],
            'session_history': [m.to_dict() for m in self.session_history],
            'readonly_files': [f.to_dict() for f in self.readonly_files],
            'safe_env': self.safe_env,
            'working_dir': self.working_dir,
            'max_context_tokens': self.max_context_tokens,
        }

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> ContextPackage:
        return ContextPackage(
            memory_chunks=[MemoryChunk.from_dict(m) for m in d['memory_chunks']],
            session_history=[ChatMessage.from_dict(m) for m in d['session_history']],
            readonly_files=[FileSnapshot.from_dict(f) for f in d['readonly_files']],
            safe_env=d['safe_env'],
            working_dir=d['working_dir'],
            max_context_tokens=d['max_context_tokens'],
        )


@dataclass
class MemoryChunk:
    id: str
    content: str
    source: str
    relevance_score: float

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> MemoryChunk:
        return cls(**d)


@dataclass
class ChatMessage:
    role: str
    content: str
    timestamp: int = None

    def __post_init__(self) -> None:
        if self.timestamp is None:
            self.timestamp = now_ts()

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> ChatMessage:
        return cls(**d)


@dataclass
class FileSnapshot:
    path: str
    content: str
    size_bytes: int
    last_modified: int

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> FileSnapshot:
        return cls(**d)


@dataclass
class ToolCapability:
    name: str
    description: str
    input_schema: Dict[str, Any]
    required_credentials: List[CredentialRef] = field(default_factory=list)
    rate_limit: Optional[RateLimit] = None

    def to_dict(self) -> Dict[str, Any]:
        d = asdict(self)
        if self.rate_limit:
            d['rate_limit'] = self.rate_limit.to_dict()
        return d

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> ToolCapability:
        return ToolCapability(
            name=d['name'],
            description=d['description'],
            input_schema=d['input_schema'],
            required_credentials=[CredentialRef.from_dict(c) for c in d.get('required_credentials', [])],
            rate_limit=RateLimit.from_dict(d['rate_limit']) if d.get('rate_limit') else None,
        )


@dataclass
class CredentialRef:
    credential_id: str
    scope: List[str]

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> CredentialRef:
        return cls(**d)


@dataclass
class RateLimit:
    requests_per_minute: int
    burst: int

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> RateLimit:
        return cls(**d)


class NetworkPolicy(str, Enum):
    NONE = "None"
    OUTBOUND_ONLY = "OutboundOnly"
    FULL = "Full"


class FilesystemPolicy(str, Enum):
    READONLY = "Readonly"
    SANDBOX_WRITABLE = "SandboxWritable"
    ISOLATED_VM = "IsolatedVM"


@dataclass
class ExecutionConstraints:
    timeout_seconds: Optional[int] = None
    max_completion_tokens: Optional[int] = None
    network_access: NetworkPolicy = NetworkPolicy.OUTBOUND_ONLY
    filesystem_policy: FilesystemPolicy = FilesystemPolicy.SANDBOX_WRITABLE
    allow_subprocess: bool = True

    def to_dict(self) -> Dict[str, Any]:
        return {
            'timeout_seconds': self.timeout_seconds,
            'max_completion_tokens': self.max_completion_tokens,
            'network_access': self.network_access.value,
            'filesystem_policy': self.filesystem_policy.value,
            'allow_subprocess': self.allow_subprocess,
        }

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> ExecutionConstraints:
        network_access = d.get('network_access', 'OutboundOnly')
        filesystem_policy = d.get('filesystem_policy', 'SandboxWritable')
        
        # Handle both string and enum values
        if isinstance(network_access, str):
            try:
                network_access = NetworkPolicy(network_access)
            except ValueError:
                network_access = NetworkPolicy.OUTBOUND_ONLY
        
        if isinstance(filesystem_policy, str):
            try:
                filesystem_policy = FilesystemPolicy(filesystem_policy)
            except ValueError:
                filesystem_policy = FilesystemPolicy.SANDBOX_WRITABLE
        
        return cls(
            timeout_seconds=d.get('timeout_seconds'),
            max_completion_tokens=d.get('max_completion_tokens'),
            network_access=network_access,
            filesystem_policy=filesystem_policy,
            allow_subprocess=d.get('allow_subprocess', True),
        )


@dataclass
class ModelConfig:
    provider: str
    model_name: str
    temperature: Optional[float] = None
    top_p: Optional[float] = None
    extra: Dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> ModelConfig:
        return cls(**d)


@dataclass
class JobMetadata:
    submitter: str
    priority: int = 1
    tags: Dict[str, str] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> JobMetadata:
        return cls(**d)


@dataclass
class AgentJob:
    job_id: str
    correlation_parent: str
    task: TaskDescription
    context: ContextPackage
    available_tool_capabilities: List[ToolCapability]
    constraints: ExecutionConstraints
    model_config: ModelConfig
    metadata: JobMetadata
    adapter_hints: Optional[Dict[str, Any]] = None
    schema_version: int = 1

    def to_dict(self) -> Dict[str, Any]:
        return {
            'job_id': self.job_id,
            'correlation_parent': self.correlation_parent,
            'task': self.task.to_dict(),
            'context': self.context.to_dict(),
            'available_tool_capabilities': [t.to_dict() for t in self.available_tool_capabilities],
            'constraints': self.constraints.to_dict(),
            'model_config': self.model_config.to_dict(),
            'metadata': self.metadata.to_dict(),
            'adapter_hints': self.adapter_hints,
            'schema_version': self.schema_version,
        }

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> AgentJob:
        return AgentJob(
            job_id=d['job_id'],
            correlation_parent=d['correlation_parent'],
            task=TaskDescription.from_dict(d['task']),
            context=ContextPackage.from_dict(d['context']),
            available_tool_capabilities=[ToolCapability.from_dict(t) for t in d['available_tool_capabilities']],
            constraints=ExecutionConstraints.from_dict(d['constraints']),
            model_config=ModelConfig.from_dict(d['model_config']),
            metadata=JobMetadata.from_dict(d['metadata']),
            adapter_hints=d.get('adapter_hints'),
            schema_version=d.get('schema_version', 1),
        )


@dataclass
class AgentOutput:
    text: Optional[str] = None
    structured: Optional[Dict[str, Any]] = None
    files_written: List[str] = field(default_factory=list)
    commands_run: List[str] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> AgentOutput:
        return cls(**d)


@dataclass
class ToolInvocation:
    tool_name: str
    parameters: Dict[str, Any]
    result: Dict[str, Any]
    duration_ms: int

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> ToolInvocation:
        return cls(**d)


@dataclass
class AdapterError:
    code: str
    message: str
    retryable: bool

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> AdapterError:
        return cls(**d)


@dataclass
class UsageMetrics:
    prompt_tokens: int
    completion_tokens: int
    total_tokens: int
    cost_cents: Optional[float] = None

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> UsageMetrics:
        return cls(**d)


@dataclass
class AgentResult:
    job_id: str
    correlation_parent: str
    output: AgentOutput
    tool_invocations: List[ToolInvocation] = field(default_factory=list)
    errors: List[AdapterError] = field(default_factory=list)
    usage: Optional[UsageMetrics] = None
    completed_at: int = None

    def __post_init__(self) -> None:
        if self.completed_at is None:
            self.completed_at = now_ts()

    def to_dict(self) -> Dict[str, Any]:
        return {
            'job_id': self.job_id,
            'correlation_parent': self.correlation_parent,
            'output': self.output.to_dict(),
            'tool_invocations': [t.to_dict() for t in self.tool_invocations],
            'errors': [e.to_dict() for e in self.errors],
            'usage': self.usage.to_dict() if self.usage else None,
            'completed_at': self.completed_at,
        }

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> AgentResult:
        return AgentResult(
            job_id=d['job_id'],
            correlation_parent=d['correlation_parent'],
            output=AgentOutput.from_dict(d['output']),
            tool_invocations=[ToolInvocation.from_dict(t) for t in d.get('tool_invocations', [])],
            errors=[AdapterError.from_dict(e) for e in d.get('errors', [])],
            usage=UsageMetrics.from_dict(d['usage']) if d.get('usage') else None,
            completed_at=d.get('completed_at'),
        )


@dataclass
class SideEffect:
    kind: str
    description: str
    path: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> SideEffect:
        return cls(**d)

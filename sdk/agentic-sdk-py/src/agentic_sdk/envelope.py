"""
agentic_sdk.envelope — Universal message envelope

Mirrors the Rust agentic_sdk::envelope::Envelope<T> struct.
All messages on the NATS bus use this wrapper.
"""

from __future__ import annotations
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from typing import TypeVar, Generic, Any, Dict
import uuid

T = TypeVar('T')


@dataclass
class Envelope(Generic[T]):
    message_id: str
    session_id: str
    correlation_id: str
    topic: str
    payload: T
    schema_version: int = 1
    timestamp: Optional[int] = None

    def __post_init__(self) -> None:
        if self.timestamp is None:
            self.timestamp = int(datetime.now(timezone.utc).timestamp())

    @classmethod
    def new(cls, topic: str, payload: T, session_id: str | None = None) -> Envelope[T]:
        """Construct a new envelope with fresh IDs."""
        return cls(
            message_id=str(uuid.uuid4()),
            session_id=session_id or f"session_{uuid.uuid4()}",
            correlation_id=str(uuid.uuid4()),
            topic=topic,
            payload=payload,
            schema_version=1,
        )

    def child(self, topic: str, payload: T, child_index: int) -> Envelope[T]:
        """Create a child envelope inheriting parent session/correlation."""
        return Envelope(
            message_id=str(uuid.uuid4()),
            session_id=self.session_id,
            correlation_id=f"{self.correlation_id}-{child_index}",
            topic=topic,
            payload=payload,
            schema_version=1,
            timestamp=int(datetime.now(timezone.utc).timestamp()),
        )

    def reply(self, topic: str, payload: T) -> Envelope[T]:
        """Create a reply envelope for responding to this message.
        Inherits session_id, generates a fresh correlation_id."""
        return Envelope(
            message_id=str(uuid.uuid4()),
            session_id=self.session_id,
            correlation_id=str(uuid.uuid4()),
            topic=topic,
            payload=payload,
            schema_version=1,
            timestamp=int(datetime.now(timezone.utc).timestamp()),
        )

    def to_dict(self) -> Dict[str, Any]:
        """Serialize envelope to dict (including payload)."""
        data = {
            'message_id': self.message_id,
            'session_id': self.session_id,
            'correlation_id': self.correlation_id,
            'topic': self.topic,
            'schema_version': self.schema_version,
            'timestamp': self.timestamp,
            'payload': self._payload_to_dict(),
        }
        return data

    def _payload_to_dict(self) -> Any:
        if hasattr(self.payload, 'to_dict'):
            return self.payload.to_dict()
        elif hasattr(self.payload, '__dict__'):
            return asdict(self.payload)
        return self.payload

    def to_json(self) -> str:
        import json
        return json.dumps(self.to_dict())

    @classmethod
    def parse_raw(cls, raw: str) -> Envelope[Any]:
        """Parse JSON into an envelope — caller must cast payload to expected type."""
        import json
        data = json.loads(raw)
        return Envelope(
            message_id=data['message_id'],
            session_id=data['session_id'],
            correlation_id=data['correlation_id'],
            topic=data['topic'],
            payload=data.get('payload'),
            schema_version=data.get('schema_version', 1),
            timestamp=data.get('timestamp'),
        )

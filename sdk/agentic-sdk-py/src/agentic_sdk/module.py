"""
agentic_sdk.module — SDK Module base class

Any Python module can extend Module and implement handle().
The base class handles:
  - NATS connection and reconnection
  - sys.module.online announcement on startup
  - sys.module.offline on graceful shutdown
  - Queue group registration
  - Envelope serialization/deserialization
"""

from __future__ import annotations

import asyncio
import signal
import sys
import uuid
from abc import ABC, abstractmethod
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any, Dict, List

from .envelope import Envelope


@dataclass
class ModuleIdentity:
    """Identity payload for sys.module.online/offline messages."""
    module_id: str
    version: str
    subscribes: List[str]
    publishes: List[str]


class Module(ABC):
    """Base class for all Wireframe AI modules.

    Subclasses must define:
      - module_id: str          — unique module identifier
      - subscribes: List[str]   — topics this module subscribes to
      - publishes: List[str]    — topics this module publishes
      - queue_group: str        — NATS queue group for load balancing

    And implement:
      - async handle(self, env: Envelope) -> List[Envelope]
    """

    module_id: str = ""
    subscribes: List[str] = []
    publishes: List[str] = []
    queue_group: str = ""

    def __init__(self):
        if not self.module_id:
            self.module_id = f"{type(self).__name__}-{uuid.uuid4().hex[:8]}"

    async def run(self, nats_url: str = "nats://localhost:4222") -> None:
        """Connect to NATS, announce, listen, and handle messages."""
        import nats

        loop = asyncio.get_running_loop()

        # Connect to NATS
        nc = await nats.connect(nats_url)
        sys.stderr.write(f"[{self.module_id}] connected to NATS at {nats_url}\n")

        # Announce online
        identity = ModuleIdentity(
            module_id=self.module_id,
            version="0.1.0",
            subscribes=self.subscribes,
            publishes=self.publishes,
        )
        online_env = Envelope.new(
            "sys.module.online",
            {
                "module_id": identity.module_id,
                "version": identity.version,
                "subscribes": identity.subscribes,
                "publishes": identity.publishes,
            },
        )
        await nc.publish("sys.module.online", online_env.to_json().encode())
        sys.stderr.write(f"[{self.module_id}] announced online\n")

        # Start heartbeat
        async def heartbeat_loop():
            while True:
                await asyncio.sleep(30)
                hb_env = Envelope.new(
                    "sys.module.heartbeat",
                    {"module_id": self.module_id, "ts": int(datetime.now(timezone.utc).timestamp())},
                )
                try:
                    await nc.publish("sys.module.heartbeat", hb_env.to_json().encode())
                except Exception:
                    pass

        heartbeat_task = asyncio.create_task(heartbeat_loop())
        sys.stderr.write(f"[{self.module_id}] heartbeat started (30s interval)\n")

        # Set up graceful shutdown
        shutdown_event = asyncio.Event()

        def _signal_handler():
            sys.stderr.write(f"\n[{self.module_id}] shutting down...\n")
            shutdown_event.set()

        try:
            loop.add_signal_handler(signal.SIGINT, _signal_handler)
            loop.add_signal_handler(signal.SIGTERM, _signal_handler)
        except NotImplementedError:
            # Windows doesn't support add_signal_handler
            pass

        # Subscribe to each topic with queue group
        subscriptions = []
        for topic in self.subscribes:
            sub = await nc.subscribe(topic, queue=self.queue_group)
            subscriptions.append(sub)
            sys.stderr.write(
                f"[{self.module_id}] subscribed to {topic} (queue: {self.queue_group})\n"
            )

        sys.stderr.write(f"[{self.module_id}] ready — waiting for messages\n")

        # Subscribe to all messages
        async def process_messages():
            async for msg in nc.messages:
                try:
                    raw = msg.data.decode()
                    env = Envelope.parse_raw(raw)
                    results = await self.handle(env)
                    for result_env in results:
                        payload = result_env.to_json().encode()
                        await nc.publish(result_env.topic, payload)
                except Exception as exc:
                    await self.publish_error(
                        nc, "PARSE_ERROR", str(exc),
                        correlation_id=None,
                    )
                    sys.stderr.write(
                        f"[{self.module_id}] error processing message: {exc}\n"
                    )

        # Run processing and wait for shutdown signal
        processor = asyncio.create_task(process_messages())

        await shutdown_event.wait()

        # Announce offline
        offline_env = Envelope.new(
            "sys.module.offline",
            {
                "module_id": self.module_id,
                "version": "0.1.0",
            },
        )
        await nc.publish("sys.module.offline", offline_env.to_json().encode())
        sys.stderr.write(f"[{self.module_id}] announced offline\n")

        # Clean up
        processor.cancel()
        heartbeat_task.cancel()
        try:
            await heartbeat_task
        except asyncio.CancelledError:
            pass
        await nc.drain()
        sys.stderr.write(f"[{self.module_id}] disconnected\n")

    async def publish_error(
        self, nc, error_code: str, error_message: str,
        correlation_id: str | None = None,
    ) -> None:
        """Publish an error event to sys.module.error."""
        payload = {
            "module_id": self.module_id,
            "error_code": error_code,
            "error_message": error_message,
            "correlation_id": correlation_id,
            "ts": int(datetime.now(timezone.utc).timestamp()),
        }
        env = Envelope.new("sys.module.error", payload)
        await nc.publish("sys.module.error", env.to_json().encode())

    @abstractmethod
    async def handle(self, env: Envelope) -> List[Envelope]:
        """Process an incoming envelope and return response envelopes."""
        ...

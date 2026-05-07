#!/usr/bin/env python3
"""Smoke test for the Python SDK — validates imports and basic operations.

Run: python tests/test_python_sdk.py
Requires: pip install -e sdk/agentic-sdk-py
"""

import sys
import os
import json

# Add SDK to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'sdk', 'agentic-sdk-py', 'src'))

def test_imports():
    """Test that all SDK modules import correctly."""
    from agentic_sdk import Envelope, Module
    from agentic_sdk import (
        TaskSubmitted, TaskEnriched, TaskComplete,
        AgentJob, AgentResult, AgentOutput,
        ContextPackage, MemoryChunk, ChatMessage, FileSnapshot,
        ToolCapability, ExecutionConstraints, ModelConfig, JobMetadata,
        TaskDescription, SubTask, OutputFormat, CredentialRef, RateLimit,
        ToolInvocation, AdapterError, UsageMetrics, SideEffect,
        NetworkPolicy, FilesystemPolicy,
    )
    print(f"  ✅ All SDK imports OK ({len(dir(AgentJob))} members in AgentJob)")


def test_envelope_basics():
    """Test Envelope creation, reply, child, and JSON roundtrip."""
    from agentic_sdk import Envelope

    # Create
    env = Envelope.new("test.topic", {"hello": "world"}, session_id="session_test")
    assert env.topic == "test.topic"
    assert env.session_id == "session_test"
    assert env.payload == {"hello": "world"}
    print(f"  ✅ Envelope created: message_id={env.message_id[:8]}...")

    # Reply
    reply = env.reply("test.reply", {"ok": True})
    assert reply.session_id == env.session_id
    assert reply.topic == "test.reply"
    assert reply.payload == {"ok": True}
    print(f"  ✅ Envelope reply: topic={reply.topic}, session matches parent")

    # Child
    child = env.child("test.child", {"index": 1}, child_index=1)
    assert child.session_id == env.session_id
    assert child.topic == "test.child"
    assert child.payload == {"index": 1}
    print(f"  ✅ Envelope child: correlation_id={child.correlation_id}")

    # JSON roundtrip
    json_str = env.to_json()
    parsed = Envelope.parse_raw(json_str)
    assert parsed.message_id == env.message_id
    assert parsed.session_id == "session_test"
    assert parsed.topic == "test.topic"
    print(f"  ✅ Envelope JSON roundtrip OK ({len(json_str)} bytes)")


def test_message_types():
    """Test that all message types can be created and serialized."""
    from agentic_sdk import (
        AgentJob, TaskDescription, ContextPackage, MemoryChunk,
        ChatMessage, ExecutionConstraints, ModelConfig, JobMetadata,
        NetworkPolicy, FilesystemPolicy,
    )

    job = AgentJob(
        job_id="test-job-001",
        correlation_parent="test-parent",
        task=TaskDescription(user_input="Test task"),
        context=ContextPackage(
            memory_chunks=[
                MemoryChunk(id="m1", content="test memory", source="test", relevance_score=0.9)
            ],
            session_history=[ChatMessage(role="user", content="hello")],
            readonly_files=[],
            safe_env={},
            working_dir="/tmp",
            max_context_tokens=1000,
        ),
        available_tool_capabilities=[],
        constraints=ExecutionConstraints(
            network_access=NetworkPolicy.OUTBOUND_ONLY,
            filesystem_policy=FilesystemPolicy.SANDBOX_WRITABLE,
        ),
        model_config=ModelConfig(provider="openai", model_name="gpt-4o"),
        metadata=JobMetadata(submitter="test"),
        adapter_hints=None,
    )
    d = job.to_dict()
    assert d["job_id"] == "test-job-001"
    assert d["task"]["user_input"] == "Test task"
    assert d["context"]["memory_chunks"][0]["content"] == "test memory"
    assert d["adapter_hints"] is None
    assert d["constraints"]["network_access"] == "OutboundOnly"
    assert d["constraints"]["filesystem_policy"] == "SandboxWritable"

    # Roundtrip
    job2 = AgentJob.from_dict(d)
    assert job2.job_id == "test-job-001"
    assert job2.task.user_input == "Test task"
    assert job2.constraints.network_access == NetworkPolicy.OUTBOUND_ONLY
    assert job2.constraints.filesystem_policy == FilesystemPolicy.SANDBOX_WRITABLE
    print(f"  ✅ AgentJob serialization roundtrip OK")


def test_module_class():
    """Test that the Module class can be extended."""
    from agentic_sdk import Module, Envelope

    class TestModule(Module):
        module_id = "test-module"
        subscribes = ["test.request"]
        publishes = ["test.response"]
        queue_group = "test_group"

        async def handle(self, env):
            return [env.reply("test.response", {"echo": env.payload})]

    module = TestModule()
    assert module.module_id == "test-module"
    assert module.subscribes == ["test.request"]
    assert module.publishes == ["test.response"]
    assert module.queue_group == "test_group"
    print(f"  ✅ Module class can be extended (handle coroutine OK)")


def test_enum_serialization():
    """Test that enum types serialize/deserialize correctly."""
    from agentic_sdk import ExecutionConstraints, NetworkPolicy, FilesystemPolicy

    # Test with enum values
    constraints = ExecutionConstraints(
        network_access=NetworkPolicy.OUTBOUND_ONLY,
        filesystem_policy=FilesystemPolicy.SANDBOX_WRITABLE,
    )
    d = constraints.to_dict()
    assert d["network_access"] == "OutboundOnly"
    assert d["filesystem_policy"] == "SandboxWritable"

    # Test roundtrip
    constraints2 = ExecutionConstraints.from_dict(d)
    assert constraints2.network_access == NetworkPolicy.OUTBOUND_ONLY
    assert constraints2.filesystem_policy == FilesystemPolicy.SANDBOX_WRITABLE

    # Test with string values (backward compatibility)
    d_string = {
        "network_access": "OutboundOnly",
        "filesystem_policy": "SandboxWritable",
        "timeout_seconds": 300,
        "max_completion_tokens": 4096,
        "allow_subprocess": True,
    }
    constraints3 = ExecutionConstraints.from_dict(d_string)
    assert constraints3.network_access == NetworkPolicy.OUTBOUND_ONLY
    assert constraints3.filesystem_policy == FilesystemPolicy.SANDBOX_WRITABLE

    print(f"  ✅ Enum serialization roundtrip OK")


if __name__ == "__main__":
    print("Python SDK Smoke Test")
    print("=" * 40)
    test_imports()
    test_envelope_basics()
    test_message_types()
    test_module_class()
    test_enum_serialization()
    print("=" * 40)
    print("✅ All Python SDK smoke tests passed!")
    sys.exit(0)

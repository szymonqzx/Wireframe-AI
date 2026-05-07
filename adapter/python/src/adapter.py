#!/usr/bin/env python3
"""
wireframe-ai-reasoning-adapter — LLM-powered agent

Subscribes to "agent.job" (queue group: agent_worker) via NATS.
Publishes "agent.result" when done.

Supports multiple LLM providers:
  - openai (gpt-4o, gpt-4o-mini, o3-mini, etc.)
  - anthropic (claude-3-5-sonnet, claude-3-haiku, etc.)
  - deepseek (deepseek-chat, deepseek-reasoner, etc.)
  - opencode-go (deepseek-v4-flash, deepseek-v4-pro, kimi-k2.5, etc.)

Invokes sandbox tools (file_read, file_write, shell_exec, file_list)
via MCP stdio as needed.

Environment variables:
  WIREFRAME_AI_NATS_URL        NATS server URL (default: nats://localhost:4222)
  OPENAI_API_KEY               OpenAI API key
  ANTHROPIC_API_KEY            Anthropic API key
  DEEPSEEK_API_KEY             DeepSeek API key
  OPENCODE_GO_API_KEY          OpenCode Go API key
  WIREFRAME_AI_SANDBOX_PATH    Path to wireframe-ai-sandbox-core binary
"""

from __future__ import annotations

import asyncio
import json
import os
import sys
import time
import uuid
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional

from agentic_sdk import (
    Envelope,
    Module,
    AgentJob,
    AgentResult,
    AgentOutput,
    ToolInvocation as ToolInvocationResult,
    AdapterError,
    UsageMetrics,
)


# ── Provider config ──────────────────────────────────────────────────────────

_PROVIDERS_PATH = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "providers.json",
)


def _load_providers_config() -> dict:
    """Load provider definitions from providers.json."""
    try:
        with open(_PROVIDERS_PATH) as f:
            return json.load(f)
    except FileNotFoundError:
        return {"providers": []}


def _get_provider_config(name: str) -> Optional[dict]:
    """Look up a provider by name in providers.json."""
    config = _load_providers_config()
    for p in config.get("providers", []):
        if p["name"] == name:
            return p
    return None


# ── LLM provider router ──────────────────────────────────────────────────────


async def call_llm(
    provider: str,
    model_name: str,
    messages: List[Dict[str, str]],
    tools: Optional[List[Dict[str, Any]]] = None,
    max_tokens: int = 4096,
    temperature: float = 0.7,
) -> Dict[str, Any]:
    """Route a chat completion request to the appropriate provider.

    Providers are defined in ``providers.json`` — run ``add_provider.py``
    to add new ones interactively.

    Returns {"role": "assistant", "content": "...", "tool_calls": [...]}
    """
    provider = provider.lower()
    cfg = _get_provider_config(provider)

    if cfg is None:
        raise ValueError(
            f"Unsupported provider: {provider}. "
            f"Run `python add_provider.py` to add it."
        )

    ptype = cfg["type"]

    if ptype == "openai_compatible":
        api_key = os.environ.get(cfg["api_key_env"])
        base_url = cfg.get("base_url")  # None → default OpenAI
        return await _call_openai_compatible(
            base_url=base_url,
            api_key=api_key,
            model=model_name,
            messages=messages,
            tools=tools,
            max_tokens=max_tokens,
            temperature=temperature,
        )

    elif ptype == "anthropic":
        return await _call_anthropic(
            model_name, messages, tools, max_tokens, temperature
        )

    else:
        raise ValueError(f"Unknown provider type '{ptype}' for {provider}")


# ── OpenAI-compatible handler (generic — used by any provider with type=openai_compatible) ──


async def _call_openai_compatible(
    base_url: Optional[str],
    api_key: Optional[str],
    model: str,
    messages: List[Dict[str, str]],
    tools: Optional[List[Dict[str, Any]]],
    max_tokens: int,
    temperature: float,
) -> Dict[str, Any]:
    """Call any OpenAI-compatible chat completion API.

    Parameters
    ----------
    base_url : str or None
        Custom base URL (e.g. ``https://api.deepseek.com``).
        When ``None``, uses the default OpenAI endpoint.
    api_key : str or None
        API key read from the environment variable configured in providers.json.
    """
    try:
        from openai import AsyncOpenAI
    except ImportError:
        raise ImportError("Install openai: pip install openai")

    client_kwargs: Dict[str, Any] = {}
    if api_key:
        client_kwargs["api_key"] = api_key
    if base_url:
        client_kwargs["base_url"] = base_url

    client = AsyncOpenAI(**client_kwargs)

    kwargs: Dict[str, Any] = {
        "model": model,
        "messages": messages,
        "max_tokens": max_tokens,
        "temperature": temperature,
    }
    if tools:
        kwargs["tools"] = tools

    response = await client.chat.completions.create(**kwargs)
    choice = response.choices[0]
    msg = choice.message

    result = {"role": "assistant", "content": msg.content or ""}

    if msg.tool_calls:
        result["tool_calls"] = [
            {
                "id": tc.id,
                "type": "function",
                "function": {
                    "name": tc.function.name,
                    "arguments": tc.function.arguments,
                },
            }
            for tc in msg.tool_calls
        ]

    return result


# ── Anthropic (specialized — different message format) ────────────────────────


async def _call_anthropic(
    model: str,
    messages: List[Dict[str, str]],
    tools: Optional[List[Dict[str, Any]]],
    max_tokens: int,
    temperature: float,
) -> Dict[str, Any]:
    try:
        from anthropic import AsyncAnthropic
    except ImportError:
        raise ImportError("Install anthropic: pip install anthropic")

    client = AsyncAnthropic(api_key=os.environ.get("ANTHROPIC_API_KEY"))

    # Convert from OpenAI format to Anthropic format
    system_msg = None
    anthropic_messages = []
    for m in messages:
        if m["role"] == "system" and system_msg is None:
            system_msg = m["content"]
        elif m["role"] == "user":
            anthropic_messages.append({"role": "user", "content": m["content"]})
        elif m["role"] == "assistant":
            anthropic_messages.append({"role": "assistant", "content": m["content"]})
        elif m["role"] == "tool":
            anthropic_messages.append({"role": "user", "content": m["content"]})

    kwargs: Dict[str, Any] = {
        "model": model,
        "messages": anthropic_messages,
        "max_tokens": max_tokens,
        "temperature": temperature,
    }
    if system_msg:
        kwargs["system"] = system_msg
    if tools:
        kwargs["tools"] = _convert_to_anthropic_tools(tools)

    response = await client.messages.create(**kwargs)

    content_blocks = response.content
    text_parts = []
    tool_calls = []

    for block in content_blocks:
        if block.type == "text":
            text_parts.append(block.text)
        elif block.type == "tool_use":
            tool_calls.append({
                "id": block.id,
                "type": "function",
                "function": {
                    "name": block.name,
                    "arguments": json.dumps(block.input),
                },
            })

    return {
        "role": "assistant",
        "content": "\n".join(text_parts),
        "tool_calls": tool_calls if tool_calls else None,
    }


# ── Anthropic tool format conversion ──────────────────────────────────────────


def _convert_to_anthropic_tools(tools: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Convert OpenAI-format tools to Anthropic format."""
    result = []
    for t in tools:
        result.append({
            "name": t["function"]["name"],
            "description": t["function"].get("description", ""),
            "input_schema": t["function"]["parameters"],
        })
    return result


# ── Tool definitions for the LLM ─────────────────────────────────────────────

SANDBOX_TOOLS_DEFINITION = [
    {
        "type": "function",
        "function": {
            "name": "shell_exec",
            "description": "Execute a shell command in the sandbox environment",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Shell command to execute (e.g., 'python3 script.py')",
                    },
                    "working_dir": {
                        "type": "string",
                        "description": "Working directory relative to sandbox root",
                    },
                },
                "required": ["command"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "file_read",
            "description": "Read the contents of a file in the sandbox",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path relative to sandbox root",
                    },
                },
                "required": ["path"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "file_write",
            "description": "Write content to a file in the sandbox (creates directories if needed)",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path relative to sandbox root",
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file",
                    },
                },
                "required": ["path", "content"],
            },
        },
    },
    {
        "type": "function",
        "function": {
            "name": "file_list",
            "description": "List files and directories in a sandbox path",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path relative to sandbox root",
                    },
                },
                "required": ["path"],
            },
        },
    },
]


# ── ReasoningAdapter ─────────────────────────────────────────────────────────


class ReasoningAdapter(Module):
    """LLM-powered reasoning adapter that processes AgentJob messages."""

    module_id = "reasoning-adapter"
    subscribes = ["agent.job"]
    publishes = ["agent.result", "exec.request"]
    queue_group = "agent_worker"

    def __init__(self):
        super().__init__()
        self.sandbox_path: Optional[str] = os.environ.get(
            "WIREFRAME_AI_SANDBOX_PATH"
        )
        self.tool_invocations: List[ToolInvocationResult] = []
        self.tool_results_cache: Dict[str, Any] = {}

    async def handle(self, env: Envelope) -> List[Envelope]:
        if env.topic != "agent.job":
            sys.stderr.write(
                f"[{self.module_id}] ignoring unexpected topic: {env.topic}\n"
            )
            return []

        job = AgentJob.from_dict(env.payload)
        sys.stderr.write(
            f"[{self.module_id}] received job {job.job_id} — "
            f"correlation {job.correlation_parent}\n"
        )

        self.tool_invocations = []
        self.tool_results_cache = {}

        try:
            output = await self.handle_task(job)
            errors = []
        except Exception as e:
            output = AgentOutput(text=f"Error: {e}")
            errors = [AdapterError(code="EXECUTION_ERROR", message=str(e), retryable=False)]

        # Count actual tokens from LLM if available
        prompt_tokens = self._estimate_input_tokens(job)
        completion_tokens = self._estimate_output_tokens(output)

        result = AgentResult(
            job_id=job.job_id,
            correlation_parent=job.correlation_parent,
            output=output,
            tool_invocations=self.tool_invocations,
            errors=errors,
            usage=UsageMetrics(
                prompt_tokens=prompt_tokens,
                completion_tokens=completion_tokens,
                total_tokens=prompt_tokens + completion_tokens,
                cost_cents=None,
            ),
            completed_at=int(datetime.now(timezone.utc).timestamp()),
        )

        result_env = Envelope.new("agent.result", result.to_dict(), session_id=env.session_id)
        return [result_env]

    async def handle_task(self, job: AgentJob) -> AgentOutput:
        """Execute a job by calling an LLM with tool-use loop.

        The LLM can call sandbox tools (shell_exec, file_read, file_write, file_list)
        in multiple rounds until a final answer is produced.
        """
        provider = job.model_config.provider or "openai"
        model = job.model_config.model_name or "gpt-4o"
        temp = job.model_config.temperature or 0.7
        max_tokens = job.constraints.max_completion_tokens or 4096

        # Build message history
        messages: List[Dict[str, str]] = [
            {
                "role": "system",
                "content": (
                    "You are a helpful AI assistant with access to a sandbox environment. "
                    "You can execute shell commands, read/write files, and list directories. "
                    "Use these capabilities to fulfill the user's request. "
                    "When you have completed the task, provide a clear summary of what you did.\n\n"
                    "Available tools:\n"
                    "- shell_exec: Execute shell commands (python, bash, etc.)\n"
                    "- file_read: Read file contents\n"
                    "- file_write: Write content to files\n"
                    "- file_list: List directory contents"
                ),
            }
        ]

        # Add session history context
        for msg in job.context.session_history:
            role = msg.role if msg.role in ("user", "assistant", "system") else "user"
            messages.append({"role": role, "content": msg.content})

        # Add memory chunks as context
        memory_text = ""
        for chunk in job.context.memory_chunks:
            memory_text += f"\n[{chunk.source} (relevance: {chunk.relevance_score:.2f})]\n{chunk.content}\n"
        if memory_text:
            messages.insert(
                1,
                {
                    "role": "system",
                    "content": f"Relevant context from previous sessions:\n{memory_text}",
                },
            )

        # Add current task
        messages.append({"role": "user", "content": job.task.user_input})

        # Tool-use loop (max 10 rounds to prevent infinite loops)
        max_rounds = 10
        tool_config = SANDBOX_TOOLS_DEFINITION

        for _round in range(max_rounds):
            response = await call_llm(
                provider=provider,
                model_name=model,
                messages=messages,
                tools=tool_config if self._sandbox_available() else None,
                max_tokens=max_tokens,
                temperature=temp,
            )

            messages.append({"role": "assistant", "content": response.get("content", "")})

            tool_calls = response.get("tool_calls")
            if not tool_calls:
                # No more tool calls — this is the final response
                break

            # Execute each tool call
            for tc in tool_calls:
                func_name = tc["function"]["name"]
                try:
                    arguments = json.loads(tc["function"]["arguments"])
                except json.JSONDecodeError as e:
                    sys.stderr.write(f"[{self.module_id}] failed to parse tool arguments: {e}\n")
                    arguments = {}

                sys.stderr.write(
                    f"[{self.module_id}] calling tool: {func_name}({json.dumps(arguments)[:100]})\n"
                )

                start_time = time.monotonic()
                try:
                    if func_name in ("shell_exec", "file_read", "file_write", "file_list"):
                        result = await self.call_sandbox_tool(func_name, arguments)
                    else:
                        result = {"error": f"Unknown tool: {func_name}"}
                except Exception as e:
                    result = {"error": str(e)}

                duration_ms = int((time.monotonic() - start_time) * 1000)

                self.tool_invocations.append(ToolInvocationResult(
                    tool_name=func_name,
                    parameters=arguments,
                    result=result,
                    duration_ms=duration_ms,
                ))

                # Add tool result as a message
                messages.append({
                    "role": "tool",
                    "tool_call_id": tc["id"],
                    "content": json.dumps(result),
                })

        # Extract final text and track side effects
        final_text = response.get("content", "")
        files_written = []
        commands_run = []

        for ti in self.tool_invocations:
            if ti.tool_name == "file_write":
                files_written.append(ti.parameters.get("path", ""))
            elif ti.tool_name == "shell_exec":
                commands_run.append(ti.parameters.get("command", ""))

        return AgentOutput(
            text=final_text,
            structured=None,
            files_written=[p for p in files_written if p],
            commands_run=commands_run,
        )

    def _sandbox_available(self) -> bool:
        import shutil
        return shutil.which("wireframe-ai-sandbox-core") is not None

    async def call_sandbox_tool(self, tool_name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
        """Call a sandbox tool via MCP over stdio."""
        if not self.sandbox_path:
            return {"error": "Sandbox not available — set WIREFRAME_AI_SANDBOX_PATH"}

        # Validate tool name
        if tool_name not in ("shell_exec", "file_read", "file_write", "file_list"):
            return {"error": f"Unknown tool: {tool_name}"}

        try:
            proc = await asyncio.create_subprocess_exec(
                self.sandbox_path,
                stdin=asyncio.subprocess.PIPE,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
        except FileNotFoundError:
            return {"error": f"Sandbox binary not found at: {self.sandbox_path}"}
        except PermissionError:
            return {"error": f"Permission denied executing sandbox: {self.sandbox_path}"}
        except Exception as e:
            return {"error": f"Failed to start sandbox process: {e}"}

        try:
            # MCP initialize with timeout
            req_id = str(uuid.uuid4())
            assert proc.stdin is not None
            proc.stdin.write(
                (json.dumps({
                    "jsonrpc": "2.0", "id": req_id, "method": "initialize",
                    "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                               "clientInfo": {"name": "wireframe-ai-adapter", "version": "0.1.0"}},
                }) + "\n").encode()
            )
            await proc.stdin.drain()

            assert proc.stdout is not None
            try:
                init_line = await asyncio.wait_for(proc.stdout.readline(), timeout=10.0)
            except asyncio.TimeoutError:
                return {"error": "Sandbox initialization timed out"}

            if not init_line:
                return {"error": "Sandbox process ended during initialization"}

            try:
                init_resp = json.loads(init_line.decode())
            except json.JSONDecodeError as e:
                return {"error": f"Failed to parse initialize response: {e}"}

            if "error" in init_resp:
                return {"error": f"Initialize failed: {init_resp['error']}"}

            # Send initialized
            proc.stdin.write((json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n").encode())
            await proc.stdin.drain()

            # Tool call with timeout
            req_id = str(uuid.uuid4())
            proc.stdin.write(
                (json.dumps({
                    "jsonrpc": "2.0", "id": req_id, "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                }) + "\n").encode()
            )
            await proc.stdin.drain()

            try:
                response_line = await asyncio.wait_for(proc.stdout.readline(), timeout=300.0)
            except asyncio.TimeoutError:
                return {"error": f"Tool '{tool_name}' timed out after 300 seconds"}

            if not response_line:
                return {"error": "Sandbox process ended during tool call"}

            try:
                response = json.loads(response_line.decode())
            except json.JSONDecodeError as e:
                return {"error": f"Failed to parse tool response: {e}"}

            if "error" in response:
                return {"error": f"Tool '{tool_name}' failed: {response['error']}"}

            result = response.get("result", {})
            # MCP returns CallToolResult with nested content
            # Validate response structure to prevent crashes on malformed data
            if not isinstance(result, dict):
                return {"error": f"Tool '{tool_name}' returned invalid result type: {type(result).__name__}"}

            content_list = result.get("content", [])
            if not isinstance(content_list, list):
                return {"error": f"Tool '{tool_name}' returned invalid content type: {type(content_list).__name__}"}

            text_contents = []
            for c in content_list:
                if isinstance(c, dict) and c.get("type") == "text":
                    text = c.get("text", "")
                    if isinstance(text, str):
                        text_contents.append(text)

            return {
                "output": "\n".join(text_contents) if text_contents else result,
                "isError": result.get("isError", False),
            }
        except asyncio.CancelledError:
            return {"error": "Tool call was cancelled"}
        except Exception as e:
            return {"error": f"Unexpected error during tool call: {e}"}
        finally:
            # Always ensure the process is terminated
            if proc.returncode is None:
                proc.terminate()
                try:
                    await asyncio.wait_for(proc.wait(), timeout=5.0)
                except asyncio.TimeoutError:
                    proc.kill()
                    await proc.wait()

    def _estimate_input_tokens(self, job: AgentJob) -> int:
        return len(job.task.user_input.split()) * 2 + len(job.context.session_history) * 50

    def _estimate_output_tokens(self, output: AgentOutput) -> int:
        return len((output.text or "").split())


def main() -> None:
    nats_url = os.environ.get("WIREFRAME_AI_NATS_URL", "nats://localhost:4222")
    adapter = ReasoningAdapter()
    asyncio.run(adapter.run(nats_url))


if __name__ == "__main__":
    main()

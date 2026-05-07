#!/usr/bin/env python3
"""
Interactive CLI to add a new LLM provider to the Wireframe AI adapter.

Providers are stored in ``providers.json`` and loaded at runtime by the
reasoning adapter. This script guides you through the addition and can
optionally run a test call to verify connectivity.

Usage::

    python add_provider.py
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

PROVIDERS_PATH = Path(__file__).parent / "providers.json"


# ── CLI helpers ──────────────────────────────────────────────────────────────


def style(text: str, code: str) -> str:
    """Simple terminal styling. Falls back to plain text on Windows if ANSI
    is not supported."""
    codes = {"green": "32", "bold": "1", "dim": "2", "red": "31", "cyan": "36"}
    c = codes.get(code, "")
    if not c or os.name == "nt" and not os.environ.get("TERM"):
        return text
    return f"\033[{c}m{text}\033[0m"


def prompt(label: str, default: str = "") -> str:
    """Ask the user for input with an optional default."""
    if default:
        hint = f" [{default}]"
    else:
        hint = ""
    val = input(f"  {label}{hint}: ").strip()
    return val or default


def confirm(label: str, default: bool = True) -> bool:
    """Ask a yes/no question."""
    hint = "Y/n" if default else "y/N"
    val = input(f"  {label} [{hint}]: ").strip().lower()
    if not val:
        return default
    return val.startswith("y")


# ── Provider config I/O ──────────────────────────────────────────────────────


def load_config() -> dict:
    if PROVIDERS_PATH.exists():
        with open(PROVIDERS_PATH) as f:
            return json.load(f)
    return {"providers": []}


def save_config(config: dict) -> None:
    with open(PROVIDERS_PATH, "w") as f:
        json.dump(config, f, indent=2)
        f.write("\n")


def list_providers(config: dict) -> None:
    providers = config.get("providers", [])
    if not providers:
        print(style("  (none configured yet)", "dim"))
        return
    for p in providers:
        ptype = {"openai_compatible": "OpenAI-compat", "anthropic": "Anthropic"}.get(
            p["type"], p["type"]
        )
        bu: str = p.get("base_url") or "-"
        print(f"  {style(p['name'], 'bold'):20s} {style(ptype, 'cyan'):16s} {bu}")


# ── Test call ────────────────────────────────────────────────────────────────


def test_provider(entry: dict) -> bool:
    """Make a lightweight test call to verify the provider works."""
    import http.client
    import urllib.parse

    api_key = os.environ.get(entry["api_key_env"])
    if not api_key:
        print(
            style(
                f"  [!] {entry['api_key_env']} is not set - skipping test",
                "red",
            )
        )
        return False

    base_url = entry.get("base_url") or "https://api.openai.com"
    model = entry.get("default_model") or "gpt-4o-mini"

    print(f"\n  Testing with model '{model}' at {base_url} ...")

    # Strip trailing slashes
    base_url = base_url.rstrip("/")
    # Build the endpoint URL
    if base_url.endswith("/v1"):
        endpoint = f"{base_url}/chat/completions"
    elif "/v1/" in base_url:
        endpoint = f"{base_url}/chat/completions" if not base_url.endswith("chat/completions") else base_url
    else:
        endpoint = f"{base_url}/v1/chat/completions"

    parsed = urllib.parse.urlparse(base_url)
    host = parsed.netloc or parsed.path
    scheme = parsed.scheme or "https"

    payload = json.dumps({
        "model": model,
        "messages": [{"role": "user", "content": "Say 'ok' and nothing else."}],
        "max_tokens": 10,
    })

    try:
        conn = http.client.HTTPSConnection(host, timeout=15)
        conn.request(
            "POST",
            urllib.parse.urlparse(endpoint).path or "/v1/chat/completions",
            body=payload,
            headers={
                "Content-Type": "application/json",
                "Authorization": f"Bearer {api_key}",
            },
        )
        resp = conn.getresponse()
        body = resp.read().decode()

        if resp.status == 200:
            data = json.loads(body)
            content = data["choices"][0]["message"]["content"]
            print(style(f"  [OK] Response: {content.strip()!r}", "green"))
            return True
        else:
            print(style(f"  [FAIL] HTTP {resp.status}: {body[:200]}", "red"))
            return False
    except Exception as e:
        print(style(f"  [FAIL] Connection failed: {e}", "red"))
        return False


# ── Main flow ────────────────────────────────────────────────────────────────


def print_header(text: str) -> None:
    width = 56
    print()
    print(style("-" * width, "dim"))
    print(style(f"  {text}", "bold"))
    print(style("-" * width, "dim"))


def main() -> None:
    config = load_config()

    print()
    print(style("+" + "-" * 56 + "+", "bold"))
    print(style("|        Wireframe AI - Add LLM Provider                 |", "bold"))
    print(style("+" + "-" * 56 + "+", "bold"))

    # ── Show existing providers ──
    print_header("Current providers")
    list_providers(config)

    # ── Gather details ──
    print_header("New provider details")

    name = prompt("Provider identifier", default="").strip().lower()
    while not name:
        name = prompt("Provider identifier (required)", default="").strip().lower()
    while any(p["name"] == name for p in config.get("providers", [])):
        print(style(f"  Provider '{name}' already exists. Choose another name.", "red"))
        name = prompt("Provider identifier", default="").strip().lower()

    display = prompt("Display name", default=name.title())

    print()
    print("  API type:")
    print(f"    {style('1', 'bold')}  OpenAI-compatible  (OpenAI SDK - works with DeepSeek, OpenCode Go, etc.)")
    print(f"    {style('2', 'bold')}  Anthropic           (Anthropic SDK - Claude models)")
    type_choice = prompt("Choice", default="1").strip()

    if type_choice == "2":
        ptype = "anthropic"
        base_url = None
    else:
        ptype = "openai_compatible"
        base_url = prompt("Base URL", default="https://api.deepseek.com").strip()
        if not base_url:
            base_url = None

    default_api_key_env = f"{name.upper().replace('-', '_')}_API_KEY"
    api_key_env = prompt("API key env var", default=default_api_key_env).strip()
    if not api_key_env:
        api_key_env = default_api_key_env

    default_model = prompt("Default model name", default=f"{name}-model").strip()
    while not default_model:
        default_model = prompt("Default model name (required)", default="").strip()

    # ── Review ──
    print_header("Summary")

    lines = [
        ("Identifier", name),
        ("Display name", display),
        ("API type", {"openai_compatible": "OpenAI-compatible", "anthropic": "Anthropic"}.get(ptype, ptype)),
    ]
    if ptype == "openai_compatible":
        lines.append(("Base URL", base_url or "(default OpenAI)"))
    lines += [
        ("API key env var", api_key_env),
        ("Default model", default_model),
    ]
    for k, v in lines:
        print(f"  {style(k + ':', 'bold'):20s} {v}")

    if not confirm("\nAdd this provider?"):
        print(style("  Cancelled.", "dim"))
        return

    # ── Save ──
    entry = {
        "name": name,
        "display_name": display,
        "type": ptype,
        "api_key_env": api_key_env,
        "default_model": default_model,
    }
    if ptype == "openai_compatible":
        entry["base_url"] = base_url

    config.setdefault("providers", []).append(entry)
    save_config(config)

    env_hint = style(f"${api_key_env}", "cyan")
    print(style(f"\n  [OK] Provider '{name}' added to providers.json", "green"))
    print(f"  Set {env_hint} and restart the adapter to use it.")

    # ── Optional test ──
    if ptype == "openai_compatible" and confirm("Test the connection now?", default=False):
        test_provider(entry)

    # ── Next steps ──
    print_header("Next steps")
    print(f"  1. Export your API key:  export {api_key_env}=<your-key>")
    print(f"  2. Restart the adapter")
    print(f"  3. Set model_config in your AgentJob to:")
    print(f'     {{"provider": "{name}", "model_name": "<one-of-your-models>"}}')
    if ptype == "openai_compatible":
        print(f"\n     The full list of available models is usually at:")
        if base_url:
            models_url = base_url.rstrip("/")
            if not models_url.endswith("/v1"):
                models_url += "/v1"
            print(f"       curl {models_url}/models")
    print()


if __name__ == "__main__":
    main()

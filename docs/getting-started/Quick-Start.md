# Wireframe AI - Quick Start

## Running the TUI

The Wireframe AI minimal TUI can be run from the tools directory:

### Build and Run
```bash
cd tools/tui-minimal
cargo build --release
./target/release/tui-minimal
```

Or run directly in development mode:
```bash
cd tools/tui-minimal
cargo run
```

### Configuration

Create a `tui-config.toml` file in the working directory:

```toml
nats_url = "nats://localhost:4222"
tick_rate_ms = 250

[[providers]]
name = "openai"
api_key_env = "OPENAI_API_KEY"
model = "gpt-4o"

current_provider = "openai"
```

### NATS Integration

The TUI integrates with the Wireframe-AI agent system via NATS.

**Start NATS server:**
```bash
nats-server
```

**Set API key:**
```bash
export OPENAI_API_KEY="your-key-here"
```

**Run TUI:**
```bash
cd tools/tui-minimal
cargo run
```

## Features

The minimal TUI provides essential features for interacting with Wireframe-AI:

- **Chat Interface**: Send and receive messages
- **NATS Integration**: Connect to Wireframe-AI agent system
- **Provider Management**: Configure LLM providers via TOML
- **Minimal UI**: Clean, simple interface with no lag

## Keyboard Shortcuts

- `Ctrl+C` or `Ctrl+Q` - Quit
- `Enter` - Submit message
- `Backspace` - Delete character
- Character keys - Type input

## Agent Integration

When NATS is connected, the TUI will:
1. Submit user messages to the agent system via `task.submitted`
2. Listen for responses on `task.complete`
3. Display agent responses as assistant messages

## Troubleshooting

**NATS connection failed:**
- Ensure NATS server is running: `nats-server`
- Check the NATS URL in `tui-config.toml`
- Verify NATS is listening on the expected port (default: 4222)

**Provider configuration:**
- Ensure API key environment variable is set
- Check provider configuration in `tui-config.toml`
- Verify `current_provider` matches a configured provider name

## Deprecated TUI

The original TUI (`tools/tui-chat`) has been deprecated due to performance issues. Use the minimal TUI (`tools/tui-minimal`) instead. See `tools/tui-chat/README.md` for deprecation details.

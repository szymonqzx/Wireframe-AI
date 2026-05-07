# Wireframe-AI Minimal TUI

A simple, fast terminal UI for Wireframe-AI focused on performance and minimalism.

## Features

- **Simple Interface**: Clean chat interface with minimal overhead
- **NATS Integration**: Connect to Wireframe-AI agent system via NATS
- **Provider Configuration**: Configure LLM providers via TOML file
- **Fast Performance**: Optimized for low latency and minimal resource usage

## Configuration

Create a `tui-config.toml` file in your working directory:

```toml
nats_url = "nats://localhost:4222"
tick_rate_ms = 250

[[providers]]
name = "openai"
api_key_env = "OPENAI_API_KEY"
model = "gpt-4o"

current_provider = "openai"
```

### Configuration Options

- `nats_url`: NATS server URL (default: `nats://localhost:4222`)
- `tick_rate_ms`: UI refresh rate in milliseconds (default: `250`)
- `providers`: List of LLM provider configurations
  - `name`: Provider identifier
  - `api_key_env`: Environment variable containing API key
  - `model`: Model name to use
- `current_provider`: Name of the currently active provider

## Building and Running

### Build

```bash
cd tools/tui-minimal
cargo build --release
```

### Run

```bash
# From project root
cargo run --release --bin tui-minimal

# Or from tui-minimal directory
cd tools/tui-minimal
cargo run --release
```

## Keyboard Shortcuts

- `Ctrl+C` or `Ctrl+Q` - Quit
- `Enter` - Submit message
- `Backspace` - Delete character
- Character keys - Type input

## Architecture

The minimal TUI is organized into focused crates:

- **tui-config**: Configuration management (TOML loading/parsing)
- **tui-nats**: NATS integration (connection, message publishing/subscribing)
- **tui-input**: Input handling (keyboard events, line editing)
- **tui-render**: UI rendering (terminal display, layout)
- **tui-core**: Application logic (coordinates between components)
- **tui-main**: Binary entry point

## Performance

The minimal TUI is designed for:

- **Fast startup**: Minimal dependencies and initialization
- **Low latency**: 250ms tick rate for responsive UI
- **Small binary**: Focused crates reduce compilation overhead
- **Efficient rendering**: Only redraw when necessary

## NATS Integration

When connected to NATS, the TUI:

1. Submits user messages as `task.submitted` messages
2. Tracks pending tasks
3. Displays connection status in the input area

If NATS is not available, the TUI will still run in disconnected mode.

## Troubleshooting

**NATS connection failed:**
- Ensure NATS server is running: `nats-server`
- Check the NATS URL in `tui-config.toml`
- Verify NATS is listening on the expected port (default: 4222)

**Provider configuration:**
- Ensure API key environment variable is set
- Check provider configuration in `tui-config.toml`
- Verify `current_provider` matches a configured provider name

## Migration from wireframe-tui

The original `wireframe-tui` has been deprecated due to performance issues. To migrate:

1. Create `tui-config.toml` with your provider settings
2. Set the `OPENAI_API_KEY` (or other provider) environment variable
3. Run `cargo run --release --bin tui-minimal`

The minimal TUI provides essential features without the complexity and overhead of the original implementation.

## License

Same as Wireframe-AI project.

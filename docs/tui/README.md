# Wireframe-AI TUI Documentation

This section contains all documentation related to the Wireframe-AI Terminal User Interface (TUI).

## Overview

The Wireframe-AI TUI provides a terminal-based interface for interacting with the agent system. It features:
- Agent chat interface
- Real-time message streaming
- Plugin system for extensibility
- Cross-platform compatibility
- Minimal dependencies

## Documentation Structure

### Implementation Plans
Detailed implementation plans for TUI enhancements:
- [TUI Implementation Plan](implementation-plans/TUI-Implementation-Plan.md) - Prioritized opportunities
- [TUI Enhancement Plan](implementation-plans/TUI-Enhancement-Plan.md) - Reliability, architecture, UI/UX analysis
- [TUI Modular Architecture Plan](implementation-plans/tui-modular-architecture-plan.md) - jcode-inspired modular architecture
- [TUI Plugin System Plan](implementation-plans/TUI-Plugin-System-Plan.md) - Detailed plugin system implementation

### Best Practices
Best practices for TUI development:
- [TUI Agent Chat Best Practices](best-practices/TUI-Agent-Chat-Best-Practices.md)

### Optimization
Performance optimization guides:
- [Rust TUI Optimizations](optimization/rust-tui-optimizations.md)

## Current Architecture

The TUI is currently implemented as a monolithic crate (`wireframe-tui`) with plans to migrate to a modular architecture inspired by jcode. See the [TUI Modular Architecture Plan](implementation-plans/tui-modular-architecture-plan.md) for details.

## Key Features

### Agent Chat
- Real-time message streaming
- Input queueing during AI turns
- Message history
- Multi-turn conversations

### Plugin System
- Extensible widget system
- Custom layouts
- Theme support
- Command palette

### Performance
- Efficient rendering pipeline
- Minimal dependencies
- Cross-platform compatibility

## Development Status

### Completed
- Basic agent chat interface
- NATS integration
- Provider configuration
- Message streaming

### In Progress
- Plugin system implementation
- Modular architecture migration
- Performance optimizations

### Planned
- Advanced widget library
- Custom layout engine
- Enhanced theme system
- Command palette

## Getting Started

To run the TUI:
```bash
cargo run --release --bin tui-minimal
```

See the [Quick Start](../getting-started/Quick-Start.md) for more information.

## Contributing

When contributing to the TUI:
1. Follow the modular architecture plan
2. Implement plugin traits for new features
3. Add tests for new functionality
4. Update this documentation
5. Test cross-platform compatibility

## Related Documentation

- [Project Architecture](../getting-started/Project-Architecture.md) - Overall system architecture
- [Provider System](../getting-started/Provider-System.md) - LLM provider integration
- [Plugin Development Guide](../guides/Plugin-Development-Guide.md) - Plugin development patterns

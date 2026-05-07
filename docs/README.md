# Wireframe-AI Documentation

Welcome to the Wireframe-AI documentation. This is a modular, event-driven agentic system built with Rust, using NATS for inter-module communication and a unified Provider system for LLM integrations.

## Quick Start

New to Wireframe-AI? Start here:

- [Quick Start](getting-started/Quick-Start.md) - Get up and running in minutes
- [Project Core](getting-started/Project-Core.md) - System overview and key concepts
- [Project Architecture](getting-started/Project-Architecture.md) - Architecture details and design principles
- [Provider System](getting-started/Provider-System.md) - LLM provider integration

## Documentation Structure

### Getting Started
Introduction and core concepts for new users:
- [Quick Start](getting-started/Quick-Start.md)
- [Project Core](getting-started/Project-Core.md)
- [Project Architecture](getting-started/Project-Architecture.md)
- [Provider System](getting-started/Provider-System.md)

### Reference
API references, schemas, and specifications:
- [API Reference](reference/API-Reference.md)
- [Schemas](reference/schemas/) - Message envelope schemas
- [Topics](reference/topics/TOPICS.md) - NATS topic definitions

### Guides
How-to guides and tutorials:
- [Plugin Development Guide](guides/Plugin-Development-Guide.md)
- [Hello World Plugin Tutorial](guides/Hello-World-Plugin-Tutorial.md)
- [Plugin Architecture Migration Guide](guides/Plugin-Architecture-Migration-Guide.md)
- [Configuration Examples](guides/Configuration-Examples.md)
- [SDK Quick Start](guides/SDK-Quick-Start.md)

### Modules
Module-specific documentation and architecture:
- [Sandbox Plugin Architecture](modules/sandbox-plugin-architecture.md) - Sandbox-core plugin system design

### SDK
SDK documentation and usage:
- [SDK README](sdk/README.md) - agentic-sdk quick start and usage
- [Plugin System](sdk/plugin-system.md) - Universal plugin system documentation

### TUI Documentation
Terminal User Interface documentation:
- [TUI Documentation Index](tui/README.md)
- [Implementation Plans](tui/implementation-plans/)
- [Best Practices](tui/best-practices/)
- [Optimization](tui/optimization/)

### Architecture
Architectural Decision Records (ADRs):
- [Selfdev Mode ADR](architecture/adr-020-selfdev-mode.md)

### Integrations
External service integrations:
- [GitHub Integration](integrations/GitHub-Integration.md)
- [Slack Integration](integrations/Slack-Integration.md)

### Operations
Deployment and operations:
- [Deployment](operations/Deployment.md)

### Security
Security documentation:
- [Security Hardening](security/Security-Hardening.md)

### Planning
Temporary implementation plans and migration notes:
- [Superpowers Migration](planning/superpowers-migration/) - Plugin architecture migration phases
- [Phase 1 Implementation Notes](planning/Phase-1-Implementation-Notes.md)

### Project
Project-level documentation:
- [Roadmap](project/ROADMAP.md)
- [2-Year Autonomous Roadmap](project/2-Year-Autonomous-Roadmap.md)
- [Universal Modularization Plan](project/Universal-Modularization-Plan.md)
- [Best Practices](project/Best-Practices.md)
- [Performance Optimization Guide](project/Performance-Optimization-Guide.md)
- [Production Deployment Guide](project/Production-Deployment-Guide.md)
- [Security](project/SECURITY.md)

## Key Concepts

### Modular Architecture
Wireframe-AI uses a modular architecture where each component is a separate crate:
- **Kernel** - Module orchestration and lifecycle
- **Modules** - Core functionality (context, orchestrator, sandbox)
- **SDK** - Rust and Python SDKs for external integrations
- **Providers** - LLM provider implementations

### Event-Driven Communication
All modules communicate through NATS using a uniform JSON envelope:
- Message envelopes with versioning
- Topic naming convention: `namespace.noun.verb`
- Module lifecycle announcements
- Correlation IDs for request/response patterns

### Provider System
Unified Provider trait for LLM integrations:
- Multiple providers (OpenAI, Anthropic, etc.)
- Capability negotiation
- Streaming support
- Session management

### Plugin Architecture
Core modules use a plugin architecture:
- Plugin traits for extensibility
- Configuration-based plugin loading
- Type-safe plugin registry
- Hot-reload support

## Development Resources

- [AGENTS.md](../AGENTS.md) - Devin CLI orchestration and behavioral standards
- [TECHNICAL-PATTERNS.md](../.devin/TECHNICAL-PATTERNS.md) - Wireframe-AI technical patterns
- [README](../README.md) - Project README with quick start guide

## Contributing

When contributing to Wireframe-AI documentation:
1. Follow the existing documentation structure
2. Use clear, concise language
3. Include code examples where appropriate
4. Update cross-references when moving files
5. Test all links before submitting

## Documentation Reorganization

This documentation was recently reorganized. See [DOCUMENTATION-REORGANIZATION-PLAN.md](DOCUMENTATION-REORGANIZATION-PLAN.md) for details on the changes made.

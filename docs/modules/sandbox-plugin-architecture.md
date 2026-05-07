# Sandbox-Core Plugin Architecture

## Overview

The sandbox-core uses a minimal but extensible plugin system to add tools, security policies, and resource limits. The design prioritizes:

1. **Minimal core**: Only essential functionality built-in
2. **Plugin extensibility**: All major features via plugins
3. **Type safety**: Strong typing with trait-based plugins
4. **Security-first**: All operations go through security layer
5. **Resource limits**: Built-in resource management

## Core Architecture

```
┌─────────────────────────────────────────┐
│         MCP Server (stdio)              │
│  - JSON-RPC 2.0 protocol               │
│  - Tool discovery & execution          │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│         SandboxCore                     │
│  - Plugin registry                      │
│  - Tool execution orchestration         │
│  - Security enforcement                 │
│  - Resource limit enforcement           │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│         Plugin System                   │
│  ┌─────────────┐  ┌──────────────┐     │
│  │   Tools     │  │  Security    │     │
│  │  (file,     │  │  (policies)  │     │
│  │   shell,    │  │              │     │
│  │   http)     │  │              │     │
│  └─────────────┘  └──────────────┘     │
│  ┌─────────────┐                        │
│  │  Resources  │                        │
│  │  (limits)   │                        │
│  └─────────────┘                        │
└─────────────────────────────────────────┘
```

## Plugin Types

### 1. Tool Plugins

**Purpose**: Implement specific tools that can be called via MCP

**Trait**: `agentic_sdk::plugins::sandbox::Tool`

**Methods**:
- `tool_name() -> &'static str`: Unique tool identifier
- `execute(params: Value, context: &SandboxContext) -> Result<Value, ToolError>`

**Built-in Tools**:
- `file_read`: Read files from sandbox
- `file_write`: Write files to sandbox
- `shell_exec`: Execute shell commands (allowlisted)

**Plugin Examples**:
- `tool-http`: HTTP requests
- `tool-database`: Database operations
- `tool-git`: Git operations

**Registration**:
```rust
sandbox.register_tool(Arc::new(HttpTool::new())).await;
```

### 2. Security Policy Plugins

**Purpose**: Enforce security rules on tool execution

**Trait**: `agentic_sdk::plugins::sandbox::SecurityPolicy`

**Methods**:
- `check_tool_access(tool_name: &str, params: &Value, context: &SandboxContext) -> Result<(), SecurityError>`

**Built-in Policies**:
- Path traversal prevention
- Command allowlisting
- Resource access control

**Plugin Examples**:
- `policy-whitelist`: Whitelist-based access control
- `policy-custom`: Custom rule-based policies

**Registration**:
```rust
sandbox.set_security(Arc::new(WhitelistPolicy::new(config))).await;
```

### 3. Resource Limiter Plugins

**Purpose**: Enforce resource limits on tool execution

**Trait**: `agentic_sdk::plugins::sandbox::ResourceLimiter`

**Methods**:
- `check_resources(tool_name: &str, context: &SandboxContext) -> Result<(), ResourceError>`
- `record_usage(tool_name: &str, usage: ResourceUsage)`

**Built-in Limits**:
- Execution time limits
- Memory usage limits
- File size limits

**Plugin Examples**:
- `limits-unix`: Unix-specific resource limits (CPU, memory, file descriptors)

**Registration**:
```rust
sandbox.set_resource_limiter(Arc::new(UnixLimits::new(config))).await;
```

## Plugin Discovery and Loading

### Configuration-Based Loading

Plugins are loaded from a configuration file (YAML):

```yaml
sandbox:
  root: "/tmp/wireframe-sandbox"
  plugins:
    tools:
      - name: "tool-http"
        path: "./plugins/sandbox/tools/tool-http"
        config:
          timeout: 30
          max_redirects: 5
    security:
      - name: "policy-whitelist"
        path: "./plugins/sandbox/security/policy-whitelist"
        config:
          allowed_paths: ["/tmp/sandbox"]
    resources:
      - name: "limits-unix"
        path: "./plugins/sandbox/resources/limits-unix"
        config:
          max_memory_mb: 512
          max_cpu_time: 60
```

### Dynamic Loading

Plugins can be loaded dynamically at runtime:

```rust
// Load plugin from shared library
let plugin = unsafe { load_plugin("./plugins/tool-http.so")? };
sandbox.register_tool(plugin).await;
```

## Tool Execution Flow

```
1. MCP Request Received
   ↓
2. Deserialize Tool Call
   ↓
3. Security Policy Check
   ├─→ Policy: check_tool_access()
   └─→ If denied, return error
   ↓
4. Resource Limit Check
   ├─→ Limiter: check_resources()
   └─→ If exceeded, return error
   ↓
5. Tool Execution
   ├─→ Tool: execute(params, context)
   └─→ Return result or error
   ↓
6. Record Resource Usage
   ├─→ Limiter: record_usage()
   └─→ Update internal state
   ↓
7. Return MCP Response
```

## Security Model

### Defense in Depth

1. **Path Isolation**: All operations confined to sandbox root
2. **Command Allowlisting**: Only approved shell commands
3. **Resource Limits**: CPU, memory, and time constraints
4. **Policy Layer**: Pluggable security policies
5. **Audit Logging**: All tool executions logged

### Default Security Policies

- **Path Traversal Prevention**: Reject paths with `..` or absolute paths
- **Command Allowlisting**: Only safe commands allowed (ls, pwd, echo, cat, grep, find, head, tail)
- **File Size Limits**: Maximum file size for read/write operations
- **Execution Time Limits**: Maximum time for tool execution

## Extensibility Points

### Adding New Tools

1. Implement the `Tool` trait
2. Register with `sandbox.register_tool()`
3. Tool automatically available via MCP

### Adding Security Policies

1. Implement the `SecurityPolicy` trait
2. Register with `sandbox.set_security()`
3. All tool executions go through policy checks

### Adding Resource Limits

1. Implement the `ResourceLimiter` trait
2. Register with `sandbox.set_resource_limiter()`
3. Resource checks applied before tool execution

## Minimal Working Sandbox

The current implementation provides:

### Built-in Tools
- `file_read`: Read files with path validation
- `file_write`: Write files with path validation
- `shell_exec`: Execute allowlisted shell commands

### Built-in Security
- Path confinement to sandbox root
- Command allowlisting
- Basic error handling

### Built-in Resource Limits
- Execution timeout (via tokio)
- Basic error handling

## Future Enhancements

### Plugin System Improvements
- [ ] Hot-reloading of plugins
- [ ] Plugin dependency management
- [ ] Plugin versioning
- [ ] Plugin marketplace

### Security Enhancements
- [ ] Network isolation
- [ ] Process sandboxing (namespaces, cgroups)
- [ ] File system quotas
- [ ] Capability-based security

### Resource Management
- [ ] Per-tool resource quotas
- [ ] Resource usage statistics
- [ ] Dynamic limit adjustment
- [ ] Resource pooling

### Tool Enhancements
- [ ] Streaming tool outputs
- [ ] Long-running tool support
- [ ] Tool cancellation
- [ ] Tool composition

## Configuration Examples

### Minimal Configuration
```yaml
sandbox:
  root: "/tmp/wireframe-sandbox"
  plugins: []  # Use built-in tools only
```

### Full Configuration
```yaml
sandbox:
  root: "/tmp/wireframe-sandbox"
  plugins:
    tools:
      - tool-http
      - tool-git
    security:
      - policy-whitelist
    resources:
      - limits-unix
```

### Custom Tool Plugin
```yaml
sandbox:
  root: "/tmp/wireframe-sandbox"
  plugins:
    tools:
      - name: "my-custom-tool"
        path: "./plugins/custom/my-tool"
        config:
          api_key: "${MY_API_KEY}"
          timeout: 60
```

## Migration Path

### From Current to Full Plugin System

1. **Phase 1**: Current minimal implementation (built-in tools)
2. **Phase 2**: Add plugin loading from configuration
3. **Phase 3**: Implement security policy plugins
4. **Phase 4**: Implement resource limiter plugins
5. **Phase 5**: Add dynamic plugin loading
6. **Phase 6**: Add plugin marketplace/hot-reloading

## Testing Strategy

### Unit Tests
- Test each tool independently
- Test security policies
- Test resource limiters

### Integration Tests
- Test MCP protocol compliance
- Test plugin loading
- Test tool execution flow

### Security Tests
- Test path traversal attempts
- Test command injection attempts
- Test resource limit bypass attempts

## Performance Considerations

- Plugin loading should be fast (lazy loading where possible)
- Tool execution should be efficient (async I/O)
- Security checks should be minimal overhead
- Resource tracking should be low overhead

## Compatibility

- **MCP Protocol**: Version 2024-11-05
- **Rust**: 1.75+
- **Platform**: Linux, macOS, Windows (with limitations)
- **Tokio**: 1.x

## Documentation

- Plugin development guide
- Security policy writing guide
- Configuration reference
- API documentation for traits
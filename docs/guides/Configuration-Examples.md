# Wireframe-AI Configuration Examples

This guide provides configuration examples for all Wireframe-AI modules and plugins.

## Configuration Structure

Wireframe-AI uses YAML configuration files located in the `configs/` directory. Each module has its own configuration file:

- `context-default.yaml` - Context module configuration
- `orchestrator-default.yaml` - Orchestrator module configuration
- `sandbox-default.yaml` - Sandbox module configuration
- `interface-default.yaml` - Interface module configuration

## Context Module Configuration

The Context module manages session storage, memory retrieval, and context enrichment.

### Default Configuration

```yaml
modules:
  context:
    enabled: true
    plugins:
      storage:
        plugin_id: "storage-sqlite"
        config:
          db_path: "./wireframe_ai.db"
      memory:
        plugin_id: "memory-fts5"
        config:
          db_path: "./wireframe_ai.db"
      enrichment_pipeline:
        - plugin_id: "enrichment-env"
          order: 1
          config: {}
```

### Storage Plugins

#### SQLite Storage

```yaml
storage:
  plugin_id: "storage-sqlite"
  config:
    db_path: "./wireframe_ai.db"
    pool_size: 10
```

**Configuration Options:**
- `db_path`: Path to SQLite database file
- `pool_size`: Connection pool size (default: 10)

#### PostgreSQL Storage

```yaml
storage:
  plugin_id: "storage-postgres"
  config:
    connection_string: "postgresql://user:password@localhost/wireframe"
    pool_size: 20
```

**Configuration Options:**
- `connection_string`: PostgreSQL connection string
- `pool_size`: Connection pool size (default: 20)

### Memory Plugins

#### FTS5 Memory

```yaml
memory:
  plugin_id: "memory-fts5"
  config:
    db_path: "./wireframe_ai.db"
    chunk_size: 512
    max_results: 10
```

**Configuration Options:**
- `db_path`: Path to SQLite database file
- `chunk_size`: Maximum chunk size in tokens (default: 512)
- `max_results`: Maximum search results (default: 10)

#### Vector Memory

```yaml
memory:
  plugin_id: "memory-vector"
  config:
    index_path: "./memory.idx"
    embedding_model: "text-embedding-ada-002"
    similarity_threshold: 0.7
```

**Configuration Options:**
- `index_path`: Path to vector index file
- `embedding_model`: Embedding model to use
- `similarity_threshold`: Minimum similarity score (0.0-1.0)

### Enrichment Plugins

#### Environment Enrichment

```yaml
enrichment_pipeline:
  - plugin_id: "enrichment-env"
    order: 1
    config:
      allowed_vars:
        - HOME
        - PATH
        - USER
```

**Configuration Options:**
- `allowed_vars`: List of environment variables to include

#### File Context Enrichment

```yaml
enrichment_pipeline:
  - plugin_id: "enrichment-file"
    order: 2
    config:
      max_file_size: 10485760  # 10MB
      allowed_extensions:
        - .rs
        - .py
        - .js
        - .md
```

**Configuration Options:**
- `max_file_size`: Maximum file size in bytes
- `allowed_extensions`: List of allowed file extensions

#### Memory Retrieval Enrichment

```yaml
enrichment_pipeline:
  - plugin_id: "enrichment-memory"
    order: 3
    config:
      max_chunks: 5
      min_relevance: 0.5
```

**Configuration Options:**
- `max_chunks`: Maximum memory chunks to retrieve
- `min_relevance`: Minimum relevance score (0.0-1.0)

### Pipeline Ordering

Enrichment plugins are executed in order specified by the `order` field:

```yaml
enrichment_pipeline:
  - plugin_id: "enrichment-env"
    order: 1  # First
    config: {}
  - plugin_id: "enrichment-memory"
    order: 2  # Second
    config: {}
  - plugin_id: "enrichment-file"
    order: 3  # Third
    config: {}
```

## Orchestrator Module Configuration

The Orchestrator module manages task planning, execution, and result synthesis.

### Default Configuration

```yaml
modules:
  orchestrator:
    enabled: true
    plugins:
      planner:
        plugin_id: "planner-linear"
        config:
          concurrency: 3
      
      execution:
        plugin_id: "execution-parallel"
        config:
          result_timeout_secs: 600
      
      synthesizer:
        plugin_id: "synthesizer-merge"
        config: {}
```

### Planner Plugins

#### Linear Planner

```yaml
planner:
  plugin_id: "planner-linear"
  config:
    concurrency: 3
    max_subtasks: 10
```

**Configuration Options:**
- `concurrency`: Number of parallel subtasks (default: 3)
- `max_subtasks`: Maximum subtasks to generate (default: 10)

#### Hierarchical Planner

```yaml
planner:
  plugin_id: "planner-hierarchical"
  config:
    max_depth: 5
    branching_factor: 3
```

**Configuration Options:**
- `max_depth`: Maximum decomposition depth (default: 5)
- `branching_factor`: Number of branches per level (default: 3)

#### LLM Planner

```yaml
planner:
  plugin_id: "planner-llm"
  config:
    model: "gpt-4"
    temperature: 0.3
    max_tokens: 1000
```

**Configuration Options:**
- `model`: LLM model to use for planning
- `temperature`: Sampling temperature (0.0-1.0)
- `max_tokens`: Maximum tokens in plan

### Execution Plugins

#### Parallel Execution

```yaml
execution:
  plugin_id: "execution-parallel"
  config:
    result_timeout_secs: 600
    max_concurrent_jobs: 10
```

**Configuration Options:**
- `result_timeout_secs`: Timeout for result collection (default: 600)
- `max_concurrent_jobs`: Maximum concurrent jobs (default: 10)

#### Sequential Execution

```yaml
execution:
  plugin_id: "execution-sequential"
  config:
    timeout_seconds: 600
    retry_attempts: 3
```

**Configuration Options:**
- `timeout_seconds`: Timeout per job (default: 600)
- `retry_attempts`: Number of retry attempts (default: 3)

#### Adaptive Execution

```yaml
execution:
  plugin_id: "execution-adaptive"
  config:
    initial_concurrency: 2
    max_concurrency: 10
    scale_up_threshold: 0.8
    scale_down_threshold: 0.3
```

**Configuration Options:**
- `initial_concurrency`: Starting concurrency (default: 2)
- `max_concurrency`: Maximum concurrency (default: 10)
- `scale_up_threshold`: CPU usage threshold to scale up (default: 0.8)
- `scale_down_threshold`: CPU usage threshold to scale down (default: 0.3)

### Synthesizer Plugins

#### Merge Synthesizer

```yaml
synthesizer:
  plugin_id: "synthesizer-merge"
  config:
    merge_strategy: "concatenate"
    separator: "\n\n"
```

**Configuration Options:**
- `merge_strategy`: Strategy for merging results (concatenate, weighted)
- `separator`: Separator between merged results

#### LLM Synthesizer

```yaml
synthesizer:
  plugin_id: "synthesizer-llm"
  config:
    model: "gpt-4"
    temperature: 0.5
    max_tokens: 2000
    system_prompt: "Synthesize the following results into a coherent answer."
```

**Configuration Options:**
- `model`: LLM model to use for synthesis
- `temperature`: Sampling temperature (0.0-1.0)
- `max_tokens`: Maximum tokens in synthesis
- `system_prompt`: System prompt for synthesis

## Sandbox Module Configuration

The Sandbox module manages tool execution, security policies, and resource limits.

### Default Configuration

```yaml
modules:
  sandbox:
    enabled: true
    plugins:
      tools:
        - plugin_id: "tool-shell"
          config:
            timeout_secs: 30
        - plugin_id: "tool-file"
          config:
            max_file_size: 10485760  # 10MB

      security:
        plugin_id: "policy-whitelist"
        config:
          allow_network: false
          filesystem_policy: "sandbox_writable"

      resources:
        plugin_id: "limits-unix"
        config:
          cpu_limit_secs: 300
          memory_limit_mb: 1024
          timeout_secs: 30
```

### Tool Plugins

#### Shell Tool

```yaml
tools:
  - plugin_id: "tool-shell"
    config:
      timeout_secs: 30
      allowed_commands:
        - ls
        - cat
        - grep
        - find
      working_dir: "/tmp/wireframe"
```

**Configuration Options:**
- `timeout_secs`: Command timeout in seconds (default: 30)
- `allowed_commands`: Whitelist of allowed commands (empty = all)
- `working_dir`: Working directory for commands

#### File Tool

```yaml
tools:
  - plugin_id: "tool-file"
    config:
      max_file_size: 10485760  # 10MB
      allowed_paths:
        - "/tmp/wireframe"
        - "./"
      denied_extensions:
        - .exe
        - .sh
        .bat
```

**Configuration Options:**
- `max_file_size`: Maximum file size in bytes (default: 10MB)
- `allowed_paths`: Whitelist of allowed paths (empty = all)
- `denied_extensions`: Blacklist of file extensions

#### HTTP Tool

```yaml
tools:
  - plugin_id: "tool-http"
    config:
      timeout_seconds: 30
      max_redirects: 5
      user_agent: "Wireframe-AI/1.0"
```

**Configuration Options:**
- `timeout_seconds`: Request timeout in seconds (default: 30)
- `max_redirects`: Maximum redirects (default: 5)
- `user_agent`: User agent string

#### Git Tool

```yaml
tools:
  - plugin_id: "tool-git"
    config:
      timeout_seconds: 60
      allowed_operations:
        - clone
        - pull
        - status
        - diff
      max_repo_size: 104857600  # 100MB
```

**Configuration Options:**
- `timeout_seconds`: Git operation timeout (default: 60)
- `allowed_operations`: Whitelist of allowed operations
- `max_repo_size`: Maximum repository size in bytes

### Security Plugins

#### Whitelist Policy

```yaml
security:
  plugin_id: "policy-whitelist"
  config:
    allow_network: false
    filesystem_policy: "sandbox_writable"
    allowed_domains:
      - "api.openai.com"
      - "cdn.example.com"
```

**Configuration Options:**
- `allow_network`: Allow network access (default: false)
- `filesystem_policy`: Filesystem policy (readonly, sandbox_writable, isolated_vm)
- `allowed_domains`: Whitelist of allowed domains

#### Custom Policy

```yaml
security:
  plugin_id: "policy-custom"
  config:
    allowed_domains:
      - "api.example.com"
      - "cdn.example.com"
    block_network: false
    command_blacklist:
      - rm
      - dd
      - mkfs
    path_whitelist:
      - "/tmp/wireframe"
      - "./"
```

**Configuration Options:**
- `allowed_domains`: Whitelist of allowed domains
- `block_network`: Block all network access
- `command_blacklist`: Blacklist of dangerous commands
- `path_whitelist`: Whitelist of allowed paths

#### Strict Policy

```yaml
security:
  plugin_id: "policy-strict"
  config:
    require_approval: true
    audit_log: true
    log_path: "./security_audit.log"
```

**Configuration Options:**
- `require_approval`: Require approval for all operations
- `audit_log`: Enable audit logging
- `log_path`: Path to audit log file

### Resource Limit Plugins

#### Unix Limits

```yaml
resources:
  plugin_id: "limits-unix"
  config:
    cpu_limit_secs: 300
    memory_limit_mb: 1024
    timeout_secs: 30
```

**Configuration Options:**
- `cpu_limit_secs`: CPU time limit in seconds (default: 300)
- `memory_limit_mb`: Memory limit in MB (default: 1024)
- `timeout_secs`: Overall timeout in seconds (default: 30)

#### Container Limits

```yaml
resources:
  plugin_id: "limits-container"
  config:
    cpu_quota: 100000  # 100ms per second
    memory_limit: 1073741824  # 1GB
    pids_limit: 100
```

**Configuration Options:**
- `cpu_quota`: CPU quota in microseconds per second
- `memory_limit`: Memory limit in bytes
- `pids_limit`: Maximum number of processes

## Interface Module Configuration

The Interface module manages user input and output formatting.

### Default Configuration

```yaml
modules:
  interface:
    enabled: true
    plugins:
      input:
        plugin_id: "input-cli"
        config:
          prompt: "wireframe> "

      output:
        plugin_id: "output-markdown"
        config:
          syntax_highlighting: true
```

### Input Plugins

#### CLI Input

```yaml
input:
  plugin_id: "input-cli"
  config:
    prompt: "wireframe> "
    history_file: "./wireframe_history.txt"
    max_history: 1000
```

**Configuration Options:**
- `prompt`: Command prompt (default: "wireframe> ")
- `history_file`: Path to command history file
- `max_history`: Maximum history entries (default: 1000)

#### Web Input

```yaml
input:
  plugin_id: "input-web"
  config:
    bind_address: "0.0.0.0"
    port: 8080
    cors_origins:
      - "http://localhost:3000"
    max_request_size: 1048576  # 1MB
```

**Configuration Options:**
- `bind_address`: Bind address (default: "0.0.0.0")
- `port`: Port number (default: 8080)
- `cors_origins`: Allowed CORS origins
- `max_request_size`: Maximum request size in bytes

#### API Input

```yaml
input:
  plugin_id: "input-api"
  config:
    api_key_required: true
    rate_limit: 100  # requests per minute
    max_concurrent: 10
```

**Configuration Options:**
- `api_key_required`: Require API key (default: true)
- `rate_limit`: Rate limit in requests per minute (default: 100)
- `max_concurrent`: Maximum concurrent requests (default: 10)

### Output Plugins

#### Markdown Output

```yaml
output:
  plugin_id: "output-markdown"
  config:
    syntax_highlighting: true
    theme: "monokai"
    max_output_size: 10485760  # 10MB
```

**Configuration Options:**
- `syntax_highlighting`: Enable syntax highlighting (default: true)
- `theme`: Syntax highlighting theme (default: "monokai")
- `max_output_size`: Maximum output size in bytes (default: 10MB)

#### JSON Output

```yaml
output:
  plugin_id: "output-json"
  config:
    pretty_print: true
    include_metadata: true
    indent_spaces: 2
```

**Configuration Options:**
- `pretty_print`: Pretty print JSON (default: true)
- `include_metadata`: Include metadata in output (default: true)
- `indent_spaces`: Indentation spaces (default: 2)

#### HTML Output

```yaml
output:
  plugin_id: "output-html"
  config:
    template: "./templates/output.html"
    include_css: true
    custom_css: "./custom.css"
```

**Configuration Options:**
- `template`: Path to HTML template
- `include_css`: Include default CSS (default: true)
- `custom_css`: Path to custom CSS file

### UI Component Plugins

#### Progress Bar

```yaml
ui_components:
  - plugin_id: "ui-progress"
    config:
      width: 50
      style: "blocks"
```

**Configuration Options:**
- `width`: Progress bar width in characters (default: 50)
- `style`: Progress bar style (blocks, line, fancy)

#### Rich Output

```yaml
ui_components:
  - plugin_id: "ui-rich"
    config:
      enable_colors: true
      enable_emoji: true
      max_lines: 1000
```

**Configuration Options:**
- `enable_colors`: Enable colored output (default: true)
- `enable_emoji`: Enable emoji in output (default: true)
- `max_lines`: Maximum output lines (default: 1000)

## Complete System Configuration

For a complete system configuration example, see `examples/configurations/complete-system.yaml`.

## Minimal System Configuration

For a minimal configuration to get started quickly, see `examples/configurations/minimal-system.yaml`.

## NATS Configuration

NATS message bus configuration is typically in a separate file:

```yaml
nats:
  url: "nats://localhost:4222"
  queue_groups:
    context: "context_group"
    orchestrator: "orchestrator_group"
    sandbox: "sandbox_group"
    interface: "interface_group"
  reconnect_interval: 5
  max_reconnects: 10
```

## Provider Configuration

LLM provider configuration for adapters:

```yaml
providers:
  openai:
    api_key: "sk-..."
    base_url: "https://api.openai.com/v1"
    models:
      - name: "gpt-4"
        max_tokens: 8192
      - name: "gpt-3.5-turbo"
        max_tokens: 4096
  
  anthropic:
    api_key: "sk-ant-..."
    base_url: "https://api.anthropic.com"
    models:
      - name: "claude-3-opus"
        max_tokens: 4096
```

## Environment Variables

Configuration can also be specified via environment variables:

```bash
export WIREFRAME_NATS_URL="nats://localhost:4222"
export WIREFRAME_DB_PATH="./wireframe_ai.db"
export WIREFRAME_OPENAI_API_KEY="sk-..."
```

## Configuration Validation

Configuration is validated on startup. Common validation errors:

- **Invalid plugin_id**: Plugin not found in registry
- **Missing required config**: Required configuration option missing
- **Invalid type**: Configuration option has wrong type
- **Out of range**: Numeric value outside valid range

## Best Practices

1. **Use version control**: Commit configuration files to version control
2. **Environment-specific configs**: Use different configs for dev/staging/prod
3. **Secrets management**: Use environment variables or secret managers for sensitive data
4. **Documentation**: Document custom configuration options
5. **Validation**: Test configuration changes in development before production
6. **Backup**: Backup configuration files before making changes
7. **Monitoring**: Monitor configuration changes and their impact

## Next Steps

- See `docs/Plugin-Development-Guide.md` for plugin development
- See `docs/API-Reference.md` for detailed API documentation
- See `examples/configurations/complete-system.yaml` for a complete system configuration

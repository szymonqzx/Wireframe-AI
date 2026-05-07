# Add context based on user prompt content
# Reads JSON from stdin, outputs context information

$inputJson = ""
if ($Console.IsInputRedirected) {
    try {
        $inputJson = [Console]::In.ReadToEnd()
    } catch {
        $inputJson = ""
    }
}

$data = $null
if ($inputJson -ne "" -and $inputJson.Length -gt 0) {
    try {
        $data = $inputJson | ConvertFrom-Json
    } catch {
        $data = $null
    }
}

$prompt = ""
if ($data -ne $null -and $data.PSObject.Properties['prompt']) {
    $prompt = $data.prompt
}

$context = ""

# Detect deployment-related prompts
if ($prompt -match "deploy|deployment|release|production|publish|ship") {
    $context = @"

=== Wireframe-AI Deployment Context ===

Deployment Checklist:
1. Build in release mode: cargo build --release
2. Run full test suite: cargo test
3. Check code quality: cargo clippy
4. Format code: cargo fmt
5. Verify NATS server availability
6. Test module startup/shutdown sequences
7. Validate message schemas
8. Check environment configuration

Environment Information:
- Platform: Windows (MINGW64)
- Project: Wireframe-AI
- Architecture: Rust core + Python adapters
- Message Bus: NATS
- SDK: agentic-sdk (Rust) and agentic-sdk-py (Python)

Deployment Commands:
- Build: cargo build --release
- Test: cargo test
- Run kernel: nats-server && cargo run --bin kernel
- Run specific module: cargo run --bin <module_name>

"@
}
# Detect schema-related prompts
elseif ($prompt -match "schema|message|envelope|topic|contract") {
    $context = @"

=== Wireframe-AI Schema Context ===

Schema Rules:
- Message envelope is immutable (see docs/Project-Core.md)
- Root fields can never be changed, only payload
- Topic naming: namespace.noun.verb or namespace.noun
- All topics lowercase, dot-separated
- Schema version in envelope for contract evolution

Key Schemas:
- Message envelope: message_id, session_id, correlation_id, topic, schema_version, timestamp, payload
- Agent job: job_id, correlation_parent, task, context, available_tool_capabilities, constraints, model_config, metadata
- Module identity: module_id, version, subscribes, publishes

Schema Location: schemas/v1/
Validation: Use /wireframe-workflow skill before schema changes

"@
}
# Detect performance-related prompts
elseif ($prompt -match "performance|optimize|slow|benchmark|speed") {
    $context = @"

=== Wireframe-AI Performance Context ===

Performance Considerations:
- NATS message bus is high-performance, but module design matters
- Use async/await patterns (Tokio for Rust, asyncio for Python)
- Minimize message payload size
- Use queue groups for horizontal scaling
- Profile with /performance-profiling skill
- Benchmark before and after changes

Key Performance Areas:
1. Message throughput (NATS handles ~10M+ msg/sec)
2. Module processing time (depends on AI model)
3. Memory usage (context chunks, session history)
4. Network latency (between modules and NATS)

Tools:
- cargo build --release for optimized builds
- Use performance-optimizer subagent for profiling
- Check NATS server metrics

"@
}
# Detect security-related prompts
elseif ($prompt -match "security|auth|permission|access|vulnerability") {
    $context = @"

=== Wireframe-AI Security Context ===

Security Considerations:
- Filesystem policy: sandbox_writable by default
- Network access: outbound_only by default
- Credential handling: Use required_credentials in tool capabilities
- Module isolation: Each module runs in its own process
- Message validation: Schema validation on envelope parsing

Security Guidelines:
- Never expose secrets in message payloads
- Validate all user inputs before processing
- Use rate limiting on external API calls
- Implement proper error handling (no sensitive data in errors)
- Regular security audits with security-auditor subagent

"@
}
# Default context for general prompts
else {
    $context = @"

=== Wireframe-AI Project Context ===

Project: Wireframe-AI - Modular event-driven agentic system
Architecture: Rust core + Python AI/ML adapters with NATS message bus

Quick Start:
1. Read docs/Project-Core.md for system overview
2. Read docs/Project-Architecture.md for architecture
3. Use /project-routing to find the right agent/skill
4. Use /wireframe-workflow before making changes
5. See .devin/SKILLS.md for complete skills index

Key Patterns:
- State ownership: Context module owns all persistent state
- Message envelope: Never change root fields, only payload
- Topic naming: namespace.noun.verb or namespace.noun, lowercase, dot-separated
- Module identity: Publish to sys.module.online on startup, sys.module.offline on shutdown

Essential Commands:
- cargo build --release  # Build
- cargo test              # Test
- cargo clippy            # Lint
- cargo fmt               # Format
- nats-server && cargo run --bin kernel  # Run

"@
}

# Output context as JSON for the agent to consume
@{
  context = $context
} | ConvertTo-Json -Depth 10

exit 0

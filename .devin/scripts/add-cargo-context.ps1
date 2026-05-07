# Add context when cargo commands are executed
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

$command = ""
if ($data -ne $null -and $data.PSObject.Properties['tool_input']) {
    $toolInput = $data.tool_input
    if ($toolInput.PSObject.Properties['command']) {
        $command = $toolInput.command
    }
}

# Context to inject based on cargo command
$context = @"

=== Wireframe-AI Cargo Context ===

Project: Wireframe-AI - Modular event-driven agentic system
Architecture: Rust core + Python AI/ML adapters with NATS message bus

Key Patterns:
- State ownership: Context module owns all persistent state
- Message envelope: Never change root fields, only payload
- Topic naming: namespace.noun.verb or namespace.noun, lowercase, dot-separated
- Module identity: Publish to sys.module.online on startup, sys.module.offline on shutdown

Essential Commands:
- cargo build --release  # Build in release mode
- cargo test              # Run test suite
- cargo clippy            # Lint with clippy
- cargo fmt               # Format code
- nats-server && cargo run --bin kernel  # Run the system

Current Command: $command

"@

# Output context as JSON for the agent to consume
@{
  context = $context
} | ConvertTo-Json -Depth 10

exit 0

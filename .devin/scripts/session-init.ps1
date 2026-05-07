# Session initialization hook
# Runs when a new Devin session starts

# Read stdin if available, otherwise use defaults
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

$source = "unknown"
if ($data -ne $null -and $data.PSObject.Properties['source']) {
    $source = $data.source
}

# Get current git branch
$branch = ""
try {
    $branch = git rev-parse --abbrev-ref HEAD 2>$null
    if ($LASTEXITCODE -ne 0) {
        $branch = "unknown"
    }
} catch {
    $branch = "unknown"
}

# Check if NATS server is running
$natsRunning = $false
try {
    $natsProcess = Get-Process -Name "nats-server" -ErrorAction SilentlyContinue
    $natsRunning = $natsProcess -ne $null
} catch {
    $natsRunning = $false
}

$context = @"

=== Wireframe-AI Session Initialized ===

Session Source: $source
Current Branch: $branch
NATS Server Status: $(if ($natsRunning) { "Running" } else { "Not running" })

Project: Wireframe-AI
Architecture: Modular event-driven agentic system (Rust core + Python AI/ML adapters)
Message Bus: NATS

Quick Reference:
- Project Core: docs/Project-Core.md
- Architecture: docs/Project-Architecture.md
- Agent Guide: AGENTS.md
- Skills Index: .devin/SKILLS.md

Essential Commands:
- cargo build --release  # Build
- cargo test              # Test
- cargo clippy            # Lint
- cargo fmt               # Format
- nats-server && cargo run --bin kernel  # Run system

Common Workflows:
- New feature: /project-routing → /orchestration-patterns → /architecture → /implementation
- Debug: /orchestration-patterns → /systematic-debugging → /run-rust-tests
- Review: /code-review-checklist → /check-rust-quality → /quality-checklist
- Performance: /orchestration-patterns → /performance-profiling → /rust-pro

"@

# Output context as JSON for the agent to consume
@{
  context = $context
} | ConvertTo-Json -Depth 10

exit 0

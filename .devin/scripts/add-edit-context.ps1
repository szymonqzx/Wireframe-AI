# Add context when editing files
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

$filePath = ""
if ($data -ne $null -and $data.PSObject.Properties['tool_input']) {
    $toolInput = $data.tool_input
    if ($toolInput.PSObject.Properties['file_path']) {
        $filePath = $toolInput.file_path
    }
}

# Context to inject based on file being edited
$context = @"

=== Wireframe-AI Edit Context ===

File: $filePath

Project Guidelines:
- Read existing code before editing
- Prefer minimal diffs over rewrites
- Never invent APIs, commands, benchmarks, or pricing
- Cite files and commands when reporting
- Run the narrowest relevant check before declaring done

Wireframe-AI Specific:
- Check docs/Project-Core.md for immutable foundation decisions
- Check docs/Project-Architecture.md for architecture patterns
- Use AGENTS.md for agent/skill guidance
- Follow topic naming: namespace.noun.verb or namespace.noun
- Never change message envelope root fields, only payload

Before editing:
1. Read the file to understand current implementation
2. Check for existing patterns in similar files
3. Ensure changes follow established conventions
4. Consider impact on message schemas and topic contracts

"@

# Output context as JSON for the agent to consume
@{
  context = $context
} | ConvertTo-Json -Depth 10

exit 0

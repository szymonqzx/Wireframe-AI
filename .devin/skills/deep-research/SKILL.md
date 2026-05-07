---
name: deep-research
description: Autonomous deep web research that self-generates questions, iteratively searches via subagents, and compiles findings into a markdown report. Use for comprehensive research on any topic. ALWAYS uses subagents for research tasks.
allowed-tools:
  - read
  - grep
  - glob
  - edit
  - write
  - exec
  - webfetch
triggers:
  - model
---

# Deep Research

"Autonomous deep web research that self-generates questions, iteratively searches, and compiles findings."

## When to Use
- User requests research on a topic you need to investigate thoroughly
- You need comprehensive information before making technical decisions
- Exploring new technologies, frameworks, or domains
- Gathering competitive intelligence or market research
- Investigating best practices or architectural patterns
- Understanding complex problems that require multiple perspectives

## When NOT to Use
- For simple factual queries (use direct web search instead)
- When the answer is already in the codebase or documentation
- For time-critical tasks requiring immediate action
- When the user wants a quick summary, not deep research
- For questions that require proprietary or internal knowledge only

## Subagent Usage

**MANDATORY:** ALWAYS use subagents for research. Never perform research directly in the main agent.

### Subagent Strategy

**Parallel Investigation Pattern:**

```powershell
# Spawn multiple read-only subagents for parallel research
# Each subagent handles a specific aspect of the research

$researchAspects = @(
    "Technical implementation details",
    "Best practices and patterns",
    "Performance considerations",
    "Security implications",
    "Ecosystem and tooling"
)

foreach ($aspect in $researchAspects) {
    # Launch read-only subagent for this aspect
    # Each subagent returns: findings, sources, confidence level
}
```

### Subagent Profiles

Use appropriate subagent profiles based on research needs:

- **fast-researcher:** Quick read-only research for broad codebase understanding
- **rust-researcher:** Read-only research for Wireframe-AI Rust codebase architecture
- **subagent_explore:** Read-only subagent for codebase exploration and search

### Subagent Coordination

1. **Divide research question** into independent sub-questions
2. **Launch subagents in parallel** using `run_subagent` with `is_background: true`
3. **Collect results** using `read_subagent` with `block: true`
4. **Synthesize findings** into coherent report
5. **Identify gaps** and spawn follow-up subagents if needed

### Subagent Guardrails

- **Read-only access:** Subagents should only use read tools (grep, glob, read)
- **Specific scope:** Each subagent gets a clearly defined research scope
- **Time limits:** Set appropriate timeouts for research tasks
- **Result validation:** Verify subagent findings before including in final report

## Pre-flight Checks

```powershell
# Verify workspace is accessible
if (-not (Test-Path .)) {
    Write-Error "Cannot access workspace directory"
    exit 1
}

# Check if research output already exists to avoid overwriting
$existingResearch = Get-ChildItem -Filter "*_research.md" -ErrorAction SilentlyContinue
if ($existingResearch) {
    Write-Host "Found existing research files:"
    $existingResearch | ForEach-Object { Write-Host "  - $($_.Name)" }
    $overwrite = Read-Host "Overwrite existing research? (Y/N)"
    if ($overwrite -ne "Y") {
        exit 0
    }
}

# Initialize research context
$RESEARCH_TOPIC = $args[0]
if (-not $RESEARCH_TOPIC) {
    Write-Error "Usage: deep-research <topic>"
    exit 1
}
```

## Loop Configuration

```powershell
$MAX_ITERS = 15
$KILLSWITCH = "$env:USERPROFILE\.workflow-stop"
$LOGDIR = ".workflow-logs/$(Get-Date -Format 'yyyyMMddHHmmss')"
New-Item -ItemType Directory -Force -Path $LOGDIR | Out-Null

$researchQuestions = @()
$answeredQuestions = @{}
$followUpQuestions = @()
$researchLog = @()
$confidenceThreshold = 0.8
```

## The Loop

```powershell
for ($i = 1; $i -le $MAX_ITERS; $i++) {
    Write-Host "── Iteration $i/$MAX_ITERS ──"

    if (Test-Path $KILLSWITCH) {
        Write-Host "Killswitch tripped — bailing."
        Remove-Item $KILLSWITCH
        exit 2
    }

    # Generate or use follow-up questions
    if ($i -eq 1 -or $followUpQuestions.Count -gt 0) {
        if ($i -eq 1) {
            Write-Host "Generating initial research questions..."
            $researchQuestions = Generate-InitialQuestions -Topic $RESEARCH_TOPIC
        } else {
            Write-Host "Processing follow-up questions..."
            $researchQuestions = $followUpQuestions
            $followUpQuestions = @()
        }
        Write-Host "Questions to research: $($researchQuestions.Count)"
    }

    # Research each unanswered question via subagents
    $unansweredQuestions = $researchQuestions | Where-Object { -not $answeredQuestions.ContainsKey($_) }
    foreach ($question in $unansweredQuestions) {
        Write-Host "Researching: $question"
        # MANDATORY: Launch subagent for research - never research directly
        $subagentTask = "Research this question thoroughly: $question. Use web search, analyze multiple sources, and provide findings with confidence scores and source citations."
        $subagentId = run_subagent -title "Research: $question" -task $subagentTask -profile "fast-researcher" -is_background $true
        $analysis = read_subagent -agent_id $subagentId -block $true

        if ($analysis.Confidence -ge $confidenceThreshold) {
            $answeredQuestions[$question] = @{
                Answer = $analysis.Answer
                Sources = $analysis.Sources
                Confidence = $analysis.Confidence
                Timestamp = Get-Date
            }
            $newQuestions = Extract-FollowUpQuestions -Analysis $analysis
            if ($newQuestions) { $followUpQuestions += $newQuestions }
        } else {
            $followUpQuestions += Refine-Query -Question $question -Analysis $analysis
        }
    }

    # Check completion
    $allAnswered = ($researchQuestions | Where-Object { -not $answeredQuestions.ContainsKey($_) }).Count -eq 0
    if ($allAnswered -and $followUpQuestions.Count -eq 0) {
        Write-Host "✓ All questions answered"
        break
    }
}

$reportPath = Generate-ResearchReport -Topic $RESEARCH_TOPIC -Answers $answeredQuestions
Write-Host "✓ Report saved to: $reportPath"
```

## Question Generation Logic

Generate questions systematically covering: definition, mechanism, benefits, limitations, alternatives, best practices, current state, and ecosystem.

```powershell
function Generate-InitialQuestions {
    param([string]$Topic)
    $questions = @(
        "What is $Topic and what are its core concepts?",
        "How does $Topic work technically?",
        "What problems does $Topic solve?",
        "What are the main benefits of using $Topic?",
        "What are the limitations or drawbacks of $Topic?",
        "When should $Topic NOT be used?",
        "What are the main alternatives to $Topic?",
        "How does $Topic compare to its alternatives?",
        "What are the best practices for using $Topic?",
        "What are common pitfalls or mistakes when using $Topic?",
        "What is the current state and maturity of $Topic?",
        "What are recent developments or trends in $Topic?",
        "What is the community sentiment around $Topic?",
        "What tools, libraries, or resources exist for $Topic?"
    )
    return $questions
}
```

## Report Structure

```markdown
# Research Report: [Topic]

**Generated:** [Date] | **Iterations:** [N] | **Questions:** [N] | **Confidence:** [X%]

## Executive Summary
[2-3 paragraph summary]

## Core Concepts
[What is X? with sources]

## Technical Overview
[How does X work? with sources]

## Benefits and Use Cases
[Why use X? with sources]

## Limitations and Trade-offs
[When NOT to use X? with sources]

## Comparison with Alternatives
[Alternatives with sources]

## Best Practices
[Recommendations with sources]

## Common Pitfalls
[Mistakes to avoid with sources]

## Current State and Trends
[Latest developments with sources]

## Community and Ecosystem
[Tools, resources, adoption with sources]

## Sources
[Cited sources list]

## Research Log
[Optional: detailed search log]
```

## Additional Resources

For guardrails, research quality criteria, edge case handling, and failure modes, see `@[skills/deep-research/SKILL-advanced.md]`.

For performance considerations, security notes, and related skills, see `@[skills/deep-research/SKILL-examples.md]`.
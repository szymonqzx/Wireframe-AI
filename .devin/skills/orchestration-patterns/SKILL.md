---
name: orchestration-patterns
description: Orchestration patterns and subagent usage guidelines for Wireframe-AI
allowed-tools:
  - read
  - grep
  - glob
triggers:
  - model
---

# Orchestration Patterns

## Purpose

Guide effective use of subagent orchestration for complex Wireframe-AI tasks. Ensure swarm research is systematic, avoids rate limits, and produces actionable synthesized results. Optimize for parallel research performance while maintaining quality.

## When to Use

Use this skill when:
- Adding new modules or features requiring comprehensive research
- Debugging complex issues spanning multiple systems
- Performance optimization requiring bottleneck analysis
- Security audits requiring comprehensive code review
- Schema migrations requiring impact analysis
- Any task where breadth of research matters and can be parallelized

**IMPORTANT RATE LIMIT CONSTRAINT:** Never spawn more than 1 subagent at once to avoid rate limit errors. Use sequential workers if more research is needed.

## Protocol

### Step 1: Task Assessment

1. **Evaluate Orchestration Need**
   - Is the task complex enough to warrant subagents?
   - Can research be parallelized across independent areas?
   - Will the cost of merging answers be less than the research value?

2. **When NOT to Use Subagents**
   - Simple, focused tasks (single file edit)
   - Need immediate interactive feedback
   - Small context understandable directly
   - Mechanical refactoring (renaming, formatting)
   - Real-time iteration required

### Step 2: Swarm Planning

1. **Define Objective**
   - Clear goal for the swarm research
   - Specific deliverables expected
   - Success criteria for the research

2. **Identify Research Areas**
   - Break down task into independent, non-overlapping areas
   - Each area should have bounded scope
   - Ensure no overlap between worker assignments

3. **Worker Allocation**
   - MAX 3 workers at once (rate limit constraint)
   - Use sequential batches for comprehensive research
   - Each worker gets specific file paths or directories
   - Set clear time limits and deliverables

### Step 3: Worker Execution

1. **Launch Workers (Sequential)**
   - Launch up to 3 workers in parallel with `is_background: true`
   - Each worker uses appropriate profile (fast-researcher, rust-researcher, etc.)
   - Each worker researches assigned area independently
   - Wait for batch completion before launching next batch

2. **Worker Deliverables**
   Each worker must report:
   - Key files (with line numbers)
   - Risky code paths
   - Tests to run
   - Unanswered questions
   - Dependencies on other modules
   - Recommendations

### Step 4: Synthesis

1. **Collect Results**
   - Gather all worker results from completed batch
   - Identify patterns and dependencies
   - Note conflicting information

2. **Create Unified Plan**
   - Produce one prioritized implementation plan
   - Identify cross-module dependencies
   - Highlight schema change impacts
   - Recommend testing strategy

3. **Iterate if Needed**
   - If gaps remain, launch next sequential batch
   - Focus new workers on unanswered questions
   - Continue until research is comprehensive

### Step 5: Implementation

1. **Execute Plan**
   - Use synthesized plan for implementation
   - Can use additional swarm (max 3 workers) for parallel implementation
   - Validate results with tests

## Wireframe-AI 1-Worker Pattern

**Lead Architect:** Opus (smartest model). **Sequential Workers (read-only, swe model):** Kernel researcher, Context module, Orchestrator, Sandbox, SDK, Schema, Python adapter, Test, Configuration researchers (one at a time).

**Each worker reports:** Key files (with line numbers), risky code paths, tests to run, unanswered questions, dependencies on other modules.

**Lead agent synthesis:** Produce prioritized implementation plan, identify cross-module dependencies, highlight schema change impacts, recommend testing strategy.

## Subagent Usage Guidelines

Use read-only subagents for broad research, implementation subagents only for isolated/parallelizable work. Subagents should return file paths/line numbers, keep responses concise. Lead agent synthesizes before editing. Subagents improve performance and reduce cost. Background subagents run parallel, foreground pause parent. No nesting (subagents can't spawn subagents).

## Custom Subagent Profiles

Wireframe-AI has specialized subagent profiles:
- `fast-researcher` - Fast read-only codebase mapper (swe model)
- `rust-researcher` - Read-only research for Wireframe-AI Rust codebase
- `backend-specialist` - Backend architect for Rust modules and Python adapters
- `database-architect` - Database architect for SQLite integration
- `performance-optimizer` - Performance optimization for NATS and database
- `security-auditor` - Security review and vulnerability assessment
- `schema-validator` - Validate schema changes

Use these profiles for targeted research instead of generic subagents.

## Model Routing Strategy

- **Architecture, high-risk refactors, final synthesis**: Use smartest available model (opus, gpt)
- **Broad read-only repo research**: Use fast/cheap model (swe, codex)
- **Lint fixes and mechanical edits**: Use fast coding model (swe, codex)
- **PR review and security pass**: Use smarter model (opus, sonnet)
- **Summarization and status updates**: Use cheap model (swe)
- **Performance benchmark**: 10 workers can achieve ~9,500 tokens/sec aggregate output vs ~950 tokens/sec for single model

## Common Pitfalls

| Pitfall | Why Bad | Correct Approach |
|---------|---------|------------------|
| Spawning too many workers at once | Causes rate limit errors | MAX 3 workers at once, use sequential batches |
| Overlapping worker scopes | Creates duplicate work and merge conflicts | Define non-overlapping areas for each worker |
| Vague worker instructions | Leads to inconsistent or incomplete research | Provide clear, specific deliverables for each worker |
| Skipping synthesis phase | Just collecting reports without creating unified plan | Always synthesize results into unified plan |
| Using subagents for simple tasks | Overhead exceeds benefit for focused work | Use subagents only for complex, parallelizable tasks |
| Letting subagents edit directly | Creates merge conflicts and inconsistent changes | Lead agent should synthesize before editing |
| Not setting time limits | Workers may run indefinitely | Set clear time limits and deliverables |
| Ignoring rate limits | Triggers API throttling | Respect rate limit constraints (MAX 3 workers) |

## Code Examples

**Launch sequential workers:**
```python
# Phase 1: Launch batch of workers (sequential, not parallel)
worker1 = run_subagent(title="Kernel researcher", task="Research kernel/ for module registration patterns, NATS setup. Report key files with line numbers.", profile="fast-researcher", is_background=True)
worker2 = run_subagent(title="Context researcher", task="Research modules/context/ for state ownership, database access. Report key files.", profile="rust-researcher", is_background=True)
worker3 = run_subagent(title="Schema researcher", task="Research schemas/v1/ for envelope contracts. Report key files.", profile="schema-validator", is_background=True)

# Wait for completion
result1 = read_subagent(agent_id=worker1, block=True)
result2 = read_subagent(agent_id=worker2, block=True)
result3 = read_subagent(agent_id=worker3, block=True)

# Phase 2: Synthesize results into unified plan
# Phase 3: Launch next batch if gaps remain
```

**Worker deliverables template:**
```python
task = """
Research assigned area and report:
1. Key files (with line numbers)
2. Risky code paths
3. Tests to run
4. Unanswered questions
5. Dependencies on other modules
6. Recommendations

Keep concise (under 2000 tokens). Focus on actionable findings.
"""

## Integration

This skill integrates with:
- `/karpathy-guidelines` - For Think Before Coding principle
- `/parallel-search` - For fast context retrieval
- `/project-routing` - For agent selection based on task type
- See `orchestration-patterns-examples.md` for detailed practical examples

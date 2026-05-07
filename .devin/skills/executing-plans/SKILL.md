---
name: executing-plans
description: Use when you have a written implementation plan to execute in a separate session with review checkpoints
allowed-tools:
  - read
  - grep
  - glob
  - edit
  - write
  - exec
triggers:
  - model
---

# Executing Plans

## Purpose

Systematically execute written implementation plans with critical review, checkpoint-based execution, and proper completion workflow. Ensures plan-based development follows a disciplined approach with verification at each step.

## When to Use

**Use this skill when:**
- You have a written implementation plan (from `writing-plans` skill) to execute
- Starting a new session to implement a pre-written plan
- The plan is comprehensive with clear tasks and verification steps
- You need checkpoint-based execution with human review

**Do NOT use when:**
- No written plan exists (use `implementation` skill instead)
- Subagents are available (use `subagent-driven-development` instead for better quality)
- The plan is incomplete or needs major revisions (return to `writing-plans`)

## Protocol

### Phase 1: Pre-Execution Setup

1. **Announce skill activation**
   - Say: "I'm using the executing-plans skill to implement this plan."

2. **Check for subagent availability**
   - If subagents are available, recommend using `subagent-driven-development` instead
   - Explain that subagent support significantly improves quality

3. **Load and review the plan**
   - Read the plan file completely
   - Review critically for:
     - Clear task definitions
     - Logical task ordering
     - Verification steps for each task
     - Dependency relationships
   - Identify any questions, concerns, or gaps

4. **Address concerns before starting**
   - If concerns exist: Raise them with your human partner
   - Wait for clarification or plan updates
   - Re-review if plan is modified
   - If no concerns: Proceed to execution

5. **Initialize task tracking**
   - Use `todo_write` to create task list from plan
   - Ensure all tasks from plan are represented
   - Set first task to `in_progress`

### Phase 2: Task Execution

For each task in the plan:

1. **Mark task as in_progress**
   - Update `todo_write` status

2. **Execute task steps exactly**
   - Follow each step in the plan precisely
   - Do not skip steps or make assumptions
   - If a step is unclear, stop and ask

3. **Run verifications as specified**
   - Execute verification commands from plan
   - Check expected outputs
   - If verification fails, stop and report

4. **Mark task as completed**
   - Update `todo_write` status to `completed`
   - Move to next task

### Phase 3: Completion Workflow

After all tasks complete and verified:

1. **Announce completion workflow**
   - Say: "I'm using the finishing-a-development-branch skill to complete this work."

2. **Invoke required sub-skill**
   - Use `superpowers:finishing-a-development-branch`
   - Follow that skill's protocol to:
     - Verify tests pass
     - Present merge/PR/cleanup options
     - Execute chosen completion path

## Stop Conditions

**STOP executing immediately when:**
- Hit a blocker (missing dependency, test fails, instruction unclear)
- Plan has critical gaps preventing starting
- You don't understand an instruction
- Verification fails repeatedly
- Plan assumptions conflict with reality

**Ask for clarification rather than guessing.**

**Return to Phase 1 (Review) when:**
- Partner updates the plan based on your feedback
- Fundamental approach needs rethinking
- Multiple blockers suggest plan needs revision

**Don't force through blockers** - stop and ask.

## Common Pitfalls

| Pitfall | Prevention Strategy |
|--------|---------------------|
| Skipping verification steps | Always run verification commands exactly as specified in plan |
| Making assumptions about unclear steps | Stop and ask for clarification before proceeding |
| Continuing after verification failure | Investigate failure cause before moving to next task |
| Not marking tasks in todo_write | Update todo_write status for each task transition |
| Starting on main branch | Ensure isolated workspace via git worktree before starting |

## Verification Examples

**Example: Running verification commands from plan**

```bash
# Plan specifies: "Run tests to verify implementation"
cargo test --lib

# Plan specifies: "Check build succeeds"
cargo build --release

# Plan specifies: "Verify schema changes are valid"
python scripts/validate_schemas.py
```

**Example: Checking verification outputs**

```bash
# Plan expects test output to show all passing
cargo test --lib
# Expected: All tests passed
# If failed: Stop and investigate before proceeding
```

## Integration

**Required workflow skills:**
- **superpowers:using-git-worktrees** - Ensures isolated workspace (creates one or verifies existing)
- **superpowers:writing-plans** - Creates the plan this skill executes
- **superpowers:finishing-a-development-branch** - Complete development after all tasks

**Related skills:**
- **superpowers:subagent-driven-development** - Alternative for environments with subagent support (higher quality)
- **superpowers:karpathy-guidelines** - Behavioral standards for plan execution

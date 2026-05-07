---
name: dispatching-parallel-agents
description: Use when facing 2+ independent tasks that can be worked on without shared state or sequential dependencies
allowed-tools:
  - read
  - grep
  - glob
triggers:
  - model
---

# Dispatching Parallel Agents

## Purpose

Accelerate debugging and development by dispatching multiple specialized agents to work on independent problems in parallel. Ensure each agent has focused scope and clear deliverables to avoid conflicts and maximize efficiency.

## When to Use

Use this skill when:
- 3+ test files failing with different root causes
- Multiple subsystems broken independently
- Each problem can be understood without context from others
- No shared state between investigations
- Tasks are truly independent (fixing one won't affect others)

**Don't use when:**
- Failures are related (fix one might fix others)
- Need to understand full system state
- Agents would interfere with each other
- Shared state or resources would cause conflicts
- Exploratory debugging where the problem is unknown

## Protocol

### Step 1: Identify Independent Domains

1. **Analyze Failures**
   - Group failures by what's broken
   - Identify root cause categories
   - Check for dependencies between failures

2. **Verify Independence**
   - Can each domain be fixed without affecting others?
   - Are there shared files or state?
   - Will agents interfere with each other?

3. **Group by Domain**
   - File A tests: Tool approval flow
   - File B tests: Batch completion behavior
   - File C tests: Abort functionality

### Step 2: Create Focused Agent Tasks

Each agent gets:
- **Specific scope:** One test file or subsystem
- **Clear goal:** Make these tests pass
- **Constraints:** Don't change other code
- **Expected output:** Summary of what you found and fixed

**Agent Prompt Structure:**
```markdown
Fix the 3 failing tests in src/agents/agent-tool-abort.test.ts:

1. "should abort tool with partial output capture" - expects 'interrupted at' in message
2. "should handle mixed completed and aborted tools" - fast tool aborted instead of completed
3. "should properly track pendingToolCount" - expects 3 results but gets 0

These are timing/race condition issues. Your task:

1. Read the test file and understand what each test verifies
2. Identify root cause - timing issues or actual bugs?
3. Fix by:
   - Replacing arbitrary timeouts with event-based waiting
   - Fixing bugs in abort implementation if found
   - Adjusting test expectations if testing changed behavior

Do NOT just increase timeouts - find the real issue.

Return: Summary of what you found and what you fixed.
```

### Step 3: Dispatch in Parallel

1. **Launch Agents**
   - Use background execution for parallel work
   - Each agent works on their assigned domain
   - Agents should not inherit session context

2. **Monitor Progress**
   - Track each agent's progress
   - Be available for questions if needed
   - Let agents work independently

### Step 4: Review and Integrate

1. **Review Summaries**
   - Read each agent's summary
   - Understand what changed
   - Verify root causes were addressed

2. **Check for Conflicts**
   - Did agents edit same code?
   - Are there conflicting fixes?
   - Do changes integrate cleanly?

3. **Run Full Suite**
   - Execute complete test suite
   - Verify all fixes work together
   - Spot check for systematic errors

4. **Integrate Changes**
   - Apply all fixes
   - Commit integrated changes
   - Document what was fixed

## Common Mistakes

**❌ Too broad:** "Fix all the tests" - agent gets lost
**✅ Specific:** "Fix agent-tool-abort.test.ts" - focused scope

**❌ No context:** "Fix the race condition" - agent doesn't know where
**✅ Context:** Paste the error messages and test names

**❌ No constraints:** Agent might refactor everything
**✅ Constraints:** "Do NOT change production code" or "Fix tests only"

**❌ Vague output:** "Fix it" - you don't know what changed
**✅ Specific:** "Return summary of root cause and changes"

## Key Benefits

1. **Parallelization** - Multiple investigations happen simultaneously
2. **Focus** - Each agent has narrow scope, less context to track
3. **Independence** - Agents don't interfere with each other
4. **Speed** - 3 problems solved in time of 1

## Verification

After agents return:
1. **Review each summary** - Understand what changed
2. **Check for conflicts** - Did agents edit same code?
3. **Run full suite** - Verify all fixes work together
4. **Spot check** - Agents can make systematic errors

## Integration

This skill integrates with:
- `/orchestration-patterns` - For broader swarm orchestration guidance
- `/karpathy-guidelines` - For Think Before Coding principle
- `/systematic-debugging` - For -phase debugging methodology

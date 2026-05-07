---
name: implementation-checkpoints
description: Detailed checkpoint protocols and test coverage guidelines for Wireframe-AI implementation
---

## Test Coverage in AI Hot Spots

Areas frequently modified by AI should have enhanced test coverage to ensure confidence in agent output:

### Wireframe-AI AI Hot Spots

**1. modules/context/ (State Management)**
- Critical for system state persistence
- High modification frequency by AI
- Test coverage needed:
  - Concurrent state updates
  - SQLite transaction handling
  - State retrieval edge cases
  - NATS message integration

**2. kernel/interface/ (NATS Integration)**
- Core message bus integration
- High modification frequency by AI
- Test coverage needed:
  - Message envelope handling
  - Module registration/deregistration
  - Connection failure scenarios
  - Topic subscription patterns

**3. sdk/agentic-sdk/ (Public API)**
- User-facing API surface
- High modification frequency by AI
- Test coverage needed:
  - All public functions
  - Macro expansion behavior
  - Error handling paths
  - Integration with kernel

**4. modules/orchestrator/ (Job Scheduling)**
- Critical for agent coordination
- High modification frequency by AI
- Test coverage needed:
  - Job scheduling logic
  - Fanout message patterns
  - Agent lifecycle management
  - Error recovery scenarios

**5. modules/sandbox/ (Execution Isolation)**
- Security-critical component
- High modification frequency by AI
- Test coverage needed:
  - Resource limit enforcement
  - MCP protocol handling
  - Security boundary violations
  - Process cleanup

### Test Coverage Strategy

**Before AI Modifications:**
- Ensure existing tests pass
- Identify test gaps in hot spots
- Add missing unit tests
- Add integration tests for critical paths

**After AI Modifications:**
- Run full test suite
- Verify no regressions
- Add tests for new functionality
- Update test documentation

### Example Test Enhancement

```powershell
# Before allowing AI to modify context module:
"Before modifying modules/context/, first add unit tests for the state management functions.
Focus on concurrent update scenarios and SQLite transaction handling.
Ensure tests pass before making any changes."

# After AI modification:
"Now that you've modified modules/context/, add integration tests that verify the NATS message flow.
Test the end-to-end state update process with real NATS messages."
```

### Test Coverage Metrics

Target test coverage for AI hot spots:
- **Unit tests:** 80%+ coverage
- **Integration tests:** All critical paths covered
- **Edge cases:** Identified and tested
- **Error paths:** All error scenarios tested

## PRD Co-Development Workflow

For complex or vaguely defined tasks, collaborate with the agent to create a detailed plan before implementation:

### When to Use PRD Co-Development

- Task is complex or vaguely defined
- Requirements are unclear or incomplete
- Multiple systems or modules are involved
- Architecture decisions need to be made
- You don't know every nuance or requirement upfront

### Co-Development Process

**1. Discovery Phase**
Ask the agent to explore discovery questions:
- "How does our authentication system function?"
- "Which services might be impacted by this change?"
- "What are the existing patterns for similar features?"
- "What are the dependencies between modules?"

**2. Code Target Identification**
Ask the agent to identify specific relevant code targets:
- "Which files will need to be modified?"
- "What are the public APIs that will change?"
- "Are there schema changes required?"
- "What are the integration points?"

**3. Planning Mode**
Use planning mode (if available) to focus on reading and exploring existing code rather than immediately modifying it:
- Use `/writing-plans` skill for structured planning
- Use `/architecture` skill for design decisions
- Use `/research-architecture` skill for codebase exploration

**4. Collaborative Planning**
Work with the agent to create a detailed plan:
- Define clear requirements
- Identify architectural decisions
- Break down into implementation tasks
- Define testing strategy
- Identify dependencies and risks

### Benefits of PRD Co-Development

- **Clearer requirements**: Unclear requirements are identified early
- **Better architecture**: Decisions are made before implementation
- **Reduced rework**: Misunderstandings are caught before coding
- **Shared understanding**: Both human and agent understand the approach
- **Faster implementation**: Clear plan leads to faster execution
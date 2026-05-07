---
name: implementation
description: Systematically implement new features using Planning With Files, team registration, and iterative development with validation
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

## Feature Implementation

"Systematically implement new features using Planning With Files, team registration, and iterative development."

## When to Use
- Implementing new features from scratch
- Adding new capabilities to existing projects
- Multi-file changes requiring coordination
- Features that need systematic planning and tracking
- Any implementation where the scope is >20 LOC or >1 file

## When NOT to Use
- Simple bug fixes (use code-fix skill instead)
- Refactoring existing code (use code-fix skill instead)
- Trivial one-off changes (<20 LOC, single file)
- Documentation-only changes (edit directly)

## Subagent Usage

**CRITICAL:** Use subagents for parallel implementation to maximize development velocity.

### When to Use Subagents

- **Independent components:** When the feature has multiple components that can be implemented separately
- **Parallel development:** When different parts of the feature don't depend on each other
- **Testing coordination:** When you need to run multiple test suites in parallel
- **Code generation:** When generating boilerplate or repetitive code across multiple files
- **Documentation:** When writing documentation alongside implementation

### Subagent Strategy

**Parallel Implementation Pattern:**

Lead agent orchestrates, subagents implement in parallel:

**Phase 1: Planning (lead agent)**
- Create detailed implementation plan with task breakdown

**Phase 2: Parallel Implementation**
- Each subagent works on isolated file set
- Launch implementation subagent for each task

**Phase 3: Integration and Testing**
- Lead agent integrates components, runs full test suite

### Subagent Profiles

Use appropriate subagent profiles based on task needs:
- **subagent_general:** General-purpose subagent with full tool access for implementation
- **rust-researcher:** Read-only research for Wireframe-AI Rust codebase architecture (investigation phase)
- **test-runner:** Execute tests and report results
- **subagent_explore:** Read-only investigation of codebase structure and patterns

### Subagent Coordination

1. **Planning Phase:**
   - Lead agent creates detailed implementation plan
   - Identify tasks that can be parallelized
   - Define clear file ownership to avoid conflicts

2. **Implementation Phase:**
   - Launch subagents with `is_background: true` for parallel execution
   - Each subagent gets specific files to modify
   - Monitor progress with `read_subagent`

3. **Integration Phase:**
   - Lead agent integrates all subagent changes
   - Resolve any conflicts or dependencies
   - Run full test suite

4. **Validation Phase:**
   - Use test-runner subagent for comprehensive testing
   - Run integration tests
   - Verify regression protection

### Subagent Guardrails

- **File isolation:** Each subagent works on distinct files to avoid conflicts
- **Clear dependencies:** Define task dependencies before launching subagents
- **Integration responsibility:** Lead agent must integrate and test all changes
- **Test coverage:** Subagents must include tests for their implementations
- **Code quality:** Follow project coding standards and patterns

## Pre-flight Checks

Ensure project builds and tests pass before starting implementation:
- Run build command (e.g., `cargo build`)
- Run test command (e.g., `cargo test`)
- Identify next team number for tracking

## Phase 1: Team Registration

Every feature must have a TEAM_XXX file for tracking:
- Create TEAM_{NUM}_feature_summary.md in .teams/
- Track progress, decisions, questions, and handoff notes
- Update file throughout implementation

## Phase 2: Planning With Files

**Note:** For systematic planning with team registration, use plan-writing skill instead. This skill is for implementation after planning is complete.

## Phase 3: Implementation Loop

- Set maximum iterations (default 15)
- Killswitch file for emergency stops
- Build and test in each iteration
- Log all attempts for debugging

## Checkpoint Protocol for Multi-Part Tasks

For complex features spanning multiple layers (database, backend, frontend, SDK), establish explicit checkpoints:

### Checkpoint Flow
```
Plan → Implement chunk → Test → Fix → Checkpoint review → Next chunk
```

### Wireframe-AI Specific Checkpoints

**1. Schema Changes Checkpoint**
- Plan schema changes in `schemas/v1/`
- Review with human before applying
- Apply migration to database
- Validate backward compatibility
- Document breaking changes

**2. Backend Changes Checkpoint**
- Implement backend logic in relevant module
- Add unit tests for new functionality
- Run integration tests
- Verify NATS message flow
- Review API contracts

**3. SDK Changes Checkpoint**
- Update SDK in `sdk/agentic-sdk/`
- Update macros in `sdk/agentic-sdk-macros/`
- Add SDK tests
- Verify public API changes
- Update documentation

**4. Frontend/Tool Changes Checkpoint**
- Implement changes in `tools/` or frontend
- Add integration tests
- Verify end-to-end flow
- Test with real data
- Review user experience

### Checkpoint Review Questions

At each checkpoint, ask:
- Does this implementation match the plan?
- Are all tests passing?
- Are there any unexpected side effects?
- Should we adjust the plan for remaining work?
- Is the code ready for the next phase?

## Phase 4: Regression Protection

- Run baseline tests before making changes
- Run baseline tests after changes
- Compare results - if different, this is a regression → fix it

## Guardrails (Non-Negotiable)

1. Team registration required - Every feature must have a TEAM_XXX file
2. Killswitch file - Create `~/.implement-feature-stop` to stop
3. Iteration cap - Default 15, reassess if not complete
4. Baseline tests - Run before and after behavior-critical changes
5. Plan persistence - plan.md must survive context compaction
6. Code comments - Add `// TEAM_XXX: Reason` to all modified code
7. No dead code - Remove unused functions, imports, commented code
8. Ask questions early - Create .questions/TEAM_XXX_* for ambiguous decisions
9. Update TODO.md - Track incomplete work globally
10. Single Source of Truth - All planning in plan.md

## Phase 5: Completion Checklist

Before marking the feature complete, ensure:

- [ ] Project builds cleanly (`cargo build --release`)
- [ ] All tests pass (`cargo test`)
- [ ] Integration tests pass (`cargo test --test integration_test`)
- [ ] Baseline regression tests pass (if applicable)
- [ ] Team file updated with final progress
- [ ] Plan.md reflects actual implementation
- [ ] All modified code has `// TEAM_XXX:` comments
- [ ] No dead code left behind
- [ ] TODO.md updated with any remaining work
- [ ] Handoff notes written in team file
- [ ] Questions resolved or documented

## References

See `implementation-checkpoints.md` for detailed checkpoint protocols and test coverage guidelines.
See `implementation-guidelines.md` for edge cases, performance considerations, and security notes.
---
name: code-fix
description: Systematic code review, refactoring, and automated fixing with iteration loops
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

# Code Fix Loop

"Review, refactor, and iteratively fix code until all checks pass."

## When to Use
- Systematic code quality improvement across multiple files
- Fixing 10+ failing tests with clear error messages
- Resolving compile errors in a large refactor
- Making a linter pass (clippy, ESLint, ruff)
- Closing out a type-check sweep
- Addressing security vulnerabilities or resource leaks
- Removing dead code and unused functions
- Improving error handling and adding proper context
- Cleaning up silent error swallowing (unwrap_or, ignore patterns)
- Any task where the checkable outcome is well-defined and machine-verifiable

## When NOT to Use
- UI flows that need human judgment
- Feature implementation from scratch (use plan-writing skill instead)
- Anything involving money, destructive data operations, or production infrastructure
- Tasks where passing depends on subjective quality
- On production systems or critical infrastructure

## Subagent Usage

**CRITICAL:** Use subagents for parallel issue investigation and fixing to maximize efficiency.

### When to Use Subagents

- **Multiple error types:** When build, lint, and test errors occur simultaneously
- **Independent issues:** When different files have unrelated errors that can be fixed independently
- **Parallel investigation:** When you need to investigate multiple code paths simultaneously
- **Test failure analysis:** When multiple tests fail for different reasons
- **Code review:** When reviewing large codebases for quality issues

### Subagent Strategy

**Parallel Investigation and Fix Pattern:**

Phase 1: Parallel Investigation - Launch read-only subagents to investigate error categories (compiler, lint, test). Each returns error list, root causes, suggested fixes.

Phase 2: Parallel Fixing - For independent errors, spawn implementation subagents on isolated files.

Phase 3: Integration and Validation - Lead agent runs full check suite, verifies all fixes.

### Subagent Profiles

Use appropriate subagent profiles based on task needs:

- **subagent_explore:** Read-only investigation of codebase and error patterns
- **rust-researcher:** Read-only research for Wireframe-AI Rust codebase architecture
- **subagent_general:** General-purpose subagent with full tool access for fixes
- **test-runner:** Execute tests and report results

### Subagent Coordination

1. **Investigation Phase:**
   - Spawn read-only subagents in parallel for error investigation
   - Each subagent focuses on a specific error category or file set
   - Collect and categorize errors by type and location

2. **Analysis Phase:**
   - Lead agent analyzes subagent findings
   - Identify independent vs. dependent errors
   - Determine which fixes can be parallelized

3. **Fixing Phase:**
   - For independent errors, spawn implementation subagents
   - Use `is_background: true` for parallel execution
   - Each subagent works on isolated files

4. **Validation Phase:**
   - Lead agent runs full check suite
   - Use test-runner subagent for test execution
   - Verify all errors are resolved

### Subagent Guardrails

- **Error isolation:** Group errors by file or module to avoid conflicts
- **Fix validation:** Subagents must verify their fixes don't introduce new errors
- **Incremental changes:** Each subagent makes minimal, targeted changes
- **Test coverage:** Subagents must run relevant tests after fixing
- **Communication:** Subagents must report what they changed and why

## Core Process

### Pre-flight Safety Checks

Optional: Create backup before running. Baseline: ensure tests pass before starting (adapt test command to your project).

### Loop Configuration

```bash
MAX_ITERS=20
COST_CAP_USD=5.00
KILLSWITCH=~/.code-fix-stop
LOGDIR=.code-fix-logs/$(date +%Y%m%d%H%M%S)
CHECK_CMD="<your-build-command> && <your-lint-command> && <your-test-command>"
```

Adapt CHECK_CMD to your project:
- Rust: `cargo check && cargo clippy && cargo test`
- Node: `npm run build && npm run lint && npm test`
- Python: `python -m py_compile src && flake8 src && pytest`

### The Loop

For each iteration (max 20):
1. Check killswitch file - if present, stop
2. Format code (refactor mode only)
3. Run check command - if green, exit success
4. If not green, analyze failure and fix
5. Repeat until green or max iterations reached

## Review Focus Areas

**Bugs and Logic Errors:**
- Logic errors and incorrect behavior
- Edge cases that aren't handled
- Null/undefined reference issues
- Race conditions or concurrency issues
- Security vulnerabilities
- Improper resource management or resource leaks
- API contract violations
- Incorrect caching behavior (cache staleness, cache key bugs, invalidation issues)
- Violations of existing code patterns or conventions

**Code Quality Issues:**
- Dead code - unused functions, unused imports, commented-out code
- Silent error swallowing - unwrap_or, ignore patterns without logging
- Poor error handling - missing error context, generic error types
- Inconsistent error patterns - mixing Result types, inconsistent anyhow/thiserror usage
- Missing or incomplete documentation - undocumented public APIs
- Code duplication - repeated logic that should be extracted
- Type safety issues - unnecessary string parsing, weak typing
- Resource cleanup issues - missing drop guards, improper cleanup ordering

## Refactoring Focus Areas

1. Clippy warnings - Address all linting suggestions systematically
2. Compilation errors - Fix type mismatches, missing imports, etc.
3. Code organization - Improve module structure and file organization
4. Error handling - Ensure proper error propagation and context
5. Resource management - Verify cleanup and RAII patterns
6. Documentation - Add missing docs and clarify complex logic
7. Performance - Address obvious performance issues

## Guardrails (Non-Negotiable)

1. Killswitch file - Create `~/.code-fix-stop` from any terminal to stop the loop
2. Iteration cap - Default 20. If it hasn't gone green by then, something is structurally wrong
3. Cost cap - Default $5. Sanity bound on runaway spend (if using paid AI services)
4. Backup recommended - Consider creating a backup before running if the project is important
5. Never weakens tests - Explicitly forbid modifying test assertions unless tests are wrong
6. Log everything - Every iteration's check output and AI response goes to `.code-fix-logs/<timestamp>/`
7. Baseline required - Tests must pass before starting (review mode)

## After It Goes Green

### Review Mode
1. Review the changes - Manually inspect modified files. Understand what was changed
2. Re-run the check manually - Sometimes caches lie
3. Check for suspicious weakenings - Search for skip/todo/xtest/only patterns in changed files
4. Verify review findings - Ensure issues identified are actually fixed
5. Commit or backup - If using git, commit the changes. Otherwise, create a backup

### Refactor Mode
1. Review the changes - Manually inspect modified files. Understand what was changed
2. Re-run the check manually - Sometimes caches lie
3. Run integration tests - adapt to your project: <integration-test-command>
4. Check for suspicious weakenings - Search for skip/todo/xtest/only patterns
5. Verify no regressions - Ensure all original functionality still works
6. Commit or backup - If using git, commit the changes. Otherwise, create a backup

## Edge Case Handling
- **Circular dependencies:** Fixing one file breaks another - identify dependency chains and fix in correct order
- **Test flakiness:** Intermittent test failures mask real issues - run tests multiple times to confirm stability
- **Multiple error types:** Compiler, linter, and test errors together - prioritize by severity (compiler > linter > tests)
- **Silent failures:** Code passes checks but still has bugs - add targeted tests for suspected issues
- **Large refactor scope:** Too many files to fix at once - break into smaller batches with intermediate verification
- **Unsafe code blocks:** Windows API or raw pointer code requires extra caution - review changes thoroughly
- **Resource leaks:** Fixed code might introduce new leaks - verify RAII patterns and cleanup paths

## Failure Modes
- **Infinite loop:** Check never goes green due to fundamental issue - stop at iteration cap and escalate
- **Weakened tests:** Fixing code by modifying test assertions instead of fixing logic - explicitly forbidden
- **New bugs introduced:** Fix resolves one issue but creates others - run full test suite after each batch
- **Incomplete fixes:** Addressing symptoms instead of root cause - ensure fix addresses underlying problem
- **Wrong guardrail:** Killswitch or iteration cap prevents completion - adjust parameters if justified
- **Cache issues:** Stale build cache causes false failures - clear cache and retry before escalating
- **Context loss:** Long-running loop exceeds context window - use persistent logs and checkpoint files

## Performance Considerations
- **Iteration speed:** Each iteration should complete within 30-60 seconds for most projects
- **Parallel checks:** Run multiple check commands in parallel when independent (e.g., build + lint)
- **Incremental builds:** Use your build system's incremental compilation to speed up repeated checks
- **Cache strategy:** Leverage build cache aggressively but clear when suspected stale
- **Target selection:** For large projects, focus on specific modules/packages rather than full workspace
- **Test parallelization:** Use parallel test flags appropriate for your framework
- **Log size:** Keep logs concise to avoid context bloat, summarize results instead of raw output

## Security Notes
- **Unsafe code review:** Review any unsafe/low-level code for memory safety and correctness
- **Secrets scanning:** Ensure no secrets are introduced during fixes (use secret-scrubber skill if available)
- **Input validation:** Fixed code should properly validate all inputs, especially from external sources
- **Error disclosure:** Error messages should not leak sensitive information (paths, internal details)
- **API security:** Platform-specific API calls must handle errors properly and validate parameters
- **Resource cleanup:** Ensure all handles, file descriptors, and resources are properly closed
- **Privilege checks:** Verify code doesn't inadvertently require elevated privileges

## Related Skills

- `error-handling` - Error handling patterns using anyhow and thiserror
- `rust-pro` - Master Rust 1.75+ with modern async patterns
- `async-tokio-patterns` - Tokio async/await patterns for Rust applications
- `systematic-debugging` - Systematic debugging methodology
- `clean-code` - Pragmatic coding standards

---
name: quality-checklist
description: Quality checks and anti-slop protocol for Wireframe-AI development
allowed-tools:
  - read
  - grep
  - glob
  - exec
triggers:
  - model
---

# Quality Checklist

## Purpose

Ensure Wireframe-AI development meets high quality standards through systematic verification. Prevent common mistakes (slop) by requiring evidence-based completion, proper tool usage, and appropriate model routing. This skill implements the anti-slop protocol to maintain code quality and development excellence.

## When to Use

Use this skill when:
- Completing any development task or feature implementation
- Before declaring work as done or complete
- Before committing changes or creating pull requests
- After making code changes to verify quality
- Before opening PRs to ensure review readiness
- When unsure if work meets quality standards

## Protocol

### Step 1: Pre-Work Verification

1. **Read Existing Code**
   - Read the existing code before editing
   - Understand current patterns and conventions
   - Identify the right place to make changes

2. **Plan Minimal Changes**
   - Prefer minimal diffs over rewrites
   - Touch only what must be changed
   - Plan the scope before starting

### Step 2: During Work Quality Control

1. **API and Command Usage**
   - Never invent APIs, commands, benchmarks, or pricing
   - Verify commands exist before using them
   - Check documentation for correct usage

2. **Subagent Usage**
   - Use read-only subagents for broad codebase research
   - Use implementation subagents only for isolated, parallelizable work
   - Ensure subagents return file paths and line numbers
   - Keep subagent responses concise
   - Lead agent synthesizes results before editing

### Step 3: Model Routing

1. **Choose Appropriate Model**
   - Architecture, high-risk refactors, final synthesis: Use smartest available model
   - Broad read-only repo research: Use fast/cheap model
   - Lint fixes and mechanical edits: Use fast coding model
   - PR review and security pass: Use smarter model
   - Summarization and status updates: Use cheap model

### Step 4: Pre-Completion Verification

1. **Run Narrowest Relevant Check**
   - Identify the specific check that verifies your work
   - Run it before declaring done
   - Use the most targeted verification command

2. **Gather Evidence**
   - Cite files and commands in your report
   - Prefer evidence over vibes
   - Show actual output, not assumptions

### Step 5: Final Quality Checks

1. **Test Verification**
   - Ensure all tests pass
   - Verify no new lint errors
   - Check that changes match the plan

2. **Impact Assessment**
   - Verify no unintended side effects
   - Check documentation is updated if needed
   - Run security review for sensitive areas

### Step 6: Completion Declaration

Only declare work complete when:
- All tests pass
- No new lint errors
- Changes match the plan
- No unintended side effects
- Documentation updated if needed
- Security review complete for sensitive areas
- Ran the narrowest relevant check
- Cited files and commands in report
- Preferred evidence over vibes

## Quality Checklist

### Before Starting Work

- [ ] Have I read the existing code before editing?
- [ ] Am I preferring minimal diffs over rewrites?
- [ ] Am I citing files and commands when reporting?
- [ ] Am I running the narrowest relevant check before declaring done?

### During Work

- [ ] Am I never inventing APIs, commands, benchmarks, or pricing?
- [ ] Am I using read-only subagents for broad codebase research?
- [ ] Am I using implementation subagents only for isolated, parallelizable work?
- [ ] Am I using the review skill before opening a PR?

### Model Routing

- [ ] Architecture, high-risk refactors, final synthesis: using smartest available model?
- [ ] Broad read-only repo research: using fast/cheap model?
- [ ] Lint fixes and mechanical edits: using fast coding model?
- [ ] PR review and security pass: using smarter model?
- [ ] Summarization and status updates: using cheap model?

### Subagent Usage

- [ ] Are subagents read-only when doing research?
- [ ] Are subagents returning file paths and line numbers?
- [ ] Are subagents keeping responses concise?
- [ ] Is the lead agent synthesizing results before editing?

### Before Declaring Done

- [ ] Did I run the narrowest relevant check?
- [ ] Did I cite files and commands in my report?
- [ ] Did I prefer evidence over vibes?
- [ ] Did I ship small, correct, reviewed changes?

### Final Quality Checks

- [ ] All tests pass
- [ ] No new lint errors
- [ ] Changes match the plan
- [ ] No unintended side effects
- [ ] Documentation updated if needed
- [ ] Security review for sensitive areas
- [ ] Ran the narrowest relevant check
- [ ] Cited files and commands in report
- [ ] Preferred evidence over vibes

## Common Anti-Patterns

| Anti-Pattern | Why It's Bad | Correct Approach |
|--------------|--------------|-------------------|
| Declaring done without testing | No verification of correctness | Run tests before declaring complete |
| Making up APIs/commands | Code won't work | Verify commands exist in documentation |
| Rewriting entire files | Unnecessary risk, large diffs | Prefer minimal, surgical changes |
| Using vibes instead of evidence | Subjective, unreliable | Cite actual files and command output |
| Skipping code review | Misses issues, poor quality | Always review before PR |
| Using wrong model for task | Wasted cost or poor results | Route based on task complexity |

## Verification Examples

| Task | Verification Command | Evidence to Cite |
|------|---------------------|------------------|
| Code change | `cargo test` | Test output showing pass/fail |
| Rust code | `cargo clippy` | Clippy output showing no warnings |
| Build | `cargo build --release` | Build exit code 0 |
| Schema change | Schema validation | Validation output showing compliance |
| Documentation | Check docs build | Build output showing success |

## Integration

This skill integrates with:
- `/karpathy-guidelines` - For Think Before Coding principle
- `/verification-before-completion` - For verification requirements
- `/code-review-checklist` - For code review guidelines
- `/final-checks` - For comprehensive final verification

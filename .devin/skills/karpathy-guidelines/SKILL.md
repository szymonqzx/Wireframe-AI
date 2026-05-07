---
name: karpathy-guidelines
description: Andrej Karpathy-inspired behavioral guidelines to reduce common LLM coding mistakes - think before coding, simplicity first, surgical changes, goal-driven execution
allowed-tools:
  - read
  - grep
  - glob
  - edit
  - write
triggers:
  - model
---

# Karpathy Guidelines - Behavioral Standards for AI Coding

**CRITICAL SKILL** - Derived from Andrej Karpathy's observations on LLM coding pitfalls. These four principles directly address common AI agent mistakes: wrong assumptions, overcomplication, orthogonal edits, and vague execution.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks (simple typo fixes, obvious one-liners), use judgment — not every change needs the full rigor.

---

## The Four Principles

### 1. Think Before Coding
**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- **State your assumptions explicitly.** If uncertain, ask.
- **If multiple interpretations exist, present them** — don't pick silently.
- **If a simpler approach exists, say so.** Push back when warranted.
- **If something is unclear, stop.** Name what's confusing. Ask.

**Example:**
```
User: "Add a feature to export user data"
❌ Wrong: Assume export ALL users, JSON format, file download
✅ Right: "I need to clarify: scope (all/filtered?), format (JSON/CSV/API?), fields (which ones?), volume (how many users?)"
```

---

### 2. Simplicity First
**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked
- No abstractions for single-use code
- No "flexibility" or "configurability" that wasn't requested
- No error handling for impossible scenarios
- If you write 200 lines and it could be 50, rewrite it

**The test:** Would a senior engineer say this is overcomplicated? If yes, simplify.

**Example:**
```
User: "Add a function to calculate discount"
❌ Wrong: Create abstract strategy pattern with 5 classes, configuration objects, validation framework
✅ Right: `def calculate_discount(amount: float, percent: float) -> float: return amount * (percent / 100)`
```

---

### 3. Surgical Changes
**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- **Don't "improve" adjacent code, comments, or formatting**
- **Don't refactor things that aren't broken**
- **Match existing style**, even if you'd do it differently
- **If you notice unrelated dead code, mention it** — don't delete it

When your changes create orphans:
- Remove imports/variables/functions that **YOUR changes made unused**
- **Don't remove pre-existing dead code** unless asked

**The test:** Every changed line should trace directly to the user's request.

**Example:**
```
User: "Fix the bug where empty emails crash the validator"
❌ Wrong: Also improve email validation, add username validation, change comments, reformat everything
✅ Right: Only change the specific lines that fix empty email handling
```

---

### 4. Goal-Driven Execution
**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:

| Instead of... | Transform to... |
|--------------|-----------------|
| "Add validation" | "Write tests for invalid inputs, then make them pass" |
| "Fix the bug" | "Write a test that reproduces it, then make it pass" |
| "Refactor X" | "Ensure tests pass before and after" |

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

**Example:**
```
User: "Fix the authentication system"
❌ Wrong: "I'll review code, identify issues, make improvements, test" (vague)
✅ Right: "If the issue is 'users stay logged in after password change':
  1. Write test: Change password → verify old session invalidated → verify: test fails
  2. Implement: Invalidate sessions on password change → verify: test passes
  3. Check edge cases: Multiple sessions, concurrent changes → verify: tests pass
  4. Verify no regression: Existing auth tests → verify: all pass"
```

---

## Wireframe-AI Specific Applications

### When Adding a New Module

**Think Before Coding:** Ask: module purpose, messages to publish/consume? Present options: new crate vs existing? Check: similar module exists?

**Simplicity First:** Start minimal (message handler only). Don't add config/logging/metrics unless asked. Use existing patterns.

**Surgical Changes:** Only add registration in kernel/interface/src/main.rs. Don't refactor other modules. Match topic naming convention.

**Goal-Driven Execution:** 1) Create skeleton → verify: cargo build, 2) Register → verify: sys.module.online published, 3) Implement handler → verify: message processed, 4) Add tests → verify: cargo test passes

---

### When Modifying Schemas

**Think Before Coding:** Ask: what fields change? Breaking change? Present tradeoffs: version vs backward compatibility? Check: which modules consume this?

**Simplicity First:** Add new fields vs modifying existing. Don't add optional fields unless needed. Use existing envelope structure.

**Surgical Changes:** Only modify specific schema file. Don't change unrelated schemas or refactor directory structure.

**Goal-Driven Execution:** 1) Update schema → verify: cargo build, 2) Update consumers → verify: no compilation errors, 3) Add migration if needed → verify: migration runs, 4) Test message flow → verify: serialize/deserialize correctly

---

### When Debugging NATS Issues

**Think Before Coding:** Ask: symptoms? Which module publishes/subscribes? Present hypotheses: subscription issue? Message format? Timing? Check: NATS server logs, module logs.

**Simplicity First:** Start with basic connectivity check. Don't add retry/backpressure/monitoring unless needed. Use existing debugging patterns.

**Surgical Changes:** Only fix specific subscription/publishing code. Don't refactor entire NATS setup. Match existing error handling.

**Goal-Driven Execution:** 1) Reproduce → verify: consistent, 2) Identify root cause → verify: hypothesis confirmed, 3) Implement fix → verify: resolved, 4) Add regression test → verify: passes

---

## How to Know It's Working

These guidelines are working if you see:
- **Fewer unnecessary changes in diffs** — Only requested changes appear
- **Fewer rewrites due to overcomplication** — Code is simple the first time
- **Clarifying questions come before implementation** — Not after mistakes
- **Clean, minimal PRs** — No drive-by refactoring or "improvements"
- **Specific success criteria stated** — Not vague "I'll fix it" statements

---

## Integration with Existing Skills

This skill complements other Wireframe-AI skills:

- **clean-code** - Simplicity First aligns with KISS, YAGNI principles
- **systematic-debugging** - Think Before Coding aligns with hypothesis-driven debugging
- **implementation** - Goal-Driven Execution aligns with iterative development
- **rust-pro** - Surgical Changes aligns with minimal, targeted edits

Use this skill alongside these for maximum effectiveness.

---

## Common Pitfalls to Avoid

| Pitfall | Wrong | Right |
|---------|-------|-------|
| Assuming Without Asking | "Optimize database" → add caching/indexes without asking | "What's slow? Read/write? Which queries? How much data?" |
| Overengineering Simple Requests | "Add config file" → create validation/defaults/hot reload system | Add simple TOML file reading with basic error handling |
| Drive-by Refactoring | "Fix null pointer" → also improve messages, add logging, reformat | Only fix the null pointer issue |
| Vague Success Criteria | "Improve performance" → "I'll optimize the code" | "Target: reduce API response time from 500ms to <100ms. Will add caching, verify with benchmarks" |

---

## When to Relax These Guidelines

For trivial tasks, use judgment:
- Simple typo fixes
- Obvious one-liners
- Adding a single import
- Fixing a clear syntax error

The goal is reducing costly mistakes on non-trivial work, not slowing down simple tasks.

---

## Key Insight from Andrej Karpathy

> "LLMs are exceptionally good at looping until they meet specific goals... Don't tell it what to do, give it success criteria and watch it go."

The "Goal-Driven Execution" principle captures this: transform imperative instructions into declarative goals with verification loops.

---

## References

- Original source: https://github.com/forrestchang/andrej-karpathy-skills
- Andrej Karpathy's observations: https://x.com/karpathy/status/2015883857489522876
- Deep insights: See `KARPATHY-INSIGHTS.md` for comprehensive analysis of Karpathy's philosophy and methods

---

## Deeper Karpathy Philosophy

For advanced practitioners, see `KARPATHY-INSIGHTS.md` covering: first-principles understanding, minimalism as art, data-driven self-improvement, educational excellence, philosophical depth, creative storytelling, pragmatic engineering, long-term obsession. These complement the four core principles.

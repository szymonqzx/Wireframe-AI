---
name: clean-code
description: Pragmatic coding standards - concise, direct, no over-engineering, no unnecessary comments
allowed-tools:
  - read
  - grep
  - glob
  - edit
  - write
triggers:
  - model
---

# Clean Code - Pragmatic AI Coding Standards

**CRITICAL SKILL** - Be concise, direct, and solution-focused.

**Integration with Karpathy Guidelines:** This skill aligns with the **Simplicity First** and **Surgical Changes** principles from `/karpathy-guidelines`. For comprehensive behavioral standards, always use both skills together.

## When to Use
- Writing any code (this is a CRITICAL skill for all code)
- Reviewing code for quality and clarity
- Refactoring to improve code maintainability
- Deciding between implementation approaches
- Naming variables, functions, and classes
- Structuring code for readability

---

## Core Principles

**Alignment with Karpathy Guidelines:**

|| Clean Code Principle | Karpathy Principle | Application |
||---------------------|-------------------|-------------|
|| **KISS** (Keep It Simple) | Simplicity First | Minimum code that solves the problem, no speculative features |
|| **YAGNI** (You Aren't Gonna Need It) | Simplicity First | No features beyond what was asked, no over-abstraction |
|| **SRP** (Single Responsibility) | Simplicity First | Each function does ONE thing, no unnecessary complexity |
|| **DRY** (Don't Repeat Yourself) | Simplicity First | Extract duplicates, reuse code, avoid redundancy |
|| **Surgical Changes** | Surgical Changes | Touch only what you must, clean up only your own mess |

|| Principle | Rule |
||-----------|------|
|| **SRP** | Single Responsibility - each function/class does ONE thing |
|| **DRY** | Don't Repeat Yourself - extract duplicates, reuse |
|| **KISS** | Keep It Simple - simplest solution that works |
|| **YAGNI** | You Aren't Gonna Need It - don't build unused features |
|| **Boy Scout** | Leave code cleaner than you found it |

---

## Naming Rules

|| Element | Convention |
||---------|------------|
|| **Variables** | Reveal intent: `userCount` not `n` |
|| **Functions** | Verb + noun: `getUserById()` not `user()` |
|| **Booleans** | Question form: `isActive`, `hasPermission`, `canEdit` |
|| **Constants** | SCREAMING_SNAKE: `MAX_RETRY_COUNT` |

**Rule:** If you need a comment to explain a name, rename it.

---

## Function Rules

|| Rule | Description |
||------|-------------|
|| **Small** | Max 20 lines, ideally 5-10 |
|| **One Thing** | Does one thing, does it well |
|| **One Level** | One level of abstraction per function |
|| **Few Args** | Max 3 arguments, prefer 0-2 |
|| **No Side Effects** | Don't mutate inputs unexpectedly |

---

## Code Structure

|| Pattern | Apply |
||---------|-------|
|| **Guard Clauses** | Early returns for edge cases |
|| **Flat > Nested** | Avoid deep nesting (max 2 levels) |
|| **Composition** | Small functions composed together |
|| **Colocation** | Keep related code close |

---

## AI Coding Style

|| Situation | Action |
||-----------|--------|
|| User asks for feature | Write it directly |
|| User reports bug | Fix it, don't explain |
|| No clear requirement | Ask, don't assume |

---

## Anti-Patterns (DON'T)

|| ❌ Pattern | ✅ Fix |
||-----------|-------|
|| Comment every line | Delete obvious comments |
|| Helper for one-liner | Inline the code |
|| Factory for 2 objects | Direct instantiation |
|| utils.ts with 1 function | Put code where used |
|| "First we import..." | Just write code |
|| Deep nesting | Guard clauses |
|| Magic numbers | Named constants |
|| God functions | Split by responsibility |

---

## 🔴 Before Editing ANY File (THINK FIRST!)

**Before changing a file, ask yourself:**

|| Question | Why |
||----------|-----|
|| **What imports this file?** | They might break |
|| **What does this file import?** | Interface changes |
|| **What tests cover this?** | Tests might fail |
|| **Is this a shared component?** | Multiple places affected |

**Quick Check:**
```
File to edit: UserService.ts
└── Who imports this? → UserController.ts, AuthController.ts
└── Do they need changes too? → Check function signatures
```

> 🔴 **Rule:** Edit the file + all dependent files in the SAME task.
> 🔴 **Never leave broken imports or missing updates.**

---

## Summary

|| Do | Don't |
||----|-------|
|| Write code directly | Write tutorials |
|| Let code self-document | Add obvious comments |
|| Fix bugs immediately | Explain the fix first |
|| Inline small things | Create unnecessary files |
|| Name things clearly | Use abbreviations |
|| Keep functions small | Write 100+ line functions |

> **Remember: The user wants working code, not a programming lesson.**

---

## 🔴 Self-Check Before Completing (MANDATORY)

**Before saying "task complete", verify:**

|| Check | Question |
||-------|----------|
|| ✅ **Goal met?** | Did I do exactly what user asked? |
|| ✅ **Files edited?** | Did I modify all necessary files? |
|| ✅ **Code works?** | Did I test/verify the change? |
|| ✅ **No errors?** | Lint and TypeScript pass? |
|| ✅ **Nothing forgotten?** | Any edge cases missed? |

> 🔴 **Rule:** If ANY check fails, fix it before completing.

---

## Common Pitfalls
- Writing tutorial-style explanations instead of just code
- Adding unnecessary comments for obvious code
- Creating helper functions for one-liners
- Writing functions longer than 20 lines
- Deep nesting beyond 2 levels
- Magic numbers without named constants
- God functions that do too many things

## Best Practices
- Write code directly, don't explain what you're doing
- Let code self-document through clear naming
- Keep functions small (5-10 lines ideally)
- Use guard clauses to avoid deep nesting
- Name things to reveal intent (userCount not n)
- One thing per function (Single Responsibility)
- Edit all dependent files in the same task

---

## Integration with Karpathy Guidelines

This skill is designed to work alongside `/karpathy-guidelines` for comprehensive behavioral standards:

**Complementary Principles:**
- **Simplicity First** (Karpathy) ↔ KISS, YAGNI, DRY (Clean Code)
- **Surgical Changes** (Karpathy) ↔ Edit only what you must (Clean Code)
- **Think Before Coding** (Karpathy) ↔ Ask, don't assume (Clean Code)

**When to Use Both:**
- For non-trivial implementation work, invoke both `/karpathy-guidelines` and this skill
- Use Karpathy guidelines for behavioral standards (assumptions, tradeoffs, success criteria)
- Use Clean Code for implementation standards (naming, structure, refactoring)
- For trivial tasks (typos, one-liners), Clean Code alone may suffice

**Example Workflow:**
1. Invoke `/karpathy-guidelines` → Establish behavioral standards
2. Invoke `/clean-code` → Apply implementation standards
3. Write code following both sets of principles
4. Verify against both guidelines before completion

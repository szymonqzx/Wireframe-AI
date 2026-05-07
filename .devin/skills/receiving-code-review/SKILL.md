---
name: receiving-code-review
description: Use when receiving code review feedback, before implementing suggestions, especially if feedback seems unclear or technically questionable - requires technical rigor and verification, not performative agreement or blind implementation
allowed-tools:
  - read
  - grep
  - glob
triggers:
  - model
---

# Code Review Reception

## Purpose

Ensure code review feedback is evaluated technically before implementation. Verify suggestions against codebase reality, ask clarifying questions when unclear, and push back on technically incorrect feedback. Prioritize technical correctness over social comfort.

## When to Use

Use this skill when:
- Receiving code review feedback before implementing suggestions
- Feedback seems unclear or technically questionable
- External reviewers suggest changes
- Multi-item feedback requires implementation
- Feedback conflicts with existing architecture or prior decisions

## Protocol

### Step 1: Read and Understand

1. **Read Complete Feedback**
   - Read all feedback without reacting
   - Don't implement anything yet
   - Identify unclear items

2. **Restate Requirements**
   - Restate each requirement in your own words
   - Ask clarifying questions on unclear items
   - Ensure full understanding before proceeding

**IF any item is unclear:**
- STOP - do not implement anything yet
- ASK for clarification on unclear items
- WHY: Items may be related. Partial understanding = wrong implementation

### Step 2: Verify Against Codebase

1. **Technical Verification**
   - Check: Technically correct for THIS codebase?
   - Check: Breaks existing functionality?
   - Check: Reason for current implementation?
   - Check: Works on all platforms/versions?
   - Check: Does reviewer understand full context?

2. **YAGNI Check**
   - If reviewer suggests "implementing properly"
   - Grep codebase for actual usage
   - IF unused: "This endpoint isn't called. Remove it (YAGNI)?"
   - IF used: Then implement properly

### Step 3: Evaluate and Decide

1. **Evaluate Feedback**
   - Is feedback technically sound for this codebase?
   - Does it conflict with architectural decisions?
   - Are there legacy/compatibility reasons for current approach?

2. **Push Back When Needed**
   Push back when:
   - Suggestion breaks existing functionality
   - Reviewer lacks full context
   - Violates YAGNI (unused feature)
   - Technically incorrect for this stack
   - Legacy/compatibility reasons exist
   - Conflicts with architectural decisions

**How to push back:**
- Use technical reasoning, not defensiveness
- Ask specific questions
- Reference working tests/code
- Involve human partner if architectural

### Step 4: Implement (If Appropriate)

1. **Implementation Order**
   For multi-item feedback:
   - Clarify anything unclear FIRST
   - Then implement in this order:
     - Blocking issues (breaks, security)
     - Simple fixes (typos, imports)
     - Complex fixes (refactoring, logic)
   - Test each fix individually
   - Verify no regressions

2. **One Item at a Time**
   - Implement one item
   - Test it
   - Verify no regressions
   - Move to next item

### Step 5: Acknowledge or Correct

1. **Acknowledging Correct Feedback**
   When feedback IS correct:
   - "Fixed. [Brief description of what changed]"
   - "Good catch - [specific issue]. Fixed in [location]."
   - Just fix it and show in the code

   **NEVER:**
   - "You're absolutely right!"
   - "Great point!" / "Excellent feedback!"
   - "Let me implement that now" (before verification)
   - ANY gratitude expression

2. **Correcting Your Pushback**
   If you pushed back and were wrong:
   - "You were right - I checked [X] and it does [Y]. Implementing now."
   - "Verified this and you're correct. My initial understanding was wrong because [reason]. Fixing."

   State the correction factually and move on.

## Source-Specific Handling

### From Human Partner
- Trusted - implement after understanding
- Still ask if scope unclear
- No performative agreement
- Skip to action or technical acknowledgment

### From External Reviewers
BEFORE implementing:
1. Check: Technically correct for THIS codebase?
2. Check: Breaks existing functionality?
3. Check: Reason for current implementation?
4. Check: Works on all platforms/versions?
5. Check: Does reviewer understand full context?

IF suggestion seems wrong:
- Push back with technical reasoning

IF can't easily verify:
- Say so: "I can't verify this without [X]. Should I [investigate/ask/proceed]?"

IF conflicts with human partner's prior decisions:
- Stop and discuss with human partner first

## GitHub Thread Replies

When replying to inline review comments on GitHub, reply in the comment thread, not as a top-level PR comment.

## Common Mistakes

| Mistake | Why Bad | Correct Approach |
|---------|---------|------------------|
| Performative agreement | Wastes time, signals weakness | State requirement or just act |
| Blind implementation | May break existing functionality | Verify against codebase first |
| Batch without testing | Hard to identify which change broke something | One at a time, test each |
| Assuming reviewer is right | Reviewer may lack context | Check if breaks things |
| Avoiding pushback | Technical correctness > comfort | Push back with technical reasoning |
| Partial implementation | Items may be related, partial = wrong | Clarify all items first |
| Can't verify, proceed anyway | Risk of introducing bugs | State limitation, ask for direction |
| Gratitude expressions | Performative, unprofessional | Skip gratitude, focus on technical content |

## Code Examples

**Example: Verifying feedback before implementation**

```bash
# Reviewer suggests: "Use async-nats instead of tokio-nats"

# Step 1: Verify current implementation
grep -r "tokio-nats" modules/
# Output: No matches found

# Step 2: Check what's actually used
grep -r "async-nats" modules/
# Output: Already using async-nats

# Step 3: Push back with technical reasoning
# "We're already using async-nats in modules/context/src/lib.rs:15.
# Can you clarify which specific location needs changing?"
```

**Example: YAGNI check**

```bash
# Reviewer suggests: "Implement proper error handling for this endpoint"

# Step 1: Check if endpoint is actually used
grep -r "endpoint_name" modules/ adapter/
# Output: No matches found

# Step 2: Push back with YAGNI reasoning
# "This endpoint isn't called anywhere in the codebase. Should we remove it
# instead (YAGNI), or is it planned for future use?"
```

**Example: Correcting your pushback**

```python
# Initial pushback:
# "This change would break the Context module's state ownership pattern.
# The current approach is intentional per architectural decision ADR-003."

# After verification, you were wrong:
# "You were right - I checked modules/context/src/lib.rs:42 and the pattern
# has evolved since ADR-003. The current implementation doesn't actually enforce
# state ownership as documented. Implementing the suggested change now."
```

**Example: GitHub thread reply**

```markdown
# Inline comment reply (correct):
@reviewer Fixed. Changed modules/context/src/lib.rs:42 to use Result instead
of unwrap(). Verified tests pass.

# Top-level PR comment (incorrect - use inline instead):
# "I've addressed all the review comments." # Don't do this
```

## Integration

This skill integrates with:
- `/code-review-checklist` - For comprehensive code review guidelines
- `/karpathy-guidelines` - For Think Before Coding principle
- `/receiving-code-review` - For technical rigor in feedback evaluation

## The Bottom Line

**External feedback = suggestions to evaluate, not orders to follow.**

Verify. Question. Then implement.

No performative agreement. Technical rigor always.

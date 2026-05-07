---
name: code-review-checklist
description: Code review guidelines covering code quality, security, and best practices.
allowed-tools:
  - read
  - grep
  - glob
triggers:
  - model
---

# Code Review Checklist

## Purpose

Provide structured guidance for reviewing code changes across multiple dimensions: correctness, security, performance, code quality, testing, and documentation. Ensure reviews are consistent, thorough, and actionable while avoiding common pitfalls that can slow down development or miss critical issues.

## When to Use

Use this skill when:
- Reviewing code for correctness, security, performance, quality, testing, and documentation
- Conducting formal code reviews before merge
- Checking AI-generated code for common issues
- Reviewing pull requests
- Auditing codebases for anti-patterns
- Providing structured feedback on code changes

## Protocol

### Step 1: Initial Assessment

1. **Understand the Change**
   - Read the pull request description or commit message
   - Understand what problem this change solves
   - Identify the scope and impact of the change

2. **Check Test Coverage**
   - Verify tests exist for new code
   - Ensure edge cases are tested
   - Check test quality and maintainability
   - Verify tests are not flaky

### Step 2: Correctness Review

1. **Logic Verification**
   - Check for logic errors and edge cases
   - Verify off-by-one errors
   - Ensure null/undefined handling
   - Validate type safety

2. **Algorithm Review**
   - Verify correct algorithm implementation
   - Check for unnecessary complexity
   - Ensure code does what it's supposed to do

### Step 3: Security Review

1. **Vulnerability Scanning**
   - Check for SQL injection vulnerabilities
   - Verify no XSS vulnerabilities
   - Validate authentication/authorization
   - Check for sensitive data exposure

2. **Input Validation**
   - Ensure input is validated and sanitized
   - Verify no hardcoded secrets
   - Check dependency vulnerabilities

### Step 4: Performance Review

1. **Efficiency Check**
   - Identify inefficient algorithms
   - Check for unnecessary computations
   - Verify no memory leaks
   - Check for N+1 queries

2. **Async Code**
   - Verify no blocking operations in async code
   - Check for appropriate caching
   - Consider bundle size impact

### Step 5: Code Quality Review

1. **Structure Assessment**
   - Verify naming conventions
   - Check for code duplication
   - Assess function/method length
   - Evaluate cyclomatic complexity

2. **Design Principles**
   - Ensure DRY principles followed
   - Verify SOLID principles
   - Check appropriate abstraction level
   - Identify dead code

### Step 6: Testing & Documentation

1. **Test Validation**
   - Ensure unit tests for new code
   - Verify edge cases tested
   - Check test isolation
   - Validate test readability

2. **Documentation Check**
   - Verify comments are not outdated
   - Check API documentation clarity
   - Ensure README is updated if needed
   - Verify public APIs are documented

### Step 7: Provide Feedback

1. **Structure Your Review**
   - Use blocking issues for critical problems (🔴)
   - Use suggestions for improvements (🟡)
   - Use nits for minor issues (🟢)
   - Use questions for clarifications (❓)

2. **Be Specific and Actionable**
   - Provide exact line numbers
   - Include code examples showing fixes
   - Explain why something is a problem
   - Suggest concrete improvements

## Review Categories

### Correctness
- Logic errors and edge cases
- Off-by-one errors
- Null/undefined handling
- Type safety violations
- Incorrect algorithm implementation
- Code does what it's supposed to do
- Edge cases handled
- Error handling in place
- No obvious bugs

### Security
- SQL injection vulnerabilities
- XSS vulnerabilities
- Authentication/authorization flaws
- Sensitive data exposure
- Dependency vulnerabilities
- Input validated and sanitized
- No SQL/NoSQL injection vulnerabilities
- No XSS or CSRF vulnerabilities
- No hardcoded secrets or sensitive credentials
- **AI-Specific:** Protection against Prompt Injection (if applicable)
- **AI-Specific:** Outputs are sanitized before being used in critical sinks

### Performance
- Inefficient algorithms
- Unnecessary computations
- Memory leaks
- N+1 queries
- Blocking operations in async code
- No N+1 queries
- No unnecessary loops
- Appropriate caching
- Bundle size impact considered

### Code Quality
- Naming conventions
- Code duplication
- Function/method length
- Cyclomatic complexity
- Dead code
- Clear naming
- DRY - no duplicate code
- SOLID principles followed
- Appropriate abstraction level

### Testing
- Missing test coverage
- Test quality issues
- Flaky tests
- Missing edge case tests
- Test isolation problems
- Unit tests for new code
- Edge cases tested
- Tests readable and maintainable

### Documentation
- Missing or outdated comments
- Unclear API documentation
- Missing README
- Inconsistent documentation
- Missing examples
- Complex logic commented
- Public APIs documented
- README updated if needed

## Anti-Patterns to Flag

| Anti-Pattern | Why It's Bad | Fix |
|--------------|--------------|-----|
| Magic numbers | Unclear meaning, hard to maintain | Use named constants |
| Deep nesting | Hard to read, cognitive load | Use early returns |
| Long functions (100+ lines) | Multiple responsibilities, hard to test | Split into smaller functions |
| `any` type | Loses type safety | Use proper types |
| SELECT * in production | Performance, security | Specify columns |
| Missing error handling | Crashes, poor UX | Add try/catch or error handling |

## Common Mistakes vs Best Practices

| Area | Common Mistake | Best Practice |
|------|----------------|---------------|
| Security | Not checking SQL injection | Always validate and sanitize input |
| Web Apps | Missing XSS vulnerabilities | Escape output, use CSP |
| Input | Not validating at boundaries | Validate at system boundaries |
| Performance | Using SELECT * | Specify only needed columns |
| Error Handling | Not reviewing error handling | Ensure comprehensive error handling |
| Performance | Ignoring performance implications | Consider complexity and scale |
| Testing | Not checking test coverage | Ensure adequate test coverage |
| Documentation | Missing or outdated docs | Keep docs in sync with code |

## AI & LLM Review Patterns (2025)

### Logic & Hallucinations
- [ ] **Chain of Thought:** Does the logic follow a verifiable path?
- [ ] **Edge Cases:** Did the AI account for empty states, timeouts, and partial failures?
- [ ] **External State:** Is the code making safe assumptions about file systems or networks?

### Prompt Engineering Review
```markdown
// ❌ Vague prompt in code
const response = await ai.generate(userInput);

// ✅ Structured & Safe prompt
const response = await ai.generate({
  system: "You are a specialized parser...",
  input: sanitize(userInput),
  schema: ResponseSchema
});
```

## Review Comments Guide

```
// Blocking issues use 🔴
🔴 BLOCKING: SQL injection vulnerability here

// Important suggestions use 🟡
🟡 SUGGESTION: Consider using useMemo for performance

// Minor nits use 🟢
🟢 NIT: Prefer const over let for immutable variable

// Questions use ❓
❓ QUESTION: What happens if user is null here?
```

## Integration

This skill integrates with:
- `/receiving-code-review` - For technical rigor when receiving feedback
- `/karpathy-guidelines` - For Think Before Coding principle
- `/clean-code` - For pragmatic coding standards

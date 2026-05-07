---
name: suggestion
description: Brainstorm and suggest features or improvements with systematic analysis, feasibility assessment, and prioritization. Use for generating actionable improvement proposals
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

# Feature Suggestion

"Brainstorm and suggest features or improvements with systematic analysis, feasibility assessment, and prioritization."

## When to Use
- User requests new feature ideas
- Project needs direction for next development phase
- Exploring potential enhancements to existing functionality
- User requests improvement suggestions
- Code review reveals patterns that need enhancement
- Performance profiling identifies bottlenecks
- Technical debt accumulation needs addressing

## When NOT to Use
- Implementing improvements (use code-fix or enhancement skill instead)
- Simple code formatting or style fixes
- Tasks requiring immediate refactoring
- Non-improvement requests

## Subagent Usage

**CRITICAL:** Use subagents for parallel analysis of different codebase aspects to maximize suggestion quality and coverage.

### When to Use Subagents

- **Multiple analysis areas:** When you need to analyze different aspects of the codebase (performance, security, architecture, UX)
- **Large codebases:** When the codebase is too large for a single agent to analyze comprehensively
- **Cross-domain analysis:** When suggestions span multiple technical domains
- **Competitive analysis:** When comparing with multiple external projects or patterns
- **Risk assessment:** When you need to assess risks from multiple perspectives

### Subagent Strategy

**Parallel Analysis Pattern:**

```powershell
# Lead agent orchestrates, subagents analyze in parallel

# Phase 1: Parallel Codebase Analysis
$analysisAspects = @(
    "Analyze performance bottlenecks",
    "Review security vulnerabilities",
    "Examine code quality issues",
    "Check architecture patterns",
    "Evaluate user experience"
)

foreach ($aspect in $analysisAspects) {
    # Launch read-only subagent for analysis
    # Each subagent returns: findings, recommendations, impact assessment
}

# Phase 2: Synthesis and Prioritization
# Lead agent synthesizes findings, applies prioritization matrix
```

### Subagent Profiles

Use appropriate subagent profiles based on analysis needs:

- **subagent_explore:** Read-only investigation of codebase structure and patterns
- **rust-researcher:** Read-only research for Wireframe-AI Rust codebase architecture
- **fast-researcher:** Quick read-only research for broad codebase understanding
- **performance-profiling:** Performance analysis and optimization recommendations

### Subagent Coordination

1. **Analysis Phase:**
   - Spawn read-only subagents in parallel for different analysis aspects
   - Each subagent focuses on a specific domain (performance, security, architecture)
   - Collect findings with impact assessments

2. **Synthesis Phase:**
   - Lead agent synthesizes all subagent findings
   - Apply prioritization matrix (impact vs. effort)
   - Identify synergies and conflicts between suggestions

3. **Documentation Phase:**
   - Organize suggestions by category and priority
   - Create structured proposals with feasibility assessments
   - Document dependencies and risks

### Subagent Guardrails

- **Read-only access:** Subagents should only use read tools for analysis
- **Specific domain:** Each subagent focuses on a specific analysis domain
- **Evidence-based:** Subagents must provide evidence for their findings
- **Actionable recommendations:** Suggestions must be specific and implementable
- **Impact assessment:** Each suggestion includes impact and effort estimates

## Feature Suggestions

### Brainstorming Process

**1. Context Analysis**
- Read README.md for project goals
- Review TODO.md for known issues
- Check recent team logs for context
- Examine codebase structure

**2. Idea Generation Categories**
- Performance optimizations - Speed, memory, efficiency
- User experience - CLI usability, error messages, documentation
- Integration - External tools, APIs, platforms
- Reliability - Error handling, testing, monitoring
- Maintainability - Code organization, refactoring, documentation
- Innovation - Novel approaches, new capabilities

**3. Feasibility Assessment**
- Technical feasibility - Can it be implemented with current stack?
- Complexity - Estimated effort and risk
- Impact - User value and project benefit
- Alignment - Does it fit project vision?
- Dependencies - External requirements or blockers

**4. Prioritization Matrix**
```
High Impact, Low Complexity    → Do First
High Impact, High Complexity   → Plan Carefully
Low Impact, Low Complexity     → Consider Later
Low Impact, High Complexity    → Discard
```

### Feature Proposal Template

```markdown
# Feature: [Name]

## Problem Statement
What problem does this solve? Why is it needed?

## Proposed Solution
High-level description of the solution.

## Benefits
- Benefit 1
- Benefit 2

## Implementation Approach
- Technical approach
- Key components
- Dependencies

## Complexity Assessment
- Estimated effort: [Low/Medium/High]
- Risk level: [Low/Medium/High]
- Breaking changes: [Yes/No]

## Alternatives Considered
- Alternative 1
- Alternative 2

## Open Questions
- Question 1
- Question 2
```

## Improvement Suggestions

### Brainstorming Process

**1. Codebase Analysis**
- Review recent commits for patterns
- Check TODO.md for known issues
- Examine error handling consistency
- Look for code duplication
- Identify performance bottlenecks
- Check test coverage gaps

**2. Improvement Categories**
- Code quality - Readability, maintainability, duplication
- Performance - Speed, memory usage, efficiency
- Error handling - Consistency, clarity, recovery
- Testing - Coverage, test quality, flakiness
- Documentation - Comments, README, API docs
- Architecture - Modularity, coupling, design patterns
- Security - Vulnerabilities, best practices
- Developer experience - Build times, error messages, debugging

**3. Impact Assessment**
- Current pain point - What problem does this solve?
- Benefit - What value does this provide?
- Effort - Estimated implementation time
- Risk - Potential for introducing bugs
- Scope - Files/modules affected
- Priority - Urgency and importance

**4. Prioritization Matrix**
```
High Benefit, Low Effort     → Quick Wins
High Benefit, High Effort    → Strategic Investments
Low Benefit, Low Effort      → Cleanup Tasks
Low Benefit, High Effort     → Defer or Discard
```

### Improvement Proposal Template

```markdown
# Improvement: [Name]

## Current State
Description of the current situation/problem.

## Proposed Improvement
High-level description of the improvement.

## Benefits
- Benefit 1
- Benefit 2

## Implementation Approach
- Technical approach
- Files/modules affected
- Breaking changes (if any)

## Effort Assessment
- Estimated effort: [Low/Medium/High]
- Risk level: [Low/Medium/High]
- Test coverage impact: [Positive/Negative/Neutral]

## Alternatives Considered
- Alternative 1
- Alternative 2

## Open Questions
- Question 1
- Question 2
```

## Output Format

Create suggestions in `.questions/` directory:
- `TEAM_XXX_feature_[name].md` for team-specific features
- `feature_[name].md` for general features
- `TEAM_XXX_improvement_[name].md` for team-specific improvements
- `improvement_[name].md` for general improvements

## Additional Resources

For guardrails, edge case handling, and failure modes, see `@[skills/suggestion/SKILL-advanced.md]`.

For performance considerations, security notes, and related skills, see `@[skills/suggestion/SKILL-examples.md]`.
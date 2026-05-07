# Skill Quality Standards

Based on analysis of high-performing vs low-performing Wireframe-AI skills, combined with research from industry best practices (Diátaxis framework, Anthropic prompt engineering, Docs as Code), these are the characteristics that distinguish excellent skills.

## Research Sources

This document incorporates best practices from:

1. **Diátaxis Framework** - Systematic approach to technical documentation with four distinct types:
   - Tutorials (learning-oriented)
   - How-to guides (goal-oriented)
   - Reference (information-oriented)
   - Explanation (understanding-oriented)

2. **Anthropic Prompt Engineering** - Best practices for AI agent prompting including:
   - Clarity and specificity
   - Step-by-step reasoning
   - Examples and patterns
   - Role prompting
   - Output control

3. **Docs as Code** - Philosophy of treating documentation with same workflows as code:
   - Version control integration
   - Plain text markup
   - Code review process
   - Automated testing

4. **GitHub Actions Patterns** - Workflow design principles:
   - Clear task definition
   - Sequential execution
   - Dependency management
   - Error handling

## What Makes a Skill "Good"

### Diátaxis Framework Applied to Skills

Wireframe-AI skills primarily follow the **How-to Guide** pattern from Diátaxis, with elements of Reference and Explanation:

**How-to Guide Characteristics (Primary):**
- Goal-oriented - helps user accomplish specific task
- Assumes user knows what they want to achieve
- Action-focused - only action, no digression
- Addresses real-world complexity
- Provides executable solutions
- Describes logical sequences

**Reference Elements (Secondary):**
- Technical descriptions of machinery
- Austere, neutral, factual
- Standard patterns for consistency
- Examples for illustration

**Explanation Elements (Tertiary):**
- Context and background
- Design decisions and rationale
- Connections to other concepts
- Alternative approaches

### 1. Proper Frontmatter (Critical)
**Required fields:**
- `name` - Skill identifier
- `description` - What the skill does
- `triggers` - When the skill should be invoked
- `allowed-tools` - Tools the skill can use (optional but recommended)

**Example:**
```yaml
---
name: karpathy-guidelines
description: Behavioral standards for AI coding
triggers:
  - model
allowed-tools:
  - read
  - grep
  - glob
---
```

### 2. Clear Purpose Statement
**High-quality skills start with a clear purpose:**
- What problem does this skill solve?
- Why does this skill exist?
- What value does it provide?

**Good example:**
```markdown
## Purpose

Guide effective use of subagent orchestration for complex Wireframe-AI tasks. Ensure swarm research is systematic, avoids rate limits, and produces actionable synthesized results.
```

### 3. Explicit "When to Use" Section
**Specific triggers for when to invoke the skill:**
- Concrete situations
- Clear conditions
- Boundary cases (when NOT to use)

**Good example:**
```markdown
## When to Use

Use this skill when:
- Adding new modules or features requiring comprehensive research
- Debugging complex issues spanning multiple systems
- Performance optimization requiring bottleneck analysis

**Don't use when:**
- Simple, focused tasks (single file edit)
- Context is small enough to understand directly
```

### 4. Structured Protocol with Steps
**Step-by-step guidance is essential:**
- Numbered steps or phases
- Clear progression
- Specific actions at each step
- Verification checkpoints

**Good example:**
```markdown
## Protocol

### Step 1: Task Assessment
1. **Evaluate Orchestration Need**
   - Is the task complex enough to warrant subagents?
   - Can research be parallelized?

### Step 2: Swarm Planning
1. **Define Objective**
   - Clear goal for the swarm research
   - Specific deliverables expected
```

### 5. Actionable Content
**Skills must be actionable, not descriptive:**
- Use action verbs (use, run, execute, implement, fix)
- Provide specific commands or patterns
- Include code examples
- Show exactly what to do

**Good example:**
```markdown
### Step 3: Launch Workers
```bash
# Launch up to 3 workers in parallel
for module in modules/*; do
    run_subagent --scope "$module" &
done
```
```

### 6. Code Examples and Patterns
**High-quality skills include:**
- Code blocks with syntax highlighting
- Concrete examples
- Comparison tables (wrong vs right)
- Pattern demonstrations

**Good example:**
```markdown
| Wrong | Correct |
|-------|---------|
| `if (Test-Path "a" -or Test-Path "b")` | `if ((Test-Path "a") -or (Test-Path "b"))` |
```

### 7. Anti-Patterns and Common Mistakes
**Excellent skills warn about pitfalls:**
- What NOT to do
- Common mistakes
- Why certain approaches fail
- How to avoid errors

**Good example:**
```markdown
## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Performative agreement | State requirement or just act |
| Blind implementation | Verify against codebase first |
```

### 8. Integration Points
**Skills connect to the broader ecosystem:**
- References to related skills
- Dependencies on other skills
- How this skill fits into workflows
- Cross-references to documentation

**Good example:**
```markdown
## Integration

This skill integrates with:
- `/karpathy-guidelines` - For Think Before Coding principle
- `/parallel-search` - For fast context retrieval
- `/project-routing` - For agent selection
```

### 9. Proper Markdown Structure
**Technical quality:**
- Consistent header hierarchy (max 4 levels)
- Proper list formatting
- Code blocks with language specifiers
- No trailing whitespace
- Tables for comparisons

### 10. Conciseness and Clarity
**Avoid verbosity (Anthropic best practices):**
- Ideal length: 500-1500 words
- No repetition
- Specific, not vague
- Direct language (avoid "should", "might", "could")
- One concept per section
- Calibrate verbosity to task complexity
- Use positive examples over negative instructions

### 11. Prompt Engineering Best Practices
**Applied from Anthropic's research:**
- **Clarity**: Be explicit about what the agent should do
- **Specificity**: Use precise language, avoid ambiguity
- **Step-by-step**: Break complex tasks into numbered phases
- **Examples**: Provide concrete examples of desired behavior
- **Role prompting**: Define the agent's role and perspective
- **Output control**: Specify expected output format
- **Context**: Provide relevant context without overwhelming
- **Constraints**: Clearly state limitations and boundaries

## Common Issues in Low-Scoring Skills

### Missing Protocol
**Problem:** Skill describes what to do but not how to do it
**Fix:** Add step-by-step protocol with numbered phases

### Vague Triggers
**Problem:** "When to use" section is generic
**Fix:** Be specific about exact situations that trigger the skill

### No Examples
**Problem:** Abstract guidance without concrete examples
**Fix:** Add code blocks, comparison tables, and specific patterns

### Information Dump
**Problem:** Lists of information without structure
**Fix:** Organize into clear sections with purpose and protocol

### Missing Integration
**Problem:** Skill exists in isolation
**Fix:** Add integration section showing how it connects to other skills

### No Anti-Patterns
**Problem:** Only shows what to do, not what to avoid
**Fix:** Add common mistakes section with wrong vs right comparisons

## Evaluation Metrics

The improved `eval_skills.py` script now measures based on industry best practices:

1. **Frontmatter Quality** (30% of quality score)
   - Proper YAML structure (Docs as Code principle)
   - Required fields present (name, description, triggers)
   - Valid metadata
   - Follows standard patterns

2. **Actionability** (30% of quality score)
   - Action verb density (Diátaxis: action-focused)
   - Step-by-step guidance (Diátaxis: logical sequences)
   - Specific examples (Anthropic: concrete examples)
   - Anti-patterns present (Diátaxis: address real-world complexity)
   - Executable solutions (Diátaxis: practical steps)

3. **Clarity** (20% of quality score)
   - Appropriate length (500-1500 words)
   - No repetition
   - Specific language (avoid vague terms - Anthropic: specificity)
   - Clear indicators
   - Calibrated verbosity (Anthropic: calibrate to task complexity)

4. **Structure Quality** (20% of quality score)
   - Comparison tables (Diátaxis: examples for illustration)
   - Integration points (Diátaxis: make connections)
   - Section hierarchy (Diátaxis: respect structure of machinery)
   - Code examples (Anthropic: examples and patterns)

5. **Coverage** (40% of overall score)
   - Required sections present (Diátaxis framework)
   - Purpose, when to use, protocol
   - Integration points
   - Reference elements where appropriate

6. **Consistency** (30% of overall score)
   - Markdown formatting (Docs as Code: plain text markup)
   - Code blocks
   - Lists and tables
   - Header hierarchy
   - Standard patterns (Diátaxis: adopt standard patterns)

## Target Scores

- **Overall SKILL_SCORE:** 0.95+
- **Coverage:** 0.95+
- **Consistency:** 0.95+
- **Quality:** 0.95+
- **Critical Errors:** 0

## Quick Checklist for Skill Authors

Before submitting a skill, verify:

- [ ] Has proper YAML frontmatter with name, description, triggers
- [ ] Has clear Purpose section explaining what the skill does
- [ ] Has explicit When to Use section with specific triggers
- [ ] Has structured Protocol with numbered steps/phases
- [ ] Includes code examples or patterns
- [ ] Provides anti-patterns or common mistakes
- [ ] Shows integration points to other skills
- [ ] Uses proper markdown structure (headers, lists, code blocks)
- [ ] Is concise (500-1500 words) without repetition
- [ ] Uses specific, actionable language (avoid vague terms)

## Examples of Excellent Skills

Study these for patterns:
- `karpathy-guidelines` - Clear principles with examples
- `implementation` - Structured protocol with subagent usage
- `systematic-debugging` - Iron law with phased approach
- `clean-code` - Comparison tables, anti-patterns
- `wireframe-workflow` - Concise, actionable steps

## Continuous Improvement

The autoimprove framework uses `eval_skills.py` to:
1. Measure skill quality objectively
2. Identify specific areas for improvement
3. Track progress over time
4. Maintain high standards across the skill ecosystem

Run evaluation: `python .devin/skills/eval_skills.py`

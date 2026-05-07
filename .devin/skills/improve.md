# autoimprove: skills quality, coverage, consistency

## Change
scope: .devin/skills/**/SKILL.md
exclude: .devin/skills/autoimprove/

## Check
test: python .devin/skills/eval_skills.py --test-only
test-files: .devin/skills/eval_skills.py
run: python .devin/skills/eval_skills.py
score: SKILL_SCORE: ([\d.]+)%
goal: higher
guard: CRITICAL_ERRORS: (\d+) < 1
keep_if_equal: true
timeout: 300

## Stop
budget: 3600
rounds: 20
target: 35%
stale: 5

## Agent
provider: claude
model: claude-sonnet-4-20250514

## Instructions

You are optimizing Wireframe-AI skills for quality, coverage, and consistency.

## Quality Dimensions

1. **Clarity & Conciseness**: Skills should be clear, direct, and avoid verbosity
   - Use active voice and imperative mood
   - Avoid redundant explanations
   - Be specific rather than vague
   - Use examples sparingly and only when helpful
   - **Score is calculated as (coverage + consistency + quality) / token_count**

2. **Scoring Formula**: SKILL_SCORE = (coverage + consistency + quality) / token_count * 10000
   - Longer skills get lower scores (divided by larger token count)
   - Shorter skills get higher scores (divided by smaller token count)
   - Perfect scores (3.0) / 1000 tokens * 10000 = 30%
   - Perfect scores (3.0) / 500 tokens * 10000 = 60%
   - No optimal word count - purely mathematical calculation

3. **Anti-Gaming Protection**: Minimum coverage threshold prevents mindless content removal
   - Coverage < 80%: 70% score penalty (prevents removing essential content)
   - Coverage < 90%: 30% score penalty (warns about borderline coverage)
   - Coverage >= 90%: No penalty (content is sufficiently complete)
   - This ensures skills remain useful while rewarding conciseness

4. **Effectiveness**: Skills should achieve their stated purpose
   - Clear trigger conditions (when to invoke)
   - Step-by-step protocols that are actionable
   - Edge cases covered
   - Integration points clearly specified

5. **Readability**: Skills should be easy to scan and understand
   - Proper markdown structure (headers, lists, code blocks)
   - Logical flow from overview to details
   - Key information prominent
   - Avoid walls of text

## Coverage Requirements

Each skill must have:
- Clear purpose statement (what problem does it solve?)
- Trigger conditions (when should this skill be invoked?)
- Protocol/steps (how to execute the skill)
- Edge cases (what to watch out for)
- Integration points (how does this connect to other skills/systems)
- Examples (if helpful, not required for simple skills)

## Consistency Standards

- All skills use the same markdown structure
- Consistent terminology (use Wireframe-AI terms correctly)
- Consistent formatting (code blocks, headers, lists)
- Consistent trigger description format
- Consistent step numbering and protocol structure

## Improvement Strategy

1. First, ensure all skills meet minimum coverage requirements
2. Then, improve clarity and conciseness by removing redundancy
3. Finally, standardize formatting and structure for consistency
4. Never remove essential content - only improve presentation
5. Preserve the core intent and protocol of each skill
6. Maintain Wireframe-AI specific patterns and terminology

## What to Avoid

- Don't change the fundamental protocol or purpose of a skill
- Don't remove critical warnings or safety constraints
- Don't make skills so concise they lose clarity
- Don't introduce new jargon or change terminology
- Don't restructure in a way that breaks existing workflows

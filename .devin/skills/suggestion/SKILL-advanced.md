# Feature Suggestion - Guardrails and Edge Cases

Guardrails, edge case handling, and failure modes for feature suggestion.

## Guardrails (Non-Negotiable)

1. **Context first** - Never suggest without understanding the project/code
2. **Feasibility check** - Only suggest technically feasible changes
3. **User value** - Every suggestion must provide clear benefit
4. **No scope creep** - Keep suggestions focused and realistic
5. **Document assumptions** - Clearly state any assumptions made
6. **Measure impact** - Only suggest improvements with clear benefits
7. **Consider risk** - Assess potential for introducing bugs

## Edge Case Handling
- **No clear direction:** When project has no obvious next steps - analyze README, TODO, and recent commits for context
- **Overwhelming options:** Too many potential suggestions - prioritize by impact and feasibility matrix
- **Technical debt vs features:** Balance between fixing issues and adding new capabilities - assess both categories
- **Resource constraints:** Limited development time or team size - focus on quick wins and high-impact items
- **Conflicting suggestions:** Multiple improvements address same area - consolidate or rank by benefit
- **Domain expertise gaps:** Suggestions require unfamiliar tech - flag for research or recommend alternatives
- **User preference unclear:** User doesn't specify feature vs improvement - ask for clarification or provide both

## Failure Modes
- **Irrelevant suggestions:** Ideas don't match project context or goals - always analyze project first
- **Over-engineering:** Suggesting complex solutions for simple problems - prioritize simplicity
- **Missing dependencies:** Suggestions require unavailable resources - validate feasibility before proposing
- **Low-value ideas:** Suggestions with minimal impact - use prioritization matrix to filter
- **Implementation blockers:** Suggestions require major refactoring - note complexity and risk
- **Duplicate suggestions:** Repeating existing ideas - check TODO.md and team logs first
- **Scope creep:** Individual suggestions too large - break down into smaller, actionable items

## Related Resources

See `@[skills/suggestion/SKILL.md]` for when to use, subagent usage, feature suggestions, improvement suggestions, and output format.

See `@[skills/suggestion/SKILL-examples.md]` for performance considerations, security notes, and related skills.
# Feature Suggestion - Performance and Security

Performance considerations, security notes, and related skills for feature suggestion.

## Performance Considerations
- **Analysis speed:** Complete suggestion generation within 5-10 minutes
- **Context loading:** Read key files efficiently (README, TODO, project config) - avoid exhaustive codebase scan
- **Suggestion quality:** Balance quantity with relevance - 5-10 high-quality suggestions better than 50 poor ones
- **Prioritization time:** Use matrix-based prioritization for quick decision-making
- **Output format:** Use structured templates for consistent, scannable output
- **Follow-up efficiency:** Store suggestions in .questions/ for easy reference and tracking
- **Review overhead:** Keep proposals concise but complete - avoid excessive detail

## Security Notes
- **Feature security:** New features should include security considerations in proposal
- **Dependency audit:** Suggested dependencies should be audited for vulnerabilities
- **Access control:** New features should consider authentication/authorization requirements
- **Data privacy:** Improvements should not expose sensitive data or weaken privacy
- **Input validation:** Suggested code should include proper input validation
- **Secrets handling:** Proposals should not introduce hardcoded secrets
- **Compliance:** Consider regulatory requirements (GDPR, SOC2, etc.) in suggestions

## Related Skills

- `brainstorming` - Socratic questioning protocol and user communication
- `architecture` - Architectural decision-making framework
- `performance-profiling` - Performance profiling principles and optimization
- `database-design` - Database design principles and decision-making

## Related Resources

See `@[skills/suggestion/SKILL.md]` for when to use, subagent usage, feature suggestions, improvement suggestions, and output format.

See `@[skills/suggestion/SKILL-advanced.md]` for guardrails, edge case handling, and failure modes.
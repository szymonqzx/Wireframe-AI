# Deep Research - Performance and Security

Performance considerations, security notes, and related skills for deep research.

## Performance Considerations
- **Research velocity:** Complete within 5-15 minutes for most topics depending on complexity
- **Question generation:** Start with 10-15 initial questions, add follow-ups as needed
- **Search efficiency:** Batch related questions to reduce API calls
- **Source caching:** Cache search results to avoid redundant queries
- **Report size:** Keep reports concise but comprehensive - aim for 2000-5000 words
- **Iteration balance:** Balance thoroughness with time - confidence threshold prevents over-researching
- **Context management:** Research logs can grow large - use rolling logs or summarize periodically

## Security Notes
- **Source validation:** Verify sources are credible and not malicious sites
- **Content sanitization:** Ensure no malicious code or scripts in research output
- **Privacy:** Don't include PII or sensitive data in research reports
- **API key protection:** If using paid search APIs, never log or expose keys
- **Phishing awareness:** Be cautious of sources that may be phishing attempts
- **Information disclosure:** Research reports may be shared - ensure no sensitive project details
- **Compliance:** Consider regulatory requirements when researching (GDPR, etc.)

## Related Skills

- `windsurf-memory` - Persist research findings
- `brainstorming` - Socratic questioning protocol and user communication

## Related Resources

See `@[skills/deep-research/SKILL.md]` for when to use, subagent usage, pre-flight checks, loop configuration, the research loop, question generation logic, and report structure.

See `@[skills/deep-research/SKILL-advanced.md]` for guardrails, research quality criteria, edge case handling, and failure modes.
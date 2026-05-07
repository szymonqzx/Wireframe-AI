# Deep Research - Guardrails and Quality

Guardrails, research quality criteria, edge case handling, and failure modes for deep research.

## Guardrails (Non-Negotiable)

1. **MANDATORY subagent usage** - ALL research must be performed by subagents, never directly by the main agent
2. Always cite sources - Every claim must reference source material
3. Confidence threshold - Don't consider answered below 80% confidence
4. Source diversity - Seek multiple sources to avoid bias
5. Recency check - Prioritize recent sources (1-2 years) for fast-moving topics
6. Killswitch file - Create `~/.workflow-stop` to stop
7. Iteration cap - Default 15 iterations
8. No hallucination - Only report what was found in search results
9. Clear uncertainty - Explicitly mark uncertain/conflicting information
10. Workspace root - Save report to workspace root
11. Unique filenames - Use timestamp or topic hash

## Research Quality Criteria

- Completeness - Addresses all aspects of the question
- Accuracy - Multiple sources agree
- Recency - Current information for fast-moving topics
- Authority - Credible and authoritative sources
- Specificity - Specific to question, not generic
- Evidence - Examples, data, or concrete evidence

## Edge Case Handling
- **Vague topics:** When research topic is too broad, narrow down to specific aspects
- **Conflicting sources:** Multiple sources disagree - present conflicting views with context
- **Outdated information:** Sources are old - prioritize recent sources and note recency
- **Sparse results:** Limited search results - broaden query or try alternative terms
- **Technical depth:** Topic too technical for general understanding - include explanatory context
- **Language barriers:** Sources in non-English languages - use translation or find English equivalents
- **Paywalled content:** Sources behind paywalls - find free alternatives or note paywall limitation

## Failure Modes
- **Insufficient confidence:** Questions remain below confidence threshold after max iterations - document uncertainty and suggest manual review
- **Search API failures:** Web search tool unavailable or errors - retry with exponential backoff, escalate if persistent
- **Hallucination risk:** AI generates content not found in sources - strict source citation requirement prevents this
- **Report generation failure:** Cannot write report to workspace - verify write permissions and disk space
- **Topic drift:** Research veers off-topic - periodically check alignment with original question
- **Source bias:** Over-reliance on single source - enforce source diversity requirement
- **Infinite loop:** Follow-up questions never resolve - iteration cap prevents runaway

## Related Resources

See `@[skills/deep-research/SKILL.md]` for when to use, subagent usage, pre-flight checks, loop configuration, the research loop, question generation logic, and report structure.

See `@[skills/deep-research/SKILL-examples.md]` for performance considerations, security notes, and related skills.
---
status: completed
created: 2026-05-06
completed: 2026-05-06
---

# TEAM_007 - Workflow Refactor and Sync

## Task
Refactor and sync all workflows in `.windsurf/workflows/` to meet quality standards and ensure cross-reference consistency. Add comprehensive subagent usage guidance.

## Progress
- [x] Create team file
- [x] Analyze current workflow structure and quality
- [x] Run quality checks on all workflows
- [x] Refactor workflows to meet quality standards
- [x] Sync cross-references between workflows and skills
- [x] Add subagent usage guidance to key workflows
- [x] Validate all changes
- [x] Move team to completed

## Decisions
- Using evolve.md pattern for systematic refactor and sync
- Created quality check script to validate workflow structure
- Updated all workflows with proper skill cross-references
- Added comprehensive subagent usage guidance to research, enhance, implement, code-fix-loop, and suggest workflows
- All workflows now meet quality standards and emphasize subagent usage

## Handoff Notes
- All workflows have proper YAML frontmatter
- All workflows have "When to Use" and "When NOT to Use" sections
- All workflows have "Guardrails" section
- All workflows have "Related Skills" sections with valid cross-references
- Key workflows (research, enhance, implement, code-fix-loop, suggest) now have comprehensive "Subagent Usage" sections
- Subagent guidance includes: when to use, strategy patterns, profile selection, coordination phases, example tasks, and guardrails
- Quality check script saved to `.evolve-logs/quality-check.ps1` for future validation
- Commits created:
  - 4a9dd0e - Refactor and sync all workflows with proper skill cross-references
  - dc78cb1 - Add comprehensive subagent usage guidance to workflows

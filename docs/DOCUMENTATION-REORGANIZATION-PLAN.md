# Documentation Reorganization Plan

**Generated:** 2025-05-07
**Purpose:** Clean up and organize Wireframe-AI documentation for better navigability and maintainability

## Current Issues Identified

### 1. TUI Documentation Scattered
Multiple TUI-related files in `docs/` root with overlapping content:
- `TUI-Implementation-Plan.md` (prioritized opportunities)
- `TUI-Enhancement-Plan.md` (reliability, architecture, UI/UX analysis)
- `tui-modular-architecture-plan.md` (jcode-inspired modular architecture)
- `TUI-Plugin-System-Plan.md` (detailed plugin system implementation)
- `TUI-Agent-Chat-Best-Practices.md` (best practices)
- `rust-tui-optimizations.md` (optimizations)

### 2. Plan Files Mixed with Documentation
Implementation plans in `docs/superpowers/plans/` (10 phase files) are temporary planning documents, not permanent documentation.

### 3. AGENTS.md Duplication
- Root `AGENTS.md`: Devin CLI orchestration for Wireframe-AI
- `.devin/agents.md`: Wireframe-AI Technical Patterns
- Different purposes but overlapping content (both cover Karpathy Guidelines)
- `.devin/agents.md` references root `AGENTS.md`

### 4. Schema Documentation Location
Schema reference files (`AGENT-JOB-SCHEMA.md`, `AGENT-RESULT-SCHEMA.md`) in `docs/` root should be in a dedicated reference section.

### 5. Inconsistent Organization
- Some files in subdirectories (`architecture/`, `integrations/`, `operations/`, `security/`, `superpowers/`)
- Many files flat in `docs/` root
- No clear categorization or hierarchy

## Proposed Organization Structure

```
docs/
├── README.md (NEW - documentation index and navigation)
│
├── getting-started/ (NEW - onboarding and core concepts)
│   ├── Quick-Start.md (move from root)
│   ├── Project-Core.md
│   ├── Project-Architecture.md
│   └── Provider-System.md
│
├── reference/ (NEW - API reference, schemas, specifications)
│   ├── API-Reference.md
│   ├── schemas/
│   │   ├── AGENT-JOB-SCHEMA.md (move from root)
│   │   ├── AGENT-RESULT-SCHEMA.md (move from root)
│   │   └── NATS-MESSAGE-ENVELOPE.md (move from root)
│   └── topics/
│       └── TOPICS.md (move from schemas/v1/)
│
├── guides/ (NEW - how-to guides and tutorials)
│   ├── Plugin-Development-Guide.md
│   ├── Hello-World-Plugin-Tutorial.md
│   ├── Plugin-Architecture-Migration-Guide.md
│   ├── Configuration-Examples.md
│   └── SDK-Quick-Start.md
│
├── tui/ (NEW - all TUI-related documentation)
│   ├── README.md (NEW - TUI documentation index)
│   ├── implementation-plans/
│   │   ├── TUI-Implementation-Plan.md (move from root)
│   │   ├── TUI-Enhancement-Plan.md (move from root)
│   │   ├── tui-modular-architecture-plan.md (move from root)
│   │   └── TUI-Plugin-System-Plan.md (move from root)
│   ├── best-practices/
│   │   └── TUI-Agent-Chat-Best-Practices.md (move from root)
│   └── optimization/
│       └── rust-tui-optimizations.md (move from root)
│
├── architecture/ (EXISTING - keep as is)
│   └── adr-020-selfdev-mode.md
│
├── integrations/ (EXISTING - keep as is)
│   ├── GitHub-Integration.md
│   └── Slack-Integration.md
│
├── operations/ (EXISTING - keep as is)
│   └── Deployment.md
│
├── security/ (EXISTING - keep as is)
│   └── Security-Hardening.md
│
├── planning/ (NEW - temporary implementation plans)
│   ├── superpowers-migration/
│   │   ├── phase1-sdk-foundation.md (move from superpowers/plans/)
│   │   ├── phase2-context-migration.md (move from superpowers/plans/)
│   │   ├── phase3-orchestrator-migration.md (move from superpowers/plans/)
│   │   ├── phase4-sandbox-migration.md (move from superpowers/plans/)
│   │   ├── phase5-interface-adapter-migration.md (move from superpowers/plans/)
│   │   ├── phase6-new-plugin-development.md (move from superpowers/plans/)
│   │   ├── phase7-testing-documentation.md (move from superpowers/plans/)
│   │   ├── phase8-advanced-features.md (move from superpowers/plans/)
│   │   ├── phase9-advanced-plugins.md (move from superpowers/plans/)
│   │   └── provider-ecosystem.md (move from superpowers/plans/)
│   └── Phase-1-Implementation-Notes.md (move from root)
│
└── project/ (NEW - project-level documentation)
    ├── ROADMAP.md (move from root)
    ├── 2-Year-Autonomous-Roadmap.md (move from root)
    ├── Universal-Modularization-Plan.md (move from root)
    ├── Best-Practices.md (move from root)
    ├── Performance-Optimization-Guide.md (move from root)
    ├── Production-Deployment-Guide.md (move from root)
    └── SECURITY.md (move from root)
```

## File Movements Summary

### Move to `getting-started/`
- `QUICKSTART.md` → `getting-started/Quick-Start.md`
- `Project-Core.md` → `getting-started/Project-Core.md`
- `Project-Architecture.md` → `getting-started/Project-Architecture.md`
- `Provider-System.md` → `getting-started/Provider-System.md`

### Move to `reference/schemas/`
- `AGENT-JOB-SCHEMA.md` → `reference/schemas/AGENT-JOB-SCHEMA.md`
- `AGENT-RESULT-SCHEMA.md` → `reference/schemas/AGENT-RESULT-SCHEMA.md`
- `NATS-MESSAGE-ENVELOPE.md` → `reference/schemas/NATS-MESSAGE-ENVELOPE.md`

### Move to `reference/topics/`
- `schemas/v1/TOPICS.md` → `reference/topics/TOPICS.md`

### Move to `guides/`
- `Plugin-Development-Guide.md` → `guides/Plugin-Development-Guide.md`
- `Hello-World-Plugin-Tutorial.md` → `guides/Hello-World-Plugin-Tutorial.md`
- `Plugin-Architecture-Migration-Guide.md` → `guides/Plugin-Architecture-Migration-Guide.md`
- `Configuration-Examples.md` → `guides/Configuration-Examples.md`
- `SDK-Quick-Start.md` → `guides/SDK-Quick-Start.md`

### Move to `tui/implementation-plans/`
- `TUI-Implementation-Plan.md` → `tui/implementation-plans/TUI-Implementation-Plan.md`
- `TUI-Enhancement-Plan.md` → `tui/implementation-plans/TUI-Enhancement-Plan.md`
- `tui-modular-architecture-plan.md` → `tui/implementation-plans/tui-modular-architecture-plan.md`
- `TUI-Plugin-System-Plan.md` → `tui/implementation-plans/TUI-Plugin-System-Plan.md`

### Move to `tui/best-practices/`
- `TUI-Agent-Chat-Best-Practices.md` → `tui/best-practices/TUI-Agent-Chat-Best-Practices.md`

### Move to `tui/optimization/`
- `rust-tui-optimizations.md` → `tui/optimization/rust-tui-optimizations.md`

### Move to `planning/superpowers-migration/`
- `superpowers/plans/2025-05-07-phase1-sdk-foundation.md` → `planning/superpowers-migration/phase1-sdk-foundation.md`
- `superpowers/plans/2025-05-07-phase2-context-migration.md` → `planning/superpowers-migration/phase2-context-migration.md`
- `superpowers/plans/2025-05-07-phase3-orchestrator-migration.md` → `planning/superpowers-migration/phase3-orchestrator-migration.md`
- `superpowers/plans/2025-05-07-phase4-sandbox-migration.md` → `planning/superpowers-migration/phase4-sandbox-migration.md`
- `superpowers/plans/2025-05-07-phase5-interface-adapter-migration.md` → `planning/superpowers-migration/phase5-interface-adapter-migration.md`
- `superpowers/plans/2025-05-07-phase6-new-plugin-development.md` → `planning/superpowers-migration/phase6-new-plugin-development.md`
- `superpowers/plans/2025-05-07-phase7-testing-documentation.md` → `planning/superpowers-migration/phase7-testing-documentation.md`
- `superpowers/plans/2025-05-07-phase8-advanced-features.md` → `planning/superpowers-migration/phase8-advanced-features.md`
- `superpowers/plans/2025-05-07-phase9-advanced-plugins.md` → `planning/superpowers-migration/phase9-advanced-plugins.md`
- `superpowers/plans/2025-05-07-provider-ecosystem.md` → `planning/superpowers-migration/provider-ecosystem.md`

### Move to `planning/`
- `Phase-1-Implementation-Notes.md` → `planning/Phase-1-Implementation-Notes.md`

### Move to `project/`
- `ROADMAP.md` → `project/ROADMAP.md`
- `2-Year-Autonomous-Roadmap.md` → `project/2-Year-Autonomous-Roadmap.md`
- `Universal-Modularization-Plan.md` → `project/Universal-Modularization-Plan.md`
- `Best-Practices.md` → `project/Best-Practices.md`
- `Performance-Optimization-Guide.md` → `project/Performance-Optimization-Guide.md`
- `Production-Deployment-Guide.md` → `project/Production-Deployment-Guide.md`
- `SECURITY.md` → `project/SECURITY.md`

### Keep in Root (for now)
- `API-Reference.md` → will move to `reference/API-Reference.md`

### Delete/Archive
- `docs/superpowers/` directory (after moving files)
- Old plan files after successful migration

## AGENTS.md Conflict Resolution

### Issue
Two AGENTS.md files with different purposes:
- Root `AGENTS.md`: Devin CLI orchestration
- `.devin/agents.md`: Technical patterns

### Resolution
1. Keep root `AGENTS.md` as the primary orchestration document
2. Rename `.devin/agents.md` to `.devin/TECHNICAL-PATTERNS.md` for clarity
3. Update cross-references in both files
4. Ensure `.devin/TECHNICAL-PATTERNS.md` references root `AGENTS.md` correctly

## New Files to Create

1. `docs/README.md` - Documentation index with:
   - Overview of documentation structure
   - Quick links to major sections
   - Getting started guide
   - Contribution guidelines

2. `docs/tui/README.md` - TUI documentation index with:
   - Overview of TUI architecture
   - Links to all TUI documentation
   - Implementation status
   - Development guidelines

## Cross-Reference Updates

After moving files, update all internal links:
1. Search for all markdown links in documentation
2. Update relative paths to reflect new structure
3. Update README.md in project root to point to new documentation structure
4. Update any code comments or docstrings that reference documentation paths

## Implementation Steps

1. **Create new directory structure**
2. **Copy files to new locations (not git mv - to remove history)**
3. **Delete old files after successful copy**
4. **Create new index files (README.md)**
5. **Resolve AGENTS.md conflict**
6. **Update all cross-references**
7. **Verify all links work**
8. **Update root README.md**
9. **Clean up old directories**
10. **Test documentation build (if applicable)**

## Benefits

1. **Better Organization**: Clear categorization by purpose (getting-started, reference, guides, etc.)
2. **Improved Discoverability**: Users can find relevant documentation faster
3. **Reduced Clutter**: Planning documents separated from permanent documentation
4. **Scalability**: Easy to add new documentation in appropriate sections
5. **Maintenance**: Easier to update and maintain specific sections

## Risks and Mitigations

### Risk: Broken Links
**Mitigation**: Comprehensive link update and verification step

### Risk: User Confusion
**Mitigation**: Clear README.md with navigation guide

### Risk: Git History Fragmentation
**Mitigation**: **INTENTIONALLY REMOVING HISTORY** - Using copy + delete operations to break git history as requested

## Timeline Estimate

- Directory creation: 5 minutes
- File moves: 15 minutes
- Index file creation: 10 minutes
- Cross-reference updates: 30 minutes
- Verification: 15 minutes
- **Total**: ~75 minutes

## Approval Needed

- [ ] Review and approve organization structure
- [ ] Review and approve file movements
- [ ] Review and approve AGENTS.md resolution
- [ ] Approve implementation timeline

## Git History Removal (Optional)

To completely remove all git history from the repository and start fresh with a single initial commit:

```bash
# Create new orphan branch with current files but no history
git checkout --orphan newmain HEAD

# Commit current state as initial commit
git commit -m "Initial commit"

# Force push over original main branch
git push origin newmain:main --force

# Delete the temporary branch
git branch -D newmain
```

### Result
- **All previous commits will be deleted**
- Repository will have a single initial commit with current files
- Git history will be completely clean

### ⚠️ Important Warnings

This is **destructive and irreversible**:

1. **All collaborators must re-clone** - their local repos will be out of sync
2. **All branches based on old main will be orphaned** - they'll need to be recreated
3. **All tags will be lost** - unless you recreate them
4. **Any open PRs will be broken** - they'll reference non-existent commits
5. **Release artifacts may be affected** - if they reference commit hashes

### Before You Do This

Make sure to:
- [ ] Inform all collaborators
- [ ] Close or merge all open PRs
- [ ] Document any important commit hashes you need (e.g., for release notes)
- [ ] Consider if you need to preserve any tags or releases
- [ ] Backup the repository (clone it elsewhere) just in case

### Alternative for Documentation Only

If you only want to remove history for the documentation files (not the entire repo):
- The documentation reorganization already broke history for those files (we used copy + delete)
- Only the moved docs files have broken history; code files retain their history
- This option preserves code history while still cleaning documentation history

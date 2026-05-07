# Wireframe-AI Repository Restructuring Analysis

**Generated:** 2025-05-07
**Purpose:** Analyze current repository structure against professional Rust standards and identify improvement opportunities

## Current Structure Analysis

### Workspace Structure (Cargo.toml)
**Current State:**
- Workspace with 65+ members
- Mixed examples, modules, plugins, tools, providers all in one workspace
- Some duplicate entries (modules/orchestrator-core, modules/sandbox-core appear twice)
- Examples are workspace members instead of separate workspace or dev-dependencies

**Professional Standards:**
- Workspaces should be focused and cohesive
- Examples should typically be separate workspaces or dev-dependencies
- Too many workspace members can slow down builds
- Related items should be grouped

### Directory Organization

**Current Issues:**

1. **Root Directory Clutter:**
   - `common_debug.txt` - PowerShell script in wrong location (should be in scripts/)
   - `wireframe_ai_context.db*` - Database files in root (should be .gitignored)
   - `provider-config.example.json` and `.toml` - Config examples in root
   - `TODO.md` - Should be in docs/ or removed

2. **Duplicate/Conflicting Modules:**
   - `kernel/interface/` vs `modules/interface-core/` - Potential duplication
   - Both appear in workspace members
   - Need to clarify relationship and purpose

3. **Examples Overload:**
   - 15 example modules as workspace members
   - Should be separate workspace or dev-dependencies
   - Slows down main workspace builds

4. **Tools Organization:**
   - `tools/` contains mixed items (tui-minimal, wireframe-cli, wireframe-debug, wireframe-replay)
   - `tools/tui-minimal/` has 6 sub-crates in workspace
   - Could be better organized

5. **Config/Configs Confusion:**
   - Both `config/` (crate) and `configs/` (configuration files) directories
   - Naming inconsistency

6. **Deploy Directory:**
   - `deploy/` contains production deployment configs
   - Should be in `ops/` or `deployment/` for clarity

## Professional Rust Project Standards

### Workspace Organization
**Best Practices:**
- **Focused workspaces**: Each workspace should have a clear purpose
- **Hierarchical workspaces**: Use multiple workspaces for different concerns
- **Examples as dev-dependencies**: Not as workspace members
- **Separate workspaces for**: Core, examples, tools, benchmarks

**Recommended Structure:**
```
wireframe-ai/
├── Cargo.toml (root workspace - core only)
├── crates/              # Core crates
│   ├── sdk/
│   ├── modules/
│   └── providers/
├── examples/            # Separate workspace or dev-dependencies
│   ├── Cargo.toml
│   └── */
├── tools/               # Separate workspace
│   ├── Cargo.toml
│   └── */
└── benches/             # Separate workspace
    ├── Cargo.toml
    └── */
```

### Directory Naming Conventions
**Best Practices:**
- Use kebab-case for directories and files
- Consistent naming (e.g., `examples/` not `example/`)
- Clear separation of concerns
- No temporary/debug files in root

### File Organization
**Best Practices:**
- Root should contain only: README, LICENSE, Cargo.toml, .gitignore
- Configuration files in dedicated directories
- Documentation in `docs/`
- Scripts in `scripts/`
- Examples in `examples/` or separate workspace

## Specific Issues Found

### 1. Database Files in Root
**Issue:** `wireframe_ai_context.db*` files in root directory
**Impact:** Should not be in version control
**Fix:** Add to .gitignore and remove from repo

### 2. PowerShell Script in Root
**Issue:** `common_debug.txt` is a PowerShell script in wrong location
**Impact:** Clutters root, wrong location
**Fix:** Move to `scripts/common_debug.ps1`

### 3. Config Examples in Root
**Issue:** `provider-config.example.json` and `.toml` in root
**Impact:** Should be in dedicated config directory
**Fix:** Move to `configs/` or `examples/configurations/`

### 4. Duplicate Workspace Members
**Issue:** `modules/orchestrator-core` and `modules/sandbox-core` appear twice in workspace
**Impact:** Confusing, potential build issues
**Fix:** Remove duplicates

### 5. Examples as Workspace Members
**Issue:** 15 example modules are workspace members
**Impact:** Slows down builds, mixes concerns
**Fix:** Move to separate workspace or make dev-dependencies

### 6. Config/Configs Confusion
**Issue:** Both `config/` (crate) and `configs/` (files) directories
**Impact:** Confusing naming
**Fix:** Rename `config/` crate to `config-core/` or similar

### 7. Kernel/Interface vs Modules/Interface-Core
**Issue:** Potential duplication between kernel/interface and modules/interface-core
**Impact:** Unclear which to use, potential maintenance burden
**Fix:** Clarify relationship or consolidate

### 8. TODO.md in Root
**Issue:** `TODO.md` in root directory
**Impact:** Should be in docs/ or tracked in issue tracker
**Fix:** Move to docs/ or remove

## Recommendations Summary

### High Priority
1. Remove database files from root and update .gitignore
2. Move PowerShell script to scripts/ directory
3. Remove duplicate workspace members
4. Move examples to separate workspace
5. Clarify kernel/interface vs modules/interface-core relationship

### Medium Priority
6. Rename config/ crate to avoid confusion
7. Move config examples to appropriate location
8. Move TODO.md to docs/ or remove
9. Reorganize tools/ directory structure
10. Consolidate deploy/ into ops/ structure

### Low Priority
11. Consider hierarchical workspace structure
12. Standardize naming conventions
13. Add CONTRIBUTING.md
14. Add LICENSE file if missing
15. Improve documentation structure

## Pending Analysis

Waiting for subagent analysis to identify:
- Unused/legacy code in modules
- Dead code or commented-out code
- Unused dependencies
- More specific structural issues
- Code that could be consolidated

---

## Subagent Analysis Results

### Critical Issues Found (18 total)

#### 1. Duplicate Workspace Members
- **Location:** Cargo.toml lines 37-38
- **Issue:** `modules/orchestrator-core` and `modules/sandbox-core` appear twice
- **Impact:** Build warnings, configuration confusion
- **Fix:** Remove duplicate entries

#### 2. Inconsistent Module Naming
- **Issue:** Mix of `-core` suffix (context-core, orchestrator-core, sandbox-core, interface-core) and no suffix (event-sourcing, integrations, observability, provider-router, tenant, webhooks)
- **Impact:** Confusing architecture, inconsistent patterns
- **Fix:** Standardize to all use `-core` suffix for plugin architecture consistency

#### 3. Duplicate Interface Modules
- **Issue:** Both `kernel/interface` (legacy, no plugin support) and `modules/interface-core` (newer, with plugin support)
- **Impact:** Developers may use wrong module, maintenance burden
- **Fix:** Deprecate kernel/interface, migrate functionality to modules/interface-core

#### 4. Redundant agentic-sdk-macros Dependencies
- **Issue:** 10 example modules use direct path dependencies instead of workspace features
- **Impact:** Redundant dependencies, harder to maintain
- **Fix:** Use `agentic-sdk = { workspace = true, features = ["macros"] }`

#### 5. Orphaned Python Components
- **Issue:** `sdk/agentic-sdk-py` and `adapter/python` not in workspace
- **Impact:** Unclear maintenance status, disconnected from build system
- **Fix:** Document as optional or move to separate Python workspace

#### 6. Orphaned Monitoring/Benchmarks
- **Issue:** `monitoring/` and `benchmarks/` have Cargo.toml but not in workspace
- **Impact:** Unclear if actively maintained
- **Fix:** Add to workspace or move to dev/tools

#### 7. Unused Templates Directory
- **Issue:** `templates/plugins/` not referenced anywhere in codebase
- **Impact:** Dead code taking up space
- **Fix:** Integrate with wireframe-cli or remove

#### 8. Misnamed PowerShell Script
- **Issue:** `common_debug.txt` is PowerShell script with wrong extension
- **Impact:** Wrong file type, confusing for users
- **Fix:** Rename to `scripts/common.ps1`

#### 9. Database Files in Root
- **Issue:** `wireframe_ai_context.db*` files in root directory
- **Impact:** Should not be in version control
- **Fix:** Add to .gitignore and delete

#### 10. Duplicate Provider Config Files
- **Issue:** Both `provider-config.example.toml` and `.json` in root
- **Impact:** Confusing which to use
- **Fix:** Keep TOML only, document JSON support

#### 11. Misplaced Configuration File
- **Issue:** `scripts/config.json` contains build settings in wrong location
- **Impact:** Configuration scattered, hard to find
- **Fix:** Move to `configs/` or root

#### 12. Module Config in Wrong Location
- **Issue:** `modules/sandbox-core/sandbox-config.yaml` should be in `configs/`
- **Impact:** Inconsistent with other module configs
- **Fix:** Move to `configs/sandbox.yaml`

#### 13. Inefficient Dockerfiles
- **Issue:** Dockerfiles copy entire source tree, breaking layer caching
- **Impact:** Slow builds, large images
- **Fix:** Copy only necessary dependencies

#### 14. Scattered Shell Scripts
- **Issue:** Scripts in `scripts/`, `.devin/hooks/`, `.devin/skills/*/scripts/`
- **Impact:** Hard to find, inconsistent organization
- **Fix:** Consolidate in `scripts/`

#### 15. Missing .gitignore Entries
- **Issue:** Missing `*.db`, `scripts/config.json`, `templates/` entries
- **Impact:** Wrong files committed to repo
- **Fix:** Add comprehensive ignores

#### 16. AGENTS.md in Root
- **Issue:** Project-specific agent config in root directory
- **Impact:** Clutters root, wrong location
- **Fix:** Move to `.devin/` or `docs/project/`

#### 17. Cargo.lock in tools/tui-minimal
- **Issue:** Individual workspace member should not have own lock file
- **Impact:** Breaks workspace dependency management
- **Fix:** Delete, rely on root Cargo.lock

#### 18. Examples Overcrowding Workspace
- **Issue:** 14 example modules as workspace members
- **Impact:** Slows builds, mixes concerns
- **Fix:** Move to separate workspace

---

## Comprehensive Restructuring Plan

### Phase 1: Critical Cleanup (2-4 hours)

#### 1.1 Fix Workspace Configuration
- [ ] Remove duplicate workspace members (Cargo.toml lines 37-38)
- [ ] Delete `tools/tui-minimal/Cargo.lock`
- [ ] Remove examples from main workspace or mark as optional

#### 1.2 Clean Root Directory
- [ ] Delete `wireframe_ai_context.db*` files
- [ ] Add `*.db`, `*.sqlite`, `*.sqlite3` to .gitignore
- [ ] Move `common_debug.txt` to `scripts/common.ps1`
- [ ] Move `AGENTS.md` to `.devin/AGENTS.md`
- [ ] Remove `TODO.md` (move to docs/ or delete)
- [ ] Keep only one provider-config.example file (TOML)

#### 1.3 Fix Configuration Locations
- [ ] Move `scripts/config.json` to `configs/build-config.json`
- [ ] Move `modules/sandbox-core/sandbox-config.yaml` to `configs/sandbox.yaml`
- [ ] Update references in code

### Phase 2: Module Standardization (8-16 hours)

#### 2.1 Standardize Module Naming
- [ ] Rename modules to use consistent `-core` suffix:
  - `event-sourcing` → `event-sourcing-core`
  - `integrations` → `integrations-core`
  - `observability` → `observability-core`
  - `provider-router` → `provider-router-core`
  - `tenant` → `tenant-core`
  - `webhooks` → `webhooks-core`
- [ ] Update all references in Cargo.toml, code, and docs

#### 2.2 Resolve Interface Module Duplication
- [ ] Audit `kernel/interface` vs `modules/interface-core`
- [ ] Migrate any unique functionality from kernel/interface
- [ ] Deprecate `kernel/interface` with deprecation notice
- [ ] Update documentation to point to modules/interface-core
- [ ] Remove from workspace after migration period

#### 2.3 Fix Example Dependencies
- [ ] Update 10 example modules to use workspace features
- [ ] Remove direct `agentic-sdk-macros` path dependencies
- [ ] Test all examples build correctly

### Phase 3: Workspace Reorganization (16-24 hours)

#### 3.1 Create Hierarchical Workspaces
- [ ] Create `examples/Cargo.toml` for examples workspace
- [ ] Move examples out of main workspace
- [ ] Create `tools/Cargo.toml` for tools workspace
- [ ] Move TUI, wireframe-cli, wireframe-debug, wireframe-replay to tools workspace
- [ ] Create `dev/Cargo.toml` for development workspace
- [ ] Move benchmarks and monitoring to dev workspace

#### 3.2 Consolidate TUI Tools
- [ ] Audit 6 TUI crates for actual separation needs
- [ ] Consolidate into 2-3 crates:
  - `tui-core` (core functionality)
  - `tui-main` (application binary)
  - Optional: `tui-widgets` if widgets are reusable
- [ ] Update workspace members

#### 3.3 Handle Orphaned Components
- [ ] Decide on Python SDK/adapter:
  - Option A: Document as optional external components
  - Option B: Create separate Python workspace
  - Option C: Deprecate if not actively maintained
- [ ] Decide on monitoring/benchmarks:
  - Option A: Add to dev workspace if actively used
  - Option B: Move to archive/ if deprecated
  - Option C: Integrate into modules/observability
- [ ] Handle templates/ directory:
  - Option A: Integrate with wireframe-cli
  - Option B: Move to tools/wireframe-cli/templates/
  - Option C: Remove if not used

### Phase 4: Directory Organization (16-24 hours)

#### 4.1 Reorganize Deploy Directory
```
deploy/
  docker/
    docker-compose.yml
    docker-compose.prod.yml
    Dockerfile.module
  k8s/
    manifests/
    helm/
  monitoring/
    grafana/
    prometheus/
```

#### 4.2 Consolidate Documentation
- [ ] Move `modules/sandbox-core/PLUGIN_ARCHITECTURE.md` to `docs/modules/sandbox/`
- [ ] Move `sdk/agentic-sdk/PLUGIN_SYSTEM.md` to `docs/sdk/`
- [ ] Move `sdk/agentic-sdk/README.md` to `docs/sdk/`
- [ ] Create `docs/modules/` and `docs/sdk/` directories
- [ ] Update all documentation cross-references

#### 4.3 Consolidate Shell Scripts
- [ ] Move all shell scripts to `scripts/`
- [ ] Create PowerShell equivalents where missing
- [ ] Document platform-specific script usage
- [ ] Remove scripts from `.devin/skills/*/scripts/`

#### 4.4 Organize Tests
```
tests/
  rust/
    integration/
    benchmark_test.rs
  python/
    test_python_sdk.py
```

### Phase 5: Optimization & Best Practices (24-40 hours)

#### 5.1 Fix Dockerfiles
- [ ] Audit all Dockerfiles
- [ ] Implement multi-stage builds with dependency caching
- [ ] Copy only necessary dependencies for each module
- [ ] Test build times improvement

#### 5.2 Update .gitignore
- [ ] Add comprehensive ignores:
  ```
  # Databases
  *.db
  *.sqlite
  *.sqlite3

  # Local configurations
  scripts/config.json
  *.local.yaml
  *.local.json

  # Build artifacts
  .build-cache/

  # Templates (if not meant to be in repo)
  templates/
  ```

#### 5.3 Establish Conventions
- [ ] Create `CONTRIBUTING.md` with:
  - Module naming conventions
  - File location standards
  - Workspace member criteria
  - Code style guidelines
- [ ] Update `AGENTS.md` with new structure
- [ ] Create architecture decision records for major changes

#### 5.4 Implement Pre-commit Hooks (Optional)
- [ ] Check for duplicate workspace members
- [ ] Validate .gitignore coverage
- [ ] Prevent committing .db files
- [ ] Validate file naming conventions

---

## Risk Assessment

### High Risk Changes
- Module renaming (breaks all imports and references)
- Workspace reorganization (affects build system)
- Interface module deprecation (affects users)

### Medium Risk Changes
- Directory reorganization (affects tooling and scripts)
- Dockerfile changes (affects deployment)
- Documentation moves (affects onboarding)

### Low Risk Changes
- File cleanup (deleting unused files)
- .gitignore updates (prevents future issues)
- Configuration consolidation (cosmetic but helpful)

---

## Success Criteria

- [ ] All critical issues resolved
- [ ] Workspace builds without warnings
- [ ] No duplicate or orphaned code
- [ ] Consistent naming conventions
- [ ] Clear directory organization
- [ ] Documentation consolidated and updated
- [ ] All tests pass
- [ ] Docker builds optimized
- [ ] .gitignore comprehensive

---

## Estimated Timeline

| Phase | Effort | Dependencies |
|-------|--------|--------------|
| Phase 1: Critical Cleanup | 2-4 hours | None |
| Phase 2: Module Standardization | 8-16 hours | Phase 1 |
| Phase 3: Workspace Reorganization | 16-24 hours | Phase 1, 2 |
| Phase 4: Directory Organization | 16-24 hours | Phase 3 |
| Phase 5: Optimization | 24-40 hours | Phase 4 |
| **Total** | **66-108 hours** | **Sequential** |

---

## Recommendations for Implementation

### Start Small
Begin with Phase 1 (Critical Cleanup) as it:
- Has low risk
- Provides immediate benefits
- Doesn't require breaking changes
- Can be done in a single session

### Test Thoroughly
After each phase:
- Run `cargo build --release`
- Run `cargo test`
- Test Docker builds
- Verify documentation links

### Communicate Changes
- Update team on breaking changes
- Provide migration guides
- Document deprecation timelines
- Update onboarding documentation

### Monitor Impact
- Track build times
- Monitor for new issues
- Gather team feedback
- Adjust approach as needed

---

## Phase 1 Completion Status

**Phase 1: Critical Cleanup** ✅ **COMPLETED** (2025-05-07)
- Commit: 506db79
- Actual Effort: 1 hour (estimated 2-4 hours)
- All tasks completed successfully
- Build verification passed
- No breaking changes introduced

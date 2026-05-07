# Wireframe-AI Scripts Enhancement Plan

## Overview
Enhance the PowerShell scripts in the Wireframe-AI repository to improve reliability, error handling, cross-platform compatibility, and user experience.

## Current State Analysis

### Existing Scripts (7 total)
1. **build-release.ps1** - Builds modules in release mode
2. **cross-build.ps1** - Cross-compiles for multiple platforms
3. **download-nats.ps1** - Downloads NATS server binary
4. **run-demo.ps1** - Starts demo with NATS + context + orchestrator
5. **smoke-test.ps1** - End-to-end smoke test
6. **run-tui-runner.ps1** - Runs TUI module runner
7. **start-all.ps1** - Starts all modules in separate cmd windows

### Identified Issues
- Inconsistent error handling across scripts
- No progress indicators for long operations
- Hardcoded values (NATS version, timeout values)
- No parallel build support (cross-build is sequential)
- Windows-only (no Linux/macOS support)
- No script validation or health checks
- Inconsistent logging/output format
- No retry logic for downloads
- Limited cleanup on failure
- Minimal inline documentation

## Enhancement Plan

### Phase 1: Core Infrastructure
**File:** `scripts/common.ps1` (NEW)
- Centralized error handling functions
- Logging utilities with consistent format
- Progress indicator functions
- Retry logic for network operations
- Cleanup handlers for graceful shutdown
- Configuration management (load from config file)

### Phase 2: Build Scripts
**Files:** `scripts/build-release.ps1`, `scripts/cross-build.ps1`

**Enhancements:**
- Add parallel build support for cross-build
- Add progress indicators for each target
- Add configurable build targets via config file
- Add build caching detection (skip if unchanged)
- Add build time measurement
- Add verbose mode for debugging
- Add error recovery (continue on individual failures)

### Phase 3: Utility Scripts
**Files:** `scripts/download-nats.ps1`, `scripts/smoke-test.ps1`

**Enhancements:**
- Add retry logic with exponential backoff for downloads
- Add version parameter support (default to latest if not specified)
- Add checksum verification for downloaded binaries
- Add progress bar for downloads
- Add configurable timeout values
- Add health check functions for service status
- Add detailed test result reporting with metrics

### Phase 4: Launcher Scripts
**Files:** `scripts/run-demo.ps1`, `scripts/start-all.ps1`, `scripts/run-tui-runner.ps1`

**Enhancements:**
- Add dependency checking before launching
- Add service health monitoring
- Add graceful shutdown with signal handling
- Add log rotation for long-running processes
- Add auto-restart on crash (optional)
- Add PID file management for tracking processes
- Add status dashboard showing running services

### Phase 5: Cross-Platform Support
**All scripts**

**Enhancements:**
- Add Linux/macOS detection and compatibility
- Use PowerShell Core (pwsh) instead of Windows PowerShell
- Replace Windows-specific commands with cross-platform alternatives
- Add shell script equivalents (.sh) for Linux/macOS
- Add platform detection and conditional logic

### Phase 6: Documentation & Configuration
**Files:** `scripts/README.md` (NEW), `configs/build-config.json` (NEW)

**Enhancements:**
- Create comprehensive README for all scripts
- Add usage examples for each script
- Add configuration file for customizable settings
- Add troubleshooting guide
- Add migration guide from old to new scripts

## Proposed File Changes

### New Files (3)
1. `scripts/common.ps1` - Shared utilities and functions
2. `scripts/README.md` - Documentation for all scripts
3. `configs/build-config.json` - Configuration file

### Modified Files (7)
1. `scripts/build-release.ps1` - Add progress, parallel builds
2. `scripts/cross-build.ps1` - Add parallel support, better error handling
3. `scripts/download-nats.ps1` - Add retry, checksum, version param
4. `scripts/run-demo.ps1` - Add health checks, graceful shutdown
5. `scripts/smoke-test.ps1` - Add detailed reporting, metrics
6. `scripts/run-tui-runner.ps1` - Add dependency checks
7. `scripts/start-all.ps1` - Add health monitoring, log rotation

### New Shell Scripts (7) - Phase 5
1. `scripts/build-release.sh`
2. `scripts/cross-build.sh`
3. `scripts/download-nats.sh`
4. `scripts/run-demo.sh`
5. `scripts/smoke-test.sh`
6. `scripts/run-tui-runner.sh`
7. `scripts/start-all.sh`

## Implementation Order

1. **Phase 1:** Create `scripts/common.ps1` infrastructure
2. **Phase 2:** Enhance build scripts
3. **Phase 3:** Enhance utility scripts
4. **Phase 4:** Enhance launcher scripts
5. **Phase 6:** Add documentation and configuration
6. **Phase 5:** Add cross-platform support (optional, based on user feedback)

## Risk Assessment

- **Risk Level:** Low - Enhancements are additive, not breaking
- **Breaking Changes:** None - All changes are backward compatible
- **Migration Path:** Drop-in replacement, no migration needed
- **Testing Strategy:** Test each script individually, then integration test with full workflow

## Estimated Effort

- Phase 1: ~30 minutes
- Phase 2: ~45 minutes
- Phase 3: ~30 minutes
- Phase 4: ~45 minutes
- Phase 6: ~30 minutes
- Phase 5: ~2 hours (optional)

**Total:** ~3 hours (without cross-platform) or ~5 hours (with cross-platform)

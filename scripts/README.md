# Wireframe-AI Scripts

This directory contains PowerShell scripts for building, testing, and running the Wireframe-AI distributed agent system.

## Common Features

All scripts now include:
- **Consistent error handling** with clear error messages
- **Progress indicators** for long-running operations
- **Retry logic** for network operations (with exponential backoff)
- **Verbose mode** (`-Verbose` flag) for debugging
- **Timing metrics** to measure operation duration
- **Graceful cleanup** on failure or interruption

## Prerequisites

- Rust toolchain (cargo) - Install from https://rustup.rs
- PowerShell 7+ (pwsh) for cross-platform support
- Python 3.8+ (for Python adapter)
- NATS server (automatically downloaded by scripts)

## Build Scripts

### build-release.ps1
Build all Wireframe-AI modules in release mode.

```powershell
.\scripts\build-release.ps1              # Standard build
.\scripts\build-release.ps1 -Verbose     # Verbose output
.\scripts\build-release.ps1 -SkipCache   # Skip build cache
```

**Features:**
- Build caching to skip unchanged modules
- Progress indicators for each target
- Build time measurement
- Error recovery (continues on individual failures)

### cross-build.ps1
Cross-compile Wireframe-AI for multiple platforms.

```powershell
.\scripts\cross-build.ps1               # Sequential build
.\scripts\cross-build.ps1 -Parallel     # Parallel build (requires cross tool)
.\scripts\cross-build.ps1 -Verbose      # Verbose output
```

**Supported Platforms:**
- Linux x86_64
- Linux ARM64
- Windows x86_64
- macOS ARM64

**Note:** Parallel builds require the `cross` tool: `cargo install cross`

## Utility Scripts

### download-nats.ps1
Download NATS server binary for the current platform.

```powershell
.\scripts\download-nats.ps1             # Download latest version
.\scripts\download-nats.ps1 -Version 2.10.25  # Specific version
.\scripts\download-nats.ps1 -Force      # Re-download even if exists
```

**Features:**
- Automatic platform detection (Windows/Linux/macOS)
- Retry logic with exponential backoff
- Version parameter support
- Check for existing installation

### smoke-test.ps1
End-to-end smoke test for Wireframe-AI.

```powershell
.\scripts\smoke-test.ps1                # Run smoke test
.\scripts\smoke-test.ps1 -TimeoutSeconds 30  # Custom timeout
.\scripts\smoke-test.ps1 -Verbose       # Verbose output
```

**Test Steps:**
1. Start NATS server (if not running)
2. Start context module
3. Run interface with test message
4. Verify pipeline completion
5. Cleanup all processes

## Launcher Scripts

### run-demo.ps1
Start a local demo environment with NATS, context, and orchestrator.

```powershell
.\scripts\run-demo.ps1                  # Start demo
.\scripts\run-demo.ps1 -Verbose         # Verbose output
```

**What it starts:**
- NATS message bus
- Context module
- Orchestrator module (optional, continues if fails)

Press Ctrl+C to stop all modules.

### start-all.ps1
Start all Wireframe-AI modules in separate cmd windows.

```powershell
.\scripts\start-all.ps1                 # Release mode
.\scripts\start-all.ps1 -BuildMode debug     # Debug mode
.\scripts\start-all.ps1 -SkipOrchestrator    # Without orchestrator
.\scripts\start-all.ps1 -SkipAdapter         # Without Python adapter
.\scripts\start-all.ps1 -SkipBuild           # Skip build step
.\scripts\start-all.ps1 -Verbose             # Verbose output
```

**What it starts:**
- NATS message bus
- Context module
- Orchestrator module (optional)
- Sandbox module
- Python adapter (optional)

The interface opens in the current window for user input. Press Ctrl+C to stop all modules.

### run-tui-runner.ps1
Run the TUI Module Runner for managing all modules in a single terminal.

```powershell
.\scripts\run-tui-runner.ps1            # Start TUI runner
.\scripts\run-tui-runner.ps1 -Verbose   # Verbose output
```

**Keyboard shortcuts:**
- `q` - Quit
- `Ctrl+C` - Stop all and quit
- `Up/Down` - Select module
- `Enter` - Toggle selected module
- `s` - Start all modules
- `x` - Stop all modules

## Configuration

### config.json
Optional configuration file for customizable settings.

```json
{
  "nats": {
    "version": "2.10.27",
    "autoDownload": true
  },
  "build": {
    "cacheEnabled": true,
    "parallelBuilds": false
  },
  "test": {
    "timeoutSeconds": 15
  }
}
```

## Common Utilities

All scripts source `common.ps1` which provides:

**Logging Functions:**
- `Write-Step` - Section headers
- `Write-Ok` - Success messages
- `Write-Warn` - Warning messages
- `Write-Error` - Error messages
- `Write-Info` - Informational messages
- `Write-Muted` - Muted/secondary messages

**Progress Indicators:**
- `Write-Spinner` - Animated spinner for long operations
- `Write-Progress` - Progress bar with percentage
- `Clear-Progress` - Clear progress display

**Error Handling:**
- `Invoke-WithErrorHandling` - Centralized error handling
- `Invoke-WithRetry` - Retry logic with exponential backoff

**Process Management:**
- `Start-TrackedProcess` - Start process with tracking
- `Stop-TrackedProcess` - Stop tracked process
- `Stop-AllTrackedProcesses` - Stop all tracked processes

**Cleanup:**
- `Register-Cleanup` - Register cleanup handler
- `Invoke-Cleanup` - Execute all cleanup handlers

**Timing:**
- `Start-Timer` - Start a timer
- `Stop-Timer` - Stop timer and get duration
- `Format-Duration` - Format duration as human-readable string

## Troubleshooting

### NATS server not found
Run `.\scripts\download-nats.ps1` to download NATS automatically.

### Build fails
- Ensure Rust toolchain is installed: `cargo --version`
- Check for compilation errors in output
- Use `-Verbose` flag for detailed error messages

### Modules fail to start
- Check if NATS is running: `Get-Process nats-server`
- Verify port 4222 is available
- Check module logs for error messages

### Python adapter issues
- Ensure Python 3.8+ is installed: `python --version`
- Verify pip packages are installed
- Check API key configuration

## Cross-Platform Support

The scripts are designed to work on:
- **Windows**: PowerShell 5.1+ or PowerShell 7+ (recommended)
- **Linux**: PowerShell 7+ (pwsh) or Bash scripts
- **macOS**: PowerShell 7+ (pwsh) or Bash scripts

### Platform-Specific Scripts

**PowerShell Scripts (.ps1):**
- Primary scripting language for Windows
- Cross-platform support via PowerShell 7 (pwsh)
- All build, test, and launcher scripts available in PowerShell
- Recommended for Windows users

**Bash Scripts (.sh):**
- Available for Linux/macOS users who prefer Bash
- Limited set of scripts (load-test.sh, start-all.sh)
- Use these if PowerShell 7 is not available

**Usage by Platform:**

Windows:
```powershell
# PowerShell (recommended)
.\scripts\build-release.ps1

# Or via PowerShell 7
pwsh .\scripts/build-release.ps1

# Bash via Git Bash (limited support)
./scripts/load-test.sh
```

Linux/macOS:
```bash
# PowerShell 7 (recommended)
pwsh ./scripts/build-release.ps1

# Bash (limited scripts)
./scripts/load-test.sh
./scripts/start-all.sh
```

Note: Some features may require platform-specific adjustments. The scripts automatically detect the current platform and adapt accordingly.

## Contributing

When adding new scripts:
1. Source `common.ps1` at the top of the script
2. Use common logging functions for consistent output
3. Add error handling with `Invoke-WithErrorHandling`
4. Include verbose mode support
5. Add timing metrics for long operations
6. Register cleanup handlers for resources
7. Update this README with usage examples

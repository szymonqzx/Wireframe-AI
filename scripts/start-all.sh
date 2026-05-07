#!/usr/bin/env bash
# Start all Wireframe AI modules in separate terminal windows.
# Opens the interface (the "UI") at the end for user input.
#
# Usage:
#   ./scripts/start-all.sh                    # default mode (release)
#   ./scripts/start-all.sh --debug            # debug mode
#   ./scripts/start-all.sh --skip-orchestrator # without the orchestrator
#   ./scripts/start-all.sh --skip-adapter     # without the Python adapter
#   ./scripts/start-all.sh --skip-build       # reuse existing binaries
#
# Each module runs in its own terminal window, title-bar labeled so you can
# quickly spot which is which. Ctrl+C in the interface window exits cleanly.

set -e

# ── Configuration ─────────────────────────────────────────────────────────────
BUILD_MODE="release"
SKIP_ORCHESTRATOR=false
SKIP_ADAPTER=false
SKIP_BUILD=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            BUILD_MODE="debug"
            shift
            ;;
        --skip-orchestrator)
            SKIP_ORCHESTRATOR=true
            shift
            ;;
        --skip-adapter)
            SKIP_ADAPTER=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --debug            Build in debug mode (default: release)"
            echo "  --skip-orchestrator Skip starting the orchestrator"
            echo "  --skip-adapter     Skip starting the Python adapter"
            echo "  --skip-build       Skip building, use existing binaries"
            echo "  --help             Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# ── Paths ───────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
NATS_BIN="$ROOT_DIR/kernel/nats/nats-server"

BUILD_FLAG=""
if [ "$BUILD_MODE" = "release" ]; then
    BUILD_FLAG="--release"
fi

# Track what we started so we can clean up
declare -A STARTED_PIDS

# ── Colors ───────────────────────────────────────────────────────────────────
if [ -t 1 ]; then
    COLOR_INFO='\033[0;36m'
    COLOR_OK='\033[0;32m'
    COLOR_WARN='\033[0;33m'
    COLOR_ERR='\033[0;31m'
    COLOR_LABEL='\033[0;35m'
    COLOR_RESET='\033[0m'
else
    COLOR_INFO=''
    COLOR_OK=''
    COLOR_WARN=''
    COLOR_ERR=''
    COLOR_LABEL=''
    COLOR_RESET=''
fi

function write_step() {
    echo -e "\n  >> $1"
}

function write_ok() {
    echo -e "     ${COLOR_OK}$1${COLOR_RESET}"
}

function write_warn() {
    echo -e "     ${COLOR_WARN}$1${COLOR_RESET}"
}

function write_err() {
    echo -e "     ${COLOR_ERR}$1${COLOR_RESET}"
}

# ── Helper: open a new terminal window ───────────────────────────────────────
function start_terminal() {
    local title="$1"
    local command="$2"
    
    # Detect the terminal emulator
    if [ -n "$WSL_DISTRO_NAME" ] || [ -n "$WSL_INTEROP" ]; then
        # Windows Subsystem for Linux
        powershell.exe -Command "Start-Process cmd.exe -ArgumentList '/c start cmd /k \"$command\"'"
        write_ok "$title (WSL)"
    elif command -v gnome-terminal &> /dev/null; then
        # GNOME Terminal (Linux)
        gnome-terminal --title="$title" -- bash -c "$command; exec bash"
        write_ok "$title (gnome-terminal)"
    elif command -v xterm &> /dev/null; then
        # xterm (Linux)
        xterm -title "$title" -e bash -c "$command; exec bash" &
        write_ok "$title (xterm)"
    elif command -v osascript &> /dev/null; then
        # macOS Terminal
        osascript -e "tell application \"Terminal\" to do script \"$command\"" &> /dev/null
        write_ok "$title (Terminal.app)"
    elif command -v kitty &> /dev/null; then
        # kitty terminal
        kitty --title "$title" bash -c "$command; exec bash" &
        write_ok "$title (kitty)"
    else
        write_warn "No suitable terminal found, running in background"
        bash -c "$command" &
        write_ok "$title (background)"
    fi
}

# ── Preflight checks ─────────────────────────────────────────────────────────
echo ""
echo -e "  ${COLOR_LABEL}+--------------------------------------------------+${COLOR_RESET}"
echo -e "  ${COLOR_LABEL}|        Wireframe AI - Launch All Modules         |${COLOR_RESET}"
echo -e "  ${COLOR_LABEL}+--------------------------------------------------+${COLOR_RESET}"
echo ""

# Check NATS — download if missing
write_step "Checking prerequisites..."
if [ ! -f "$NATS_BIN" ]; then
    write_warn "NATS binary not found at kernel/nats/nats-server"
    echo "     Downloading NATS server..."
    if command -v curl &> /dev/null; then
        curl -sL https://github.com/nats-io/nats-server/releases/download/v2.10.22/nats-server-v2.10.22-linux-amd64.tar.gz -o /tmp/nats-server.tar.gz
        tar -xzf /tmp/nats-server.tar.gz -C /tmp
        mkdir -p "$ROOT_DIR/kernel/nats"
        mv /tmp/nats-server-v2.10.22-linux-amd64/nats-server "$NATS_BIN"
        rm /tmp/nats-server.tar.gz
        rm -rf /tmp/nats-server-v2.10.22-linux-amd64
        write_ok "NATS binary downloaded"
    elif command -v wget &> /dev/null; then
        wget -qO- https://github.com/nats-io/nats-server/releases/download/v2.10.22/nats-server-v2.10.22-linux-amd64.tar.gz -O /tmp/nats-server.tar.gz
        tar -xzf /tmp/nats-server.tar.gz -C /tmp
        mkdir -p "$ROOT_DIR/kernel/nats"
        mv /tmp/nats-server-v2.10.22-linux-amd64/nats-server "$NATS_BIN"
        rm /tmp/nats-server.tar.gz
        rm -rf /tmp/nats-server-v2.10.22-linux-amd64
        write_ok "NATS binary downloaded"
    else
        write_warn "Neither curl nor wget found. Cannot download NATS automatically."
        write_warn "Start NATS manually (docker run -p 4222:4222 nats:latest) and re-run."
        write_warn "Continuing anyway - modules will fail to connect."
    fi
else
    write_ok "NATS binary found at kernel/nats/nats-server"
fi

# Check Rust
if ! command -v cargo &> /dev/null; then
    write_err "Rust (cargo) not found. Install from https://rustup.rs"
    exit 1
fi
write_ok "Rust toolchain found"

# Check Python + install packages (only if adapter is enabled)
if [ "$SKIP_ADAPTER" = false ]; then
    if ! command -v python3 &> /dev/null && ! command -v python &> /dev/null; then
        write_warn "Python not found - adapter will be skipped"
        SKIP_ADAPTER=true
    else
        PYTHON_CMD=$(command -v python3 || command -v python)
        write_ok "Python found ($PYTHON_CMD)"

        # Install SDK + adapter packages (idempotent — pip skips if already installed)
        SDK_DIR="$ROOT_DIR/sdk/agentic-sdk-py"
        ADAPTER_DIR="$ROOT_DIR/adapter/python"

        pushd "$ROOT_DIR" > /dev/null
        echo "     Installing agentic-sdk-py..."
        if $PYTHON_CMD -m pip install -e "$SDK_DIR" -q; then
            write_ok "agentic-sdk-py ready"
        else
            write_warn "Failed to install agentic-sdk-py"
        fi

        echo "     Installing wireframe-ai-adapter..."
        if $PYTHON_CMD -m pip install -e "$ADAPTER_DIR" -q; then
            write_ok "wireframe-ai-adapter ready"
        else
            write_warn "Failed to install wireframe-ai-adapter"
        fi
        popd > /dev/null
    fi
fi

# ── Build modules ────────────────────────────────────────────────────────────
if [ "$SKIP_BUILD" = false ]; then
    write_step "Building modules ($BUILD_MODE mode)..."
    echo "     (this may take a while the first time)" | sed "s/^/     ${COLOR_WARN}/" | sed "s/$/${COLOR_RESET}/"

    BUILD_START=$(date +%s)

    pushd "$ROOT_DIR" > /dev/null
    if [ "$BUILD_MODE" = "release" ]; then
        cargo build --release
    else
        cargo build
    fi
    BUILD_END=$(date +%s)
    BUILD_TIME=$((BUILD_END - BUILD_START))
    popd > /dev/null

    write_ok "Build finished in ${BUILD_TIME}s"
else
    write_step "Skipping build (--skip-build)"
fi

# ── Start NATS ──────────────────────────────────────────────────────────────
write_step "Starting NATS message bus..."

if pgrep -x "nats-server" > /dev/null; then
    write_ok "NATS already running"
elif [ -f "$NATS_BIN" ]; then
    "$NATS_BIN" &
    NATS_PID=$!
    STARTED_PIDS["nats-server"]=$NATS_PID
    write_ok "NATS started (PID $NATS_PID)"
else
    write_warn "NATS binary not available - start NATS manually"
fi

sleep 2

# ── Start modules ────────────────────────────────────────────────────────────
write_step "Starting modules..."

CARGO_RUN="cargo run $BUILD_FLAG -p"

# Context module
start_terminal "wireframe-context" "cd '$ROOT_DIR' && $CARGO_RUN wireframe-ai-context-core"
sleep 1

# Orchestrator (optional)
if [ "$SKIP_ORCHESTRATOR" = false ]; then
    start_terminal "wireframe-orchestrator" "cd '$ROOT_DIR' && $CARGO_RUN wireframe-ai-orchestrator-core"
    sleep 1
else
    write_ok "Orchestrator skipped (--skip-orchestrator)"
fi

# Sandbox
start_terminal "wireframe-sandbox" "cd '$ROOT_DIR' && $CARGO_RUN wireframe-ai-sandbox-core"
sleep 1

# Python adapter (optional) — via installed entry point
if [ "$SKIP_ADAPTER" = false ]; then
    PYTHON_CMD=$(command -v python3 || command -v python)
    start_terminal "wireframe-adapter-python" "wireframe-ai-adapter-python"
    sleep 1
else
    write_ok "Python adapter skipped (--skip-adapter)"
fi

# ── Summary ──────────────────────────────────────────────────────────────────
write_step "Launch summary"
echo ""
echo -e "     ${COLOR_LABEL}Module              Window Title                PID${COLOR_RESET}"
echo -e "     ${COLOR_LABEL}------------------------------------------------------${COLOR_RESET}"
for title in "${!STARTED_PIDS[@]}"; do
    pid="${STARTED_PIDS[$title]}"
    printf "     %-20s %-26s %s\n" "$title" "$title" "$pid"
done

echo ""
echo -e "  ${COLOR_LABEL}-----------------------------------------------------${COLOR_RESET}"
echo ""

# ── Open the interface (the "UI") ──────────────────────────────────────────
write_step "Opening interface (the UI)"
echo ""
echo "  Type or paste your task below. Press Ctrl+D to submit (or pass a task as an argument)."
echo ""

# Run the interface in the current window so the user can interact
pushd "$ROOT_DIR" > /dev/null
if [ "$BUILD_MODE" = "release" ]; then
    cargo run --release -p wireframe-ai-interface "$@"
else
    cargo run -p wireframe-ai-interface "$@"
fi
popd > /dev/null

# ── Cleanup ──────────────────────────────────────────────────────────────────
write_step "Shutting down..."

# Kill all tracked processes
for title in "${!STARTED_PIDS[@]}"; do
    pid="${STARTED_PIDS[$title]}"
    if kill -0 "$pid" 2>/dev/null; then
        kill "$pid"
        write_ok "Stopped $title (PID $pid)"
    fi
done

echo ""
echo -e "  ${COLOR_OK}All modules stopped. Goodbye!${COLOR_RESET}"
echo ""

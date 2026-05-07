#!/bin/bash
# Session start hook for Wireframe-AI
# This script runs when a new Devin CLI session starts

echo "Wireframe-AI session started"
echo "Working directory: $(pwd)"

# Check if Rust toolchain is available
if command -v cargo &> /dev/null; then
    echo "Rust toolchain: $(rustc --version)"
else
    echo "Warning: Rust toolchain not found"
fi

# Check if NATS server is running
if pgrep -x "nats-server" > /dev/null; then
    echo "NATS server: Running"
else
    echo "NATS server: Not running"
fi

# Display project info
echo "Wireframe-AI: Modular event-driven agentic system"
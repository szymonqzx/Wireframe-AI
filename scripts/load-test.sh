#!/bin/bash
# Load Testing Script for Wireframe-AI
#
# This script performs load testing on the Wireframe-AI system
# by submitting concurrent tasks and measuring performance.

set -e

# Configuration
NATS_URL="${NATS_URL:-nats://localhost:4222}"
CONCURRENT_TASKS="${CONCURRENT_TASKS:-10}"
TASKS_PER_WORKER="${TASKS_PER_WORKER:-10}"
TOTAL_TASKS=$((CONCURRENT_TASKS * TASKS_PER_WORKER))
TASK_DURATION="${TASK_DURATION:-30}"
OUTPUT_DIR="${OUTPUT_DIR:-./load-test-results}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Wireframe-AI Load Testing Script"
echo "================================"
echo "NATS URL: $NATS_URL"
echo "Concurrent Workers: $CONCURRENT_TASKS"
echo "Tasks per Worker: $TASKS_PER_WORKER"
echo "Total Tasks: $TOTAL_TASKS"
echo "Task Duration: ${TASK_DURATION}s"
echo "Output Directory: $OUTPUT_DIR"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Check if NATS is available
echo "Checking NATS connection..."
if ! nats pub test.test "ping" -s "$NATS_URL" > /dev/null 2>&1; then
    echo -e "${RED}Error: Cannot connect to NATS at $NATS_URL${NC}"
    echo "Make sure NATS is running and accessible"
    exit 1
fi
echo -e "${GREEN}NATS connection successful${NC}"
echo ""

# Function to submit a task
submit_task() {
    local worker_id=$1
    local task_id=$2
    local start_time=$(date +%s%N)
    
    # Create a test task
    local task_data=$(cat <<EOF
{
  "session_id": "load-test-session-$worker_id-$task_id",
  "user_input": "Test task $task_id",
  "submitted_at": $(date +%s)
}
EOF
)
    
    # Publish to NATS
    if nats pub task.submitted "$task_data" -s "$NATS_URL" > /dev/null 2>&1; then
        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 ))
        echo "$worker_id,$task_id,$duration,success" >> "$OUTPUT_DIR/results_$TIMESTAMP.csv"
    else
        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 ))
        echo "$worker_id,$task_id,$duration,failed" >> "$OUTPUT_DIR/results_$TIMESTAMP.csv"
    fi
}

# Function to run a worker
run_worker() {
    local worker_id=$1
    echo "Worker $worker_id started"
    
    for ((task_id=1; task_id<=TASKS_PER_WORKER; task_id++)); do
        submit_task "$worker_id" "$task_id"
        sleep 0.1  # Small delay between tasks
    done
    
    echo "Worker $worker_id completed"
}

# Create results header
echo "worker_id,task_id,duration_ms,status" > "$OUTPUT_DIR/results_$TIMESTAMP.csv"

echo "Starting load test..."
echo "Starting $CONCURRENT_TASKS concurrent workers, each submitting $TASKS_PER_WORKER tasks"
echo ""

# Start workers in background
worker_pids=()
for ((worker_id=1; worker_id<=CONCURRENT_TASKS; worker_id++)); do
    run_worker "$worker_id" &
    worker_pids+=($!)
done

# Wait for all workers to complete
echo "Waiting for all workers to complete..."
for pid in "${worker_pids[@]}"; do
    wait "$pid"
done

echo ""
echo -e "${GREEN}Load test completed${NC}"
echo ""

# Analyze results
echo "Analyzing results..."
total_submitted=$(wc -l < "$OUTPUT_DIR/results_$TIMESTAMP.csv")
total_submitted=$((total_submitted - 1))  # Subtract header

successful=$(grep ",success$" "$OUTPUT_DIR/results_$TIMESTAMP.csv" | wc -l)
failed=$(grep ",failed$" "$OUTPUT_DIR/results_$TIMESTAMP.csv" | wc -l)

# Calculate statistics
if [ $successful -gt 0 ]; then
    total_duration=$(awk -F',' '$3 ~ /^[0-9]+/ {sum+=$3} END {print sum}' "$OUTPUT_DIR/results_$TIMESTAMP.csv")
    avg_duration=$((total_duration / successful))
    max_duration=$(awk -F',' '$3 ~ /^[0-9]+/ {if ($3 > max) max=$3} END {print max}' "$OUTPUT_DIR/results_$TIMESTAMP.csv")
    min_duration=$(awk -F',' '$3 ~ /^[0-9]+/ {if ($3 < min || min == 0) min=$3} END {print min}' "$OUTPUT_DIR/results_$TIMESTAMP.csv")
fi

# Print summary
echo "Load Test Summary"
echo "================"
echo "Total Tasks Submitted: $total_submitted"
echo "Successful: $successful"
echo "Failed: $failed"
echo "Success Rate: $(echo "scale=2; $successful * 100 / $total_submitted" | bc)%"
echo ""

if [ $successful -gt 0 ]; then
    echo "Performance Metrics"
    echo "------------------"
    echo "Average Duration: ${avg_duration}ms"
    echo "Min Duration: ${min_duration}ms"
    echo "Max Duration: ${max_duration}ms"
    echo "Throughput: $(echo "scale=2; $successful / $TASK_DURATION" | bc) tasks/sec"
    echo ""
fi

# Generate report
cat > "$OUTPUT_DIR/report_$TIMESTAMP.md" <<EOF
# Load Test Report

**Date:** $(date)
**Configuration:**
- NATS URL: $NATS_URL
- Concurrent Workers: $CONCURRENT_TASKS
- Tasks per Worker: $TASKS_PER_WORKER
- Total Tasks: $TOTAL_TASKS
- Task Duration: ${TASK_DURATION}s

## Results

- Total Tasks Submitted: $total_submitted
- Successful: $successful
- Failed: $failed
- Success Rate: $(echo "scale=2; $successful * 100 / $total_submitted" | bc)%

## Performance Metrics

- Average Duration: ${avg_duration}ms
- Min Duration: ${min_duration}ms
- Max Duration: ${max_duration}ms
- Throughput: $(echo "scale=2; $successful / $TASK_DURATION" | bc) tasks/sec

## Recommendations

EOF

# Add recommendations based on results
if [ $failed -gt 0 ]; then
    cat >> "$OUTPUT_DIR/report_$TIMESTAMP.md" <<EOF
- **High failure rate detected**: Consider increasing resource limits or optimizing plugin performance
EOF
fi

if [ $avg_duration -gt 1000 ]; then
    cat >> "$OUTPUT_DIR/report_$TIMESTAMP.md" <<EOF
- **High latency detected**: Consider implementing caching or optimizing database queries
EOF
fi

if [ $successful -eq $total_submitted ]; then
    cat >> "$OUTPUT_DIR/report_$TIMESTAMP.md" <<EOF
- **All tasks successful**: System is performing well under load
EOF
fi

echo "Report saved to: $OUTPUT_DIR/report_$TIMESTAMP.md"
echo "Raw data saved to: $OUTPUT_DIR/results_$TIMESTAMP.csv"
echo ""

# Cleanup
echo "Cleaning up worker processes..."
for pid in "${worker_pids[@]}"; do
    kill "$pid" 2>/dev/null || true
done

echo -e "${GREEN}Load test complete${NC}"

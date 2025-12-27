#!/bin/bash
# Trinity Brain Daemon - For Multi-Day Autonomous Operation
#
# Usage:
#   ./trinity-daemon.sh start   # Start brain in background
#   ./trinity-daemon.sh stop    # Stop brain
#   ./trinity-daemon.sh status  # Check if running
#   ./trinity-daemon.sh logs    # View logs
#
# This script enables Trinity to work autonomously for days.

BRAIN_DIR="/home/joshua/antigravity/trinity-genesis"
PID_FILE="/tmp/trinity-brain.pid"
LOG_FILE="/home/joshua/.trinity/brain.log"

# Ensure log directory exists
mkdir -p /home/joshua/.trinity

case "$1" in
    start)
        if [ -f "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
            echo "Trinity Brain is already running (PID: $(cat "$PID_FILE"))"
            exit 1
        fi
        
        echo "Starting Trinity Brain..."
        cd "$BRAIN_DIR" || exit 1
        
        # Load VRAM as configured for Strix Halo
        export HSA_OVERRIDE_GFX_VERSION=11.0.3
        export GGML_CUDA_ENABLE_UNIFIED_MEMORY=1
        
        nohup cargo run -p trinity-brain --release >> "$LOG_FILE" 2>&1 &
        echo $! > "$PID_FILE"
        
        echo "Trinity Brain started (PID: $(cat "$PID_FILE"))"
        echo "Logs: $LOG_FILE"
        ;;
        
    stop)
        if [ -f "$PID_FILE" ]; then
            PID=$(cat "$PID_FILE")
            if kill -0 "$PID" 2>/dev/null; then
                echo "Stopping Trinity Brain (PID: $PID)..."
                kill "$PID"
                rm -f "$PID_FILE"
                echo "Stopped."
            else
                echo "Trinity Brain is not running (stale PID file)"
                rm -f "$PID_FILE"
            fi
        else
            echo "Trinity Brain is not running"
        fi
        ;;
        
    status)
        if [ -f "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
            echo "Trinity Brain is running (PID: $(cat "$PID_FILE"))"
            echo "Uptime: $(ps -o etime= -p "$(cat "$PID_FILE")" | tr -d ' ')"
        else
            echo "Trinity Brain is not running"
        fi
        ;;
        
    logs)
        if [ -f "$LOG_FILE" ]; then
            tail -f "$LOG_FILE"
        else
            echo "No log file found at $LOG_FILE"
        fi
        ;;
        
    *)
        echo "Usage: $0 {start|stop|status|logs}"
        exit 1
        ;;
esac

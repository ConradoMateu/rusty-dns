#!/bin/bash
# monitor-updates.sh - Monitor DDNS updates in real-time

set -euo pipefail

echo "📡 Monitoring DDNS updates (Ctrl+C to stop)"
echo "=========================================="
echo ""

journalctl -u rusty-dns -f | grep --line-buffered -E "IP changed|Update successful|Update failed|Error"

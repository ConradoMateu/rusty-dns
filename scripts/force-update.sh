#!/bin/bash
# force-update.sh - Force immediate DDNS update

set -euo pipefail

echo "🔄 Forcing DDNS update..."
echo ""

rusty-dns update --force

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Update successful"
    echo ""
    rusty-dns status
else
    echo ""
    echo "❌ Update failed"
    exit 1
fi

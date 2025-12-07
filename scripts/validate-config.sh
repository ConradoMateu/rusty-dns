#!/bin/bash
# validate-config.sh - Validate rusty-dns configuration

set -euo pipefail

CONFIG_FILE="${1:-$HOME/.config/rusty-dns/config.toml}"

echo "✅ Validating rusty-dns configuration..."
echo "Config file: $CONFIG_FILE"
echo ""

if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ Config file not found: $CONFIG_FILE"
    exit 1
fi

# Test config loading (rusty-dns has validate subcommand)
if rusty-dns validate --config "$CONFIG_FILE" 2>/dev/null; then
    echo "✅ Configuration is valid"
else
    echo "❌ Configuration is invalid"
    echo ""
    echo "Check for:"
    echo "  - Correct TOML syntax"
    echo "  - Valid provider configuration"
    echo "  - Environment variables set"
    exit 1
fi

#!/bin/bash
# test-provider.sh - Test DDNS provider connectivity

set -euo pipefail

PROVIDER="$1"

if [ -z "$PROVIDER" ]; then
    echo "Usage: $0 <provider>"
    echo "Providers: cloudflare, duckdns, namecheap, godaddy"
    exit 1
fi

echo "🔍 Testing $PROVIDER connectivity..."
echo ""

case "$PROVIDER" in
    cloudflare)
        if [ -z "${CF_API_TOKEN:-}" ]; then
            echo "❌ CF_API_TOKEN not set"
            exit 1
        fi
        curl -X GET "https://api.cloudflare.com/client/v4/user/tokens/verify" \
            -H "Authorization: Bearer $CF_API_TOKEN" \
            -H "Content-Type: application/json"
        ;;
    duckdns)
        if [ -z "${DUCKDNS_TOKEN:-}" ]; then
            echo "❌ DUCKDNS_TOKEN not set"
            exit 1
        fi
        curl "https://www.duckdns.org/update?domains=test&token=$DUCKDNS_TOKEN&ip=1.2.3.4&verbose=true"
        ;;
    namecheap)
        echo "⚠️  Namecheap test requires actual domain - check logs"
        ;;
    godaddy)
        if [ -z "${GODADDY_API_KEY:-}" ] || [ -z "${GODADDY_API_SECRET:-}" ]; then
            echo "❌ GODADDY_API_KEY or GODADDY_API_SECRET not set"
            exit 1
        fi
        curl -X GET "https://api.godaddy.com/v1/domains" \
            -H "Authorization: sso-key $GODADDY_API_KEY:$GODADDY_API_SECRET"
        ;;
    *)
        echo "❌ Unknown provider: $PROVIDER"
        exit 1
        ;;
esac

echo ""
echo "✅ Test complete"

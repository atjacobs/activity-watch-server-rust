#!/bin/bash
# Generate API key hash for ActivityWatch server authentication
#
# Usage:
#   ./generate-api-key.sh [api-key]
#
# If no api-key is provided, a random one will be generated.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "ActivityWatch API Key Generator"
echo "================================"
echo ""

# Check if API key was provided
if [ -z "$1" ]; then
    echo "No API key provided. Generating a random secure key..."
    echo ""

    # Generate a random API key (32 characters, URL-safe)
    if command -v openssl &> /dev/null; then
        API_KEY=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-32)
    elif command -v /dev/urandom &> /dev/null; then
        API_KEY=$(head -c 32 /dev/urandom | base64 | tr -d "=+/" | cut -c1-32)
    else
        echo -e "${RED}Error: Could not generate random key. Please provide one as an argument.${NC}"
        echo "Usage: $0 [your-api-key]"
        exit 1
    fi

    echo -e "${GREEN}Generated API Key:${NC}"
    echo -e "${YELLOW}$API_KEY${NC}"
    echo ""
    echo -e "${RED}⚠️  IMPORTANT: Save this key securely! You will need it to access the server.${NC}"
    echo ""
else
    API_KEY="$1"
    echo "Using provided API key..."
    echo ""
fi

# Generate SHA256 hash
if command -v sha256sum &> /dev/null; then
    HASH=$(echo -n "$API_KEY" | sha256sum | awk '{print $1}')
elif command -v shasum &> /dev/null; then
    HASH=$(echo -n "$API_KEY" | shasum -a 256 | awk '{print $1}')
else
    echo -e "${RED}Error: sha256sum or shasum not found.${NC}"
    exit 1
fi

echo -e "${GREEN}SHA256 Hash:${NC}"
echo "$HASH"
echo ""

echo "Configuration Instructions:"
echo "==========================="
echo ""
echo "1. Open your ActivityWatch config file:"
echo "   - Linux: ~/.config/activitywatch/aw-server-rust/config.toml"
echo "   - macOS: ~/Library/Application Support/activitywatch/aw-server-rust/config.toml"
echo "   - Windows: %APPDATA%\\activitywatch\\aw-server-rust\\config.toml"
echo ""
echo "2. Add or update the [security] section:"
echo ""
echo "[security]"
echo "require_auth = true"
echo "allow_remote = true"
echo "api_keys = [\"$HASH\"]"
echo ""
echo "3. To allow remote connections, also update the address:"
echo ""
echo "address = \"0.0.0.0\"  # Listen on all network interfaces"
echo ""
echo "4. (Optional) Enable TLS for encrypted connections:"
echo ""
echo "[tls]"
echo "enabled = true"
echo "cert = \"/path/to/cert.pem\""
echo "key = \"/path/to/key.pem\""
echo ""
echo "5. Restart the ActivityWatch server"
echo ""
echo "Client Usage:"
echo "============="
echo "When connecting from a client, include the API key in your requests:"
echo ""
echo "  Authorization: Bearer $API_KEY"
echo ""
echo "or"
echo ""
echo "  X-API-Key: $API_KEY"
echo ""

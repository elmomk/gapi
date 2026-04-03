#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CACHE_FILE="$PROJECT_DIR/.garmin-setup-cache"

# Load .env from project root
if [ -f "$PROJECT_DIR/.env" ]; then
    set -a
    source "$PROJECT_DIR/.env"
    set +a
fi

# Extract test API key from API_KEYS if GARMIN_API_KEY not set directly
if [ -z "${GARMIN_API_KEY:-}" ] && [ -n "${API_KEYS:-}" ]; then
    # Use the last key (test key)
    GARMIN_API_KEY=$(echo "$API_KEYS" | tr ',' '\n' | tail -1 | cut -d: -f1)
fi

# Auto-detect Tailscale IP if running in Docker and no URL set
if [ -z "${GARMIN_API_URL:-}" ]; then
    TS_IP=$(docker compose -f "$PROJECT_DIR/docker-compose.yaml" exec -T tailscale tailscale ip -4 2>/dev/null || true)
    if [ -n "$TS_IP" ]; then
        GARMIN_API_URL="http://${TS_IP}:${PORT:-3000}"
    fi
fi
BASE_URL="${GARMIN_API_URL:-http://localhost:${PORT:-3000}}"
API_KEY="${GARMIN_API_KEY:-}"

if [ -z "$API_KEY" ]; then
    echo "Error: No API key found. Set GARMIN_API_KEY or add API_KEYS to .env"
    exit 1
fi

# Load cached values
CACHED_USER_ID=""
CACHED_EMAIL=""
if [ -f "$CACHE_FILE" ]; then
    source "$CACHE_FILE"
    CACHED_USER_ID="${USER_ID:-}"
    CACHED_EMAIL="${GARMIN_EMAIL:-}"
fi

echo "=== Garmin Account Setup ==="
echo "API: $BASE_URL"
echo ""

# Prompt with cached defaults
if [ -n "$CACHED_USER_ID" ]; then
    read -rp "User ID (UUID) [$CACHED_USER_ID]: " USER_ID
    USER_ID="${USER_ID:-$CACHED_USER_ID}"
else
    read -rp "User ID (UUID): " USER_ID
fi

if [ -n "$CACHED_EMAIL" ]; then
    read -rp "Garmin email [$CACHED_EMAIL]: " GARMIN_EMAIL
    GARMIN_EMAIL="${GARMIN_EMAIL:-$CACHED_EMAIL}"
else
    read -rp "Garmin email: " GARMIN_EMAIL
fi

read -rsp "Garmin password: " GARMIN_PASS
echo ""

# Save to cache
cat > "$CACHE_FILE" <<EOF
USER_ID="$USER_ID"
GARMIN_EMAIL="$GARMIN_EMAIL"
EOF
chmod 600 "$CACHE_FILE"

echo ""
echo "Registering credentials..."
RESP=$(curl -s -X POST \
    -H "X-API-Key: $API_KEY" \
    -H "Content-Type: application/json" \
    -d "{\"garmin_username\":\"$GARMIN_EMAIL\",\"garmin_password\":\"$GARMIN_PASS\"}" \
    "$BASE_URL/api/v1/users/$USER_ID/credentials")

STATUS=$(echo "$RESP" | grep -o '"status":"[^"]*"' | cut -d'"' -f4)
MESSAGE=$(echo "$RESP" | grep -o '"message":"[^"]*"' | cut -d'"' -f4)

echo "Status: $STATUS"
echo "Message: $MESSAGE"

if [ "$STATUS" = "connected" ]; then
    echo ""
    echo "Login successful -- no MFA required."
    echo "You can now trigger a sync:"
    echo "  curl -X POST -H 'X-API-Key: $API_KEY' $BASE_URL/api/v1/users/$USER_ID/sync"
    exit 0
fi

if [ "$STATUS" != "mfa_required" ]; then
    echo ""
    echo "Login failed. Check your credentials."
    exit 1
fi

# MFA retry loop
while true; do
    echo ""
    echo "MFA required. Check your authenticator app."
    read -rp "MFA code (or 'q' to quit): " MFA_CODE

    if [ "$MFA_CODE" = "q" ]; then
        echo "Aborted."
        exit 1
    fi

    echo ""
    echo "Submitting MFA code..."
    RESP=$(curl -s -X POST \
        -H "X-API-Key: $API_KEY" \
        -H "Content-Type: application/json" \
        -d "{\"mfa_code\":\"$MFA_CODE\"}" \
        "$BASE_URL/api/v1/users/$USER_ID/mfa")

    STATUS=$(echo "$RESP" | grep -o '"status":"[^"]*"' | cut -d'"' -f4)
    MESSAGE=$(echo "$RESP" | grep -o '"message":"[^"]*"' | cut -d'"' -f4)

    echo "Status: $STATUS"
    echo "Message: $MESSAGE"

    if [ "$STATUS" = "connected" ]; then
        echo ""
        echo "MFA verified. Account is connected."
        echo "Background sync will start within the hour, or trigger manually:"
        echo "  curl -X POST -H 'X-API-Key: $API_KEY' $BASE_URL/api/v1/users/$USER_ID/sync"
        exit 0
    fi

    echo ""
    echo "MFA failed. Try again with a fresh code."
done

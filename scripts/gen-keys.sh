#!/usr/bin/env bash
set -euo pipefail

# Generate API keys and MASTER_KEY for garmin_api, then update .env

ENV_FILE="${1:-.env}"

if [ ! -f "$ENV_FILE" ]; then
    echo "Error: $ENV_FILE not found. Run from the project root or pass the path."
    exit 1
fi

MASTER_KEY=$(openssl rand -hex 32)
GORILLA_KEY=$(openssl rand -hex 32)
LIFE_KEY=$(openssl rand -hex 32)
TEST_KEY=$(openssl rand -hex 32)

# Update .env
sed -i "s|^MASTER_KEY=.*|MASTER_KEY=${MASTER_KEY}|" "$ENV_FILE"
sed -i "s|^API_KEYS=.*|API_KEYS=${GORILLA_KEY}:gorilla_coach,${LIFE_KEY}:life_manager,${TEST_KEY}:test|" "$ENV_FILE"

echo "Keys generated and written to $ENV_FILE"
echo ""
echo "MASTER_KEY = ${MASTER_KEY}"
echo ""
echo "gorilla_coach API key: ${GORILLA_KEY}"
echo "  -> Add to gorilla_coach .env: GARMIN_API_KEY=${GORILLA_KEY}"
echo ""
echo "life_manager API key:  ${LIFE_KEY}"
echo "  -> Add to life_manager .env:  GARMIN_API_KEY=${LIFE_KEY}"
echo ""
echo "test API key:          ${TEST_KEY}"
echo "  -> For manual testing: curl -H 'X-API-Key: ${TEST_KEY}' ..."

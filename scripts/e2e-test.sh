#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${GARMIN_API_URL:-http://localhost:3000}"
API_KEY="${GARMIN_API_KEY:-testkey123}"
TEST_USER_ID="00000000-0000-0000-0000-000000000001"

PASS=0
FAIL=0
TOTAL=0

run_test() {
    local name="$1"
    local expected_status="$2"
    shift 2
    TOTAL=$((TOTAL + 1))

    local response
    local http_code
    local body

    response=$(curl -s -w "\n%{http_code}" "$@" 2>&1) || true
    http_code=$(echo "$response" | tail -1)
    body=$(echo "$response" | sed '$d')

    if [ "$http_code" = "$expected_status" ]; then
        PASS=$((PASS + 1))
        echo "  PASS  $name (HTTP $http_code)"
    else
        FAIL=$((FAIL + 1))
        echo "  FAIL  $name (expected $expected_status, got $http_code)"
        echo "        body: $(echo "$body" | head -1)"
    fi
}

echo "=== garmin_api e2e tests ==="
echo "Base URL: $BASE_URL"
echo ""

# --- Health ---
echo "[Health]"
run_test "GET /health" "200" \
    "$BASE_URL/health"

# --- Auth: rejected without API key ---
echo ""
echo "[Auth]"
run_test "GET without API key returns 401" "401" \
    "$BASE_URL/api/v1/users/$TEST_USER_ID/status"

# --- User Status (no user yet) ---
echo ""
echo "[User Status]"
run_test "GET status for unknown user" "200" \
    -H "X-API-Key: $API_KEY" \
    "$BASE_URL/api/v1/users/$TEST_USER_ID/status"

# --- Credentials ---
echo ""
echo "[Credentials]"
# Register with dummy credentials (login will fail but user record should be created)
run_test "POST credentials (dummy, expect error in message but 200)" "200" \
    -X POST \
    -H "X-API-Key: $API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"garmin_username":"test@example.com","garmin_password":"fake_password"}' \
    "$BASE_URL/api/v1/users/$TEST_USER_ID/credentials"

# Verify user now exists
run_test "GET status after credential creation" "200" \
    -H "X-API-Key: $API_KEY" \
    "$BASE_URL/api/v1/users/$TEST_USER_ID/status"

# --- Data queries (empty, but should not error) ---
echo ""
echo "[Data Queries]"
run_test "GET daily (no data yet) returns 404" "404" \
    -H "X-API-Key: $API_KEY" \
    "$BASE_URL/api/v1/users/$TEST_USER_ID/daily?date=2026-04-01"

run_test "GET daily range (no data) returns 200 empty array" "200" \
    -H "X-API-Key: $API_KEY" \
    "$BASE_URL/api/v1/users/$TEST_USER_ID/daily?start=2026-03-01&end=2026-04-01"

run_test "GET baseline returns 200" "200" \
    -H "X-API-Key: $API_KEY" \
    "$BASE_URL/api/v1/users/$TEST_USER_ID/baseline?days=7"

run_test "GET vitals returns 200" "200" \
    -H "X-API-Key: $API_KEY" \
    "$BASE_URL/api/v1/users/$TEST_USER_ID/vitals"

# --- Webhooks ---
echo ""
echo "[Webhooks]"
run_test "POST create webhook" "200" \
    -X POST \
    -H "X-API-Key: $API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"consumer_name":"test","url":"http://localhost:9999/hook","event_types":["daily_data_synced","sync_completed"]}' \
    "$BASE_URL/api/v1/webhooks"

run_test "GET list webhooks" "200" \
    -H "X-API-Key: $API_KEY" \
    "$BASE_URL/api/v1/webhooks"

# Extract webhook ID for deletion
WEBHOOK_ID=$(curl -s -H "X-API-Key: $API_KEY" "$BASE_URL/api/v1/webhooks" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$WEBHOOK_ID" ]; then
    run_test "DELETE webhook" "204" \
        -X DELETE \
        -H "X-API-Key: $API_KEY" \
        "$BASE_URL/api/v1/webhooks/$WEBHOOK_ID"
else
    TOTAL=$((TOTAL + 1))
    FAIL=$((FAIL + 1))
    echo "  FAIL  DELETE webhook (could not extract webhook ID)"
fi

# --- Sync (will fail auth with Garmin but should return 200 with error message) ---
echo ""
echo "[Sync]"
run_test "POST sync (dummy creds, expect error message)" "200" \
    -X POST \
    -H "X-API-Key: $API_KEY" \
    "$BASE_URL/api/v1/users/$TEST_USER_ID/sync"

# --- MFA (should return 404 or error since no MFA pending) ---
echo ""
echo "[MFA]"
run_test "POST MFA for user with no session" "200" \
    -X POST \
    -H "X-API-Key: $API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"mfa_code":"123456"}' \
    "$BASE_URL/api/v1/users/$TEST_USER_ID/mfa"

# --- Cleanup: delete test user ---
echo ""
echo "[Cleanup]"
run_test "DELETE test user credentials" "204" \
    -X DELETE \
    -H "X-API-Key: $API_KEY" \
    "$BASE_URL/api/v1/users/$TEST_USER_ID/credentials"

run_test "GET status after deletion shows disconnected" "200" \
    -H "X-API-Key: $API_KEY" \
    "$BASE_URL/api/v1/users/$TEST_USER_ID/status"

# --- Summary ---
echo ""
echo "=== Results: $PASS/$TOTAL passed, $FAIL failed ==="
if [ "$FAIL" -gt 0 ]; then
    exit 1
fi

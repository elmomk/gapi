#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# This script migrates an existing Garmin session from gorilla_coach to garmin_api.
# It decrypts the session with gorilla_coach's MASTER_KEY, then imports it into
# garmin_api's SQLite DB re-encrypted with garmin_api's MASTER_KEY.
#
# Requires: a small Rust helper (compiled inline below) since the encryption
# is ChaCha20Poly1305 which bash can't do natively.

GORILLA_DIR="${GORILLA_COACH_DIR:-/home/mo/data/Documents/git/gorilla_coach}"
GARMIN_DIR="$PROJECT_DIR"

# Load keys
GORILLA_KEY=$(grep "^MASTER_KEY=" "$GORILLA_DIR/.env" | cut -d= -f2-)
GARMIN_KEY=$(grep "^MASTER_KEY=" "$GARMIN_DIR/.env" | cut -d= -f2-)
USER_ID="${1:?Usage: migrate-session.sh <user-uuid>}"

echo "=== Migrate Garmin Session ==="
echo "From: gorilla_coach"
echo "To:   garmin_api"
echo "User: $USER_ID"
echo ""

# Extract from gorilla_coach DB
echo "Extracting session from gorilla_coach..."
ROW=$(docker compose -f "$GORILLA_DIR/docker-compose.yaml" exec -T db \
    psql -U gorilla gorilla_hq -t -A -F'|' -c \
    "SELECT garmin_oauth_token, garmin_oauth_token_nonce, garmin_username, encrypted_garmin_password, nonce FROM user_settings WHERE user_id = '$USER_ID';")

if [ -z "$ROW" ]; then
    echo "Error: No Garmin credentials found for user $USER_ID in gorilla_coach"
    exit 1
fi

IFS='|' read -r ENC_SESSION SESSION_NONCE GARMIN_USERNAME ENC_PASSWORD PASS_NONCE <<< "$ROW"

echo "  Username: $GARMIN_USERNAME"
echo "  Session:  ${#ENC_SESSION} chars encrypted"
echo "  Password: ${#ENC_PASSWORD} chars encrypted"

# Build a tiny Rust helper to decrypt/re-encrypt
HELPER_DIR=$(mktemp -d)
mkdir -p "$HELPER_DIR/src"
cat > "$HELPER_DIR/src/main.rs" << 'RUSTEOF'
use base64::prelude::*;
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit, Nonce};

fn make_cipher(key: &str) -> ChaCha20Poly1305 {
    let kb = key.as_bytes();
    let k: [u8; 32] = kb[..32].try_into().unwrap();
    ChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(&k))
}

fn decrypt(cipher: &ChaCha20Poly1305, ct_b64: &str, nonce_b64: &str) -> String {
    let ct = BASE64_STANDARD.decode(ct_b64.trim()).unwrap();
    let nb = BASE64_STANDARD.decode(nonce_b64.trim()).unwrap();
    let nonce = Nonce::from_slice(&nb);
    let pt = cipher.decrypt(nonce, ct.as_slice()).unwrap();
    String::from_utf8(pt).unwrap()
}

fn encrypt(cipher: &ChaCha20Poly1305, data: &str) -> (String, String) {
    use rand::RngCore;
    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher.encrypt(nonce, data.as_bytes()).unwrap();
    (BASE64_STANDARD.encode(ct), BASE64_STANDARD.encode(nonce_bytes))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let old_key = &args[1];
    let new_key = &args[2];
    let enc_session = &args[3];
    let session_nonce = &args[4];
    let enc_password = &args[5];
    let pass_nonce = &args[6];

    let old_cipher = make_cipher(old_key);
    let new_cipher = make_cipher(new_key);

    let session_plain = decrypt(&old_cipher, enc_session, session_nonce);
    let password_plain = decrypt(&old_cipher, enc_password, pass_nonce);

    let (new_enc_session, new_session_nonce) = encrypt(&new_cipher, &session_plain);
    let (new_enc_password, new_pass_nonce) = encrypt(&new_cipher, &password_plain);

    // Output as pipe-separated values
    println!("{}|{}|{}|{}", new_enc_session, new_session_nonce, new_enc_password, new_pass_nonce);
}
RUSTEOF

cat > "$HELPER_DIR/Cargo.toml" << 'CARGOEOF'
[package]
name = "migrate_helper"
version = "0.1.0"
edition = "2024"
[dependencies]
base64 = "0.22"
chacha20poly1305 = "0.10"
rand = "0.9"
CARGOEOF

echo ""
echo "Building migration helper..."
cargo build --quiet --release --manifest-path "$HELPER_DIR/Cargo.toml" 2>&1 | tail -3

echo "Re-encrypting credentials..."
RESULT=$("$HELPER_DIR/target/release/migrate_helper" \
    "$GORILLA_KEY" "$GARMIN_KEY" \
    "$ENC_SESSION" "$SESSION_NONCE" \
    "$ENC_PASSWORD" "$PASS_NONCE")

IFS='|' read -r NEW_ENC_SESSION NEW_SESSION_NONCE NEW_ENC_PASSWORD NEW_PASS_NONCE <<< "$RESULT"

echo "  New session:  ${#NEW_ENC_SESSION} chars"
echo "  New password: ${#NEW_ENC_PASSWORD} chars"

# Insert directly into garmin_api SQLite
DB_PATH="$GARMIN_DIR/data/garmin_api.db"
echo ""
echo "Importing into garmin_api ($DB_PATH)..."

NOW=$(date +%s)
sqlite3 "$DB_PATH" <<SQL
INSERT INTO garmin_users (user_id, garmin_username, encrypted_password, password_nonce, encrypted_session, session_nonce, status, created_at, updated_at)
VALUES ('$USER_ID', '$GARMIN_USERNAME', '$NEW_ENC_PASSWORD', '$NEW_PASS_NONCE', '$NEW_ENC_SESSION', '$NEW_SESSION_NONCE', 'connected', $NOW.0, $NOW.0)
ON CONFLICT(user_id) DO UPDATE SET
    garmin_username = excluded.garmin_username,
    encrypted_password = excluded.encrypted_password,
    password_nonce = excluded.password_nonce,
    encrypted_session = excluded.encrypted_session,
    session_nonce = excluded.session_nonce,
    status = 'connected',
    updated_at = excluded.updated_at;
SQL

echo ""
echo "Done! Session migrated. Test with:"
echo "  ./scripts/garmin-setup.sh  (check status)"
echo "  Or trigger a sync directly via the API"

# Cleanup
rm -rf "$HELPER_DIR"

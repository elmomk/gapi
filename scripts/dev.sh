#!/usr/bin/env bash
set -euo pipefail
export RUST_LOG=${RUST_LOG:-info}
export MASTER_KEY=${MASTER_KEY:-dev_master_key_at_least_32_bytes_long}
export DATABASE_PATH=${DATABASE_PATH:-garmin_api.db}
export API_KEYS=${API_KEYS:-devkey:dev}
cargo run

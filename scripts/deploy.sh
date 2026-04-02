#!/usr/bin/env bash
set -euo pipefail
cargo build --release
docker compose up -d --build

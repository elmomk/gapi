# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Standalone Garmin Connect API proxy/aggregator. Syncs health data from Garmin's undocumented mobile API via reverse-engineered OAuth1/OAuth2 + MFA authentication, stores in SQLite, exposes REST endpoints, and dispatches webhook events. Extracted from gorilla_coach to serve multiple consumers over Tailscale.

## Build & Run

```bash
# Backend
cargo build --release                    # Release build
cargo check                              # Type check only
cargo clippy --workspace                 # Lint both backend + frontend
./scripts/dev.sh                         # Run locally (sets MASTER_KEY, DATABASE_PATH, API_KEYS defaults)

# Frontend (Leptos WASM, separate crate — NOT a Cargo workspace member)
cd frontend && trunk build --release     # Build WASM bundle
cd frontend && trunk serve               # Dev server with hot reload

# Docker
./scripts/deploy.sh                      # Build + docker compose up
docker compose up -d                     # Backend only
cd frontend && docker compose up -d      # Frontend (nginx)

# Tests (no unit tests — e2e only)
./scripts/e2e-test.sh                    # Curl-based API tests against running server

# Utilities
./scripts/gen-keys.sh                    # Generate MASTER_KEY + API keys, writes .env
./scripts/garmin-setup.sh                # Interactive Garmin credential registration
```

## Architecture

**Two independent Rust crates** (not a workspace):
- Root: `garmin_api` (Axum 0.8 backend, edition 2024)
- `frontend/`: `garmin_dashboard` (Leptos 0.7 CSR WASM, edition 2024)

The frontend knows nothing about Garmin — it fetches from the backend REST API using `X-API-Key` auth.

### Backend module structure

```
src/
├── main.rs          # Entry: tracing, AppConfig, AppState, router, background sync spawn
├── config.rs        # AppConfig::from_env() — all config from env vars
├── state.rs         # AppState: config, repo, vault, http_client, webhook_dispatcher
├── db.rs            # r2d2 + rusqlite pool, schema DDL, migrations (ALTER TABLE)
├── vault.rs         # ChaCha20Poly1305 encrypt/decrypt for credentials
├── domain.rs        # Core types: GarminDailyData (44 fields), VitalsResponse, Intraday*, Baseline
├── sync.rs          # Background hourly sync loop: per-user, smart date-skip, rate-limit aware
├── garmin/
│   ├── mod.rs       # OAuth1/OAuth2 auth, SSO login, MFA, token refresh, GarminApiError enum
│   └── api.rs       # fetch_all_daily_data(): 12 parallel API calls via tokio::join!, response parsing
├── handlers/
│   ├── mod.rs       # Router tree + API key auth middleware (X-API-Key header)
│   ├── credentials.rs, sync.rs, data.rs, intraday.rs, activities.rs, webhooks.rs, health.rs
├── repository/      # SQLite data access (users, data, intraday, extended, webhooks)
└── events/          # WebhookDispatcher: HMAC-SHA256 signed delivery, 3-retry backoff
```

### Key data flow

1. **Sync loop** (`sync.rs`) runs hourly per user → calls `fetch_all_daily_data` (`garmin/api.rs`)
2. `fetch_all_daily_data` fires 12 Garmin endpoints in parallel (`tokio::join!`), parses into `GarminDailyData` + intraday vectors
3. Repository layer upserts to SQLite (COALESCE-based — new NULLs don't overwrite existing data)
4. Webhook events dispatched to subscribers after successful sync
5. Frontend fetches `/api/v1/users/{id}/daily`, `/vitals`, `/intraday/*` etc.

### Auth chain

- **API consumers → backend**: `X-API-Key` header, keys seeded from `API_KEYS` env var, stored as SHA-256 hashes
- **Backend → Garmin**: OAuth1 ticket → OAuth2 token exchange, tokens cached in encrypted SQLite column, proactive refresh before expiry

### Environment variables

| Var | Required | Purpose |
|-----|----------|---------|
| `MASTER_KEY` | Yes | ChaCha20Poly1305 encryption key (≥32 bytes) |
| `API_KEYS` | Yes | Comma-separated `key:consumer_name` pairs |
| `DATABASE_PATH` | No | SQLite file path (default: `garmin_api.db`) |
| `PORT` | No | Listen port (default: `3000`) |
| `HOST` | No | Bind address (default: `0.0.0.0`) |
| `RUST_LOG` | No | Tracing filter (default: `info`) |

## Conventions

- **No unit tests** — validation is via `scripts/e2e-test.sh` and `cargo clippy`
- **DB migrations**: new columns added via `ALTER TABLE` in `db.rs` `init_pool()`, errors silently ignored (idempotent)
- **Garmin API is undocumented**: response parsing is defensive with `.or_else()` chains for field name variations across API versions
- **All Garmin API responses parsed as `serde_json::Value`** — not strongly typed, because Garmin changes field names/shapes without notice
- **Frontend deployment**: `trunk build --release` → static files served by nginx, connects to backend via `config.json` (api_url, api_key)
- **Tailscale networking**: both Docker Compose files use `network_mode: service:tailscale` sidecar pattern

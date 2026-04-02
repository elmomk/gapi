# Garmin API

Standalone Garmin Connect API service with Event-Driven Architecture. Handles OAuth authentication, background data sync, encrypted credential storage, and webhook-based event dispatch.

Extracted from [gorilla_coach](https://github.com/elmomk/gorilla_coach) to serve as a shared data source for multiple consumers (gorilla_coach, life_manager, etc.) over a Tailscale private network.

## What it does

- Authenticates with Garmin Connect via SSO (OAuth1/OAuth2 + MFA support)
- Syncs 12 health endpoints in parallel: steps, heart rate, HRV, sleep, stress, body battery, weight, SpO2, respiration, training readiness, training status, activities
- Stores 42+ daily health metrics in SQLite
- Emits webhook events (`daily_data_synced`, `sync_completed`, `sync_failed`) to registered consumers
- Exposes REST API for data queries, on-demand sync, credential management

## Stack

- **Rust** (Axum 0.8, Tokio)
- **SQLite** (rusqlite + r2d2, WAL mode) — no DB container needed
- **ChaCha20Poly1305** encryption for credentials at rest
- **Docker + Tailscale** (2 containers: tailscale + app)

## Quick Start

```bash
# Generate keys and configure .env
cp .env.example .env
./scripts/gen-keys.sh

# Build and deploy
cargo build --release
docker compose up -d --build

# Set up Garmin account
./scripts/garmin-setup.sh

# Or migrate existing session from gorilla_coach
./scripts/migrate-session.sh
```

## REST API

All endpoints require `X-API-Key` header.

| Method | Endpoint | Purpose |
|--------|----------|---------|
| POST | `/api/v1/users/{id}/credentials` | Register Garmin credentials |
| POST | `/api/v1/users/{id}/mfa` | Submit MFA code |
| DELETE | `/api/v1/users/{id}/credentials` | Remove credentials |
| GET | `/api/v1/users/{id}/status` | Connection status |
| POST | `/api/v1/users/{id}/sync` | Trigger on-demand sync |
| GET | `/api/v1/users/{id}/daily?date=YYYY-MM-DD` | Single day data |
| GET | `/api/v1/users/{id}/daily?start=...&end=...` | Date range |
| GET | `/api/v1/users/{id}/baseline?days=7` | N-day averages |
| GET | `/api/v1/users/{id}/vitals` | Today + 7-day baseline |
| POST | `/api/v1/webhooks` | Register webhook |
| GET | `/api/v1/webhooks` | List webhooks |
| DELETE | `/api/v1/webhooks/{id}` | Remove webhook |
| GET | `/health` | Health check |

## Events

Webhooks are dispatched with HMAC-SHA256 signing and 3-retry exponential backoff.

| Event | When | Payload |
|-------|------|---------|
| `daily_data_synced` | Each day's data saved | Full daily data |
| `sync_completed` | Full sync finishes | `{ days_synced, errors, duration_secs }` |
| `sync_failed` | Auth/rate limit failure | `{ reason }` |
| `credentials_updated` | Credentials saved/MFA done | `{ status }` |

## Configuration

See [.env.example](.env.example) for all options. Key settings:

| Variable | Default | Description |
|----------|---------|-------------|
| `MASTER_KEY` | required | Encryption key (min 32 bytes) |
| `API_KEYS` | required | `key:name` pairs, comma-separated |
| `SYNC_DAYS` | 30 | Days of history to sync |
| `SYNC_RATE_LIMIT_MINS` | 60 | Min interval between syncs per user |
| `GARMIN_API_DELAY_SECS` | 5 | Delay between per-day API calls |

## Scripts

| Script | Purpose |
|--------|---------|
| `scripts/gen-keys.sh` | Generate MASTER_KEY and API keys, update .env |
| `scripts/garmin-setup.sh` | Interactive Garmin account setup with MFA |
| `scripts/migrate-session.sh` | Migrate session from gorilla_coach |
| `scripts/e2e-test.sh` | Run end-to-end API tests |
| `scripts/build.sh` | Build release binary |
| `scripts/deploy.sh` | Build and deploy to Docker |
| `scripts/dev.sh` | Run locally for development |

## Dashboard

Open `dashboard.html` in a browser for a test dashboard with vitals cards, trend charts (configurable 1-365 days), and activity view.

## Architecture

```
garmin_api
├── Garmin Connect SSO ──── OAuth1/OAuth2 auth, MFA, token refresh
├── Sync Engine ─────────── Hourly background sync + on-demand
├── SQLite ──────────────── Encrypted credentials, daily health data
├── Webhook Dispatcher ──── HMAC-signed event delivery with retries
└── REST API ────────────── Axum handlers with API key auth
```

Consumers (gorilla_coach, life_manager) subscribe to webhook events and/or query the REST API. No direct Garmin API knowledge needed in consumers.

# Open Credential Ledger

[![CI](https://github.com/marco-pcg/Open-Credential-Ledger/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/marco-pcg/Open-Credential-Ledger/actions/workflows/ci.yml)

Research monorepo comparing conventional issuer trust with blockchain-backed issuer trust for signed academic credential bytes. This is a working research prototype, not an accredited credential service. Repository: [marco-pcg/Open-Credential-Ledger](https://github.com/marco-pcg/Open-Credential-Ledger).

## Layout

- `backend/`: Rust, Axum, Tokio, SQLx/PostgreSQL and Alloy contract calls.
- `contracts/`: Foundry issuer/key authorization registry and tests; no external Solidity libraries.
- `frontend/`: React/TypeScript verifier, Vite and Vitest.
- `paper/`: editable CEFET/RJ-oriented LaTeX boilerplate and research protocol.
- `docs/`: API, deployment, security model, experiment plan and validation results.

## Quick start

Install Docker with Compose. Copy `.env.example` to `.env`, then run:

```sh
cp .env.example .env
docker compose up --build -d
```

Open http://localhost:3000. API liveness: http://localhost:8080/healthz; database readiness: `/readyz`. The backend runs migrations at startup. An empty registry address leaves blockchain mode unavailable (HTTP 503), while conventional mode works. Database is initially empty: an unregistered issuer is untrusted.

See [deployment](docs/DEPLOYMENT.md) to generate a synthetic credential, enroll its public key, deploy the contract and use both modes. Use only synthetic data. `docker compose down` preserves database data; `docker compose down -v` deletes it.

## Local development

Requires Rust 1.96+, Node 22+, PostgreSQL 17, and Foundry with Solidity 0.8.28.

```sh
docker compose up -d db anvil
# Export variables manually; the Rust service does not automatically read .env.
export DATABASE_URL=postgres://ocl:local-development-only@localhost:5432/ocl
cargo run -p ocl-api
# In another terminal:
cd frontend
npm ci
npm run dev
```

Vite serves port 5173 and proxies `/api` to port 8080. The production frontend uses nginx for the same-origin API proxy. No broad CORS policy is required.

## Verification

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
# Requires a disposable PostgreSQL instance:
cargo test --test database --locked -- --ignored
# Requires forge build artifacts and a running Anvil:
RPC_URL=http://localhost:8545 cargo test --test chain --locked -- --ignored
(cd frontend && npm ci && npm test && npm run build)
(cd contracts && forge fmt --check && forge test -vvv)
```

See [test results](docs/TEST_RESULTS.md) for checks actually run in this environment. CI runs four jobs: backend (including PostgreSQL and Alloy/Anvil integration tests), frontend, contracts, and container builds. Deployment is manual until a destination, network and secret management policy are selected; no automatic publication is configured.

## Scope

The signature is Ed25519 over the exact decoded payload bytes; transport uses standard Base64. The institutional Ethereum address is a registry identifier, not the Ed25519 signing key. A registry entry authorizes the Keccak-256 hash of that Ed25519 public key. Conventional mode compares the same key against PostgreSQL.

`valid` means signature integrity plus current issuer/key authorization. It does not assert the truth of a degree, holder identity, expiry, historical authorization, or credential-level revocation. No W3C VC conformance, zero-knowledge proof or selective-disclosure claim is made. Credentials are submitted transiently to this server; verification is not client-only.

MIT licensed; see [CONTRIBUTING](CONTRIBUTING.md) and [SECURITY](SECURITY.md).

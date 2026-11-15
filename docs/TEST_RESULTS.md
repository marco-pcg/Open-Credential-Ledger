# Validation results — 2026-11-15

Local environment: Rust 1.96.0, Node 22.23.5, npm 10.9.9, temporary Foundry 1.7.1, Solidity 0.8.28, PostgreSQL 17 container. Dependencies are recorded in Cargo.lock and frontend/package-lock.json.

| Check | Result |
| --- | --- |
| `cargo test --workspace --locked` | 5 unit tests passed; environment-dependent database/chain tests are explicitly ignored in the ordinary suite |
| `DATABASE_URL=… cargo test --test database --locked -- --ignored` | 1 passed against local PostgreSQL: enrolled key accepted, revoked key rejected |
| `RPC_URL=… cargo test --test chain --locked -- --ignored` | 1 passed against Anvil: deploy registry, unauthorized/authorized/revoked key decisions, wrong-chain 503 |
| `cargo fmt --all --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `npm test` | 3 tests passed: mode selection, outage handling, input validation |
| `npm run build` | Strict TypeScript check and production bundle passed |
| `npm audit` | 0 reported vulnerabilities after updating Vite/Vitest |
| `forge test -vvv` | 5 tests passed, including 256 fuzz cases; unauthorized mutation, invalid issuer, ownership transfer and revocation covered |
| `forge fmt --check` | Passed after formatting |
| `POSTGRES_PASSWORD=… docker compose config --quiet` | Passed |
| `docker compose build` | Backend release image and frontend production image built successfully |
| Container HTTP smoke test | Liveness, database readiness and frontend returned 200; nginx API proxy rejected an unregistered synthetic key and accepted it after enrollment; exact fixture row removed afterward |

The ordinary Rust suite does not silently require PostgreSQL or Anvil. Integration tests must be invoked explicitly as above; CI provisions both dependencies. Chain tests require `forge build --root contracts` artifacts and a fresh local Anvil (chain ID 31337). The chain test deploys its own registry and uses unlocked local accounts only.

Not performed: TeX/PDF compilation or visual review (no TeX compiler installed), browser interaction/accessibility audit, comparative research benchmarks, public-network deployment, external security audit, GitHub-hosted CI execution. No empirical research results are claimed. npm audit is a point-in-time advisory check, not a security certification.

Docker initially required access outside the restricted execution context; PostgreSQL and integration tests subsequently ran successfully. No GitHub remote, push, or publication was performed.

Temporary application/database containers were stopped after validation; the local database volume was preserved. The test-only Anvil node was also stopped. The application is not left running.

# Validation results

## Local validation — 2026-11-15

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

Not performed during this local validation: TeX/PDF compilation or visual review (no TeX compiler installed), browser interaction/accessibility audit, comparative research benchmarks, public-network deployment, external security audit, GitHub-hosted CI execution. No empirical research results are claimed. npm audit is a point-in-time advisory check, not a security certification.

Docker initially required access outside the restricted execution context; PostgreSQL and integration tests subsequently ran successfully. No GitHub remote, push, or publication was performed during the local validation above.

Temporary application/database containers were stopped after validation; the local database volume was preserved. The test-only Anvil node was also stopped. The application is not left running.

## Hosted GitHub Actions verification — 2026-11-20

Issue [#3](https://github.com/marco-pcg/Open-Credential-Ledger/issues/3) was verified against the initial hosted [CI run #43743565984](https://github.com/marco-pcg/Open-Credential-Ledger/actions/runs/43743565984) on `main`, commit `6671071fe319cb28d2cfcdb91b02844494a6b409`. The run completed successfully. Inspection of all four jobs confirmed that each required validation step passed:

| Hosted job | Successful checks |
| --- | --- |
| backend | Rust formatting, Clippy with warnings denied, workspace tests, PostgreSQL enrollment/revocation integration, contract compilation, and Alloy integration against Anvil |
| frontend | Clean dependency installation with `npm ci`, Vitest tests, TypeScript checking and production build |
| contracts | Foundry formatting and Solidity tests |
| containers | Docker Compose configuration and builds of both application images |

No workflow failure needed correction. The README now links a live CI badge scoped to `main`; it reflects future runs rather than a fixed historical result. The earlier local-validation exclusions remain historical: hosted CI has now been verified, while paper rendering, browser review, research benchmarks, public-network deployment, and an external security audit remain outside this check.

## Credential-profile validation — 2026-11-20 (issue #4)

Implemented `ocl.academic.v1` on the issue #4 branch. This changes the signed-message format: regenerate old demo credentials using the updated fixture example. No dependency or Solidity storage change was needed.

| Check | Result |
| --- | --- |
| Rust workspace tests | 5 existing unit tests and 11 credential-policy acceptance tests passed; the ordinary run explicitly skips the two infrastructure-dependent tests |
| Published policy vectors | All 20 payload examples match the expected decisions at the fixed manifest clock, including duplicate members, unknown/missing claims, validity dates and issuer mismatch |
| PostgreSQL integration | 1 passed against an isolated PostgreSQL 17 test container: profile-v1 credential accepted with active issuer key, rejected after key revocation |
| Alloy/Anvil integration | 1 passed: profile-v1 credential decisions for unauthorized/authorized/revoked keys, plus wrong-chain rejection |
| Rust formatting and Clippy | Passed, with Clippy warnings denied |
| Frontend tests | 12 passed, including the displayed policy decisions |
| Frontend production build | TypeScript checks and Vite bundle passed |
| Foundry contracts | 5 passed, including 256 fuzz cases; contract code unchanged |
| Fixture generator | Generated a signed v1 envelope with matching issuer and a 24-hour validity window |
| Real HTTP smoke test | Accepted the Rust-generated credential after enrollment; also accepted an independently signed Python/cryptography envelope using the specified context; correctly rejected signed expiry, future validity, issuer substitution, unsupported version and unknown claims |

The deterministic policy tests cover inclusive start/exclusive expiry boundaries, signed/envelope issuer binding, profile/context separation, rejection after byte changes, strict claim parsing, and skipped trust lookups for signed invalid claims in both modes. HTTP errors for malformed envelope structure do not echo submitted fields or values. A valid credential cannot turn a trust-source outage into a successful result.

Browser review, paper rendering and comparative benchmarks were not performed for this change. Hosted PR CI results are linked from the pull request once available.

The issue #4 HTTP smoke test used an isolated PostgreSQL container and the publicly known synthetic key only. Its exact fixture row was removed afterward. The temporary API, Anvil node and isolated PostgreSQL container/storage were stopped/removed after validation; the normal project database was not used.

## Object-shape review regression — 2026-11-20

Post-merge review found that Serde's derived struct deserializer also accepts positional JSON arrays. New regression tests reproduced the mismatch with the profile's object-only contract before the fix: array claims passed policy validation, and array envelopes reached the trust lookup instead of returning a client error.

The follow-up requires JSON maps for the request envelope, claims root, and nested award, plus JSON strings for award kind and mode (also rejecting externally tagged enum objects). It preserves direct typed deserialization so duplicate-member rejection is retained. Three additional published vectors cover positional claims/award arrays and an object-encoded award kind (23 vectors total). All 14 policy tests and the 5 original Rust unit tests passed after the fix, along with formatting and Clippy with warnings denied. No signing rules, required fields, or trust storage changed.

## Credential-level revocation — 2026-11-20 (issue #5)

| Check | Result |
| --- | --- |
| Rust workspace unit/policy/handle tests | 21 passed; infrastructure tests are explicitly ignored in the ordinary run |
| PostgreSQL integration | 2 passed against isolated PostgreSQL 17: active/revoked credential decisions, unaffected sibling credentials, same-ID issuer isolation, idempotence, key-change persistence, and append-only update/delete rejection |
| Alloy/Anvil integration | 3 passed: key authorization/wrong-chain baseline; individual revocation lifecycle and issuer isolation; old registry ABI rejected with 503 |
| Foundry tests | 11 passed, including two fuzz tests with 256 runs each and the Rust/Solidity handle interoperability vector |
| Frontend tests | 13 passed, including revoked-credential feedback |
| Frontend build | TypeScript checking and production bundle passed |
| Rust formatting and Clippy | Passed with warnings denied |
| Live HTTP smoke test | Two generated credentials passed initially; revoking one rejected only that credential while the sibling and issuer key remained valid |
| Fixture generation | Fresh, distinct 256-bit random credential IDs; handle printed for the exact generated credential |

Total: 50 automated tests passed across these suites, plus the live HTTP smoke check. The new direct dependency on getrandom 0.3.4 uses the version already present in Cargo.lock. Claims/signing remain profile v1; the registry ABI and response status fields change as documented in REVOCATION.md.

Tests use synthetic data only. No public-chain transaction, real-wallet operation, browser review, paper rendering, or comparative benchmark was performed. The isolated database/API/Anvil processes are removed/stopped after validation; the normal project database is not used. Append-only test tombstones remain only in the disposable test database until that container/storage is removed. Hosted CI evidence is recorded in the PR.

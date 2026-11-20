# OCL academic credential profile v1

Status: implemented research profile, defined by issue #4. Profile identifier: `ocl.academic.v1`. This document specifies the signed payload and verification policy, independent of the chosen issuer trust source. It supersedes the prototype's acceptance of arbitrary signed bytes.

## Signed payload

The payload is exactly one UTF-8 JSON object with these required fields. No field is optional or nullable. Unknown or duplicate members are rejected, including inside `award`. Trailing whitespace is allowed as part of the signed bytes; trailing JSON values are rejected.

| Field | Type and constraints |
| --- | --- |
| `schema_version` | Exact string `ocl.academic.v1` |
| `credential_id` | `urn:ocl:credential:` followed by 1–96 ASCII letters, digits, hyphens or underscores |
| `issuer` | Nonzero institutional Ethereum address: `0x` followed by exactly 40 lowercase hexadecimal characters; must equal the envelope's `issuer` |
| `subject_id` | `urn:ocl:subject:` followed by 1–96 ASCII letters, digits, hyphens or underscores |
| `award` | Object containing exactly `kind` and `name` |
| `award.kind` | `degree` or `course_completion` |
| `award.name` | 1–256 Unicode scalar values, no leading/trailing whitespace, no Unicode control characters |
| `issued_at` | Integer UTC Unix timestamp in seconds, at least 1 |
| `valid_from` | Integer UTC Unix timestamp in seconds |
| `valid_until` | Integer UTC Unix timestamp in seconds, required (no indefinite validity) |

All timestamps must be unsigned integers, at most `9007199254740991` (2^53−1, the JSON client safe-integer ceiling). Decimal/exponent notation, fractional, negative, string and null values are rejected. Relations must satisfy `issued_at <= valid_from < valid_until`. At the server's evaluation time `t`, acceptance requires `valid_from <= t < valid_until`. The start is inclusive and expiry is exclusive. There is no clock-skew grace period. Future issuance cannot pass: it either violates date ordering or places the start in the future. Operators must synchronize the verifier's clock.

Use issuer-generated opaque, random identifiers, unique in the issuer's scope; avoid email, CPF, registration numbers, names, or other real personal data. Syntax checking does not prove identifier uniqueness, holder identity, possession, enrollment, or the award's truth. Only synthetic identifiers are used by this project. Pairwise subject identifiers reduce correlation; these URNs are not decentralized identifiers or externally resolved resources.

A valid payload is in [valid.json](../examples/credentials/v1/valid.json). Its dates are designed for the fixture clock, not the current date. Generate a current, signed demonstration envelope with:

```sh
cargo run -p ocl-api --example fixture > fixture.json
```

This example uses the publicly known test seed `[7;32]` and expires after 24 hours. Its issuer key must still be enrolled in the selected trust source. Never use this seed for real issuance.

## Envelope and signing

The request envelope retains `issuer`, `public_key`, `payload_base64`, `signature_base64`, and `mode`. The envelope's issuer uses the same canonical nonzero address form above. `public_key` is exactly 64 lowercase hexadecimal characters without `0x`, encoding an Ed25519 public key. Standard padded Base64 encodes the original payload bytes and the 64-byte signature. The decoded payload is 1–32768 bytes; the whole request is limited to 64 KiB.

The Ed25519 signature is computed over:

```text
UTF8("OpenCredentialLedger:academic:v1\n") || original_payload_bytes
```

Here `\n` is one LF byte (`0x0a`), not two literal characters. This fixed context separates these signatures from signatures over unrelated documents. Rust issuers and tests use `credential::signing_message`. Strict Ed25519 verification is used. The service never reserializes/canonicalizes a payload before verifying its signature. Reordering fields, changing whitespace, normalization or escaping after signing invalidates the signature, even if the JSON would have the same semantic values. Raw-byte signatures from the initial prototype are intentionally unsupported; there is no legacy bypass.

The signed `issuer` must equal the envelope's issuer, preventing the envelope from relabeling a signed credential as another institution. The Ethereum address is an institutional registry identifier, not the Ed25519 key. Conventional trust requires an active PostgreSQL row for that issuer and the exact public key; blockchain trust requires the same issuer and `keccak256(public_key_bytes)` to be authorized in the configured registry on the expected chain. Registry trust is current, not historical.

`mode` is a verifier-selected policy outside the signature and changes only the source of issuer trust. It cannot bypass claim validation. The credential does not claim a blockchain chain ID or registry address: verifiers configure their trust domain independently.

## Verification order and decision contract

1. Validate envelope encodings and limits. Malformed requests receive a client error, not a validity verdict.
2. Verify the signature over the signing context and the exact payload bytes. Invalid signatures produce `invalid_signature`; claims and trust are not assessed.
3. Deserialize into strict typed claims, validate profile version, bind issuer, validate claim constraints/date ordering, then evaluate the validity window.
4. Only if steps 2–3 pass, query the selected current issuer/key authorization source. Failed/missing trust sources return HTTP 503; a configured source rejecting authorization produces `untrusted_issuer`.

Structural errors have precedence over profile-version errors. Otherwise the order is version, issuer binding, identifier/award constraints, date ordering/range, start time, expiry, then trust. Stable reason codes are `verified`, `invalid_signature`, `invalid_claims`, `unsupported_version`, `issuer_mismatch`, `invalid_dates`, `not_yet_valid`, `expired`, and `untrusted_issuer`. Reasons never echo claim values or parser diagnostics.

A completed decision returns HTTP 200 with `valid`, `signature_valid`, `claims_valid`, `issuer_trusted`, `reason`, `policy_version`, and `evaluated_at`. `issuer_trusted` is null when not assessed and otherwise boolean. `claims_valid:false` also covers claims not assessed after an invalid signature. `evaluated_at` is the server's UTC Unix timestamp used for the claims check, captured before trust lookup; this is a point-in-time policy verdict, not a guarantee of continuing validity or an atomic snapshot of all systems.

`valid:true` means **all** of: a valid signature, structurally valid profile-v1 claims, matching signed/envelope issuer, an active validity window at `evaluated_at`, and current issuer/key authorization from the selected source. It does not establish academic truth, holder identity/possession, credential uniqueness, credential-level revocation, historic issuance authority, or replay protection. Repeated presentations can pass. Per-credential revocation belongs to issue #5, historical/key-rotation policy to #6, and finality/RPC policy to #7.

## Versioning, compatibility and W3C intent

This is an application-specific JSON profile, **not W3C Verifiable Credentials Data Model 2.0 compatible**. It does not implement JSON-LD contexts, VC securing mechanisms, credential status, presentations or holder proofs. Field resemblance does not imply interoperability. W3C compatibility is not intended for v1; a future interoperable profile needs a separate design, version, signing context and conformance tests. The [W3C Recommendation](https://www.w3.org/TR/vc-data-model-2.0/) is background, not a claim of compliance.

Any incompatible change to required claims, interpretation or signing rules requires a new profile version. Unsupported versions fail closed. The initial arbitrary-byte prototype is replaced by v1, not treated as a second supported profile. Existing issuer enrollments and on-chain authorization entries remain usable because the public-key trust model is unchanged; credentials must be regenerated with the v1 claims/context. Clients must handle the new decision fields and nullable trust status.

## Acceptance vectors

[manifest.json](../examples/credentials/v1/manifest.json) sets the fixed `evaluated_at` and envelope issuer for the published valid/invalid payload examples, and the expected reason for each. Examples include expiry, future dates, issuer substitution, unsupported versions, invalid/missing/unknown fields, duplicate members, invalid time types and unsafe timestamp ranges. Keep duplicate-key examples as raw bytes: loading them into a generic map and rewriting them can erase the ambiguity being tested.

`backend/tests/credential_policy.rs` consumes those exact files, tests boundary times and signing-byte/context behavior, and exercises invalid signed credentials through the API in both trust modes. PostgreSQL and Anvil integration tests verify successful profile-v1 credentials and current issuer authorization/revocation. The frontend tests cover the displayed decision messages.

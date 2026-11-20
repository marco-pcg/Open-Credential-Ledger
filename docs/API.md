# API — academic credential profile v1

`GET /healthz`: process liveness, 200 `ok`. `GET /readyz`: database availability, 200 `ready` or 503. Readiness does not certify chain availability.

`POST /api/verify`, Content-Type `application/json`:

```json
{"issuer":"0x0000000000000000000000000000000000000001","public_key":"64 lowercase hex characters without 0x","payload_base64":"standard padded Base64 of signed profile-v1 UTF-8 JSON","signature_base64":"standard padded Base64 of 64-byte Ed25519 signature","mode":"conventional"}
```

Mode is `conventional` or `blockchain`. Issuer is a nonzero lowercase address; public key is 32 bytes encoded as lowercase hex. Maximum body is 64 KiB and decoded payload is 1–32768 bytes. Unknown/duplicate envelope fields are rejected. The payload must satisfy [OCL academic profile v1](CREDENTIALS_V1.md); its signed issuer must match the envelope. The signature covers the fixed context `OpenCredentialLedger:academic:v1` plus LF plus the exact payload bytes. Never reformat a payload after signing. Old arbitrary-byte credentials must be regenerated.

Successful policy/authorization decision:

```json
{"valid":true,"signature_valid":true,"claims_valid":true,"issuer_trusted":true,"credential_revoked":false,"reason":"verified","policy_version":"ocl.academic.v1","evaluated_at":1800000000}
```

A correctly signed but expired credential returns HTTP 200, without querying issuer trust:

```json
{"valid":false,"signature_valid":true,"claims_valid":false,"issuer_trusted":null,"credential_revoked":null,"reason":"expired","policy_version":"ocl.academic.v1","evaluated_at":1800000000}
```

`valid` is the conjunction of signature validity, profile/issuer/date checks, current issuer authorization, and a non-revoked credential ID. `claims_valid:false` includes a skipped claims check after an invalid signature; `issuer_trusted:null` explicitly means no trust lookup took place. `evaluated_at` is the clock used for claim validity, before trust lookup. The verdict includes [current credential-level revocation](REVOCATION.md); it does not establish academic truth or holder identity.

Other reasons: `invalid_signature`, `invalid_claims`, `unsupported_version`, `issuer_mismatch`, `invalid_dates`, `not_yet_valid`, `untrusted_issuer`, and `revoked_credential`. Failed signatures skip claims/trust/status; failed claims skip trust/status. `credential_revoked` is null until signature, claims and key trust pass. A trusted key with a revoked credential returns `issuer_trusted:true`, `credential_revoked:true`, `reason:revoked_credential`, and `valid:false`. An unauthorized key returns `signature_valid:true`, `claims_valid:true`, `issuer_trusted:false`, `valid:false`.

Malformed envelope/encoding: 400 (Axum JSON extraction may return 415/422); oversized body: 413; trust source error, missing status storage, old registry ABI, missing/wrong-chain configuration, or unavailable clock: 503; timeout: 408. Errors are generic and do not echo credential data, database details, or RPC diagnostics. A service error makes no validity decision.

No API endpoint enrolls issuers. Administrators update the database or contract using separate authenticated tooling. No credential payload, digest, signature, subject identifier, or request log is persisted by the application.

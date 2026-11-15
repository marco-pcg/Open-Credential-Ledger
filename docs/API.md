# API v0.1

`GET /healthz`: process liveness, 200 `ok`. `GET /readyz`: database availability, 200 `ready` or 503. Readiness does not certify chain availability.

`POST /api/verify`, Content-Type `application/json`:

```json
{"issuer":"0x0000000000000000000000000000000000000001","public_key":"64 lowercase hex characters without 0x","payload_base64":"standard Base64 of exact signed bytes","signature_base64":"standard Base64 of 64-byte Ed25519 signature","mode":"conventional"}
```

Mode is `conventional` or `blockchain`. Public key is 32 bytes. Maximum request body is 64 KiB and decoded payload is 1–32768 bytes. Unknown JSON fields are rejected. Reformatting a JSON payload after signing changes its bytes and invalidates its signature. The service does not parse payload claims.

200 returns `{"valid":true,"signature_valid":true,"issuer_trusted":true}` for a trusted signature. Invalid cryptographic signatures return 200 with all flags false; trust lookup is skipped. Therefore `issuer_trusted:false` in that case is not an independent trust verdict. Valid signatures with unauthorized keys return `signature_valid:true`, remaining flags false. Malformed data: 400 (Axum JSON extraction may return 415/422); oversized request: 413; trust source error or missing/wrong-chain configuration: 503; timeout: 408. Errors are generic and do not expose credentials, database details or RPC diagnostics.

No API endpoint enrolls issuers. Administrators update the database or contract using separate authenticated tooling. No credential payload, digest, signature, holder identifier, or request log is persisted by the application.

# Credential-level revocation v1

Issue #5 adds an issuer-scoped, permanent denylist independent of issuer-key authorization. The signed academic profile remains `ocl.academic.v1`; its existing `credential_id` identifies the credential being revoked. No new signed claim or signing rule is introduced. Revocation is a current verifier-side status check, like current key authorization, not a claim that an award was false or that its holder is known.

## Privacy decision

The implemented design stores only a domain-separated opaque handle for each revoked credential. It never writes the subject identifier, award, signed payload, signature, raw credential ID, revocation reason, or a hash of credential contents to the chain. Active credentials are not individually registered. A handle is derived from the institutional issuer and its signed credential ID:

```text
keccak256(UTF8("OpenCredentialLedger:revocation:v1\n") || issuer_address_bytes_20 || UTF8(credential_id))
```

The LF is one byte `0x0a`. The issuer is exactly 20 decoded address bytes, not its hexadecimal text. The fixed prefix/address lengths make this concatenation unambiguous. The [published interoperability vector](../examples/revocation/handle-v1.json) is checked independently by Rust and Solidity tests. Its predictable ID is test data only.

**Identifiers must be unpredictable for privacy.** The fixture generator now uses 32 bytes from the operating system random source and encodes them as `urn:ocl:credential:<64 lowercase hex characters>`. Never derive credential IDs from student numbers, email, names, course codes, or a sequence. Existing v1 IDs remain syntactically compatible, but hashes of predictable IDs can be guessed; hashing alone does not make them private. Before public-chain use, reissue legacy credentials with fresh random IDs. The service can validate syntax but cannot prove that an issuer generated an ID randomly.

Even with random IDs, handles are stable correlation tags. The registry reveals the institution, revocation transaction timing, and revoked-handle counts. Anyone who already has a credential can compute its handle and recognize its revocation. A verifier/RPC operator can correlate repeated lookups, and the API sees the submitted credential. Do not describe this as anonymous or selectively disclosed verification. Do not publish real student data or revocation metadata in events, logs, scripts, issues, or examples.

Alternatives considered: raw identifiers or full-payload hashes increase exposure or unnecessarily bind status to serialization; a shared off-chain denylist for both modes would add a database dependency to blockchain verification; issuer-wide bitstring lists can offer group-level retrieval but require status-list distribution and signed indexing claims beyond the current profile. [W3C Bitstring Status List](https://www.w3.org/TR/vc-bitstring-status-list/) is a potential future design reference; this implementation does not implement or claim compatibility with it.

## State and governance

- Conventional mode: PostgreSQL `credential_revocations(issuer, handle, revoked_at)`. Handles are 64 lowercase hex characters **without** `0x`. The primary key makes duplicate inserts idempotent with `ON CONFLICT DO NOTHING`. Update/delete/truncate triggers protect append-only operation. There is no foreign key that could delete a tombstone when issuer enrollment changes.
- Blockchain mode: `IssuerRegistry.revokeCredential(issuer, handle)` accepts a `bytes32` handle. Only the registry owner can revoke, consistent with existing key-enrollment governance. The `CredentialRevoked` event contains only issuer and handle. Repeated revocation is idempotent. There is no un-revoke operation. State survives key and ownership changes within the same registry.
- A normal database writer cannot remove a tombstone through ordinary update/delete/truncate. A database owner/superuser can still disable triggers, drop storage or restore stale backups; administrator integrity remains a conventional-mode trust assumption. On-chain persistence assumes the selected registry's canonical chain history; reorganizations, malicious RPC replies and registry replacement remain concerns addressed by deployment policy and issue #7.

Absence of a tombstone means “not currently listed as revoked,” not proof of issuance. An authenticated signature and active key authorization are still required. Revoking an unissued handle is allowed, so a tombstone also applies to later enrollment/re-signing of that same credential identity. Reissue corrected credentials with a **new** random ID; never reuse a revoked ID or attempt to undo a revocation.

The two modes have independent administrators and stores. A PostgreSQL insert does not broadcast a transaction, and a contract transaction does not update PostgreSQL. To compare equivalent policy state, operators must populate both stores consistently. This research prototype does not implement cross-store synchronization or an authenticated administrative HTTP API.

## Verification behavior

After signature and claims checks, the API derives the handle from the signed issuer and credential ID. Conventional verification reads key authorization and tombstone existence in one SQL statement; blockchain verification uses one `verificationStatus(issuer, keyHash, handle)` contract call. These avoid mixing separate key/status snapshots within the selected source. Chain identity is still checked before the contract call; no finalized-block policy is implemented yet.

If key trust fails, the result is `untrusted_issuer` and `credential_revoked:null` (no status verdict is exposed). If key trust passes, `credential_revoked` is boolean: true produces `revoked_credential` and `valid:false`; false permits `verified` and `valid:true`. Invalid signatures/claims skip both lookups and leave status null. Missing or failed status storage/RPC/ABI returns HTTP 503, not a successful result. There is no fallback to the old key-only check.

`policy_version` still identifies the signed claims profile; the handle derivation is separately versioned by its `revocation:v1` context. Existing signed credentials need not be re-signed for functional compatibility, although predictable legacy IDs should be replaced before public publication for privacy. The response adds `credential_revoked` and the `revoked_credential` reason, and success now requires a non-revoked status in addition to existing checks.

## Local operation (synthetic data only)

Generate a current fixture with `cargo run -p ocl-api --example fixture > fixture.json`. The key hash and revocation handle are printed to stderr. Each generation has a fresh credential ID; retain the exact fixture whose status you intend to change.

Alternatively derive a handle from the signed issuer and ID:

```sh
cargo run -p ocl-api --example revocation_handle -- \
  0x0000000000000000000000000000000000000001 \
  urn:ocl:credential:REPLACE_WITH_THE_RANDOM_ID_SUFFIX
```

This helper validates identifier syntax and computes a handle only. It does not verify signatures, authorize a credential, write the database, or publish a transaction. Arguments can remain in shell history; use only synthetic IDs in this demonstration.

Conventional revocation, after confirming the correct issuer and handle:

```sql
INSERT INTO credential_revocations(issuer,handle)
VALUES ('0x0000000000000000000000000000000000000001','REPLACE_WITH_64_HEX_HANDLE_WITHOUT_0x')
ON CONFLICT DO NOTHING;
```

Blockchain revocation on local Anvil, using the current registry owner's disposable account:

```sh
cast send "$REGISTRY_ADDRESS" 'revokeCredential(address,bytes32)' \
  0x0000000000000000000000000000000000000001 REPLACE_WITH_0x_PREFIXED_HANDLE \
  --rpc-url http://localhost:8545 --interactive
```

Confirm transaction success and then verify that the targeted credential returns `revoked_credential`, while a second credential from the same active issuer remains valid. Do not use real wallets or public networks for this exercise.

## Migration and recovery

The backend applies migration `0002_credential_revocations.sql` at startup; existing issuer rows remain intact and the denylist initially has no rows. Preserve tombstones during backups, restores and schema changes: losing status records can make revoked credentials appear active.

Existing deployed contracts cannot gain the new methods automatically. Deploy the updated registry, restore the intended owner/key authorizations **and all applicable revocations**, then update `REGISTRY_ADDRESS` and restart the API. An old ABI deliberately returns 503. Do not point the verifier at an empty replacement registry and assume prior credential status was carried over. Registry redeployment/chain reset is a trust-domain migration, not a revocation reset procedure.

## Acceptance coverage

PostgreSQL and Anvil integration tests each verify active credentials, single-credential revocation, unaffected sibling credentials, issuer isolation for a shared ID, idempotence, key changes, and hiding status when the key is untrusted. Database tests exercise append-only update/delete guards. Foundry tests cover owner permissions, zero-input rejection, state persistence through ownership/key changes, an interoperability hash vector and fuzzed isolation. A legacy-contract integration test proves that missing status support fails closed. Unit tests cover handle derivation/issuer binding and random identifier generation; frontend tests cover the displayed revoked decision.

# Local demonstration and deployment

1. Copy `.env.example` to `.env`; run `docker compose up --build -d`.
2. Generate a public deterministic synthetic credential:
   `cargo run -p ocl-api --example fixture > fixture.json`.
   The example generates a signed profile-v1 synthetic credential valid for 24 hours and prints the key hash to stderr. Regenerate expired fixtures. It uses the publicly known seed `[7;32]` and must never issue real credentials.
3. Enroll the `public_key` from fixture.json in PostgreSQL, using the address in that fixture:
   `docker compose exec db psql -U ocl -d ocl`

```sql
INSERT INTO issuers(address,public_key) VALUES
('0x0000000000000000000000000000000000000001','REPLACE_WITH_FIXTURE_PUBLIC_KEY');
```

4. Verify with `curl -sS http://localhost:8080/api/verify -H 'Content-Type: application/json' --data-binary @fixture.json`, or paste fixture JSON into the UI.
5. With Foundry installed, deploy locally:

```sh
cd contracts
forge create src/IssuerRegistry.sol:IssuerRegistry --rpc-url http://localhost:8545 --interactive --broadcast --constructor-args YOUR_LOCAL_ANVIL_ACCOUNT
```

Use only a disposable account from Anvil's startup output. Never reuse its key elsewhere. Copy the deployed address into `REGISTRY_ADDRESS` in `.env`, then run `docker compose up -d --force-recreate backend` from the repository root.

6. Authorize the fixture's key hash with the registry owner's local account:

```sh
cast send "$REGISTRY_ADDRESS" 'setAuthorization(address,bytes32,bool)' 0x0000000000000000000000000000000000000001 FIXTURE_KEY_HASH true --rpc-url http://localhost:8545 --interactive
```

Export REGISTRY_ADDRESS in your shell or replace it in this command; Compose's .env is not a shell export. Choose Blockchain in the UI to verify. Passing `false` revokes the key; all credentials signed by it become untrusted. For conventional revocation, set the row's `active=false`.

Anvil is ephemeral. Restarting/recreating it may require redeployment and updating REGISTRY_ADDRESS. No automatic contract deployment or transaction signing is done by the API.

Before a public deployment: choose a chain and expected chain ID, verify deployed bytecode and owner, use a protected administrator/multisig, provision authenticated PostgreSQL and trusted RPC access, terminate HTTPS, enforce gateway request limits, disable body/access logging of credential data, establish backups and retention policy, and review the [implemented profile-v1 claims/expiry rules](CREDENTIALS_V1.md) plus any application-specific and credential-revocation requirements. A compromised RPC can falsify chain reads; current code checks chain ID but is not a light client. No finality threshold or historical block pinning is implemented. Keep this prototype local until those requirements are addressed.

The confirmed GitHub destination is `git@github.com:marco-pcg/Open-Credential-Ledger.git` (`origin`). Changes are committed and pushed to `main` only when authorized; review files and secrets before publication.

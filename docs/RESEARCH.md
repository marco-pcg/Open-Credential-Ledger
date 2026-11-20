# Research protocol (proposed, not measured)

Question: What changes when issuer/key trust is resolved through an Ethereum registry instead of a conventional database, keeping signed credentials and Ed25519 verification constant?

Use only generated synthetic credentials. Keep payload sizes (1, 4, 16, 32 KiB), issuer populations, signature algorithm, workload and hardware identical. Compare warm/cold caches, concurrency 1/10/100, valid/invalid signatures and trusted/revoked keys. Include database outage, RPC outage, wrong-chain configuration and stale/compromised trust-source scenarios. Conventional database administration and on-chain owner governance are distinct trust assumptions and must be reported.

Run at least five independent trials after warmup; retain raw timings, environment metadata, commit identifier and random seeds. Report p50/p95/p99 end-to-end latency, completed requests/s, error/timeout rate, availability during fault injection, database CPU/memory/storage, RPC request costs, registry deployment and update gas, and owner key management costs. Record chain ID, block number, finality policy, gas price and exchange-rate timestamp separately; local Anvil costs are not mainnet costs. Keep API verification throughput separate from registry update throughput.

Security analysis: signature forgery, unauthorized enrollment, owner compromise, key theft/rotation, key revocation, replay and stale state. Privacy analysis: ledger permanence, correlation of public institutional keys, RPC access patterns, credential exposure to the verifier, logging and backups. Do not infer privacy merely from use of hashes. Registry stores no credential hashes or student identifiers.

The prototype enforces [profile-v1 claims, signed issuer binding and validity dates](CREDENTIALS_V1.md), plus current key authorization. It does not implement credential-level revocation, holder proof, historical authorization, selective disclosure, finality enforcement, or offline verification. Analyze these limitations explicitly. No benchmark observations or statistical conclusions have yet been produced.

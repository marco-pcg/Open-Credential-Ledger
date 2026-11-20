# Profile-v1 acceptance examples

These are **unsigned synthetic payloads**, not ready-to-submit API envelopes. `manifest.json` fixes the evaluator clock at Unix timestamp `1800000000`, names the envelope issuer and records expected policy decisions. `valid.json` is valid only under that fixture clock; the policy verifier still requires a signature and current issuer authorization.

The Rust acceptance tests consume every manifest case directly. Duplicate-member files intentionally contain duplicate JSON members and must not be normalized or converted into a map before verification.

Generate a currently valid signed envelope with `cargo run -p ocl-api --example fixture > fixture.json`. See [the versioned specification](../../../docs/CREDENTIALS_V1.md) for claims, date rules, signing context and non-W3C compatibility.

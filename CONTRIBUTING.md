# Contributing

Open an issue describing the problem and scope, then submit a focused pull request with tests for changed behavior. Use synthetic data only. Keep credential and student data out of issues, fixtures, logs and commits.

Run the README checks. Rust follows rustfmt and Clippy; Solidity follows forge fmt; TypeScript uses strict type checking. Explain schema migrations and contract ABI changes. Update API documentation and the paper's limitations when verification semantics change. Use descriptive commit messages and disclose validation gaps in the PR. Never commit .env, wallet keys, credentials or database dumps.

By submitting a contribution you agree it is licensed under the repository's MIT license. Discuss incompatible dependencies and major architecture changes before implementation. Report vulnerabilities privately as described in SECURITY.md.

# Security policy

This is an unaudited research prototype. Only the current development branch receives fixes. Do not use real student records or production funds.

For a vulnerability, do not post exploit details or personal data in a public issue. A private reporting destination is not yet confirmed. Before public hosting, the maintainer must enable GitHub private vulnerability reporting or add a verified security contact here. Until then, share only a non-sensitive request for a private contact with the repository owner. No email address is invented.

Trust boundaries: PostgreSQL administrators govern conventional keys; registry owner governs on-chain keys; RPC operator supplies chain state; API operator can observe submitted payloads. Database/RPC unavailability produces an error, not a successful trust decision. Contract state is public. Revocation is per issuer/key, not per credential. No private signing keys are accepted by the API.

Known limitations and deployment requirements are in docs/DEPLOYMENT.md and docs/RESEARCH.md. Signature verification alone does not validate academic claims. Protect administrator access, use HTTPS and gateway abuse controls, rotate keys under a documented policy, and avoid payload logging.

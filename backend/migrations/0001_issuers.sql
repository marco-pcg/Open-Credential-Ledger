-- Public institutional keys only. Never insert student identifiers or credentials.
CREATE TABLE issuers (
 address TEXT PRIMARY KEY CHECK (address ~ '^0x[0-9a-f]{40}$'),
 public_key TEXT NOT NULL CHECK (public_key ~ '^[0-9a-f]{64}$'),
 active BOOLEAN NOT NULL DEFAULT true
);

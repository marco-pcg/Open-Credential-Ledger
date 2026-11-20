-- Only issuer-scoped opaque status handles. No student identifiers, claim data or reasons.
-- No issuer foreign key: retain tombstones across key/issuer removal and re-enrollment.
CREATE TABLE credential_revocations (
 issuer TEXT NOT NULL CHECK (issuer ~ '^0x[0-9a-f]{40}$' AND issuer <> '0x0000000000000000000000000000000000000000'),
 handle TEXT NOT NULL CHECK (handle ~ '^[0-9a-f]{64}$' AND handle <> repeat('0',64)),
 revoked_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
 PRIMARY KEY (issuer, handle)
);
CREATE FUNCTION reject_revocation_mutation() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
 RAISE EXCEPTION 'credential revocations are append-only';
END;
$$;
CREATE TRIGGER immutable_credential_revocations BEFORE UPDATE OR DELETE
 ON credential_revocations FOR EACH ROW EXECUTE FUNCTION reject_revocation_mutation();
CREATE TRIGGER no_truncate_credential_revocations BEFORE TRUNCATE
 ON credential_revocations FOR EACH STATEMENT EXECUTE FUNCTION reject_revocation_mutation();

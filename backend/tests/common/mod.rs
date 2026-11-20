use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use ocl_api::{
    Mode, Verification,
    credential::{PROFILE_VERSION, signing_message},
};
use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
pub fn claims(issuer: &str, now: u64) -> Value {
    json!({"schema_version":PROFILE_VERSION,"credential_id":"urn:ocl:credential:synthetic-001",
        "issuer":issuer,"subject_id":"urn:ocl:subject:synthetic-001",
        "award":{"kind":"degree","name":"Synthetic degree for testing"},
        "issued_at":now-60,"valid_from":now-30,"valid_until":now+3600})
}
pub fn envelope(key: &SigningKey, issuer: &str, payload: &[u8], mode: Mode) -> Verification {
    Verification {
        issuer: issuer.into(),
        public_key: hex::encode(key.verifying_key().to_bytes()),
        payload_base64: STANDARD.encode(payload),
        signature_base64: STANDARD.encode(key.sign(&signing_message(payload)).to_bytes()),
        mode,
    }
}

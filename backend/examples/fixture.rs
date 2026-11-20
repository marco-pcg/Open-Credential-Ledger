use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use ocl_api::credential::{Award, AwardKind, ClaimsV1, PROFILE_VERSION, signing_message};
use std::time::{SystemTime, UNIX_EPOCH};
fn main() {
    // Public deterministic fixture key. NEVER use it for real credentials.
    let key = SigningKey::from_bytes(&[7; 32]);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before Unix epoch")
        .as_secs();
    let issuer = "0x0000000000000000000000000000000000000001";
    let claims = ClaimsV1 {
        schema_version: PROFILE_VERSION.into(),
        credential_id: "urn:ocl:credential:synthetic-001".into(),
        issuer: issuer.into(),
        subject_id: "urn:ocl:subject:synthetic-001".into(),
        award: Award {
            kind: AwardKind::Degree,
            name: "Synthetic degree for research only".into(),
        },
        issued_at: now,
        valid_from: now,
        valid_until: now + 86400,
    };
    let payload = serde_json::to_vec(&claims).unwrap();
    println!(
        "{}",
        serde_json::json!({"issuer":issuer,"public_key":hex::encode(key.verifying_key().to_bytes()),
        "payload_base64":STANDARD.encode(&payload),"signature_base64":STANDARD.encode(key.sign(&signing_message(&payload)).to_bytes()),"mode":"conventional"})
    );
    eprintln!(
        "keyHash: {:#x}",
        alloy::primitives::keccak256(key.verifying_key().to_bytes())
    );
    eprintln!("Synthetic profile v1 fixture expires in 24 hours; regenerate as needed.");
}

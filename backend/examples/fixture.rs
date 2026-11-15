use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
fn main() {
    // Public deterministic fixture. NEVER use this key for real credentials.
    let key = SigningKey::from_bytes(&[7; 32]);
    let public_key = hex::encode(key.verifying_key().to_bytes());
    let payload = b"synthetic diploma for research only";
    println!(
        "{}",
        serde_json::json!({"issuer":"0x0000000000000000000000000000000000000001","public_key":public_key,"payload_base64":STANDARD.encode(payload),"signature_base64":STANDARD.encode(key.sign(payload).to_bytes()),"mode":"conventional"})
    );
    eprintln!(
        "keyHash: {:#x}",
        alloy::primitives::keccak256(key.verifying_key().to_bytes())
    );
}

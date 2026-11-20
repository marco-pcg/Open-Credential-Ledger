use ocl_api::credential::{is_credential_id, new_credential_id, revocation_handle};
#[test]
fn handle_vector_and_issuer_scope() {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../../examples/revocation/handle-v1.json")).unwrap();
    let issuer = v["issuer"].as_str().unwrap().parse().unwrap();
    let id = v["credential_id"].as_str().unwrap();
    assert_eq!(format!("{:#x}", revocation_handle(issuer, id)), v["handle"]);
    assert_ne!(
        revocation_handle(issuer, id),
        revocation_handle(
            "0x0000000000000000000000000000000000000002"
                .parse()
                .unwrap(),
            id
        )
    );
    assert_ne!(
        revocation_handle(issuer, id),
        revocation_handle(issuer, "urn:ocl:credential:other")
    );
}
#[test]
fn generated_identifiers_are_random_256_bit_tokens_in_the_profile_namespace() {
    let first = new_credential_id().unwrap();
    let second = new_credential_id().unwrap();
    assert!(is_credential_id(&first));
    assert_ne!(first, second);
    assert_eq!(first.strip_prefix("urn:ocl:credential:").unwrap().len(), 64);
    assert_eq!(
        hex::decode(first.strip_prefix("urn:ocl:credential:").unwrap())
            .unwrap()
            .len(),
        32
    );
}

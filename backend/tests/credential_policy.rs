mod common;
use alloy::primitives::Address;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use ocl_api::{
    AppState, Mode, Verification, app, check_signature,
    credential::{Decision, PROFILE_VERSION, parse_issuer, signing_message, validate_claims},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;
use tower::ServiceExt;
const ISSUER: &str = "0x0000000000000000000000000000000000000001";
const NOW: u64 = 1_800_000_000;
#[derive(Deserialize)]
struct Manifest {
    evaluated_at: u64,
    envelope_issuer: String,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    file: String,
    reason: Decision,
}
fn vectors() -> (std::path::PathBuf, Manifest) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/credentials/v1");
    let manifest =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    (root, manifest)
}
fn state() -> AppState {
    AppState {
        db: sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://unused:unused@localhost/unused")
            .unwrap(),
        rpc_url: None,
        registry: None,
        chain_id: None,
    }
}
async fn request(v: &Verification) -> (StatusCode, Value) {
    common::request(state(), v).await
}

#[test]
fn published_examples_match_the_policy() {
    let (root, manifest) = vectors();
    let issuer = manifest.envelope_issuer.parse().unwrap();
    for case in manifest.cases {
        let payload = std::fs::read(root.join(&case.file)).unwrap();
        let actual = validate_claims(&payload, issuer, manifest.evaluated_at)
            .err()
            .unwrap_or(Decision::Verified);
        assert_eq!(actual, case.reason, "{}", case.file);
    }
}
#[test]
fn validity_boundaries_are_inclusive_start_exclusive_end() {
    let mut c = common::claims(ISSUER, NOW);
    c["issued_at"] = json!(NOW);
    c["valid_from"] = json!(NOW);
    c["valid_until"] = json!(NOW + 1);
    let raw = serde_json::to_vec(&c).unwrap();
    let issuer = ISSUER.parse().unwrap();
    assert!(validate_claims(&raw, issuer, NOW).is_ok());
    assert_eq!(
        validate_claims(&raw, issuer, NOW - 1).unwrap_err(),
        Decision::NotYetValid
    );
    assert_eq!(
        validate_claims(&raw, issuer, NOW + 1).unwrap_err(),
        Decision::Expired
    );
}
#[test]
fn missing_fields_and_ambiguous_types_fail_closed() {
    for field in [
        "schema_version",
        "credential_id",
        "issuer",
        "subject_id",
        "award",
        "issued_at",
        "valid_from",
        "valid_until",
    ] {
        let mut c = common::claims(ISSUER, NOW);
        c.as_object_mut().unwrap().remove(field);
        assert_eq!(
            validate_claims(
                &serde_json::to_vec(&c).unwrap(),
                ISSUER.parse().unwrap(),
                NOW
            )
            .unwrap_err(),
            Decision::InvalidClaims,
            "{field}"
        );
    }
    for token in ["1800000000.0", "1.8e9", "\"1800000000\""] {
        let raw = serde_json::to_string(&common::claims(ISSUER, NOW))
            .unwrap()
            .replace(
                &format!("\"issued_at\":{}", NOW - 60),
                &format!("\"issued_at\":{token}"),
            );
        assert_eq!(
            validate_claims(raw.as_bytes(), ISSUER.parse().unwrap(), NOW).unwrap_err(),
            Decision::InvalidClaims
        );
    }
    for payload in [b"not JSON".as_slice(), b"[]", b"{} {}", &[0xff]] {
        assert_eq!(
            validate_claims(payload, ISSUER.parse().unwrap(), NOW).unwrap_err(),
            Decision::InvalidClaims
        );
    }
}
#[test]
fn rejects_bad_ids_names_and_dates() {
    for (field, value, reason) in [
        (
            "credential_id",
            json!("urn:ocl:credential:"),
            Decision::InvalidClaims,
        ),
        (
            "credential_id",
            json!(format!("urn:ocl:credential:{}", "x".repeat(97))),
            Decision::InvalidClaims,
        ),
        (
            "subject_id",
            json!("urn:ocl:subject:a b"),
            Decision::InvalidClaims,
        ),
        ("issued_at", json!(0), Decision::InvalidDates),
    ] {
        let mut c = common::claims(ISSUER, NOW);
        c[field] = value;
        assert_eq!(
            validate_claims(
                &serde_json::to_vec(&c).unwrap(),
                ISSUER.parse().unwrap(),
                NOW
            )
            .unwrap_err(),
            reason
        );
    }
    for name in [
        "".into(),
        " name".into(),
        "name ".into(),
        "line\nbreak".into(),
        "x".repeat(257),
    ] {
        let mut c = common::claims(ISSUER, NOW);
        c["award"]["name"] = json!(name);
        assert_eq!(
            validate_claims(
                &serde_json::to_vec(&c).unwrap(),
                ISSUER.parse().unwrap(),
                NOW
            )
            .unwrap_err(),
            Decision::InvalidClaims
        );
    }
    let mut c = common::claims(ISSUER, NOW);
    c["award"] = json!({"kind":"course_completion","name":"Formação sintética"});
    assert!(
        validate_claims(
            &serde_json::to_vec(&c).unwrap(),
            ISSUER.parse().unwrap(),
            NOW
        )
        .is_ok()
    );
}
#[test]
fn institutional_addresses_have_one_nonzero_canonical_form() {
    assert_eq!(parse_issuer(ISSUER), Some(ISSUER.parse().unwrap()));
    for value in [
        "0x0000000000000000000000000000000000000000",
        "0X0000000000000000000000000000000000000001",
        "0x000000000000000000000000000000000000000A",
        "0x1",
    ] {
        assert!(parse_issuer(value).is_none());
    }
    let claims = common::claims(ISSUER, NOW);
    assert_eq!(
        validate_claims(
            &serde_json::to_vec(&claims).unwrap(),
            Address::from([2; 20]),
            NOW
        )
        .unwrap_err(),
        Decision::IssuerMismatch
    );
}
#[tokio::test]
async fn signed_invalid_claims_are_rejected_before_either_trust_lookup() {
    let key = SigningKey::from_bytes(&[7; 32]);
    let now = common::now();
    for blockchain in [false, true] {
        for (field, value, reason) in [
            ("valid_until", json!(now - 1), "expired"),
            ("valid_from", json!(now + 3600), "invalid_dates"),
            ("valid_from", json!(now + 60), "not_yet_valid"),
            (
                "issuer",
                json!("0x0000000000000000000000000000000000000002"),
                "issuer_mismatch",
            ),
            (
                "schema_version",
                json!("ocl.academic.v2"),
                "unsupported_version",
            ),
            ("subject_id", Value::Null, "invalid_claims"),
        ] {
            let mut c = common::claims(ISSUER, now);
            c[field] = value;
            let mode = if blockchain {
                Mode::Blockchain
            } else {
                Mode::Conventional
            };
            let v = common::envelope(&key, ISSUER, &serde_json::to_vec(&c).unwrap(), mode);
            let (status, body) = request(&v).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body["valid"], false);
            assert_eq!(body["signature_valid"], true);
            assert_eq!(body["claims_valid"], false);
            assert!(body["issuer_trusted"].is_null());
            assert!(body["credential_revoked"].is_null());
            assert_eq!(body["reason"], reason);
            assert_eq!(body["policy_version"], PROFILE_VERSION);
            assert!(body["evaluated_at"].as_u64().unwrap() >= now);
        }
    }
}
#[tokio::test]
async fn changing_signed_bytes_fails_before_claims_or_trust() {
    let key = SigningKey::from_bytes(&[7; 32]);
    let raw = serde_json::to_vec(&common::claims(ISSUER, common::now())).unwrap();
    let mut v = common::envelope(&key, ISSUER, &raw, Mode::Conventional);
    let mut changed = raw;
    changed.push(b' ');
    v.payload_base64 = STANDARD.encode(changed);
    let (status, body) = request(&v).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["reason"], "invalid_signature");
    assert_eq!(body["claims_valid"], false);
    assert!(body["issuer_trusted"].is_null());
    assert!(body["credential_revoked"].is_null());
}
#[tokio::test]
async fn valid_claims_do_not_hide_a_trust_source_outage() {
    let key = SigningKey::from_bytes(&[7; 32]);
    let raw = serde_json::to_vec(&common::claims(ISSUER, common::now())).unwrap();
    let v = common::envelope(&key, ISSUER, &raw, Mode::Blockchain);
    assert_eq!(request(&v).await.0, StatusCode::SERVICE_UNAVAILABLE);
}
#[test]
fn signing_context_and_original_bytes_are_required() {
    let key = SigningKey::from_bytes(&[7; 32]);
    let c = common::claims(ISSUER, NOW);
    let raw = serde_json::to_vec(&c).unwrap();
    let mut v = common::envelope(&key, ISSUER, &raw, Mode::Conventional);
    assert!(check_signature(&v).unwrap().2);
    v.payload_base64 = STANDARD.encode(serde_json::to_vec_pretty(&c).unwrap());
    assert!(!check_signature(&v).unwrap().2);
    v.payload_base64 = STANDARD.encode(&raw);
    v.signature_base64 = STANDARD.encode(key.sign(&raw).to_bytes());
    assert!(!check_signature(&v).unwrap().2);
    assert!(signing_message(&raw).ends_with(&raw));
}
#[tokio::test]
async fn malformed_envelopes_are_client_errors() {
    let key = SigningKey::from_bytes(&[7; 32]);
    let raw = serde_json::to_vec(&common::claims(ISSUER, common::now())).unwrap();
    for field in ["issuer", "public_key", "payload_base64", "signature_base64"] {
        let mut v = common::envelope(&key, ISSUER, &raw, Mode::Conventional);
        match field {
            "issuer" => v.issuer = "0x0".into(),
            "public_key" => v.public_key = "zz".into(),
            "payload_base64" => v.payload_base64 = "!".into(),
            _ => v.signature_base64 = "!".into(),
        }
        assert_eq!(request(&v).await.0, StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn envelope_structure_errors_do_not_echo_submitted_data() {
    let r = app(state())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/verify")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"private-submitted-field":"sensitive-value"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let bytes = to_bytes(r.into_body(), 65536).await.unwrap();
    assert_eq!(bytes.as_ref(), b"invalid verification envelope");
}

#[test]
fn rejects_positional_arrays_for_claims_and_award() {
    let c = common::claims(ISSUER, NOW);
    let positional = json!([
        c["schema_version"],
        c["credential_id"],
        c["issuer"],
        c["subject_id"],
        c["award"],
        c["issued_at"],
        c["valid_from"],
        c["valid_until"]
    ]);
    let mut nested = c.clone();
    nested["award"] = json!(["degree", "Synthetic degree"]);
    let mut kind_object = c.clone();
    kind_object["award"]["kind"] = json!({"degree":null});
    for value in [positional, nested, kind_object] {
        assert_eq!(
            validate_claims(
                &serde_json::to_vec(&value).unwrap(),
                ISSUER.parse().unwrap(),
                NOW
            )
            .err(),
            Some(Decision::InvalidClaims)
        );
    }
}
#[tokio::test]
async fn rejects_positional_envelope_arrays() {
    let key = SigningKey::from_bytes(&[7; 32]);
    let v = common::envelope(
        &key,
        ISSUER,
        &serde_json::to_vec(&common::claims(ISSUER, common::now())).unwrap(),
        Mode::Blockchain,
    );
    let positional = json!([
        v.issuer,
        v.public_key,
        v.payload_base64,
        v.signature_base64,
        v.mode
    ]);
    let r = app(state())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/verify")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&positional).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn rejects_object_encoded_mode_variants() {
    let key = SigningKey::from_bytes(&[7; 32]);
    let v = common::envelope(
        &key,
        ISSUER,
        &serde_json::to_vec(&common::claims(ISSUER, common::now())).unwrap(),
        Mode::Blockchain,
    );
    let mut envelope = serde_json::to_value(&v).unwrap();
    envelope["mode"] = json!({"blockchain":null});
    let r = app(state())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/verify")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&envelope).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

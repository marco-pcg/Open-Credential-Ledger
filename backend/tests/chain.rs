mod common;
use alloy::{
    primitives::{Address, Bytes, keccak256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use ed25519_dalek::SigningKey;
use ocl_api::{AppState, IIssuerRegistry, Mode, app};
use tower::ServiceExt;
#[tokio::test]
#[ignore = "requires Anvil RPC_URL and forge build artifacts"]
async fn onchain_authorization_revocation_and_wrong_chain() {
    let url = std::env::var("RPC_URL").unwrap();
    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());
    let owner = provider.get_accounts().await.unwrap()[0];
    let artifact: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string("../contracts/out/IssuerRegistry.sol/IssuerRegistry.json")
            .unwrap(),
    )
    .unwrap();
    let mut code = hex::decode(
        artifact["bytecode"]["object"]
            .as_str()
            .unwrap()
            .trim_start_matches("0x"),
    )
    .unwrap();
    code.extend_from_slice(&[0; 12]);
    code.extend_from_slice(owner.as_slice());
    let receipt = provider
        .send_transaction(
            TransactionRequest::default()
                .from(owner)
                .input(Bytes::from(code).into()),
        )
        .await
        .unwrap()
        .get_receipt()
        .await
        .unwrap();
    assert!(receipt.status());
    let address = receipt.contract_address.unwrap();
    let registry = IIssuerRegistry::new(address, provider.clone());
    let issuer = Address::from([1; 20]);
    let key = SigningKey::from_bytes(&[7; 32]);
    let hash = keccak256(key.verifying_key().to_bytes());
    let db = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@localhost/unused")
        .unwrap();
    let mut state = AppState {
        db,
        rpc_url: Some(url),
        registry: Some(address),
        chain_id: Some(31337),
    };
    let claims = common::claims(&format!("{issuer:#x}"), common::now());
    let v = common::envelope(
        &key,
        &format!("{issuer:#x}"),
        &serde_json::to_vec(&claims).unwrap(),
        Mode::Blockchain,
    );
    for trusted in [false, true, false] {
        assert!(
            registry
                .setAuthorization(issuer, hash, trusted)
                .from(owner)
                .send()
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap()
                .status()
        );
        let response = app(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/verify")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&v).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
        assert_eq!(body["valid"], trusted);
        assert_eq!(body["claims_valid"], true);
        assert_eq!(body["issuer_trusted"], trusted);
    }
    state.chain_id = Some(1);
    let response = app(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/verify")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&v).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

async fn deploy(
    provider: &impl Provider,
    owner: Address,
    artifact: &str,
    with_owner: bool,
) -> Address {
    let artifact: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(format!("../contracts/out/{artifact}.sol/{artifact}.json"))
            .unwrap(),
    )
    .unwrap();
    let mut code = hex::decode(
        artifact["bytecode"]["object"]
            .as_str()
            .unwrap()
            .trim_start_matches("0x"),
    )
    .unwrap();
    if with_owner {
        code.extend_from_slice(&[0; 12]);
        code.extend_from_slice(owner.as_slice());
    }
    let receipt = provider
        .send_transaction(
            TransactionRequest::default()
                .from(owner)
                .input(Bytes::from(code).into()),
        )
        .await
        .unwrap()
        .get_receipt()
        .await
        .unwrap();
    assert!(receipt.status());
    receipt.contract_address.unwrap()
}
#[tokio::test]
#[ignore = "requires Anvil RPC_URL and forge build artifacts"]
async fn credential_revocation_is_independent_of_key_and_other_credentials() {
    use ocl_api::credential::{new_credential_id, revocation_handle};
    let url = std::env::var("RPC_URL").unwrap();
    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());
    let owner = provider.get_accounts().await.unwrap()[1];
    let address = deploy(&provider, owner, "IssuerRegistry", true).await;
    let registry = IIssuerRegistry::new(address, provider.clone());
    let issuer = "0x0000000000000000000000000000000000000044";
    let other = "0x0000000000000000000000000000000000000045";
    let key = SigningKey::from_bytes(&[7; 32]);
    let key_hash = keccak256(key.verifying_key().to_bytes());
    for address in [issuer, other] {
        assert!(
            registry
                .setAuthorization(address.parse().unwrap(), key_hash, true)
                .from(owner)
                .send()
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap()
                .status()
        );
    }
    let state = AppState {
        db: sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://unused:unused@localhost/unused")
            .unwrap(),
        rpc_url: Some(url),
        registry: Some(address),
        chain_id: Some(31337),
    };
    let first = common::claims(issuer, common::now());
    let mut second = first.clone();
    second["credential_id"] = serde_json::json!(new_credential_id().unwrap());
    let mut other_claims = first.clone();
    other_claims["issuer"] = serde_json::json!(other);
    let envelopes: [ocl_api::Verification; 3] = std::array::from_fn(|i| {
        let c = [&first, &second, &other_claims][i];
        common::envelope(
            &key,
            c["issuer"].as_str().unwrap(),
            &serde_json::to_vec(c).unwrap(),
            Mode::Blockchain,
        )
    });
    for v in &envelopes {
        let (status, body) = common::request(state.clone(), v).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["valid"], true);
        assert_eq!(body["credential_revoked"], false);
    }
    let handle = revocation_handle(
        issuer.parse().unwrap(),
        first["credential_id"].as_str().unwrap(),
    );
    for _ in 0..2 {
        assert!(
            registry
                .revokeCredential(issuer.parse().unwrap(), handle)
                .from(owner)
                .send()
                .await
                .unwrap()
                .get_receipt()
                .await
                .unwrap()
                .status()
        );
    }
    for (i, v) in envelopes.iter().enumerate() {
        let (status, body) = common::request(state.clone(), v).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["issuer_trusted"], true);
        assert_eq!(body["valid"], i != 0);
        assert_eq!(body["credential_revoked"], i == 0);
        assert_eq!(
            body["reason"],
            if i == 0 {
                "revoked_credential"
            } else {
                "verified"
            }
        );
    }
    assert!(
        registry
            .isAuthorized(issuer.parse().unwrap(), key_hash)
            .call()
            .await
            .unwrap()
    );
    let rotated = SigningKey::from_bytes(&[8; 32]);
    assert!(
        registry
            .setAuthorization(
                issuer.parse().unwrap(),
                keccak256(rotated.verifying_key().to_bytes()),
                true
            )
            .from(owner)
            .send()
            .await
            .unwrap()
            .get_receipt()
            .await
            .unwrap()
            .status()
    );
    let v = common::envelope(
        &rotated,
        issuer,
        &serde_json::to_vec(&first).unwrap(),
        Mode::Blockchain,
    );
    assert_eq!(
        common::request(state.clone(), &v).await.1["reason"],
        "revoked_credential"
    );
    let unauthorized = SigningKey::from_bytes(&[9; 32]);
    let v = common::envelope(
        &unauthorized,
        issuer,
        &serde_json::to_vec(&first).unwrap(),
        Mode::Blockchain,
    );
    let (_, body) = common::request(state, &v).await;
    assert_eq!(body["reason"], "untrusted_issuer");
    assert!(body["credential_revoked"].is_null());
}
#[tokio::test]
#[ignore = "requires Anvil RPC_URL and forge build artifacts"]
async fn old_registry_without_revocation_support_fails_closed() {
    let url = std::env::var("RPC_URL").unwrap();
    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());
    let owner = provider.get_accounts().await.unwrap()[2];
    let address = deploy(&provider, owner, "LegacyIssuerRegistry", false).await;
    let state = AppState {
        db: sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://unused:unused@localhost/unused")
            .unwrap(),
        rpc_url: Some(url),
        registry: Some(address),
        chain_id: Some(31337),
    };
    let issuer = "0x0000000000000000000000000000000000000001";
    let key = SigningKey::from_bytes(&[7; 32]);
    let v = common::envelope(
        &key,
        issuer,
        &serde_json::to_vec(&common::claims(issuer, common::now())).unwrap(),
        Mode::Blockchain,
    );
    assert_eq!(
        common::request(state, &v).await.0,
        StatusCode::SERVICE_UNAVAILABLE
    );
}

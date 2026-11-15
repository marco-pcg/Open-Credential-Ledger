use alloy::{
    primitives::{Address, Bytes, keccak256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use ocl_api::{AppState, IIssuerRegistry, Mode, Verification, app};
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
    let v = Verification {
        issuer: format!("{issuer:#x}"),
        public_key: hex::encode(key.verifying_key().to_bytes()),
        payload_base64: STANDARD.encode(b"synthetic"),
        signature_base64: STANDARD.encode(key.sign(b"synthetic").to_bytes()),
        mode: Mode::Blockchain,
    };
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

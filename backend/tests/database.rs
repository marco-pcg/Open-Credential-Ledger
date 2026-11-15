use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use ocl_api::{AppState, Mode, Verification, app};
use tower::ServiceExt;
#[tokio::test]
#[ignore = "requires disposable PostgreSQL DATABASE_URL"]
async fn issuer_trust_and_revocation() {
    let db = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&db).await.unwrap();
    let key = SigningKey::from_bytes(&[42; 32]);
    let issuer = "0x0000000000000000000000000000000000000042";
    sqlx::query("INSERT INTO issuers(address,public_key,active) VALUES ($1,$2,true) ON CONFLICT(address) DO UPDATE SET public_key=EXCLUDED.public_key,active=true")
 .bind(issuer).bind(hex::encode(key.verifying_key().to_bytes())).execute(&db).await.unwrap();
    let state = AppState {
        db: db.clone(),
        rpc_url: None,
        registry: None,
        chain_id: None,
    };
    let v = Verification {
        issuer: issuer.into(),
        public_key: hex::encode(key.verifying_key().to_bytes()),
        payload_base64: STANDARD.encode(b"synthetic"),
        signature_base64: STANDARD.encode(key.sign(b"synthetic").to_bytes()),
        mode: Mode::Conventional,
    };
    for expected in [true, false] {
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
        assert_eq!(body["valid"], expected);
        sqlx::query("UPDATE issuers SET active=false WHERE address=$1")
            .bind(issuer)
            .execute(&db)
            .await
            .unwrap();
    }
    sqlx::query("DELETE FROM issuers WHERE address=$1")
        .bind(issuer)
        .execute(&db)
        .await
        .unwrap();
}

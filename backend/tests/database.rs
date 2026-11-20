mod common;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use ed25519_dalek::SigningKey;
use ocl_api::{AppState, Mode, app};
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
    let claims = common::claims(issuer, common::now());
    let v = common::envelope(
        &key,
        issuer,
        &serde_json::to_vec(&claims).unwrap(),
        Mode::Conventional,
    );
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
        assert_eq!(body["claims_valid"], true);
        assert_eq!(body["issuer_trusted"], expected);
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

#[tokio::test]
#[ignore = "requires disposable PostgreSQL DATABASE_URL"]
async fn individual_revocation_is_scoped_and_survives_key_changes() {
    use ocl_api::credential::{new_credential_id, revocation_handle};
    let db = sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&db).await.unwrap();
    let issuer = "0x0000000000000000000000000000000000000044";
    let other = "0x0000000000000000000000000000000000000045";
    let key = SigningKey::from_bytes(&[7; 32]);
    for address in [issuer, other] {
        sqlx::query("INSERT INTO issuers(address,public_key,active) VALUES ($1,$2,true) ON CONFLICT(address) DO UPDATE SET public_key=EXCLUDED.public_key,active=true")
          .bind(address).bind(hex::encode(key.verifying_key().to_bytes())).execute(&db).await.unwrap();
    }
    let state = AppState {
        db: db.clone(),
        rpc_url: None,
        registry: None,
        chain_id: None,
    };
    let first = common::claims(issuer, common::now());
    let mut second = first.clone();
    second["credential_id"] = serde_json::json!(new_credential_id().unwrap());
    let mut other_claims = first.clone();
    other_claims["issuer"] = serde_json::json!(other);
    let envelopes: [ocl_api::Verification; 3] = std::array::from_fn(|i| {
        let claims = [&first, &second, &other_claims][i];
        common::envelope(
            &key,
            claims["issuer"].as_str().unwrap(),
            &serde_json::to_vec(claims).unwrap(),
            Mode::Conventional,
        )
    });
    for v in &envelopes {
        let (status, body) = common::request(state.clone(), v).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["valid"], true);
        assert_eq!(body["credential_revoked"], false);
    }
    let handle = hex::encode(revocation_handle(
        issuer.parse().unwrap(),
        first["credential_id"].as_str().unwrap(),
    ));
    for _ in 0..2 {
        sqlx::query("INSERT INTO credential_revocations(issuer,handle) VALUES ($1,$2) ON CONFLICT DO NOTHING")
            .bind(issuer).bind(&handle).execute(&db).await.unwrap();
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
    // A different authorized key cannot revive the revoked credential identity.
    let rotated = SigningKey::from_bytes(&[8; 32]);
    sqlx::query("UPDATE issuers SET public_key=$2 WHERE address=$1")
        .bind(issuer)
        .bind(hex::encode(rotated.verifying_key().to_bytes()))
        .execute(&db)
        .await
        .unwrap();
    let rotated_envelope = common::envelope(
        &rotated,
        issuer,
        &serde_json::to_vec(&first).unwrap(),
        Mode::Conventional,
    );
    let (_, body) = common::request(state.clone(), &rotated_envelope).await;
    assert_eq!(body["reason"], "revoked_credential");
    assert_eq!(body["issuer_trusted"], true);
    let (_, body) = common::request(state, &envelopes[0]).await;
    assert_eq!(body["reason"], "untrusted_issuer");
    assert!(body["credential_revoked"].is_null());
    assert!(
        sqlx::query("DELETE FROM credential_revocations WHERE issuer=$1 AND handle=$2")
            .bind(issuer)
            .bind(&handle)
            .execute(&db)
            .await
            .is_err()
    );
    assert!(sqlx::query("UPDATE credential_revocations SET revoked_at=CURRENT_TIMESTAMP WHERE issuer=$1 AND handle=$2").bind(issuer).bind(&handle).execute(&db).await.is_err());
    // Keep the append-only tombstone; remove only these synthetic issuer enrollments.
    for address in [issuer, other] {
        sqlx::query("DELETE FROM issuers WHERE address=$1")
            .bind(address)
            .execute(&db)
            .await
            .unwrap();
    }
}

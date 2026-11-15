use alloy::{
    primitives::{Address, keccak256},
    providers::{Provider, ProviderBuilder},
    sol,
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::time::Duration;

sol! { #[sol(rpc)] interface IIssuerRegistry {
    function isAuthorized(address issuer, bytes32 keyHash) external view returns (bool);
    function setAuthorization(address issuer, bytes32 keyHash, bool active) external;
} }
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub rpc_url: Option<String>,
    pub registry: Option<Address>,
    pub chain_id: Option<u64>,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Conventional,
    Blockchain,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Verification {
    pub issuer: String,
    pub public_key: String,
    pub payload_base64: String,
    pub signature_base64: String,
    pub mode: Mode,
}
#[derive(Serialize)]
pub struct Outcome {
    pub valid: bool,
    pub signature_valid: bool,
    pub issuer_trusted: bool,
}
type ApiError = (StatusCode, &'static str);
fn bad() -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        "invalid issuer, key, payload or signature encoding",
    )
}
fn unavailable() -> ApiError {
    (StatusCode::SERVICE_UNAVAILABLE, "trust source unavailable")
}
pub fn check_signature(v: &Verification) -> Result<(Address, [u8; 32], bool), ApiError> {
    let issuer = v.issuer.parse::<Address>().map_err(|_| bad())?;
    let key: [u8; 32] = hex::decode(&v.public_key)
        .map_err(|_| bad())?
        .try_into()
        .map_err(|_| bad())?;
    let key_obj = VerifyingKey::from_bytes(&key).map_err(|_| bad())?;
    let payload = STANDARD.decode(&v.payload_base64).map_err(|_| bad())?;
    if payload.is_empty() || payload.len() > 32768 {
        return Err(bad());
    }
    let signature =
        Signature::from_slice(&STANDARD.decode(&v.signature_base64).map_err(|_| bad())?)
            .map_err(|_| bad())?;
    Ok((
        issuer,
        key,
        key_obj.verify_strict(&payload, &signature).is_ok(),
    ))
}
async fn verify(
    State(s): State<AppState>,
    Json(v): Json<Verification>,
) -> Result<Json<Outcome>, ApiError> {
    let (issuer, key, signature_valid) = check_signature(&v)?;
    if !signature_valid {
        return Ok(Json(Outcome {
            valid: false,
            signature_valid: false,
            issuer_trusted: false,
        }));
    }
    let trusted = match v.mode {
        Mode::Conventional => {
            let row = sqlx::query("SELECT public_key, active FROM issuers WHERE address = $1")
                .bind(format!("{issuer:#x}"))
                .fetch_optional(&s.db)
                .await
                .map_err(|_| unavailable())?;
            row.is_some_and(|r| {
                r.get::<bool, _>("active") && r.get::<String, _>("public_key") == hex::encode(key)
            })
        }
        Mode::Blockchain => {
            let url = s
                .rpc_url
                .as_ref()
                .ok_or_else(unavailable)?
                .parse()
                .map_err(|_| unavailable())?;
            let provider = ProviderBuilder::new().connect_http(url);
            if provider.get_chain_id().await.map_err(|_| unavailable())?
                != s.chain_id.ok_or_else(unavailable)?
            {
                return Err(unavailable());
            }
            let registry = IIssuerRegistry::new(s.registry.ok_or_else(unavailable)?, provider);
            registry
                .isAuthorized(issuer, keccak256(key))
                .call()
                .await
                .map_err(|_| unavailable())?
        }
    };
    Ok(Json(Outcome {
        valid: trusted,
        signature_valid,
        issuer_trusted: trusted,
    }))
}
async fn ready(State(s): State<AppState>) -> Result<&'static str, ApiError> {
    sqlx::query("SELECT 1")
        .execute(&s.db)
        .await
        .map_err(|_| unavailable())?;
    Ok("ready")
}
pub fn app(s: AppState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(ready))
        .route("/api/verify", post(verify))
        .layer(DefaultBodyLimit::max(65536))
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .with_state(s)
}
#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    fn fixture() -> Verification {
        let key = SigningKey::from_bytes(&[7; 32]);
        Verification {
            issuer: format!("{:#x}", Address::ZERO),
            public_key: hex::encode(key.verifying_key().to_bytes()),
            payload_base64: STANDARD.encode(b"synthetic diploma"),
            signature_base64: STANDARD.encode(key.sign(b"synthetic diploma").to_bytes()),
            mode: Mode::Conventional,
        }
    }
    #[test]
    fn accepts_signed_bytes() {
        assert!(check_signature(&fixture()).unwrap().2);
    }
    #[test]
    fn rejects_tampering() {
        let mut v = fixture();
        v.payload_base64 = STANDARD.encode(b"different");
        assert!(!check_signature(&v).unwrap().2);
    }
    #[test]
    fn rejects_malformed() {
        let mut v = fixture();
        v.public_key = "bad".into();
        assert!(check_signature(&v).is_err());
    }
    #[test]
    fn rejects_empty_payload() {
        let mut v = fixture();
        v.payload_base64.clear();
        assert!(check_signature(&v).is_err());
    }
    #[tokio::test]
    async fn liveness_does_not_require_database() {
        use tower::ServiceExt;
        let s = AppState {
            db: sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://unused:unused@localhost/unused")
                .unwrap(),
            rpc_url: None,
            registry: None,
            chain_id: None,
        };
        let r = app(s)
            .oneshot(
                axum::http::Request::builder()
                    .uri("/healthz")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
    }
}

//! OCL academic credential profile v1. See docs/CREDENTIALS_V1.md.
use alloy::primitives::Address;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{MapAccess, Visitor, value::MapAccessDeserializer},
};
use std::{fmt, marker::PhantomData};

/// JSON struct deserialization also accepts positional arrays by default. Require a map
/// without converting to Value, so duplicate-member detection remains intact.
pub(crate) fn deserialize_object<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct ObjectVisitor<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> Visitor<'de> for ObjectVisitor<T> {
        type Value = T;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a JSON object")
        }
        fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<T, M::Error> {
            T::deserialize(MapAccessDeserializer::new(map))
        }
    }
    deserializer.deserialize_map(ObjectVisitor(PhantomData))
}

pub(crate) struct JsonObject<T>(pub T);
impl<'de, T: Deserialize<'de>> Deserialize<'de> for JsonObject<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize_object(deserializer).map(Self)
    }
}

pub const PROFILE_VERSION: &str = "ocl.academic.v1";
pub const SIGNING_CONTEXT: &[u8] = b"OpenCredentialLedger:academic:v1\n";
// Safe integer ceiling for interoperable JSON clients (2^53 - 1).
pub const MAX_TIMESTAMP: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimsV1 {
    pub schema_version: String,
    pub credential_id: String,
    pub issuer: String,
    pub subject_id: String,
    #[serde(deserialize_with = "deserialize_object")]
    pub award: Award,
    pub issued_at: u64,
    pub valid_from: u64,
    pub valid_until: u64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Award {
    pub kind: AwardKind,
    pub name: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AwardKind {
    Degree,
    CourseCompletion,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Verified,
    InvalidSignature,
    InvalidClaims,
    UnsupportedVersion,
    IssuerMismatch,
    InvalidDates,
    NotYetValid,
    Expired,
    UntrustedIssuer,
}

/// Sign the context followed by the original UTF-8 JSON bytes, never reserialized JSON.
pub fn signing_message(payload: &[u8]) -> Vec<u8> {
    [SIGNING_CONTEXT, payload].concat()
}

/// Require one nonzero, lowercase 0x-prefixed 20-byte institutional address.
pub fn parse_issuer(value: &str) -> Option<Address> {
    if value.len() != 42 || !value.starts_with("0x") {
        return None;
    }
    if !value[2..]
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return None;
    }
    value.parse::<Address>().ok().filter(|a| !a.is_zero())
}
fn opaque_id(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(|id| {
        (1..=96).contains(&id.len())
            && id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    })
}

/// Exact boundaries: issued_at <= valid_from <= now < valid_until; no clock tolerance.
pub fn validate_claims(
    payload: &[u8],
    envelope_issuer: Address,
    now: u64,
) -> Result<ClaimsV1, Decision> {
    // Deserialize directly into strict structs: duplicate/unknown members, missing fields,
    // invalid UTF-8, trailing data, fractional/negative timestamps and nesting errors fail.
    let claims = serde_json::from_slice::<JsonObject<ClaimsV1>>(payload)
        .map_err(|_| Decision::InvalidClaims)?
        .0;
    if claims.schema_version != PROFILE_VERSION {
        return Err(Decision::UnsupportedVersion);
    }
    if parse_issuer(&claims.issuer) != Some(envelope_issuer) {
        return Err(Decision::IssuerMismatch);
    }
    if !opaque_id(&claims.credential_id, "urn:ocl:credential:")
        || !opaque_id(&claims.subject_id, "urn:ocl:subject:")
        || claims.award.name.trim() != claims.award.name
        || claims.award.name.is_empty()
        || claims.award.name.chars().count() > 256
        || claims.award.name.chars().any(char::is_control)
    {
        return Err(Decision::InvalidClaims);
    }
    if claims.issued_at == 0
        || claims.issued_at > claims.valid_from
        || claims.valid_from >= claims.valid_until
        || claims.valid_until > MAX_TIMESTAMP
    {
        return Err(Decision::InvalidDates);
    }
    if now < claims.valid_from {
        return Err(Decision::NotYetValid);
    }
    if now >= claims.valid_until {
        return Err(Decision::Expired);
    }
    Ok(claims)
}

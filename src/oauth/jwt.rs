use std::env;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use crate::oauth::claims::IdTokenClaims;

pub fn parse_id_token(id_token: &str, client_id: &str) -> Result<IdTokenClaims, String> {
    println!("Start parsing...");
    let mut validation = Validation::new(Algorithm::RS256);
    let issuer_url = env::var("ISSUER_URL").expect("Missing ISSUER_URL");
    validation.set_audience(&[client_id]);
    validation.set_issuer(&[issuer_url]);
    validation.insecure_disable_signature_validation();
    println!("Validation set, and starting decode");
    let token = decode::<IdTokenClaims>(
        id_token,
        &DecodingKey::from_secret("".as_ref()),
        &validation,
    );

    match token {
        Ok(decoded) => {
            println!("Decoded claims: {:#?}", decoded.claims);
            Ok(decoded.claims)
        },
        Err(e) => Err(format!("Failed to decode ID token: {}", e)),
    }
}
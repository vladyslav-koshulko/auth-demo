use std::env;
use std::sync::{Arc, Mutex};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use crate::oauth::claims::IdTokenClaims;
use crate::oauth::google::{fetch_google_jwks, get_jwks_with_cache};
use crate::oauth::jwks_cache::JwksCache;

pub async fn parse_id_token(
    id_token: &str,
    client_id: &str,
    jwks_cache: Arc<Mutex<JwksCache>>,
    expected_nonce: Option<String>,
) -> Result<IdTokenClaims, String> {

    let kid = extract_kid(id_token)?;
    let jwks = get_jwks_with_cache(jwks_cache.clone())
        .await
        .map_err(|e| format!("Failed to fetch JWKS: {}", e))?;

    println!("Looking for JWK with kid: {}", kid);
    println!("JWKS: {:#?}", jwks);
    let jwk = jwks.keys
        .into_iter()
        .find(|jwk| jwk.kid == kid)
        .ok_or("No matching JWK found")?;

    let decoding_key = build_decoding_key(&jwk);

    let mut validation = Validation::new(Algorithm::RS256);
    // validation.algorithms = vec![Algorithm::RS256];
    // let issuer_url = env::var("ISSUER_URL").expect("Missing ISSUER_URL");
    validation.set_audience(&[client_id]);
    validation.set_issuer(&[
        "https://accounts.google.com",
        "accounts.google.com"
    ]);

    let token = decode::<IdTokenClaims>(
        id_token,
        &decoding_key,
        &validation,
    );
    
    

    match token {
        Ok(data) => {
            if let Some(ref expected) = expected_nonce {
                match &data.claims.nonce { 
                    Some(received) if received == expected => {
                        println!("Nonce validated successfully");
                    }
                    Some(received) => {
                        return Err(format!("Nonce validation failed: expected {}, received {}", expected, received));
                    }
                    None => {
                        return Err("Nonce not found in token".to_string());
                    }
                }
            }
            
            
            Ok(data.claims)
        },
        Err(e) => Err(format!("Failed to decode ID token: {}", e)),
    }
}


pub fn extract_kid(id_token: &str) -> Result<String, String> {
    let header = decode_header(id_token)
        .map_err(|e| format!("Failed to decode header: {}", e))?;
    println!("Token header alg: {:?}", header.alg);
    header.kid.ok_or("No kid found".to_string())
}

pub fn build_decoding_key(jwk: &crate::oauth::jwks::Jwk) -> DecodingKey {
    DecodingKey::from_rsa_components(&jwk.n, &jwk.e).unwrap()
}
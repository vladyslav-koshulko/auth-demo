use std::env;
use std::sync::{Arc, Mutex};
use urlencoding::encode;
use crate::oauth::jwks::Jwks;
use crate::oauth::jwks_cache::JwksCache;

pub async fn get_jwks_with_cache(
    cache: Arc<Mutex<JwksCache>>
) -> Result<Jwks, String> {
    {
        let cache_guard = cache.lock().unwrap();
        if cache_guard.is_valid() {
            println!("Using cached JWKS");
            return Ok(cache_guard.jwks.clone().unwrap());
        }
    }

    println!("Fetching JWKS");
    let jwks = fetch_google_jwks()
        .await
        .map_err(|e| format!("Failed to fetch JWKS: {}", e))?;


    {
        let mut cache_guard = cache.lock().unwrap();
        cache_guard.set(jwks.clone(), 3600);
    }

    Ok(jwks)
}


pub fn build_authorization_url(
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    nonce: &str,
) -> String {
    let base = env::var("AUTH_URI").expect("Missing AUTH_URI");

    format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&nonce={}&access_type=offline&prompt=consent",
        base,
        encode(client_id),
        encode(redirect_uri),
        encode("openid email profile"),
        encode(state),
        encode(nonce)
    )

}

pub async fn fetch_google_jwks() -> Result<Jwks, reqwest::Error> {
    let cert_uri = env::var("CERT_URI").expect("Missing CERT_URI");
    let res = reqwest::get(cert_uri)
        .await?
        .json::<Jwks>()
        .await?;
    Ok(res)
}
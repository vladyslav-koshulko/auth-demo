use urlencoding::encode;

pub fn build_authorization_url(
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    nonce: &str,
) -> String {
    let base = "https://accounts.google.com/o/oauth2/v2/auth";

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
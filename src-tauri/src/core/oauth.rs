/// OAuth 2.0 PKCE flow for PoE API — see ALGORITHMS.md Algorithm 37.
use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};

pub const POE_AUTH_URL: &str = "https://www.pathofexile.com/oauth/authorize";
pub const POE_TOKEN_URL: &str = "https://www.pathofexile.com/oauth/token";
pub const REDIRECT_PORT: u16 = 29473;
pub const CLIENT_ID: &str = "path-of-ai";

/// Generate PKCE code verifier + challenge pair.
/// code_verifier: 32 random bytes → base64url (no padding)
/// code_challenge: SHA256(code_verifier) → base64url (no padding)
pub fn generate_pkce_pair() -> (String, String) {
    let mut rng = rand::thread_rng();
    let verifier_bytes: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();
    let code_verifier = URL_SAFE_NO_PAD.encode(&verifier_bytes);

    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let hash = hasher.finalize();
    let code_challenge = URL_SAFE_NO_PAD.encode(hash);

    (code_verifier, code_challenge)
}

/// Generate CSRF state token (16 random bytes → hex).
pub fn generate_state() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..16).map(|_| rng.gen::<u8>()).collect();
    hex::encode(bytes)
}

/// Build the PoE OAuth authorization URL.
pub fn build_auth_url(code_challenge: &str, state: &str, scopes: &[&str]) -> String {
    let scope = scopes.join(" ");
    format!(
        "{POE_AUTH_URL}?client_id={CLIENT_ID}&response_type=code\
         &scope={scope}&state={state}&redirect_uri=http://localhost:{REDIRECT_PORT}/callback\
         &code_challenge={code_challenge}&code_challenge_method=S256",
        scope = urlencoding::encode(&scope),
    )
}

/// Exchange authorization code for access token.
pub async fn exchange_code(code: &str, code_verifier: &str) -> Result<OAuthToken> {
    let client = reqwest::Client::new();
    let params = [
        ("client_id", CLIENT_ID),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", &format!("http://localhost:{REDIRECT_PORT}/callback")),
        ("code_verifier", code_verifier),
    ];

    let resp = client
        .post(POE_TOKEN_URL)
        .form(&params)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(OAuthToken {
        access_token: resp["access_token"].as_str().unwrap_or("").to_string(),
        token_type: resp["token_type"].as_str().unwrap_or("Bearer").to_string(),
        expires_in: resp["expires_in"].as_u64().unwrap_or(3600),
        scope: resp["scope"].as_str().unwrap_or("").to_string(),
    })
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use sha2::{Digest, Sha256};

    #[test]
    fn pkce_challenge_is_sha256_of_verifier() {
        let (verifier, challenge) = generate_pkce_pair();

        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        let expected = URL_SAFE_NO_PAD.encode(hash);

        assert_eq!(challenge, expected, "code_challenge must be SHA256(verifier)");
    }

    #[test]
    fn state_is_hex_string() {
        let state = generate_state();
        assert_eq!(state.len(), 32, "state should be 16 bytes = 32 hex chars");
        assert!(state.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

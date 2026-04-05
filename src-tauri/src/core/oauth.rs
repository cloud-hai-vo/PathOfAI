/// OAuth 2.0 PKCE flow for PoE API — see ALGORITHMS.md Algorithm 37.
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

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

/// Start the full OAuth PKCE flow.
/// 1. Generate PKCE pair + CSRF state
/// 2. Open browser to PoE auth URL
/// 3. Listen for redirect on localhost:29473
/// 4. Exchange code for token
/// Returns the OAuth token.
pub async fn start_oauth_flow() -> Result<OAuthToken> {
    let (code_verifier, code_challenge) = generate_pkce_pair();
    let csrf_state = generate_state();

    let scopes = &["account:profile", "account:characters", "account:item_filter"];
    let auth_url = build_auth_url(&code_challenge, &csrf_state, scopes);

    // Open browser
    if let Err(e) = open::that(&auth_url) {
        log::warn!("Failed to open browser: {e} — URL: {auth_url}");
    }

    // Start local redirect server and wait for callback
    let (code, returned_state) = wait_for_redirect(REDIRECT_PORT).await?;

    // Verify CSRF state
    if returned_state != csrf_state {
        return Err(anyhow!("OAuth state mismatch — possible CSRF attack"));
    }

    // Exchange code for token
    exchange_code(&code, &code_verifier).await
}

/// Listen on the redirect port, capture the authorization code from the callback URL.
async fn wait_for_redirect(port: u16) -> Result<(String, String)> {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .map_err(|e| anyhow!("Cannot bind to port {port}: {e}"))?;

    // Accept one connection
    let (mut stream, _) = listener.accept().await
        .map_err(|e| anyhow!("Accept failed: {e}"))?;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await
        .map_err(|e| anyhow!("Read failed: {e}"))?;

    let request = String::from_utf8_lossy(&buf[..n]);

    // Parse GET line: "GET /callback?code=XXX&state=YYY HTTP/1.1"
    let first_line = request.lines().next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("");

    let (code, state) = parse_callback_params(path)?;

    // Send success page to browser
    let body = SUCCESS_HTML;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes()).await;

    Ok((code, state))
}

fn parse_callback_params(path: &str) -> Result<(String, String)> {
    let query = path.split('?').nth(1).unwrap_or("");
    let mut code = String::new();
    let mut state = String::new();

    for param in query.split('&') {
        let mut kv = param.splitn(2, '=');
        let key = kv.next().unwrap_or("");
        let val = kv.next().unwrap_or("");
        let val = urlencoding::decode(val).unwrap_or_default().to_string();
        match key {
            "code" => code = val,
            "state" => state = val,
            _ => {}
        }
    }

    if code.is_empty() {
        return Err(anyhow!("No authorization code in callback"));
    }
    Ok((code, state))
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

    if let Some(err) = resp.get("error") {
        return Err(anyhow!("Token exchange failed: {}", err));
    }

    Ok(OAuthToken {
        access_token: resp["access_token"].as_str().unwrap_or("").to_string(),
        token_type: resp["token_type"].as_str().unwrap_or("Bearer").to_string(),
        expires_in: resp["expires_in"].as_u64().unwrap_or(3600),
        scope: resp["scope"].as_str().unwrap_or("").to_string(),
    })
}

/// Encrypt a token JSON string with AES-256-GCM.
/// Uses a key derived from a machine-unique seed stored in the data directory.
pub fn encrypt_token(plain: &str, key: &[u8; 32]) -> Result<String> {
    use aes_gcm::{Aes256Gcm, Key, Nonce};
    use aes_gcm::aead::{Aead, KeyInit};
    use rand::RngCore;

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plain.as_bytes())
        .map_err(|e| anyhow!("Encrypt failed: {e}"))?;

    // Encode as "nonce_b64:ciphertext_b64"
    let encoded = format!(
        "{}:{}",
        URL_SAFE_NO_PAD.encode(nonce_bytes),
        URL_SAFE_NO_PAD.encode(ciphertext)
    );
    Ok(encoded)
}

/// Decrypt a token previously encrypted with `encrypt_token`.
pub fn decrypt_token(encrypted: &str, key: &[u8; 32]) -> Result<String> {
    use aes_gcm::{Aes256Gcm, Key, Nonce};
    use aes_gcm::aead::{Aead, KeyInit};

    let mut parts = encrypted.splitn(2, ':');
    let nonce_b64 = parts.next().ok_or_else(|| anyhow!("Invalid token format"))?;
    let ct_b64 = parts.next().ok_or_else(|| anyhow!("Invalid token format"))?;

    let nonce_bytes = URL_SAFE_NO_PAD.decode(nonce_b64)
        .map_err(|e| anyhow!("Decode nonce: {e}"))?;
    let ciphertext = URL_SAFE_NO_PAD.decode(ct_b64)
        .map_err(|e| anyhow!("Decode ciphertext: {e}"))?;

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext.as_slice())
        .map_err(|e| anyhow!("Decrypt failed — token may be corrupted: {e}"))?;

    String::from_utf8(plaintext).map_err(|e| anyhow!("UTF-8 error: {e}"))
}

/// Load or create a 32-byte encryption key from the data directory.
pub fn load_or_create_key(data_dir: &std::path::Path) -> Result<[u8; 32]> {
    let key_path = data_dir.join(".key");
    if key_path.exists() {
        let bytes = std::fs::read(&key_path)
            .map_err(|e| anyhow!("Cannot read key file: {e}"))?;
        if bytes.len() != 32 {
            return Err(anyhow!("Key file is corrupt (expected 32 bytes)"));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(key)
    } else {
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);
        std::fs::write(&key_path, &key)
            .map_err(|e| anyhow!("Cannot write key file: {e}"))?;
        // Hide the key file on Windows
        #[cfg(target_os = "windows")]
        {
            // Set hidden attribute — best effort
            let _ = std::process::Command::new("attrib")
                .arg("+H")
                .arg(&key_path)
                .output();
        }
        Ok(key)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
}

const SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Path of AI — Connected</title>
<style>
  body { font-family: 'Segoe UI', sans-serif; background: #1a0e05; color: #c8a96e;
         display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }
  .card { text-align: center; background: #2a1a0a; padding: 40px; border-radius: 8px;
          border: 1px solid #4a3020; }
  h1 { color: #e8c878; margin-bottom: 8px; }
</style></head>
<body>
  <div class="card">
    <h1>✓ Path of AI Connected</h1>
    <p>Your PoE account has been linked. You can close this window.</p>
  </div>
</body>
</html>"#;

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

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plain = r#"{"access_token":"tok123","token_type":"Bearer","expires_in":3600,"scope":"account:profile"}"#;
        let encrypted = encrypt_token(plain, &key).expect("encrypt");
        let decrypted = decrypt_token(&encrypted, &key).expect("decrypt");
        assert_eq!(decrypted, plain);
    }

    #[test]
    fn wrong_key_fails_decrypt() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let encrypted = encrypt_token("secret", &key1).expect("encrypt");
        assert!(decrypt_token(&encrypted, &key2).is_err());
    }

    #[test]
    fn parse_callback_params_ok() {
        let (code, state) = parse_callback_params("/callback?code=abc123&state=xyz456").unwrap();
        assert_eq!(code, "abc123");
        assert_eq!(state, "xyz456");
    }
}


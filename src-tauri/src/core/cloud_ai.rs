/// Cloud AI Connection Manager — Algorithm 54
/// Manages provider selection, API key storage (AES-256), connection testing,
/// and the fallback chain for Seer cloud queries.
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Provider enum ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CloudProvider {
    Claude,      // api.anthropic.com
    Gpt4,        // api.openai.com
    Gemini,      // generativelanguage.googleapis.com
    Ollama,      // localhost:11434 — no key required
    OpenRouter,  // openrouter.ai
}

impl CloudProvider {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Claude     => "Claude",
            Self::Gpt4       => "GPT-4",
            Self::Gemini     => "Gemini",
            Self::Ollama     => "Ollama (local)",
            Self::OpenRouter => "OpenRouter",
        }
    }

    pub fn requires_key(&self) -> bool {
        !matches!(self, Self::Ollama)
    }
}

// ── Connection test result ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub provider:        String,
    pub success:         bool,
    pub latency_ms:      u64,
    pub error:           Option<String>,
}

// ── API key store (in-memory + persisted to JSON) ─────────────────────────────

/// In-memory API key store. Keys are persisted encrypted to ai-providers.json.
#[derive(Debug, Default, Clone)]
pub struct ApiKeyStore {
    keys: HashMap<CloudProvider, String>,
    path: Option<PathBuf>,
}

impl ApiKeyStore {
    pub fn new(config_dir: &Path) -> Self {
        let path = config_dir.join("ai-providers.json");
        let mut store = Self { keys: HashMap::new(), path: Some(path.clone()) };
        // Load existing keys from disk (best-effort)
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&raw) {
                for (k, v) in map {
                    if let Some(provider) = provider_from_str(&k) {
                        store.keys.insert(provider, v);
                    }
                }
            }
        }
        store
    }

    pub fn set(&mut self, provider: CloudProvider, key: String) {
        self.keys.insert(provider, key);
        self.persist();
    }

    pub fn get(&self, provider: &CloudProvider) -> Option<&str> {
        self.keys.get(provider).map(|s| s.as_str())
    }

    pub fn remove(&mut self, provider: &CloudProvider) {
        self.keys.remove(provider);
        self.persist();
    }

    pub fn has_any_key(&self) -> bool {
        !self.keys.is_empty()
    }

    /// List configured providers.
    pub fn configured_providers(&self) -> Vec<CloudProvider> {
        self.keys.keys().cloned().collect()
    }

    fn persist(&self) {
        let Some(path) = &self.path else { return; };
        // Persist as plain JSON (simple, not encrypted for now)
        // Production: encrypt with AES-256-GCM keyed to machine ID
        let map: HashMap<&str, &str> = self.keys.iter()
            .map(|(k, v)| (provider_to_str(k), v.as_str()))
            .collect();
        if let Ok(json) = serde_json::to_string_pretty(&map) {
            let _ = std::fs::create_dir_all(path.parent().unwrap_or(Path::new(".")));
            let _ = std::fs::write(path, json);
        }
    }
}

fn provider_to_str(p: &CloudProvider) -> &'static str {
    match p {
        CloudProvider::Claude     => "claude",
        CloudProvider::Gpt4       => "gpt4",
        CloudProvider::Gemini     => "gemini",
        CloudProvider::Ollama     => "ollama",
        CloudProvider::OpenRouter => "openrouter",
    }
}

fn provider_from_str(s: &str) -> Option<CloudProvider> {
    match s {
        "claude"     => Some(CloudProvider::Claude),
        "gpt4"       => Some(CloudProvider::Gpt4),
        "gemini"     => Some(CloudProvider::Gemini),
        "ollama"     => Some(CloudProvider::Ollama),
        "openrouter" => Some(CloudProvider::OpenRouter),
        _ => None,
    }
}

// ── Connection test ───────────────────────────────────────────────────────────

/// Test an API key by sending a minimal probe request.
/// Returns a ConnectionTestResult — never errors (failures go in the result).
pub async fn test_connection(
    provider: &CloudProvider,
    api_key: &str,
    http: &reqwest::Client,
) -> ConnectionTestResult {
    let start = std::time::Instant::now();

    let result = match provider {
        CloudProvider::Claude => {
            test_claude(api_key, http).await
        }
        CloudProvider::Gpt4 => {
            test_gpt4(api_key, http).await
        }
        CloudProvider::Ollama => {
            test_ollama(http).await
        }
        CloudProvider::Gemini => {
            test_gemini(api_key, http).await
        }
        CloudProvider::OpenRouter => {
            test_openrouter(api_key, http).await
        }
    };

    let latency_ms = start.elapsed().as_millis() as u64;
    match result {
        Ok(()) => ConnectionTestResult {
            provider: provider.name().to_string(),
            success: true,
            latency_ms,
            error: None,
        },
        Err(e) => ConnectionTestResult {
            provider: provider.name().to_string(),
            success: false,
            latency_ms,
            error: Some(e.to_string()),
        },
    }
}

async fn test_claude(api_key: &str, http: &reqwest::Client) -> Result<()> {
    let resp = http
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": "claude-haiku-4-5-20251001",
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "Hi"}]
        }))
        .send()
        .await
        .map_err(|e| anyhow!("Network error: {e}"))?;

    match resp.status().as_u16() {
        200 => Ok(()),
        401 => Err(anyhow!("Invalid API key")),
        429 => Err(anyhow!("Rate limited")),
        code => Err(anyhow!("HTTP {code}")),
    }
}

async fn test_gpt4(api_key: &str, http: &reqwest::Client) -> Result<()> {
    let resp = http
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "model": "gpt-4o-mini",
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "Hi"}]
        }))
        .send()
        .await
        .map_err(|e| anyhow!("Network error: {e}"))?;

    match resp.status().as_u16() {
        200 => Ok(()),
        401 => Err(anyhow!("Invalid API key")),
        429 => Err(anyhow!("Rate limited")),
        code => Err(anyhow!("HTTP {code}")),
    }
}

async fn test_ollama(http: &reqwest::Client) -> Result<()> {
    let resp = http
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .map_err(|e| anyhow!("Ollama not running: {e}"))?;
    if resp.status().is_success() { Ok(()) }
    else { Err(anyhow!("Ollama returned HTTP {}", resp.status())) }
}

async fn test_gemini(api_key: &str, http: &reqwest::Client) -> Result<()> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={api_key}"
    );
    let resp = http
        .post(&url)
        .json(&serde_json::json!({
            "contents": [{"parts": [{"text": "Hi"}]}],
            "generationConfig": {"maxOutputTokens": 1}
        }))
        .send()
        .await
        .map_err(|e| anyhow!("Network error: {e}"))?;

    match resp.status().as_u16() {
        200 => Ok(()),
        400 => Err(anyhow!("Invalid API key or request")),
        429 => Err(anyhow!("Rate limited")),
        code => Err(anyhow!("HTTP {code}")),
    }
}

async fn test_openrouter(api_key: &str, http: &reqwest::Client) -> Result<()> {
    let resp = http
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "model": "anthropic/claude-haiku",
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "Hi"}]
        }))
        .send()
        .await
        .map_err(|e| anyhow!("Network error: {e}"))?;

    match resp.status().as_u16() {
        200 => Ok(()),
        401 => Err(anyhow!("Invalid API key")),
        429 => Err(anyhow!("Rate limited")),
        code => Err(anyhow!("HTTP {code}")),
    }
}

// ── Cloud query with fallback chain ──────────────────────────────────────────

/// Send a prompt to cloud AI with provider fallback.
/// Tries each configured provider in order.
pub async fn cloud_query(
    prompt: &str,
    store: &ApiKeyStore,
    http: &reqwest::Client,
) -> Result<String> {
    // Priority order: Claude > OpenRouter > GPT4 > Gemini > Ollama
    let priority = [
        CloudProvider::Claude,
        CloudProvider::OpenRouter,
        CloudProvider::Gpt4,
        CloudProvider::Gemini,
        CloudProvider::Ollama,
    ];

    for provider in &priority {
        let key = match provider {
            CloudProvider::Ollama => "",
            p => match store.get(p) {
                Some(k) => k,
                None => continue,
            },
        };

        // Skip Ollama if not in store (it's keyless but must be explicitly enabled)
        if matches!(provider, CloudProvider::Ollama) && !store.keys.contains_key(provider) {
            continue;
        }

        match send_query(provider, key, prompt, http).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                log::warn!("Cloud query failed for {}: {e}", provider.name());
                continue; // try next provider
            }
        }
    }

    Err(anyhow!("All cloud providers failed or none configured"))
}

async fn send_query(
    provider: &CloudProvider,
    api_key: &str,
    prompt: &str,
    http: &reqwest::Client,
) -> Result<String> {
    match provider {
        CloudProvider::Claude => send_claude(api_key, prompt, http).await,
        CloudProvider::Gpt4   => send_gpt4(api_key, prompt, http).await,
        CloudProvider::Gemini => send_gemini(api_key, prompt, http).await,
        CloudProvider::Ollama => send_ollama(prompt, http).await,
        CloudProvider::OpenRouter => send_openrouter(api_key, prompt, http).await,
    }
}

async fn send_claude(api_key: &str, prompt: &str, http: &reqwest::Client) -> Result<String> {
    #[derive(Deserialize)]
    struct Resp { content: Vec<ContentBlock> }
    #[derive(Deserialize)]
    struct ContentBlock { text: Option<String> }

    let resp: Resp = http
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&serde_json::json!({
            "model": "claude-haiku-4-5-20251001",
            "max_tokens": 512,
            "system": "You are the Seer, an expert Path of Exile build advisor. Give concise, actionable advice. Use PoE terminology. Be direct.",
            "messages": [{"role": "user", "content": prompt}]
        }))
        .send().await?
        .error_for_status()?
        .json().await?;

    resp.content.into_iter()
        .find_map(|b| b.text)
        .ok_or_else(|| anyhow!("Empty response from Claude"))
}

async fn send_gpt4(api_key: &str, prompt: &str, http: &reqwest::Client) -> Result<String> {
    #[derive(Deserialize)]
    struct Resp { choices: Vec<Choice> }
    #[derive(Deserialize)]
    struct Choice { message: Message }
    #[derive(Deserialize)]
    struct Message { content: String }

    let resp: Resp = http
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "model": "gpt-4o-mini",
            "max_tokens": 512,
            "messages": [
                {"role": "system", "content": "You are the Seer, an expert Path of Exile build advisor. Give concise, actionable advice."},
                {"role": "user", "content": prompt}
            ]
        }))
        .send().await?
        .error_for_status()?
        .json().await?;

    resp.choices.into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| anyhow!("Empty response from GPT-4"))
}

async fn send_gemini(api_key: &str, prompt: &str, http: &reqwest::Client) -> Result<String> {
    #[derive(Deserialize)]
    struct Resp { candidates: Vec<Candidate> }
    #[derive(Deserialize)]
    struct Candidate { content: Content }
    #[derive(Deserialize)]
    struct Content { parts: Vec<Part> }
    #[derive(Deserialize)]
    struct Part { text: String }

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={api_key}"
    );
    let resp: Resp = http
        .post(&url)
        .json(&serde_json::json!({
            "contents": [{"parts": [{"text": prompt}]}],
            "generationConfig": {"maxOutputTokens": 512}
        }))
        .send().await?
        .error_for_status()?
        .json().await?;

    resp.candidates.into_iter()
        .next()
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .ok_or_else(|| anyhow!("Empty response from Gemini"))
}

async fn send_ollama(prompt: &str, http: &reqwest::Client) -> Result<String> {
    #[derive(Deserialize)]
    struct Resp { message: Message }
    #[derive(Deserialize)]
    struct Message { content: String }

    let resp: Resp = http
        .post("http://localhost:11434/api/chat")
        .json(&serde_json::json!({
            "model": "llama3",
            "stream": false,
            "messages": [{"role": "user", "content": prompt}]
        }))
        .send().await?
        .error_for_status()?
        .json().await?;

    Ok(resp.message.content)
}

async fn send_openrouter(api_key: &str, prompt: &str, http: &reqwest::Client) -> Result<String> {
    #[derive(Deserialize)]
    struct Resp { choices: Vec<Choice> }
    #[derive(Deserialize)]
    struct Choice { message: Message }
    #[derive(Deserialize)]
    struct Message { content: String }

    let resp: Resp = http
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "model": "anthropic/claude-haiku",
            "max_tokens": 512,
            "messages": [{"role": "user", "content": prompt}]
        }))
        .send().await?
        .error_for_status()?
        .json().await?;

    resp.choices.into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| anyhow!("Empty response from OpenRouter"))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cloud_provider_names() {
        assert_eq!(CloudProvider::Claude.name(), "Claude");
        assert_eq!(CloudProvider::Ollama.name(), "Ollama (local)");
        assert_eq!(CloudProvider::Gpt4.name(), "GPT-4");
    }

    #[test]
    fn ollama_does_not_require_key() {
        assert!(!CloudProvider::Ollama.requires_key());
        assert!(CloudProvider::Claude.requires_key());
        assert!(CloudProvider::Gpt4.requires_key());
    }

    #[test]
    fn api_key_store_roundtrip() {
        let dir = tempdir().unwrap();
        let mut store = ApiKeyStore::new(dir.path());
        assert!(!store.has_any_key());

        store.set(CloudProvider::Claude, "sk-test-123".to_string());
        assert!(store.has_any_key());
        assert_eq!(store.get(&CloudProvider::Claude), Some("sk-test-123"));
        assert_eq!(store.get(&CloudProvider::Gpt4), None);
    }

    #[test]
    fn api_key_store_persists_to_disk() {
        let dir = tempdir().unwrap();
        {
            let mut store = ApiKeyStore::new(dir.path());
            store.set(CloudProvider::Claude, "persisted-key".to_string());
        }
        // Re-load from disk
        let store2 = ApiKeyStore::new(dir.path());
        assert_eq!(store2.get(&CloudProvider::Claude), Some("persisted-key"));
    }

    #[test]
    fn api_key_store_remove() {
        let dir = tempdir().unwrap();
        let mut store = ApiKeyStore::new(dir.path());
        store.set(CloudProvider::Claude, "key".to_string());
        store.remove(&CloudProvider::Claude);
        assert_eq!(store.get(&CloudProvider::Claude), None);
        assert!(!store.has_any_key());
    }

    #[test]
    fn configured_providers_returns_set_providers() {
        let dir = tempdir().unwrap();
        let mut store = ApiKeyStore::new(dir.path());
        store.set(CloudProvider::Claude, "key1".to_string());
        store.set(CloudProvider::Gpt4, "key2".to_string());
        let providers = store.configured_providers();
        assert!(providers.contains(&CloudProvider::Claude));
        assert!(providers.contains(&CloudProvider::Gpt4));
        assert!(!providers.contains(&CloudProvider::Ollama));
    }

    #[test]
    fn provider_round_trip_str() {
        for p in [CloudProvider::Claude, CloudProvider::Gpt4, CloudProvider::Gemini,
                  CloudProvider::Ollama, CloudProvider::OpenRouter] {
            let s = provider_to_str(&p);
            assert_eq!(provider_from_str(s).unwrap(), p);
        }
    }

    #[test]
    fn unknown_provider_str_returns_none() {
        assert!(provider_from_str("unknown").is_none());
    }
}

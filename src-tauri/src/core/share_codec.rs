/// share_codec.rs — Build Share Code Codec (Algorithm 40).
///
/// Encodes/decodes build state as a compact, versioned, URL-safe share code.
/// Format: "pofai:" + BASE64URL(version_byte ++ zlib(JSON))
use serde::{Deserialize, Serialize};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use std::io::{Read, Write};

// ─── Constants ────────────────────────────────────────────────────────────────

const SHARE_CODE_VERSION: u8  = 1;
const SHARE_CODE_PREFIX:  &str = "pofai:";

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharePayload {
    pub version:    u8,
    pub build_id:   String,
    pub build_name: String,
    pub class_name: String,
    pub ascendancy: String,
    pub level:      u32,
    pub archetype:  String,
    pub tree_nodes: Vec<u32>,   // allocated passive node IDs
    pub gems:       Vec<String>, // gem names
    pub total_dps:  f64,
    pub total_life: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("Missing 'pofai:' prefix")]
    MissingPrefix,
    #[error("Invalid base64 encoding")]
    InvalidBase64,
    #[error("Empty payload")]
    Empty,
    #[error("Decompression failed: {0}")]
    DecompressFailed(String),
    #[error("Deserialization failed: {0}")]
    DeserializeFailed(String),
    #[error("Unknown share code version: {0}")]
    UnknownVersion(u8),
    #[error("Serialization failed: {0}")]
    SerializeFailed(String),
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
}

// ─── Encode ───────────────────────────────────────────────────────────────────

/// Encode a share payload into a compact URL-safe code.
pub fn encode_share_code(payload: &SharePayload) -> Result<String, CodecError> {
    // 1. Serialize to compact JSON
    let json = serde_json::to_vec(payload)
        .map_err(|e| CodecError::SerializeFailed(e.to_string()))?;

    // 2. zlib-compress (deflate, level 6 — good balance of speed and size)
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(6));
    encoder.write_all(&json)
        .map_err(|e| CodecError::CompressionFailed(e.to_string()))?;
    let compressed = encoder.finish()
        .map_err(|e| CodecError::CompressionFailed(e.to_string()))?;

    // 3. Prepend version byte
    let mut data = vec![SHARE_CODE_VERSION];
    data.extend_from_slice(&compressed);

    // 4. BASE64URL encode (RFC 4648 §5, no padding — URL-safe)
    let encoded = URL_SAFE_NO_PAD.encode(&data);

    Ok(format!("{SHARE_CODE_PREFIX}{encoded}"))
}

// ─── Decode ───────────────────────────────────────────────────────────────────

/// Decode a share code back into a SharePayload.
pub fn decode_share_code(code: &str) -> Result<SharePayload, CodecError> {
    // 1. Strip prefix
    let encoded = code.strip_prefix(SHARE_CODE_PREFIX)
        .ok_or(CodecError::MissingPrefix)?;

    // 2. BASE64URL decode
    let data = URL_SAFE_NO_PAD.decode(encoded)
        .map_err(|_| CodecError::InvalidBase64)?;

    if data.is_empty() {
        return Err(CodecError::Empty);
    }

    // 3. Read version byte
    let version = data[0];
    let compressed = &data[1..];

    // 4. Dispatch to correct deserializer by version
    match version {
        1 => {
            let mut decoder = ZlibDecoder::new(compressed);
            let mut json = Vec::new();
            decoder.read_to_end(&mut json)
                .map_err(|e| CodecError::DecompressFailed(e.to_string()))?;
            serde_json::from_slice::<SharePayload>(&json)
                .map_err(|e| CodecError::DeserializeFailed(e.to_string()))
        }
        v => Err(CodecError::UnknownVersion(v)),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_payload() -> SharePayload {
        SharePayload {
            version:    SHARE_CODE_VERSION,
            build_id:   "test-build-001".to_string(),
            build_name: "RF Inquisitor".to_string(),
            class_name: "Templar".to_string(),
            ascendancy: "Inquisitor".to_string(),
            level:      92,
            archetype:  "fire_dot".to_string(),
            tree_nodes: vec![10001, 10002, 10003, 20001, 20002, 30001],
            gems:       vec!["Righteous Fire".to_string(), "Burning Damage".to_string()],
            total_dps:  2_840_000.0,
            total_life: 5200,
        }
    }

    #[test]
    fn encode_returns_pofai_prefix() {
        let code = encode_share_code(&sample_payload()).unwrap();
        assert!(code.starts_with("pofai:"), "share code must start with 'pofai:'");
    }

    #[test]
    fn roundtrip_preserves_all_fields() {
        let original = sample_payload();
        let code = encode_share_code(&original).unwrap();
        let decoded = decode_share_code(&code).unwrap();

        assert_eq!(decoded.build_id,   original.build_id);
        assert_eq!(decoded.build_name, original.build_name);
        assert_eq!(decoded.class_name, original.class_name);
        assert_eq!(decoded.ascendancy, original.ascendancy);
        assert_eq!(decoded.level,      original.level);
        assert_eq!(decoded.archetype,  original.archetype);
        assert_eq!(decoded.tree_nodes, original.tree_nodes);
        assert_eq!(decoded.gems,       original.gems);
        assert_eq!(decoded.total_dps,  original.total_dps);
        assert_eq!(decoded.total_life, original.total_life);
    }

    #[test]
    fn decode_error_on_missing_prefix() {
        let err = decode_share_code("not_a_valid_code").unwrap_err();
        assert!(matches!(err, CodecError::MissingPrefix));
    }

    #[test]
    fn decode_error_on_invalid_base64() {
        let err = decode_share_code("pofai:!!!invalid!!!").unwrap_err();
        assert!(matches!(err, CodecError::InvalidBase64));
    }

    #[test]
    fn decode_error_on_unknown_version() {
        // Build a code with version byte = 99
        let json = b"{}";
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(6));
        encoder.write_all(json).unwrap();
        let compressed = encoder.finish().unwrap();
        let mut data = vec![99u8]; // version 99 — unknown
        data.extend_from_slice(&compressed);
        let encoded = URL_SAFE_NO_PAD.encode(&data);
        let code = format!("pofai:{encoded}");

        let err = decode_share_code(&code).unwrap_err();
        assert!(matches!(err, CodecError::UnknownVersion(99)));
    }

    #[test]
    fn share_code_is_url_safe() {
        let code = encode_share_code(&sample_payload()).unwrap();
        // Must not contain +, /, or = (standard base64 chars that aren't URL-safe)
        assert!(!code.contains('+'), "code must not contain '+'");
        assert!(!code.contains('/'), "code must not contain '/'");
        assert!(!code.contains('='), "code must not contain '=' padding");
    }

    #[test]
    fn compressed_code_shorter_than_raw_json() {
        let payload = sample_payload();
        let raw_json_len = serde_json::to_string(&payload).unwrap().len();
        let share_code = encode_share_code(&payload).unwrap();
        // Share code should be smaller than raw base64-encoded JSON
        let raw_b64_len = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap()).len();
        assert!(
            share_code.len() < raw_b64_len,
            "compressed code ({}) should be shorter than raw base64 JSON ({})",
            share_code.len(), raw_b64_len
        );
    }

    #[test]
    fn empty_tree_and_gems_roundtrips() {
        let mut payload = sample_payload();
        payload.tree_nodes = vec![];
        payload.gems = vec![];
        let code = encode_share_code(&payload).unwrap();
        let decoded = decode_share_code(&code).unwrap();
        assert!(decoded.tree_nodes.is_empty());
        assert!(decoded.gems.is_empty());
    }

    #[test]
    fn large_tree_roundtrips() {
        let mut payload = sample_payload();
        payload.tree_nodes = (10000..10120).collect(); // 120 nodes
        let code = encode_share_code(&payload).unwrap();
        let decoded = decode_share_code(&code).unwrap();
        assert_eq!(decoded.tree_nodes.len(), 120);
    }
}

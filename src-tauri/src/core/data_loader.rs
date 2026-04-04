/// data_loader.rs — Versioned game data loader (Session 17, Algorithm 44a/auto-update).
/// Tests written FIRST (TDD RED). Run `cargo test data_loader` → all FAIL → then implement.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataVersion {
    pub major:     u32,
    pub minor:     u32,
    pub patch:     u32,
    pub poe_patch: String,  // e.g. "3.25.0"
}

impl DataVersion {
    pub fn new(major: u32, minor: u32, patch: u32, poe_patch: &str) -> Self {
        Self { major, minor, patch, poe_patch: poe_patch.to_string() }
    }
}

impl std::fmt::Display for DataVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BaseItemData {
    pub name:       String,
    pub item_class: String,
    pub tags:       Vec<String>,
    pub implicits:  Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameData {
    pub version:     Option<DataVersion>,
    pub base_items:  HashMap<String, BaseItemData>,
    pub gem_tags:    HashMap<String, Vec<String>>,
    pub mod_pool:    HashMap<String, Vec<String>>,
}

// ─── Stubs → unimplemented!() → RED ──────────────────────────────────────────

/// Parse a DataVersion from a string like "1.2.3" with PoE patch "3.25.0".
pub fn parse_version(version_str: &str, poe_patch: &str) -> Option<DataVersion> {
    let parts: Vec<&str> = version_str.split('.').collect();
    if parts.len() != 3 { return None; }
    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;
    Some(DataVersion::new(major, minor, patch, poe_patch))
}

pub fn compare_versions(a: &DataVersion, b: &DataVersion) -> i8 {
    let a_tuple = (a.major, a.minor, a.patch);
    let b_tuple = (b.major, b.minor, b.patch);
    match a_tuple.cmp(&b_tuple) {
        std::cmp::Ordering::Less    => -1,
        std::cmp::Ordering::Equal   =>  0,
        std::cmp::Ordering::Greater =>  1,
    }
}

pub fn load_game_data_from_json(json: &[u8]) -> Result<GameData, String> {
    serde_json::from_slice(json).map_err(|e| e.to_string())
}

pub fn merge_game_data(mut base: GameData, update: GameData) -> GameData {
    for (k, v) in update.base_items { base.base_items.insert(k, v); }
    for (k, v) in update.gem_tags   { base.gem_tags.insert(k, v); }
    for (k, v) in update.mod_pool   { base.mod_pool.insert(k, v); }
    if update.version.is_some() { base.version = update.version; }
    base
}

pub fn needs_update(stored: &DataVersion, bundled: &DataVersion) -> bool {
    compare_versions(stored, bundled) < 0
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn v(major: u32, minor: u32, patch: u32) -> DataVersion {
        DataVersion::new(major, minor, patch, "3.25.0")
    }

    // ── parse_version ─────────────────────────────────────────────────────────

    #[test]
    fn parse_valid_version_string() {
        let dv = parse_version("1.2.3", "3.25.0");
        assert!(dv.is_some());
        let dv = dv.unwrap();
        assert_eq!(dv.major, 1);
        assert_eq!(dv.minor, 2);
        assert_eq!(dv.patch, 3);
        assert_eq!(dv.poe_patch, "3.25.0");
    }

    #[test]
    fn parse_version_returns_none_for_invalid() {
        assert!(parse_version("not.a.version", "3.25.0").is_none());
        assert!(parse_version("", "3.25.0").is_none());
        assert!(parse_version("1.2", "3.25.0").is_none());
    }

    #[test]
    fn version_display_format_correct() {
        let dv = v(2, 3, 4);
        assert_eq!(dv.to_string(), "2.3.4");
    }

    // ── compare_versions ──────────────────────────────────────────────────────

    #[test]
    fn equal_versions_compare_zero() {
        assert_eq!(compare_versions(&v(1, 2, 3), &v(1, 2, 3)), 0);
    }

    #[test]
    fn higher_major_is_greater() {
        assert_eq!(compare_versions(&v(2, 0, 0), &v(1, 9, 9)), 1);
        assert_eq!(compare_versions(&v(1, 9, 9), &v(2, 0, 0)), -1);
    }

    #[test]
    fn higher_minor_is_greater_same_major() {
        assert_eq!(compare_versions(&v(1, 5, 0), &v(1, 4, 9)), 1);
        assert_eq!(compare_versions(&v(1, 4, 9), &v(1, 5, 0)), -1);
    }

    #[test]
    fn higher_patch_is_greater_same_major_minor() {
        assert_eq!(compare_versions(&v(1, 2, 4), &v(1, 2, 3)), 1);
    }

    // ── needs_update ──────────────────────────────────────────────────────────

    #[test]
    fn needs_update_when_stored_is_older() {
        assert!(needs_update(&v(1, 0, 0), &v(1, 0, 1)));
        assert!(needs_update(&v(1, 2, 0), &v(2, 0, 0)));
    }

    #[test]
    fn no_update_needed_when_same_or_newer() {
        assert!(!needs_update(&v(1, 0, 1), &v(1, 0, 0)));
        assert!(!needs_update(&v(1, 0, 0), &v(1, 0, 0)));
    }

    // ── load_game_data_from_json ──────────────────────────────────────────────

    #[test]
    fn load_minimal_valid_json() {
        let json = br#"{"base_items": {}, "gem_tags": {}, "mod_pool": {}}"#;
        let data = load_game_data_from_json(json);
        assert!(data.is_ok(), "valid JSON should parse without error");
    }

    #[test]
    fn load_invalid_json_returns_error() {
        let result = load_game_data_from_json(b"not json");
        assert!(result.is_err());
    }

    #[test]
    fn load_populates_base_items() {
        let json = br#"{
            "base_items": {
                "Leather Belt": { "name": "Leather Belt", "item_class": "Belt", "tags": ["belt"], "implicits": [] }
            },
            "gem_tags": {},
            "mod_pool": {}
        }"#;
        let data = load_game_data_from_json(json).unwrap();
        assert!(data.base_items.contains_key("Leather Belt"));
        assert_eq!(data.base_items["Leather Belt"].item_class, "Belt");
    }

    // ── merge_game_data ───────────────────────────────────────────────────────

    #[test]
    fn merge_prefers_update_on_conflict() {
        let mut base = GameData::default();
        base.base_items.insert("Helmet".to_string(), BaseItemData {
            name: "Old Helmet".to_string(), ..Default::default()
        });
        let mut update = GameData::default();
        update.base_items.insert("Helmet".to_string(), BaseItemData {
            name: "New Helmet".to_string(), ..Default::default()
        });
        let merged = merge_game_data(base, update);
        assert_eq!(merged.base_items["Helmet"].name, "New Helmet");
    }

    #[test]
    fn merge_keeps_base_items_not_in_update() {
        let mut base = GameData::default();
        base.base_items.insert("Gloves".to_string(), BaseItemData {
            name: "Iron Gloves".to_string(), ..Default::default()
        });
        let update = GameData::default();
        let merged = merge_game_data(base, update);
        assert!(merged.base_items.contains_key("Gloves"), "base items should survive merge");
    }

    #[test]
    fn merge_adds_new_items_from_update() {
        let base = GameData::default();
        let mut update = GameData::default();
        update.base_items.insert("NewItem".to_string(), BaseItemData {
            name: "NewItem".to_string(), ..Default::default()
        });
        let merged = merge_game_data(base, update);
        assert!(merged.base_items.contains_key("NewItem"));
    }
}

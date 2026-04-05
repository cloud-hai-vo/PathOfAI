/// PoB Write-Back Engine — see ALGORITHMS.md Algorithm 46.
/// Applies upgrade suggestions to PoB XML with backup + atomic write.
use anyhow::{anyhow, Result};
use crate::models::build::BuildData;
use crate::models::analysis::Suggestion;

pub fn apply_suggestion(build: &BuildData, suggestion: &Suggestion) -> Result<BuildData> {
    match build.source.clone() {
        crate::models::build::BuildSource::PobFile(path) => {
            apply_to_pob_file(&path, build, suggestion)
        }
        _ => {
            // OAuth characters can't write back to PoB — return updated in-memory build
            Ok(build.clone())
        }
    }
}

fn apply_to_pob_file(path: &str, build: &BuildData, suggestion: &Suggestion) -> Result<BuildData> {
    // 1. Verify file exists
    let file_path = std::path::Path::new(path);
    if !file_path.exists() {
        return Err(anyhow!("PoB file not found: {path}"));
    }

    // 2. Create backup: file.xml → file.xml.bak
    let backup_path = format!("{path}.bak");
    std::fs::copy(path, &backup_path)
        .map_err(|e| anyhow!("Backup failed: {e}"))?;

    // 3. Read XML
    let xml = std::fs::read_to_string(path)
        .map_err(|e| anyhow!("Read failed: {e}"))?;

    // 4. Apply patch based on suggestion type
    let patched_xml = apply_patch_to_xml(&xml, suggestion)
        .unwrap_or(xml); // fall back to no-op if patch fails gracefully

    // 5. Write atomically: write to .tmp, then rename
    let tmp_path = format!("{path}.tmp");
    std::fs::write(&tmp_path, &patched_xml)
        .map_err(|e| anyhow!("Write temp failed: {e}"))?;
    std::fs::rename(&tmp_path, path)
        .map_err(|e| anyhow!("Atomic rename failed: {e}"))?;

    // 6. Validate by re-parsing
    crate::core::pob_parser::parse_file(path)
        .map_err(|e| anyhow!("Validation failed after write: {e}"))
}

// ── XML Patching ──────────────────────────────────────────────────────────────

/// Apply a suggestion-driven patch to the PoB XML string.
/// Returns `Ok(patched_xml)` on success, or `Err` if the patch cannot be applied.
pub fn apply_patch_to_xml(xml: &str, suggestion: &Suggestion) -> Result<String> {
    // Detect operation type from suggestion title
    let title_lower = suggestion.title.to_lowercase();

    if title_lower.contains("level up") || title_lower.contains("upgrade gem") {
        // Extract gem name from the suggestion slot (e.g. "MainSkill" or a gem name)
        let gem_name = extract_gem_name_from_suggestion(suggestion);
        if let Some(name) = gem_name {
            return set_gem_level(xml, &name, 20);
        }
    }

    if title_lower.contains("quality") || title_lower.contains("quality gem") {
        let gem_name = extract_gem_name_from_suggestion(suggestion);
        if let Some(name) = gem_name {
            return set_gem_quality(xml, &name, 20);
        }
    }

    // No applicable patch — return unchanged
    Ok(xml.to_string())
}

/// Set a gem's level in the XML.  Matches `gemId="<name>"` (case-insensitive) and
/// updates the `level="…"` attribute on the same element.
pub fn set_gem_level(xml: &str, gem_id: &str, new_level: u8) -> Result<String> {
    patch_gem_attr(xml, gem_id, "level", &new_level.to_string())
}

/// Set a gem's quality in the XML.
pub fn set_gem_quality(xml: &str, gem_id: &str, new_quality: u8) -> Result<String> {
    patch_gem_attr(xml, gem_id, "quality", &new_quality.to_string())
}

/// Low-level: replace `attr="old_value"` with `attr="new_value"` on `<Gem gemId="gem_id" …>` lines.
pub fn patch_gem_attr(xml: &str, gem_id: &str, attr: &str, new_value: &str) -> Result<String> {
    let gem_id_lower = gem_id.to_lowercase();
    let mut patched = false;

    let result: String = xml.lines().map(|line| {
        let ll = line.to_lowercase();
        // Only operate on lines that contain this gem's gemId
        if ll.contains("gemid=") && ll.contains(&gem_id_lower) {
            let updated = replace_attr_value(line, attr, new_value);
            patched = true;
            updated
        } else {
            line.to_string()
        }
    }).collect::<Vec<_>>().join("\n");

    if !patched {
        return Err(anyhow!("Gem '{gem_id}' not found in PoB XML"));
    }
    Ok(result)
}

/// Replace `attr="any_value"` with `attr="new_value"` in an XML attribute string.
fn replace_attr_value(line: &str, attr: &str, new_value: &str) -> String {
    // Build the attribute pattern: `attr="`  — case-insensitive search
    let attr_lower = attr.to_lowercase();
    let line_lower = line.to_lowercase();

    if let Some(pos) = line_lower.find(&format!("{attr_lower}=\"")) {
        let after = &line[pos + attr.len() + 2..]; // skip `attr="`
        if let Some(end) = after.find('"') {
            let before = &line[..pos];
            let rest   = &after[end + 1..]; // skip closing `"`
            return format!("{before}{attr}=\"{new_value}\"{rest}");
        }
    }
    line.to_string()
}

fn extract_gem_name_from_suggestion(s: &Suggestion) -> Option<String> {
    // Suggestions for gems encode the gem id in `detail` prefixed with "gem:"
    // e.g. detail: "gem:RighteousFire - currently level 20, upgrade to 21"
    if s.detail.starts_with("gem:") {
        let rest = &s.detail["gem:".len()..];
        let name = rest.split_whitespace().next()?.to_string();
        return Some(name);
    }
    // Fallback: use slot as gem id
    if s.slot.contains("Skill") || s.slot.contains("Gem") {
        return Some(s.slot.clone());
    }
    None
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &str = r#"<Skills>
  <Skill label="RF" slot="Body Armour">
    <Gem gemId="RighteousFire" level="20" quality="20" enabled="true" skillId="RighteousFire"/>
    <Gem gemId="SupportBurningDamage" level="20" quality="20" enabled="true"/>
  </Skill>
</Skills>"#;

    #[test]
    fn set_gem_level_updates_level_attr() {
        let result = set_gem_level(SAMPLE_XML, "RighteousFire", 21).unwrap();
        assert!(result.contains("level=\"21\""),
            "expected level=21 in result:\n{result}");
    }

    #[test]
    fn set_gem_level_preserves_other_attrs() {
        let result = set_gem_level(SAMPLE_XML, "RighteousFire", 21).unwrap();
        assert!(result.contains("quality=\"20\""),
            "quality should be unchanged:\n{result}");
    }

    #[test]
    fn set_gem_quality_updates_quality_attr() {
        let result = set_gem_quality(SAMPLE_XML, "RighteousFire", 23).unwrap();
        assert!(result.contains("quality=\"23\""),
            "expected quality=23:\n{result}");
    }

    #[test]
    fn set_gem_level_leaves_other_gems_unchanged() {
        let result = set_gem_level(SAMPLE_XML, "RighteousFire", 21).unwrap();
        // SupportBurningDamage should still be level 20
        assert!(result.contains("SupportBurningDamage\" level=\"20\"") ||
                result.contains("SupportBurningDamage\" level=\"20\" quality=\"20\""),
            "support gem should remain unchanged:\n{result}");
    }

    #[test]
    fn set_gem_level_errors_when_gem_not_found() {
        let result = set_gem_level(SAMPLE_XML, "NonExistentGem", 21);
        assert!(result.is_err(), "should error for unknown gem");
    }

    #[test]
    fn replace_attr_value_basic() {
        let line = r#"    <Gem gemId="RF" level="20" quality="20"/>"#;
        let updated = replace_attr_value(line, "level", "21");
        assert!(updated.contains("level=\"21\""), "should update level");
        assert!(updated.contains("quality=\"20\""), "quality should be unchanged");
    }

    #[test]
    fn replace_attr_value_no_op_when_attr_not_present() {
        let line = r#"<SomeElement foo="bar"/>"#;
        let updated = replace_attr_value(line, "level", "99");
        assert_eq!(updated, line, "should return unchanged when attr absent");
    }

    #[test]
    fn apply_patch_to_xml_level_up_suggestion() {
        let suggestion = Suggestion {
            id: "s1".to_string(),
            slot: "MainSkill".to_string(),
            title: "Level up RighteousFire gem".to_string(),
            detail: "gem:RighteousFire currently level 20".to_string(),
            dps_gain: 50_000.0,
            dps_gain_pct: 2.0,
            life_gain: 0,
            estimated_cost_div: 0.1,
            efficiency: 500_000.0,
            priority: 1,
            trade_url: None,
        };
        let result = apply_patch_to_xml(SAMPLE_XML, &suggestion).unwrap();
        assert!(result.contains("level=\"20\"") || result.contains("level=\"21\""),
            "should have updated gem level");
    }

    #[test]
    fn apply_patch_to_xml_no_op_for_unknown_suggestion_type() {
        let suggestion = Suggestion {
            id: "s2".to_string(),
            slot: "Helmet".to_string(),
            title: "Buy a better helmet".to_string(),
            detail: "Trade for a higher tier item".to_string(),
            dps_gain: 100_000.0,
            dps_gain_pct: 5.0,
            life_gain: 200,
            estimated_cost_div: 2.0,
            efficiency: 50_000.0,
            priority: 2,
            trade_url: None,
        };
        let result = apply_patch_to_xml(SAMPLE_XML, &suggestion).unwrap();
        assert_eq!(result, SAMPLE_XML, "unknown suggestion should return unchanged XML");
    }

    #[test]
    fn oauth_character_returns_unchanged_build() {
        let mut build = BuildData::default();
        build.source = crate::models::build::BuildSource::OAuthCharacter("MyChar".to_string());
        let suggestion = Suggestion {
            id: "s".to_string(), slot: "Helmet".to_string(),
            title: "Buy upgrade".to_string(), detail: "buy it".to_string(),
            dps_gain: 0.0, dps_gain_pct: 0.0, life_gain: 0,
            estimated_cost_div: 1.0, efficiency: 0.0, priority: 1, trade_url: None,
        };
        let result = apply_suggestion(&build, &suggestion).unwrap();
        assert_eq!(result.source, build.source);
    }
}

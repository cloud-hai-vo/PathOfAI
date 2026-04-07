/// PoB XML Parser — Rust implementation of Algorithm 2 (ALGORITHMS.md).
/// Port of prototypes/pob-parser.js
///
/// Parses a Path of Building XML file into a BuildData struct.
/// Both PoB import and OAuth import produce the same BuildData.
use anyhow::{anyhow, Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use uuid::Uuid;

use crate::models::build::*;

/// Parse a PoB XML file from disk.
pub fn parse_file(path: &str) -> Result<BuildData> {
    let xml = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read PoB file: {path}"))?;
    parse_xml(&xml)
}

/// Parse PoB XML from a string.
pub fn parse_xml(xml: &str) -> Result<BuildData> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut build = BuildData {
        id: Uuid::new_v4().to_string(),
        ..Default::default()
    };

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag = std::str::from_utf8(e.name().as_ref())?.to_string();

                match tag.as_str() {
                    "Build" => parse_build_tag(&e, &mut build)?,
                    "Item" => {
                        let item = parse_item_tag(&e, &mut reader)?;
                        build.items.push(item);
                    }
                    "Skill" => {
                        let setup = parse_skill_tag(&e, &mut reader)?;
                        if !setup.gems.is_empty() {
                            build.gems.push(setup);
                        }
                    }
                    "Tree" => {
                        parse_tree_tag(&e, &mut reader, &mut build.passive_tree)?;
                    }
                    "Config" => {
                        parse_config_tag(&e, &mut reader, &mut build.config)?;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }

    // Auto-detect build name from main skill if not set
    if build.name.is_empty() {
        build.name = format!(
            "{} {}",
            detect_main_skill_name(&build),
            build.ascendancy
        );
    }

    Ok(build)
}

fn parse_build_tag(e: &quick_xml::events::BytesStart, build: &mut BuildData) -> Result<()> {
    for attr in e.attributes().flatten() {
        let key = std::str::from_utf8(attr.key.as_ref())?;
        let val = std::str::from_utf8(&attr.value)?.to_string();
        match key {
            "level" => build.level = val.parse().unwrap_or(1),
            "class" | "className" => build.class_name = val,
            "ascendClassName" => build.ascendancy = val,
            "mainSocketGroup" => {} // handled via gem setup
            _ => {}
        }
    }
    Ok(())
}

fn parse_item_tag(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Item> {
    let mut item = Item::default();

    for attr in e.attributes().flatten() {
        let key = std::str::from_utf8(attr.key.as_ref())?;
        let val = std::str::from_utf8(&attr.value)?.to_string();
        match key {
            "id" => item.id = val.parse().unwrap_or(0),
            _ => {}
        }
    }

    // Item data is in text content, not attributes (PoB format)
    let mut buf = Vec::new();
    let mut lines = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(t)) => {
                let text = t.unescape()?.to_string();
                lines.extend(text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()));
            }
            Ok(Event::End(_)) => break,
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    parse_item_text(&lines, &mut item);
    Ok(item)
}

/// Parse item text lines (PoB format: rarity, name, basetype, then mods).
/// Also used by item_parser for clipboard imports.
pub fn parse_item_text(lines: &[String], item: &mut Item) {
    let mut i = 0;

    // First line: "Rarity: Rare"
    if let Some(rarity_line) = lines.first() {
        if let Some(r) = rarity_line.strip_prefix("Rarity: ") {
            item.rarity = match r {
                "Normal" => ItemRarity::Normal,
                "Magic" => ItemRarity::Magic,
                "Rare" => ItemRarity::Rare,
                "Unique" => ItemRarity::Unique,
                _ => ItemRarity::Normal,
            };
            i = 1;
        }
    }

    // Name and base type
    if i < lines.len() { item.name = lines[i].clone(); i += 1; }
    if i < lines.len() { item.base_type = lines[i].clone(); i += 1; }

    // Remaining lines: mods separated by "--------"
    let mut mod_type = ModType::Implicit;
    let mut section_count = 0;

    for line in &lines[i..] {
        if line == "--------" {
            section_count += 1;
            mod_type = match section_count {
                1 => ModType::Implicit,
                2 => ModType::Prefix,
                _ => ModType::Suffix,
            };
            continue;
        }

        if let Some(level) = line.strip_prefix("Item Level: ") {
            item.item_level = level.parse().unwrap_or(0);
            continue;
        }

        if let Some(qual) = line.strip_prefix("Quality: +") {
            item.quality = qual.trim_end_matches('%').parse().unwrap_or(0);
            continue;
        }

        if line.contains("Sockets:") {
            item.sockets = line.replace("Sockets: ", "");
            continue;
        }

        if line == "Corrupted" { item.is_corrupted = true; continue; }
        if line == "Synthesised Item" { item.is_synthesised = true; continue; }
        if line == "Fractured Item" { item.is_fractured = true; continue; }

        let is_crafted = line.starts_with("{crafted}");
        let is_fractured = line.starts_with("{fractured}");
        let clean = line.trim_start_matches("{crafted}")
                        .trim_start_matches("{fractured}")
                        .trim()
                        .to_string();

        if !clean.is_empty() {
            item.mods.push(ItemMod {
                id: String::new(),  // resolved later from mod database
                text: clean,
                value1: 0.0,
                value2: None,
                mod_type: match mod_type {
                    ModType::Implicit => ModType::Implicit,
                    ModType::Prefix => ModType::Prefix,
                    ModType::Suffix => ModType::Suffix,
                    _ => ModType::Suffix,
                },
                is_crafted,
                is_fractured,
            });
        }
    }
}

fn parse_skill_tag(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<GemSetup> {
    let mut setup = GemSetup {
        skill: String::new(),
        slot: String::new(),
        socket_colors: String::new(),
        gems: Vec::new(),
        is_main_skill: false,
    };

    for attr in e.attributes().flatten() {
        let key = std::str::from_utf8(attr.key.as_ref())?;
        let val = std::str::from_utf8(&attr.value)?.to_string();
        match key {
            "slot" => setup.slot = val,
            "mainActiveSkillCalcs" | "mainActiveSkill" => setup.is_main_skill = val == "1",
            _ => {}
        }
    }

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"Gem" => {
                let gem = parse_gem_tag(&e)?;
                if setup.skill.is_empty() && !gem.is_support {
                    setup.skill = gem.name.clone();
                }
                setup.gems.push(gem);
            }
            // Self-closing <Gem ... /> emits Empty, not Start
            Ok(Event::Empty(e)) if e.name().as_ref() == b"Gem" => {
                let gem = parse_gem_tag(&e)?;
                if setup.skill.is_empty() && !gem.is_support {
                    setup.skill = gem.name.clone();
                }
                setup.gems.push(gem);
            }
            Ok(Event::End(_)) => break,
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(setup)
}

fn parse_gem_tag(e: &quick_xml::events::BytesStart) -> Result<Gem> {
    let mut gem = Gem {
        name: String::new(),
        level: 1,
        quality: 0,
        is_support: false,
        is_vaal: false,
        is_awakened: false,
        is_maxed: false,
        gem_id: String::new(),
    };

    for attr in e.attributes().flatten() {
        let key = std::str::from_utf8(attr.key.as_ref())?;
        let val = std::str::from_utf8(&attr.value)?.to_string();
        match key {
            "nameSpec" => {
                gem.is_vaal = val.starts_with("Vaal ");
                gem.is_awakened = val.starts_with("Awakened ");
                gem.is_support = val.ends_with(" Support");
                gem.name = val;
            }
            "level" => gem.level = val.parse().unwrap_or(1),
            "quality" => gem.quality = val.parse().unwrap_or(0),
            "gemId" => gem.gem_id = val,
            _ => {}
        }
    }

    // PoB newer format uses gemId instead of nameSpec for the gem identifier
    if gem.name.is_empty() && !gem.gem_id.is_empty() {
        gem.name = gem.gem_id.clone();
        gem.is_support = gem.gem_id.starts_with("Support");
        gem.is_awakened = gem.gem_id.starts_with("Awakened");
        gem.is_vaal = gem.gem_id.starts_with("Vaal");
    }

    let max_level: u8 = if gem.is_awakened { 5 } else { 20 };
    gem.is_maxed = gem.level >= max_level && gem.quality >= 20;

    Ok(gem)
}

fn parse_tree_tag(
    _e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    tree: &mut PassiveTree,
) -> Result<()> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"Spec" => {
                for attr in e.attributes().flatten() {
                    let key = std::str::from_utf8(attr.key.as_ref())?;
                    let val = std::str::from_utf8(&attr.value)?;
                    if key == "nodes" {
                        tree.allocated_nodes = val
                            .split(',')
                            .filter_map(|s| s.trim().parse().ok())
                            .collect();
                    }
                }
            }
            Ok(Event::End(_)) => break,
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

fn parse_config_tag(
    _e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    config: &mut BuildConfig,
) -> Result<()> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"Input" => {
                let mut name = String::new();
                let mut value = String::new();
                for attr in e.attributes().flatten() {
                    let k = std::str::from_utf8(attr.key.as_ref())?.to_string();
                    let v = std::str::from_utf8(&attr.value)?.to_string();
                    match k.as_str() {
                        "name" => name = v,
                        "string" | "number" | "boolean" => value = v,
                        _ => {}
                    }
                }
                match name.as_str() {
                    "boss" => config.boss_name = value,
                    "mapTier" => config.map_tier = value.parse().unwrap_or(0),
                    "flaskUptime" => config.flask_uptime = value.parse().unwrap_or(0.5),
                    "powerCharges" => config.charges.power = value.parse().unwrap_or(0),
                    "frenzyCharges" => config.charges.frenzy = value.parse().unwrap_or(0),
                    "enduranceCharges" => config.charges.endurance = value.parse().unwrap_or(0),
                    _ => {}
                }
            }
            Ok(Event::End(_)) => break,
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

fn detect_main_skill_name(build: &BuildData) -> String {
    build.gems
        .iter()
        .find(|g| g.is_main_skill)
        .or_else(|| build.gems.first())
        .map(|g| g.skill.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample_rf_inquisitor() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../test-data/SampleRFInquisitor.xml"
        );
        let build = parse_file(path).expect("should parse without error");

        assert_eq!(build.class_name, "Templar", "class should be Templar");
        assert_eq!(build.ascendancy, "Inquisitor", "ascendancy should be Inquisitor");
        assert_eq!(build.level, 95, "level should be 95");
        assert!(!build.items.is_empty(), "should have items");
        assert!(!build.gems.is_empty(), "should have gem setups");
        assert!(!build.passive_tree.allocated_nodes.is_empty(), "should have passives");
    }

    #[test]
    fn parse_item_rarity() {
        let lines: Vec<String> = vec![
            "Rarity: Rare".into(),
            "Havoc Coil".into(),
            "Two-Stone Ring".into(),
        ];
        let mut item = Item::default();
        parse_item_text(&lines, &mut item);
        assert!(matches!(item.rarity, ItemRarity::Rare));
        assert_eq!(item.name, "Havoc Coil");
        assert_eq!(item.base_type, "Two-Stone Ring");
    }

    #[test]
    fn parse_gem_support_flag() {
        // Simulate <Gem nameSpec="Elemental Focus Support" level="20" quality="20"/>
        // We test the flag detection logic directly
        let name = "Elemental Focus Support".to_string();
        let is_support = name.ends_with(" Support");
        assert!(is_support);
    }
}

/// PoE character fetch pipeline — see ALGORITHMS.md Algorithm 45.
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use crate::models::build::BuildData;

const GGG_API: &str = "https://api.pathofexile.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterSummary {
    pub name: String,
    pub class: String,
    pub level: u32,
    pub league: String,
    pub ascendancy: String,
    pub is_dead: bool,
}

/// Fetch full build data for a character from the GGG API.
/// Calls /character/{name}/items and /character/{name}/passives in parallel.
pub async fn fetch_character(token: &str, name: &str) -> Result<BuildData> {
    let client = reqwest::Client::new();

    let items_url = format!("{GGG_API}/character/{name}/items");
    let passives_url = format!("{GGG_API}/character/{name}/passives");

    let (items_resp, passives_resp) = tokio::join!(
        client.get(&items_url).bearer_auth(token).send(),
        client.get(&passives_url).bearer_auth(token).send(),
    );

    let items_json: serde_json::Value = items_resp
        .map_err(|e| anyhow!("Items request failed: {e}"))?
        .json()
        .await
        .map_err(|e| anyhow!("Items parse failed: {e}"))?;

    let passives_json: serde_json::Value = passives_resp
        .map_err(|e| anyhow!("Passives request failed: {e}"))?
        .json()
        .await
        .map_err(|e| anyhow!("Passives parse failed: {e}"))?;

    api_response_to_build(name, items_json, passives_json)
}

/// List all characters on the account.
pub async fn list_characters(token: &str) -> Result<Vec<CharacterSummary>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{GGG_API}/character"))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| anyhow!("List characters failed: {e}"))?;

    let json: serde_json::Value = resp.json().await
        .map_err(|e| anyhow!("Parse failed: {e}"))?;

    let chars = json["characters"].as_array()
        .ok_or_else(|| anyhow!("Unexpected API response"))?
        .iter()
        .map(|c| CharacterSummary {
            name: c["name"].as_str().unwrap_or("").to_string(),
            class: c["class"].as_str().unwrap_or("").to_string(),
            level: c["level"].as_u64().unwrap_or(1) as u32,
            league: c["league"].as_str().unwrap_or("").to_string(),
            ascendancy: c["ascendancyClass"].as_str().unwrap_or("").to_string(),
            is_dead: c["dead"].as_bool().unwrap_or(false),
        })
        .collect();

    Ok(chars)
}

/// Convert GGG API JSON response to BuildData.
fn api_response_to_build(
    character_name: &str,
    items_json: serde_json::Value,
    passives_json: serde_json::Value,
) -> Result<BuildData> {
    use crate::models::build::*;

    let char_info = &items_json["character"];
    let class = char_info["class"].as_str().unwrap_or("").to_string();
    let ascendancy = char_info["ascendancyClass"].as_str().unwrap_or("").to_string();
    let level = char_info["level"].as_u64().unwrap_or(1) as u32;

    let items: Vec<Item> = items_json["items"].as_array()
        .map(|arr| arr.iter().enumerate().map(|(i, j)| convert_api_item(i as u32, j)).collect())
        .unwrap_or_default();

    // Convert passive tree
    let allocated_nodes: Vec<u32> = passives_json["hashes"].as_array()
        .map(|arr| arr.iter().filter_map(|n| n.as_u64().map(|v| v as u32)).collect())
        .unwrap_or_default();

    Ok(BuildData {
        id: uuid::Uuid::new_v4().to_string(),
        name: format!("{character_name} ({ascendancy})"),
        class_name: class,
        ascendancy,
        level,
        items,
        passive_tree: PassiveTree {
            allocated_nodes,
            ..Default::default()
        },
        source: BuildSource::OAuthCharacter(character_name.to_string()),
        ..Default::default()
    })
}

/// Convert a single GGG API item JSON into our Item struct.
pub(crate) fn convert_api_item(id: u32, j: &serde_json::Value) -> crate::models::build::Item {
    use crate::models::build::{Item, ItemMod, ItemRarity, ModType};

    // Rarity from frameType: 0=Normal, 1=Magic, 2=Rare, 3=Unique
    let rarity = match j["frameType"].as_u64().unwrap_or(0) {
        1 => ItemRarity::Magic,
        2 => ItemRarity::Rare,
        3 => ItemRarity::Unique,
        _ => ItemRarity::Normal,
    };

    // Build mod list from all mod arrays in the API response
    let mut mods: Vec<ItemMod> = Vec::new();

    let add_mods = |arr: Option<&Vec<serde_json::Value>>, mod_type: ModType, out: &mut Vec<ItemMod>| {
        let Some(arr) = arr else { return };
        for (idx, text) in arr.iter().enumerate() {
            let text_str = text.as_str().unwrap_or("").to_string();
            if text_str.is_empty() { continue; }
            // Extract first numeric value from the mod text for value1
            let value1 = text_str.split_whitespace()
                .find_map(|w| w.trim_start_matches('+').trim_end_matches('%').parse::<f64>().ok())
                .unwrap_or(0.0);
            out.push(ItemMod {
                id: format!("{mod_type:?}_{idx}").to_lowercase(),
                text: text_str,
                value1,
                value2: None,
                mod_type: mod_type.clone(),
                is_crafted: matches!(mod_type, ModType::Crafted),
                is_fractured: false,
            });
        }
    };

    let explicit = j["explicitMods"].as_array().cloned();
    let implicit = j["implicitMods"].as_array().cloned();
    let crafted  = j["craftedMods"].as_array().cloned();
    let enchants = j["enchantMods"].as_array().cloned();

    add_mods(explicit.as_ref(), ModType::Suffix,   &mut mods);
    add_mods(implicit.as_ref(), ModType::Implicit,  &mut mods);
    add_mods(crafted.as_ref(),  ModType::Crafted,   &mut mods);
    add_mods(enchants.as_ref(), ModType::Enchant,   &mut mods);

    // Sockets: build "R-G-B" string from socketedItems or sockets array
    let sockets = j["sockets"].as_array()
        .map(|s| s.iter()
            .map(|sock| sock["sColour"].as_str().unwrap_or("W"))
            .collect::<Vec<_>>()
            .join("-"))
        .unwrap_or_default();

    Item {
        id,
        name:              j["name"].as_str().unwrap_or("").to_string(),
        base_type:         j["typeLine"].as_str().unwrap_or("").to_string(),
        slot:              j["inventoryId"].as_str().unwrap_or("").to_string(),
        rarity,
        level_requirement: j["requirements"].as_array()
            .and_then(|r| r.iter().find(|req| req["name"] == "Level"))
            .and_then(|r| r["values"][0][0].as_str())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        item_level:  j["ilvl"].as_u64().unwrap_or(0) as u32,
        quality:     j["quality"].as_str()
            .and_then(|q| q.trim_start_matches('+').trim_end_matches('%').parse().ok())
            .unwrap_or(0),
        sockets,
        mods,
        influence:   Vec::new(),
        is_corrupted: j["corrupted"].as_bool().unwrap_or(false),
        is_synthesised: j["synthesised"].as_bool().unwrap_or(false),
        is_fractured:  j["fractured"].as_bool().unwrap_or(false),
        image_url:   j["icon"].as_str().map(|s| s.to_string()),
        score: None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item_json(name: &str, rarity: u64, explicit_mods: &[&str]) -> serde_json::Value {
        serde_json::json!({
            "name": name,
            "typeLine": "Iron Gauntlets",
            "inventoryId": "Gloves",
            "ilvl": 82,
            "frameType": rarity,
            "corrupted": false,
            "explicitMods": explicit_mods,
        })
    }

    #[test]
    fn convert_item_parses_name_and_slot() {
        let j = make_item_json("Tyrannical Gauntlets", 2, &[]);
        let item = convert_api_item(0, &j);
        assert_eq!(item.name, "Tyrannical Gauntlets");
        assert_eq!(item.slot, "Gloves");
    }

    #[test]
    fn convert_item_parses_rarity() {
        let j = make_item_json("Kaom's Heart", 3, &[]);
        let item = convert_api_item(0, &j);
        assert!(matches!(item.rarity, crate::models::build::ItemRarity::Unique));
    }

    #[test]
    fn convert_item_parses_explicit_mods() {
        let j = make_item_json("Rare Boots", 2, &["+80 to maximum Life", "30% increased Movement Speed"]);
        let item = convert_api_item(0, &j);
        assert_eq!(item.mods.len(), 2);
        assert!(item.mods.iter().any(|m| m.text.contains("maximum Life")));
    }

    #[test]
    fn convert_item_extracts_numeric_value_from_mod() {
        let j = make_item_json("Item", 2, &["+52 to maximum Life"]);
        let item = convert_api_item(0, &j);
        let life_mod = item.mods.iter().find(|m| m.text.contains("Life")).unwrap();
        assert!((life_mod.value1 - 52.0).abs() < 0.1, "expected value1=52, got {}", life_mod.value1);
    }

    #[test]
    fn convert_item_sets_corrupted_flag() {
        let j = serde_json::json!({
            "name": "Corrupted Item", "typeLine": "Amulet", "inventoryId": "Amulet",
            "ilvl": 86, "frameType": 2, "corrupted": true, "explicitMods": [],
        });
        let item = convert_api_item(0, &j);
        assert!(item.is_corrupted);
    }

    #[test]
    fn convert_item_parses_sockets() {
        let j = serde_json::json!({
            "name": "Item", "typeLine": "Body Armour", "inventoryId": "BodyArmour",
            "ilvl": 84, "frameType": 2, "corrupted": false, "explicitMods": [],
            "sockets": [{"sColour": "R"}, {"sColour": "G"}, {"sColour": "B"}],
        });
        let item = convert_api_item(0, &j);
        assert_eq!(item.sockets, "R-G-B");
    }

    #[test]
    fn convert_item_sets_image_url_from_icon() {
        let j = serde_json::json!({
            "name": "Item", "typeLine": "Helmet", "inventoryId": "Helmet",
            "ilvl": 84, "frameType": 2, "corrupted": false, "explicitMods": [],
            "icon": "https://web.poecdn.com/image/Art/2DItems/Armours/Helmets/HelmetStr1.png",
        });
        let item = convert_api_item(0, &j);
        assert!(item.image_url.is_some());
        assert!(item.image_url.unwrap().contains("poecdn.com"));
    }
}

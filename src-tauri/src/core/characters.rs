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

    // Convert API items → Item structs
    // TODO: full item conversion (requires base item data)
    let items: Vec<Item> = items_json["items"].as_array()
        .map(|arr| arr.iter().enumerate().map(|(i, item_json)| {
            Item {
                id: i as u32,
                name: item_json["name"].as_str().unwrap_or("").to_string(),
                base_type: item_json["typeLine"].as_str().unwrap_or("").to_string(),
                slot: item_json["inventoryId"].as_str().unwrap_or("").to_string(),
                item_level: item_json["ilvl"].as_u64().unwrap_or(0) as u32,
                is_corrupted: item_json["corrupted"].as_bool().unwrap_or(false),
                ..Default::default()
            }
        }).collect())
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

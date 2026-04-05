use serde::{Deserialize, Serialize};

/// Top-level build data parsed from PoB XML or PoE OAuth character API.
/// This is the single source of truth for all calculator inputs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildData {
    pub id: String,
    pub name: String,
    pub class_name: String,
    pub ascendancy: String,
    pub level: u32,
    pub items: Vec<Item>,
    pub gems: Vec<GemSetup>,
    pub passive_tree: PassiveTree,
    pub config: BuildConfig,
    pub source: BuildSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum BuildSource {
    #[default]
    Unknown,
    PobFile(String),   // file path
    OAuthCharacter(String), // character name
}

/// A single equipped or inventory item.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub base_type: String,
    pub slot: String,           // "Helmet", "BodyArmour", "Ring1", etc.
    pub rarity: ItemRarity,
    pub level_requirement: u32,
    pub item_level: u32,
    pub quality: u32,
    pub sockets: String,        // e.g. "R-G-B-B" (links separated by -)
    pub mods: Vec<ItemMod>,
    pub influence: Vec<String>, // "Shaper", "Elder", etc.
    pub is_corrupted: bool,
    pub is_synthesised: bool,
    pub is_fractured: bool,
    pub image_url: Option<String>,
    pub score: Option<u8>,      // 0-100, filled by item_scorer
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ItemRarity {
    #[default]
    Normal,
    Magic,
    Rare,
    Unique,
}

/// A single mod on an item (prefix, suffix, implicit, crafted, enchant).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMod {
    pub id: String,             // internal mod ID from poedb
    pub text: String,           // display text: "+52 to maximum Life"
    pub value1: f64,
    pub value2: Option<f64>,    // for ranges: "adds X to Y damage"
    pub mod_type: ModType,
    pub is_crafted: bool,
    pub is_fractured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModType {
    Prefix,
    Suffix,
    Implicit,
    Enchant,
    Corrupted,
}

/// A linked gem setup (main skill or utility).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GemSetup {
    pub skill: String,          // main skill name
    pub slot: String,           // which item slot this lives in
    pub socket_colors: String,  // "RRGB"
    pub gems: Vec<Gem>,
    pub is_main_skill: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gem {
    pub name: String,
    pub level: u8,              // 1-21
    pub quality: u8,            // 0-23
    pub is_support: bool,
    pub is_vaal: bool,
    pub is_awakened: bool,
    pub is_maxed: bool,         // level == max_level && quality == 20
    pub gem_id: String,         // internal ID
}

/// Allocated passive tree nodes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PassiveTree {
    pub allocated_nodes: Vec<u32>,
    pub jewels: Vec<TreeJewel>,
    pub masteries: Vec<MasterySelection>,
    pub cluster_jewels: Vec<ClusterJewel>,
    pub ascendancy_nodes: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeJewel {
    pub socket_id: u32,
    pub item: Item,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterySelection {
    pub node_id: u32,
    pub effect_id: u32,
    pub effect_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterJewel {
    pub socket_id: u32,
    pub small_nodes: Vec<String>,
    pub notable: String,
    pub keystone: Option<String>,
}

/// PoB Config section — assumptions for calculation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    pub boss_name: String,      // "Shaper", "Maven", etc.
    pub map_tier: u32,
    pub is_uberlab: bool,
    pub charges: ChargeConfig,
    pub flask_uptime: f64,      // 0.0-1.0
    pub aura_uptime: f64,
    pub is_moving: bool,
    pub is_on_full_life: bool,
    pub is_on_low_life: bool,
    pub minion_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChargeConfig {
    pub endurance: u32,
    pub frenzy: u32,
    pub power: u32,
}

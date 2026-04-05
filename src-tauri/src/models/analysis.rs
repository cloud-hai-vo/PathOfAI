use serde::{Deserialize, Serialize};
use crate::models::build::GemSetup;

/// Full analysis result returned by `analyze_build` and `load_character`.
/// This is what the frontend receives and renders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub build_id: String,
    pub build_name: String,
    pub class_name: String,
    pub ascendancy: String,
    pub level: u32,
    pub archetype: String,          // "fire_dot", "cold_dot", "attack", etc.
    pub archetype_label: String,    // "RF Inquisitor"
    pub overall_score: u8,          // 0-100
    pub defenses: DefenseStats,
    pub offense: OffenseStats,
    pub issues: Vec<Issue>,
    pub suggestions: Vec<Suggestion>,
    pub item_scores: Vec<ItemScore>,
    pub gem_setups: Vec<GemSetup>,  // gem links — for the Gems panel
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefenseStats {
    pub life: u32,
    pub energy_shield: u32,
    pub mana: u32,
    pub life_regen_flat: f64,       // per second
    pub life_regen_pct: f64,        // % of max life per second
    pub resistances: ResistanceProfile,
    pub armour: u32,
    pub armour_phys_reduction: f64, // % vs 5000 hit (standard reference)
    pub evasion: u32,
    pub evasion_chance: f64,        // 0.0-1.0
    pub block_chance: f64,
    pub spell_block_chance: f64,
    pub effective_hp: EffectiveHP,
    pub ailment_immunity: AilmentImmunity,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResistanceProfile {
    pub fire: i32,      // can be negative (chaos res)
    pub cold: i32,
    pub lightning: i32,
    pub chaos: i32,
    pub max_fire: i32,  // default 75
    pub max_cold: i32,
    pub max_lightning: i32,
    pub max_chaos: i32,
    pub fire_overcap: i32,     // how much over max
    pub cold_overcap: i32,
    pub lightning_overcap: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EffectiveHP {
    pub vs_physical: u32,
    pub vs_elemental: u32,
    pub vs_chaos: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AilmentImmunity {
    pub freeze: bool,
    pub freeze_source: Option<String>,
    pub shock: bool,
    pub shock_source: Option<String>,
    pub ignite: bool,
    pub ignite_source: Option<String>,
    pub bleed: bool,
    pub bleed_source: Option<String>,
    pub corrupted_blood: bool,
    pub corrupted_blood_source: Option<String>,
    pub poison: bool,
    pub stun: bool,
    pub curse_immune: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OffenseStats {
    pub total_dps: f64,
    pub dps_label: String,          // "2.84M"
    pub main_skill: String,
    pub hit_dps: f64,
    pub dot_dps: f64,
    pub crit_chance: f64,
    pub crit_multiplier: f64,
    pub attack_speed: f64,          // attacks per second (0 for spells/DoT)
    pub cast_speed: f64,
    pub hit_chance: f64,
    pub sources: Vec<DpsSource>,
    pub multiplier_chain: Vec<MultiplierStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpsSource {
    pub source: String,             // "Righteous Fire", "Scorching Ray"
    pub value: f64,
    pub percent_of_total: f64,
    pub color: String,              // CSS color for UI bar
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplierStep {
    pub label: String,              // "Increased Damage"
    pub multiplier: f64,            // 2.34
    pub step_type: MultiplierType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MultiplierType {
    Base,
    Increased,
    More,
    Penetration,
}

/// A detected build problem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub severity: Severity,
    pub title: String,
    pub detail: String,
    pub fix: String,
    pub slot: Option<String>,       // which equipment slot is affected (if any)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Critical,   // will die — must fix
    Major,      // significant damage/defense loss
    Minor,      // small improvement available
    Info,       // informational
}

/// An upgrade suggestion with price and impact estimate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: String,
    pub slot: String,
    pub title: String,
    pub detail: String,
    pub dps_gain: f64,
    pub dps_gain_pct: f64,
    pub life_gain: i32,
    pub estimated_cost_div: f64,
    pub efficiency: f64,            // DPS-per-divine
    pub priority: u32,              // 1 = highest priority
    pub trade_url: Option<String>,
}

/// Per-item score for the equipment list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemScore {
    pub slot: String,
    pub item_name: String,
    pub score: u8,                  // 0-100
    pub tier: ScoreTier,
    pub top_issue: Option<String>,  // worst mod problem on this item
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScoreTier {
    BiS,        // 90-100
    Excellent,  // 75-89
    Good,       // 60-74
    Acceptable, // 40-59
    Upgrade,    // 20-39
    Replace,    // 0-19
}

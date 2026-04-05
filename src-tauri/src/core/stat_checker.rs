/// stat_checker.rs — Stat Requirement Checker (Algorithm 30).
///
/// Verifies that the character meets attribute requirements for all equipped items
/// and gems. Also checks if swapping in a new item would break requirements.
use serde::{Deserialize, Serialize};
use crate::models::build::{BuildData, Item};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Attribute {
    Strength,
    Dexterity,
    Intelligence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatDeficiency {
    pub stat:          Attribute,
    pub required:      u32,
    pub available:     u32,
    pub shortfall:     i32,       // always negative for deficiencies
    pub blocking_slot: String,    // which item slot imposes the requirement
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeTotals {
    pub strength:     u32,
    pub dexterity:    u32,
    pub intelligence: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatCheckResult {
    pub totals:       AttributeTotals,
    pub deficiencies: Vec<StatDeficiency>,
    pub all_met:      bool,
}

// ─── Class base attributes ────────────────────────────────────────────────────

fn class_base_attrs(class_name: &str) -> (u32, u32, u32) {
    // (str, dex, int) — PoE base class starting attributes
    match class_name.to_lowercase().as_str() {
        "marauder"  => (23, 10, 10),
        "ranger"    => (10, 23, 10),
        "witch"     => (10, 10, 23),
        "duelist"   => (16, 16, 10),
        "templar"   => (16, 10, 16),
        "shadow"    => (10, 16, 16),
        "scion"     => (14, 14, 14),
        _           => (14, 14, 14), // fallback
    }
}

// ─── Attribute extraction from item mods ─────────────────────────────────────

fn extract_item_attributes(item: &Item) -> (u32, u32, u32) {
    let mut str_bonus = 0u32;
    let mut dex_bonus = 0u32;
    let mut int_bonus = 0u32;

    for m in &item.mods {
        let text = m.text.to_lowercase();
        let val = m.value1.max(0.0) as u32;

        if text.contains("to strength") && !text.contains("dexterity") && !text.contains("intelligence") {
            str_bonus += val;
        } else if text.contains("to dexterity") && !text.contains("strength") && !text.contains("intelligence") {
            dex_bonus += val;
        } else if text.contains("to intelligence") && !text.contains("strength") && !text.contains("dexterity") {
            int_bonus += val;
        } else if text.contains("to all attributes") {
            str_bonus += val;
            dex_bonus += val;
            int_bonus += val;
        }
    }
    (str_bonus, dex_bonus, int_bonus)
}

// ─── Requirement extraction from item level_requirement ──────────────────────

/// Infer attribute requirements from item mods.
/// In PoB XML, requirements are sometimes stored as mods with text like
/// "Requires Level X, Y Str" or as separate fields.
/// We use a simple heuristic: items named or modded explicitly.
fn infer_requirements(item: &Item) -> (u32, u32, u32) {
    let mut req_str = 0u32;
    let mut req_dex = 0u32;
    let mut req_int = 0u32;

    for m in &item.mods {
        let text = m.text.to_lowercase();
        let val = m.value1.max(0.0) as u32;

        if text.contains("requires") {
            if text.contains("str") || text.contains("strength") {
                req_str = req_str.max(val);
            }
            if text.contains("dex") || text.contains("dexterity") {
                req_dex = req_dex.max(val);
            }
            if text.contains("int") || text.contains("intelligence") {
                req_int = req_int.max(val);
            }
        }
    }
    (req_str, req_dex, req_int)
}

// ─── Core algorithm ───────────────────────────────────────────────────────────

/// Calculate total available attributes from all equipped items + class base.
pub fn calculate_attribute_totals(build: &BuildData) -> AttributeTotals {
    let (base_str, base_dex, base_int) = class_base_attrs(&build.class_name);

    let mut total_str = base_str;
    let mut total_dex = base_dex;
    let mut total_int = base_int;

    for item in &build.items {
        let (s, d, i) = extract_item_attributes(item);
        total_str += s;
        total_dex += d;
        total_int += i;
    }

    // Each 10 strength grants +5 life but also implicitly: item base 30 Str = +15 life
    // No attribute-from-tree (would need game data), so tree bonus not counted here.

    AttributeTotals { strength: total_str, dexterity: total_dex, intelligence: total_int }
}

/// Check all items for unmet requirements, optionally simulating a swap.
/// `candidate` — if provided, replaces the item in the same slot before checking.
pub fn check_requirements(
    build: &BuildData,
    candidate: Option<&Item>,
) -> StatCheckResult {
    let totals = calculate_attribute_totals(build);

    // Build the effective item list (swap candidate in if provided)
    let mut items: Vec<&Item> = build.items.iter().collect();
    if let Some(c) = candidate {
        items.retain(|i| i.slot != c.slot);
        items.push(c);
    }

    let mut deficiencies = Vec::new();

    for item in &items {
        let (req_str, req_dex, req_int) = infer_requirements(item);
        let slot = item.slot.as_str();

        if req_str > 0 && totals.strength < req_str {
            deficiencies.push(StatDeficiency {
                stat:          Attribute::Strength,
                required:      req_str,
                available:     totals.strength,
                shortfall:     totals.strength as i32 - req_str as i32,
                blocking_slot: slot.to_string(),
            });
        }
        if req_dex > 0 && totals.dexterity < req_dex {
            deficiencies.push(StatDeficiency {
                stat:          Attribute::Dexterity,
                required:      req_dex,
                available:     totals.dexterity,
                shortfall:     totals.dexterity as i32 - req_dex as i32,
                blocking_slot: slot.to_string(),
            });
        }
        if req_int > 0 && totals.intelligence < req_int {
            deficiencies.push(StatDeficiency {
                stat:          Attribute::Intelligence,
                required:      req_int,
                available:     totals.intelligence,
                shortfall:     totals.intelligence as i32 - req_int as i32,
                blocking_slot: slot.to_string(),
            });
        }
    }

    // Sort most severe first (most negative shortfall first)
    deficiencies.sort_by_key(|d| d.shortfall);

    let all_met = deficiencies.is_empty();
    StatCheckResult { totals, deficiencies, all_met }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::build::{BuildData, Item, ItemMod, ModType};

    fn mod_text(text: &str, val: f64) -> ItemMod {
        ItemMod {
            id: text.to_lowercase().replace(' ', "_"),
            text: text.to_string(),
            value1: val,
            value2: None,
            mod_type: ModType::Suffix,
            is_crafted: false,
            is_fractured: false,
        }
    }

    fn item_with_mods(slot: &str, mods: Vec<ItemMod>) -> Item {
        Item {
            id: 1,
            name: "Test Item".to_string(),
            base_type: "Ring".to_string(),
            slot: slot.to_string(),
            mods,
            ..Default::default()
        }
    }

    fn build_with_items(class: &str, items: Vec<Item>) -> BuildData {
        BuildData {
            class_name: class.to_string(),
            items,
            ..Default::default()
        }
    }

    // ── Attribute totals ──────────────────────────────────────────────────────

    #[test]
    fn templar_base_str_dex_int_14_10_16() {
        let build = build_with_items("Templar", vec![]);
        let totals = calculate_attribute_totals(&build);
        assert_eq!(totals.strength, 16);
        assert_eq!(totals.dexterity, 10);
        assert_eq!(totals.intelligence, 16);
    }

    #[test]
    fn marauder_base_has_high_str() {
        let build = build_with_items("Marauder", vec![]);
        let totals = calculate_attribute_totals(&build);
        assert_eq!(totals.strength, 23);
        assert!(totals.strength > totals.dexterity);
    }

    #[test]
    fn item_mod_adds_to_strength() {
        let ring = item_with_mods("Ring1", vec![
            mod_text("+30 to Strength", 30.0),
        ]);
        let build = build_with_items("Templar", vec![ring]);
        let totals = calculate_attribute_totals(&build);
        assert_eq!(totals.strength, 16 + 30);
    }

    #[test]
    fn all_attributes_mod_adds_to_all() {
        let amulet = item_with_mods("Amulet", vec![
            mod_text("+20 to all Attributes", 20.0),
        ]);
        let build = build_with_items("Templar", vec![amulet]);
        let totals = calculate_attribute_totals(&build);
        assert_eq!(totals.strength,     16 + 20);
        assert_eq!(totals.dexterity,    10 + 20);
        assert_eq!(totals.intelligence, 16 + 20);
    }

    #[test]
    fn multiple_items_stack_attributes() {
        let items = vec![
            item_with_mods("Ring1", vec![mod_text("+10 to Strength", 10.0)]),
            item_with_mods("Ring2", vec![mod_text("+15 to Dexterity", 15.0)]),
            item_with_mods("Amulet", vec![mod_text("+25 to Intelligence", 25.0)]),
        ];
        let build = build_with_items("Scion", vec![]);
        let base = calculate_attribute_totals(&build);
        let build2 = build_with_items("Scion", items);
        let totals = calculate_attribute_totals(&build2);
        assert_eq!(totals.strength,     base.strength + 10);
        assert_eq!(totals.dexterity,    base.dexterity + 15);
        assert_eq!(totals.intelligence, base.intelligence + 25);
    }

    // ── Requirement checks ────────────────────────────────────────────────────

    #[test]
    fn no_requirements_means_all_met() {
        let build = build_with_items("Templar", vec![
            item_with_mods("Ring1", vec![]),
        ]);
        let result = check_requirements(&build, None);
        assert!(result.all_met);
        assert!(result.deficiencies.is_empty());
    }

    #[test]
    fn unmet_str_requirement_creates_deficiency() {
        let sword = item_with_mods("Weapon", vec![
            mod_text("Requires 200 Str", 200.0),
        ]);
        let build = build_with_items("Witch", vec![sword]); // Witch: 10 str
        let result = check_requirements(&build, None);
        assert!(!result.all_met);
        assert_eq!(result.deficiencies.len(), 1);
        assert_eq!(result.deficiencies[0].stat, Attribute::Strength);
        assert!(result.deficiencies[0].shortfall < 0);
    }

    #[test]
    fn candidate_swap_check_uses_new_item() {
        // Base build has a low-req ring
        let old_ring = item_with_mods("Ring1", vec![]);
        let build = build_with_items("Witch", vec![old_ring]);

        // Swap in a high-req ring
        let new_ring = item_with_mods("Ring1", vec![
            mod_text("Requires 150 Intelligence", 150.0),
        ]);
        let result = check_requirements(&build, Some(&new_ring));
        // Witch has 23 int base — deficient
        assert!(!result.all_met);
        let def = &result.deficiencies[0];
        assert_eq!(def.stat, Attribute::Intelligence);
        assert_eq!(def.blocking_slot, "Ring1");
    }

    #[test]
    fn candidate_swap_removes_old_slot() {
        // Old item in Ring1 has a high requirement
        let old_ring = item_with_mods("Ring1", vec![
            mod_text("Requires 150 Strength", 150.0),
        ]);
        let build = build_with_items("Marauder", vec![old_ring]); // Marauder: 23 str
        // Without candidate: would fail
        let before = check_requirements(&build, None);
        assert!(!before.all_met);

        // Swap Ring1 for a ring with no requirements
        let new_ring = item_with_mods("Ring1", vec![]);
        let after = check_requirements(&build, Some(&new_ring));
        assert!(after.all_met);
    }

    #[test]
    fn deficiencies_sorted_by_severity() {
        let items = vec![
            item_with_mods("Weapon", vec![mod_text("Requires 200 Str", 200.0)]),
            item_with_mods("Ring1",  vec![mod_text("Requires 100 Dex", 100.0)]),
        ];
        let build = build_with_items("Witch", vec![]);
        let build_with = BuildData { items, ..build };
        let result = check_requirements(&build_with, None);
        // Witch: 10 str → shortfall for weapon = 10 - 200 = -190
        // Witch: 10 dex → shortfall for ring = 10 - 100 = -90
        // -190 should come first (more severe)
        assert_eq!(result.deficiencies[0].stat, Attribute::Strength);
        assert!(result.deficiencies[0].shortfall < result.deficiencies[1].shortfall);
    }

    #[test]
    fn unknown_class_uses_fallback_14_14_14() {
        let build = build_with_items("CustomClass", vec![]);
        let totals = calculate_attribute_totals(&build);
        assert_eq!(totals.strength, 14);
        assert_eq!(totals.dexterity, 14);
        assert_eq!(totals.intelligence, 14);
    }
}

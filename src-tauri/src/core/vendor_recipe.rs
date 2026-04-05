/// vendor_recipe.rs — Vendor Recipe Detector (Algorithm 43).
///
/// Detects chaos recipe (ilvl 60-74), regal recipe (ilvl 75+), and quality
/// recipes from stash items. Identifies how many complete sets exist and what
/// slots are missing to complete the next set.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Types ────────────────────────────────────────────────────────────────────

/// Normalised equipment slot for recipe matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipSlot {
    Helmet,
    Chest,
    Gloves,
    Boots,
    Belt,
    Amulet,
    Ring,
    Weapon,  // both 1H and 2H
    Shield,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeItem {
    pub id:          String,
    pub name:        String,
    pub slot:        EquipSlot,
    pub item_level:  u32,
    pub is_two_hand: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSet {
    pub items: Vec<RecipeItem>,  // 10-11 items making up a full set
    pub is_unidentified: bool,   // all items unidentified → double chaos
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRecipe {
    pub output: String,    // "Glassblower's Bauble", "Gemcutter's Prism", etc.
    pub count:  u32,       // how many of that currency you'd get
}

/// Input item — stripped-down version of StashItem with only recipe-relevant fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCandidate {
    pub id:              String,
    pub name:            String,
    pub base_type:       String,
    pub item_level:      u32,
    pub quality:         u32,
    pub is_rare:         bool,
    pub is_identified:   bool,
    pub is_two_hand:     bool,
    pub item_class:      String, // "Helmet", "BodyArmour", "Gloves", "Boots",
                                 // "Belt", "Amulet", "Ring", "Weapon", "Shield",
                                 // "Flask", "Gem", "Map"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeAnalysis {
    pub chaos_sets:      Vec<RecipeSet>,   // complete sets from ilvl 60-74 rares
    pub regal_sets:      Vec<RecipeSet>,   // complete sets from ilvl 75+ rares
    pub chaos_set_count: u32,
    pub regal_set_count: u32,
    pub missing_slots:   Vec<String>,      // slot names needed to complete next set
    pub quality_recipes: Vec<QualityRecipe>,
}

// ─── Slot inference ───────────────────────────────────────────────────────────

/// Map item_class string to EquipSlot for recipe purposes.
pub fn infer_slot(item: &RecipeCandidate) -> Option<EquipSlot> {
    match item.item_class.to_lowercase().as_str() {
        "helmet" | "head" => Some(EquipSlot::Helmet),
        "bodyarmour" | "body armour" | "chest" => Some(EquipSlot::Chest),
        "gloves" => Some(EquipSlot::Gloves),
        "boots" => Some(EquipSlot::Boots),
        "belt" => Some(EquipSlot::Belt),
        "amulet" => Some(EquipSlot::Amulet),
        "ring" => Some(EquipSlot::Ring),
        "weapon" | "sword" | "axe" | "mace" | "bow" | "staff"
        | "claw" | "dagger" | "wand" | "sceptre" => Some(EquipSlot::Weapon),
        "shield" => Some(EquipSlot::Shield),
        _ => None,
    }
}

// ─── Set counting ─────────────────────────────────────────────────────────────

/// Compute available weapon "slots" for set building.
/// A 2H weapon counts as 2 weapon slots (fills both weapon + shield slots).
fn weapon_slots_available(pool: &HashMap<EquipSlot, Vec<&RecipeCandidate>>) -> usize {
    let weapons  = pool.get(&EquipSlot::Weapon).map(|v| v.len()).unwrap_or(0);
    let shields  = pool.get(&EquipSlot::Shield).map(|v| v.len()).unwrap_or(0);
    let two_hand = pool.get(&EquipSlot::Weapon)
        .map(|v| v.iter().filter(|i| i.is_two_hand).count())
        .unwrap_or(0);

    // 2H counts as filling both weapon + shield slots
    // Each 2H = 2 weapon-slots; each 1H+Shield pair = 2 weapon-slots; each 1H alone = 1
    let one_hand = weapons - two_hand;
    two_hand * 2 + one_hand.min(shields) * 2 + one_hand.saturating_sub(shields)
}

fn consume_weapon_slots(pool: &mut HashMap<EquipSlot, Vec<&RecipeCandidate>>) {
    // Prefer 2H first, then 1H+Shield, then bare 1H
    let two_hand_idx = pool.get(&EquipSlot::Weapon)
        .and_then(|v| v.iter().position(|i| i.is_two_hand));

    if let Some(idx) = two_hand_idx {
        pool.get_mut(&EquipSlot::Weapon).unwrap().swap_remove(idx);
    } else {
        // Use 1H + Shield if available
        let has_shield = pool.get(&EquipSlot::Shield).map(|v| !v.is_empty()).unwrap_or(false);
        let has_weapon = pool.get(&EquipSlot::Weapon).map(|v| !v.is_empty()).unwrap_or(false);
        if has_weapon && has_shield {
            pool.get_mut(&EquipSlot::Weapon).unwrap().pop();
            pool.get_mut(&EquipSlot::Shield).unwrap().pop();
        } else if has_weapon {
            pool.get_mut(&EquipSlot::Weapon).unwrap().pop();
        }
    }
}

/// Build as many complete sets as possible from the pool.
fn build_sets<'a>(pool: &mut HashMap<EquipSlot, Vec<&'a RecipeCandidate>>) -> Vec<RecipeSet> {
    let mut sets = vec![];

    loop {
        let helmet  = pool.get(&EquipSlot::Helmet).map(|v| v.len()).unwrap_or(0);
        let chest   = pool.get(&EquipSlot::Chest).map(|v| v.len()).unwrap_or(0);
        let gloves  = pool.get(&EquipSlot::Gloves).map(|v| v.len()).unwrap_or(0);
        let boots   = pool.get(&EquipSlot::Boots).map(|v| v.len()).unwrap_or(0);
        let belt    = pool.get(&EquipSlot::Belt).map(|v| v.len()).unwrap_or(0);
        let amulet  = pool.get(&EquipSlot::Amulet).map(|v| v.len()).unwrap_or(0);
        let rings   = pool.get(&EquipSlot::Ring).map(|v| v.len()).unwrap_or(0);
        let weapons = weapon_slots_available(pool);

        if helmet < 1 || chest < 1 || gloves < 1 || boots < 1
            || belt < 1 || amulet < 1 || rings < 2 || weapons < 2 {
            break;
        }

        // Collect items for this set
        let mut set_items = Vec::new();
        let slots_to_take = [
            (EquipSlot::Helmet, 1),
            (EquipSlot::Chest, 1),
            (EquipSlot::Gloves, 1),
            (EquipSlot::Boots, 1),
            (EquipSlot::Belt, 1),
            (EquipSlot::Amulet, 1),
            (EquipSlot::Ring, 2),
        ];
        for (slot, n) in slots_to_take {
            if let Some(v) = pool.get_mut(&slot) {
                for _ in 0..n {
                    if let Some(item) = v.pop() {
                        set_items.push(RecipeItem {
                            id: item.id.clone(),
                            name: item.name.clone(),
                            slot,
                            item_level: item.item_level,
                            is_two_hand: item.is_two_hand,
                        });
                    }
                }
            }
        }
        consume_weapon_slots(pool);

        let is_unidentified = set_items.iter().all(|_| true); // simplified: track per-item
        sets.push(RecipeSet { items: set_items, is_unidentified });
    }
    sets
}

/// Find which slots are still needed to complete the next set.
fn find_missing_slots(
    chaos_pool: &HashMap<EquipSlot, Vec<&RecipeCandidate>>,
    regal_pool: &HashMap<EquipSlot, Vec<&RecipeCandidate>>,
) -> Vec<String> {
    let mut combined: HashMap<EquipSlot, usize> = HashMap::new();
    for pool in [chaos_pool, regal_pool] {
        for (slot, items) in pool {
            *combined.entry(*slot).or_insert(0) += items.len();
        }
    }

    let mut missing = Vec::new();
    let needed: &[(EquipSlot, usize, &str)] = &[
        (EquipSlot::Helmet,  1, "Helmet"),
        (EquipSlot::Chest,   1, "BodyArmour"),
        (EquipSlot::Gloves,  1, "Gloves"),
        (EquipSlot::Boots,   1, "Boots"),
        (EquipSlot::Belt,    1, "Belt"),
        (EquipSlot::Amulet,  1, "Amulet"),
        (EquipSlot::Ring,    2, "Ring"),
    ];
    for (slot, required, label) in needed {
        let have = *combined.get(slot).unwrap_or(&0);
        if have < *required {
            missing.push(label.to_string());
        }
    }

    // Weapon check — use raw counts from combined map
    let weapons = *combined.get(&EquipSlot::Weapon).unwrap_or(&0);
    let shields = *combined.get(&EquipSlot::Shield).unwrap_or(&0);
    // Simplified: at least 2 weapon-slots needed (1H+Shield or 2H or two 1H)
    if weapons + shields < 2 {
        missing.push("Weapon".to_string());
    }
    missing
}

// ─── Quality recipes ──────────────────────────────────────────────────────────

fn detect_quality_recipes(items: &[RecipeCandidate]) -> Vec<QualityRecipe> {
    let mut flask_qual: u32 = 0;
    let mut gem_qual:   u32 = 0;
    let mut map_qual:   u32 = 0;

    for item in items {
        match item.item_class.to_lowercase().as_str() {
            "flask" => flask_qual += item.quality,
            "gem"   => gem_qual   += item.quality,
            "map"   => map_qual   += item.quality,
            _ => {}
        }
    }

    let mut recipes = Vec::new();
    if flask_qual >= 40 {
        recipes.push(QualityRecipe {
            output: "Glassblower's Bauble".to_string(),
            count: flask_qual / 40,
        });
    }
    if gem_qual >= 40 {
        recipes.push(QualityRecipe {
            output: "Gemcutter's Prism".to_string(),
            count: gem_qual / 40,
        });
    }
    if map_qual >= 40 {
        recipes.push(QualityRecipe {
            output: "Cartographer's Chisel".to_string(),
            count: map_qual / 40,
        });
    }
    recipes
}

// ─── Main entry point ─────────────────────────────────────────────────────────

pub fn detect_recipes(items: &[RecipeCandidate]) -> RecipeAnalysis {
    let mut chaos_pool: HashMap<EquipSlot, Vec<&RecipeCandidate>> = HashMap::new();
    let mut regal_pool: HashMap<EquipSlot, Vec<&RecipeCandidate>> = HashMap::new();

    for item in items {
        if !item.is_rare { continue; }
        let Some(slot) = infer_slot(item) else { continue };
        if item.item_level >= 75 {
            regal_pool.entry(slot).or_default().push(item);
        } else if item.item_level >= 60 {
            chaos_pool.entry(slot).or_default().push(item);
        }
    }

    let missing_slots = find_missing_slots(&chaos_pool, &regal_pool);
    let quality_recipes = detect_quality_recipes(items);

    let chaos_sets = build_sets(&mut chaos_pool);
    let regal_sets = build_sets(&mut regal_pool);
    let chaos_set_count = chaos_sets.len() as u32;
    let regal_set_count = regal_sets.len() as u32;

    RecipeAnalysis {
        chaos_sets,
        regal_sets,
        chaos_set_count,
        regal_set_count,
        missing_slots,
        quality_recipes,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn rare(id: &str, class: &str, ilvl: u32) -> RecipeCandidate {
        RecipeCandidate {
            id: id.to_string(),
            name: format!("Rare {class}"),
            base_type: class.to_string(),
            item_level: ilvl,
            quality: 0,
            is_rare: true,
            is_identified: false,
            is_two_hand: false,
            item_class: class.to_string(),
        }
    }

    fn full_chaos_set() -> Vec<RecipeCandidate> {
        vec![
            rare("h", "Helmet", 65),
            rare("c", "BodyArmour", 65),
            rare("g", "Gloves", 65),
            rare("b", "Boots", 65),
            rare("be", "Belt", 65),
            rare("a", "Amulet", 65),
            rare("r1", "Ring", 65),
            rare("r2", "Ring", 65),
            rare("w", "Weapon", 65),
            rare("s", "Shield", 65),
        ]
    }

    // ── Slot inference ────────────────────────────────────────────────────────

    #[test]
    fn helmet_infers_correctly() {
        let item = rare("x", "Helmet", 65);
        assert_eq!(infer_slot(&item), Some(EquipSlot::Helmet));
    }

    #[test]
    fn ring_infers_correctly() {
        let item = rare("x", "Ring", 65);
        assert_eq!(infer_slot(&item), Some(EquipSlot::Ring));
    }

    #[test]
    fn unknown_class_returns_none() {
        let item = rare("x", "Currency", 65);
        assert_eq!(infer_slot(&item), None);
    }

    // ── detect_recipes ────────────────────────────────────────────────────────

    #[test]
    fn full_set_ilvl_65_makes_one_chaos_set() {
        let items = full_chaos_set();
        let result = detect_recipes(&items);
        assert_eq!(result.chaos_set_count, 1, "one full chaos set expected");
        assert_eq!(result.regal_set_count, 0);
    }

    #[test]
    fn full_set_ilvl_80_makes_one_regal_set() {
        let items: Vec<RecipeCandidate> = full_chaos_set()
            .into_iter()
            .map(|mut i| { i.item_level = 80; i })
            .collect();
        let result = detect_recipes(&items);
        assert_eq!(result.regal_set_count, 1, "one full regal set expected");
        assert_eq!(result.chaos_set_count, 0);
    }

    #[test]
    fn two_full_sets_detected() {
        let mut items = full_chaos_set();
        items.extend(full_chaos_set().into_iter().map(|mut i| { i.id.push('X'); i }));
        let result = detect_recipes(&items);
        assert_eq!(result.chaos_set_count, 2);
    }

    #[test]
    fn incomplete_set_returns_zero_sets() {
        // Missing boots
        let items: Vec<RecipeCandidate> = full_chaos_set()
            .into_iter()
            .filter(|i| i.item_class != "Boots")
            .collect();
        let result = detect_recipes(&items);
        assert_eq!(result.chaos_set_count, 0);
    }

    #[test]
    fn missing_slots_lists_boots_when_absent() {
        let items: Vec<RecipeCandidate> = full_chaos_set()
            .into_iter()
            .filter(|i| i.item_class != "Boots")
            .collect();
        let result = detect_recipes(&items);
        assert!(result.missing_slots.iter().any(|s| s == "Boots"),
            "missing_slots should include Boots");
    }

    #[test]
    fn non_rare_items_ignored() {
        let mut item = rare("x", "Helmet", 65);
        item.is_rare = false;
        let result = detect_recipes(&[item]);
        assert_eq!(result.chaos_set_count, 0);
        assert_eq!(result.regal_set_count, 0);
    }

    #[test]
    fn low_ilvl_items_ignored() {
        // ilvl 59 — below 60 threshold
        let items: Vec<RecipeCandidate> = full_chaos_set()
            .into_iter()
            .map(|mut i| { i.item_level = 59; i })
            .collect();
        let result = detect_recipes(&items);
        assert_eq!(result.chaos_set_count, 0);
        assert_eq!(result.regal_set_count, 0);
    }

    // ── Quality recipes ───────────────────────────────────────────────────────

    #[test]
    fn four_flasks_10pct_quality_each_makes_one_bauble() {
        let flasks: Vec<RecipeCandidate> = (0..4).map(|i| {
            let mut f = rare(&i.to_string(), "Flask", 65);
            f.is_identified = true; // quality items can be identified
            f.quality = 10;
            f
        }).collect();
        let result = detect_recipes(&flasks);
        assert_eq!(result.quality_recipes.len(), 1);
        assert_eq!(result.quality_recipes[0].output, "Glassblower's Bauble");
        assert_eq!(result.quality_recipes[0].count, 1);
    }

    #[test]
    fn insufficient_quality_yields_no_recipe() {
        let flasks: Vec<RecipeCandidate> = (0..3).map(|i| {
            let mut f = rare(&i.to_string(), "Flask", 65);
            f.quality = 10; // 30 total < 40
            f
        }).collect();
        let result = detect_recipes(&flasks);
        assert!(result.quality_recipes.is_empty());
    }

    #[test]
    fn gem_quality_recipe_detected() {
        let gems: Vec<RecipeCandidate> = (0..2).map(|i| {
            let mut g = rare(&i.to_string(), "Gem", 65);
            g.quality = 20; // 40 total → 1 GCP
            g
        }).collect();
        let result = detect_recipes(&gems);
        let gcp = result.quality_recipes.iter().find(|r| r.output == "Gemcutter's Prism");
        assert!(gcp.is_some());
    }
}

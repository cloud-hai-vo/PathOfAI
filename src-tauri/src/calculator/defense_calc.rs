/// Defense calculator — computes all defensive stats from a BuildData.
/// See ALGORITHMS.md Algorithm 14 (Effective HP) and Algorithm 5 (Resistances).
use crate::models::build::BuildData;
use crate::models::analysis::*;
use super::formulas;

/// Standard hit size for armour reduction reference (5,000 damage hit).
const REFERENCE_HIT: f64 = 5_000.0;
/// Monster accuracy baseline for evasion calculation.
const MONSTER_ACCURACY: f64 = 2_000.0;

pub fn calculate(build: &BuildData) -> DefenseStats {
    let life = calculate_life(build);
    let energy_shield = calculate_es(build);
    let mana = calculate_mana(build);
    let resistances = calculate_resistances(build);
    let armour = calculate_armour(build);
    let armour_phys_reduction = formulas::armour_phys_reduction(armour as f64, REFERENCE_HIT);
    let evasion = calculate_evasion(build);
    let evasion_chance = formulas::evasion_chance(evasion as f64, MONSTER_ACCURACY);
    let (block_chance, spell_block_chance) = calculate_block(build);
    let (life_regen_flat, life_regen_pct) = calculate_life_regen(build);
    let ailment_immunity = calculate_ailment_immunity(build);

    let effective_hp = calculate_effective_hp(
        life, energy_shield, &resistances, armour_phys_reduction,
    );

    DefenseStats {
        life,
        energy_shield,
        mana,
        life_regen_flat,
        life_regen_pct,
        resistances,
        armour,
        armour_phys_reduction,
        evasion,
        evasion_chance,
        block_chance,
        spell_block_chance,
        effective_hp,
        ailment_immunity,
    }
}

/// Per-class base life at level 1.
/// Source: PoE wiki — Class#Base_stats
pub(crate) fn class_base_life(class_name: &str) -> f64 {
    match class_name.to_lowercase().as_str() {
        "marauder"   => 63.0,
        "duelist"    => 53.0,
        "ranger"     => 53.0,
        "shadow"     => 38.0,
        "witch"      => 50.0,
        "templar"    => 55.0,
        "scion"      => 57.0,
        // Ascendancies — inherit from base class
        "juggernaut" | "berserker" | "chieftain" => 63.0,  // Marauder
        "slayer" | "gladiator" | "champion"      => 53.0,  // Duelist
        "deadeye" | "pathfinder" | "raider"      => 53.0,  // Ranger
        "assassin" | "saboteur" | "trickster"    => 38.0,  // Shadow
        "elementalist" | "occultist" | "necromancer" => 50.0, // Witch
        "inquisitor" | "hierophant" | "guardian" => 55.0,  // Templar
        "ascendant"  => 57.0,                               // Scion
        _            => 50.0,   // unknown — use reasonable default
    }
}

fn calculate_life(build: &BuildData) -> u32 {
    let mut base: f64 = class_base_life(&build.class_name);
    let mut flat_life: f64 = 0.0;
    let mut pct_increased: f64 = 0.0;

    // Base life scaling: +12 per level after 1 (same for all classes)
    base += (build.level.saturating_sub(1) as f64) * 12.0;

    // Aggregate mods from all items
    for item in &build.items {
        for mod_ in &item.mods {
            let text = mod_.text.to_lowercase();
            if text.contains("to maximum life") {
                flat_life += parse_flat_value(&mod_.text);
            } else if text.contains("increased maximum life") {
                pct_increased += parse_pct_value(&mod_.text);
            }
        }
    }

    // TODO: passive tree nodes (requires tree data loaded — stub for now)
    // TODO: aura effects, flasks

    ((base + flat_life) * (1.0 + pct_increased / 100.0)) as u32
}

fn calculate_es(build: &BuildData) -> u32 {
    let mut flat_es: f64 = 0.0;
    let mut pct_increased: f64 = 0.0;

    for item in &build.items {
        // Base ES from item itself (approximated from item type — proper impl needs base data)
        for mod_ in &item.mods {
            let text = mod_.text.to_lowercase();
            if text.contains("to maximum energy shield") {
                flat_es += parse_flat_value(&mod_.text);
            } else if text.contains("increased maximum energy shield") {
                pct_increased += parse_pct_value(&mod_.text);
            }
        }
    }

    (flat_es * (1.0 + pct_increased / 100.0)) as u32
}

fn calculate_mana(build: &BuildData) -> u32 {
    let mut base: f64 = 34.0;
    let mut flat: f64 = 0.0;
    let mut pct: f64 = 0.0;

    base += (build.level.saturating_sub(1) as f64) * 6.0;

    for item in &build.items {
        for mod_ in &item.mods {
            let text = mod_.text.to_lowercase();
            if text.contains("to maximum mana") { flat += parse_flat_value(&mod_.text); }
            else if text.contains("increased maximum mana") { pct += parse_pct_value(&mod_.text); }
        }
    }

    ((base + flat) * (1.0 + pct / 100.0)) as u32
}

fn calculate_resistances(build: &BuildData) -> ResistanceProfile {
    let mut fire: i32 = 0;
    let mut cold: i32 = 0;
    let mut lightning: i32 = 0;
    let mut chaos: i32 = -60; // default chaos res is -60%
    let mut max_fire: i32 = 75;
    let mut max_cold: i32 = 75;
    let mut max_lightning: i32 = 75;
    let max_chaos: i32 = 75;

    for item in &build.items {
        for mod_ in &item.mods {
            let text = mod_.text.to_lowercase();
            let val = parse_res_value(&mod_.text);
            if text.contains("to fire resistance") { fire += val; }
            else if text.contains("to cold resistance") { cold += val; }
            else if text.contains("to lightning resistance") { lightning += val; }
            else if text.contains("to chaos resistance") { chaos += val; }
            else if text.contains("to all elemental resistances") || text.contains("to all resistances") {
                fire += val; cold += val; lightning += val;
            }
            else if text.contains("to maximum fire resistance") { max_fire += val; }
            else if text.contains("to maximum cold resistance") { max_cold += val; }
            else if text.contains("to maximum lightning resistance") { max_lightning += val; }
        }
    }

    // Class base resistances (Templar gets +0 by default, varies by class)
    // TODO: load per-class base stats from game data

    let fire_overcap = (fire - max_fire).max(0);
    let cold_overcap = (cold - max_cold).max(0);
    let lightning_overcap = (lightning - max_lightning).max(0);

    ResistanceProfile {
        fire: fire.min(max_fire),
        cold: cold.min(max_cold),
        lightning: lightning.min(max_lightning),
        chaos: chaos.min(max_chaos),
        max_fire,
        max_cold,
        max_lightning,
        max_chaos,
        fire_overcap,
        cold_overcap,
        lightning_overcap,
    }
}

fn calculate_armour(build: &BuildData) -> u32 {
    let mut flat: f64 = 0.0;
    let mut pct: f64 = 0.0;

    for item in &build.items {
        // Base armour from item body (requires base data — approximated)
        for mod_ in &item.mods {
            let text = mod_.text.to_lowercase();
            if text.contains("to armour") && !text.contains("evasion") {
                flat += parse_flat_value(&mod_.text);
            } else if text.contains("increased armour") {
                pct += parse_pct_value(&mod_.text);
            } else if text.contains("increased armour and evasion") {
                pct += parse_pct_value(&mod_.text) * 0.5; // split
            }
        }
    }

    (flat * (1.0 + pct / 100.0)) as u32
}

fn calculate_evasion(build: &BuildData) -> u32 {
    let mut flat: f64 = 0.0;
    let mut pct: f64 = 0.0;

    for item in &build.items {
        for mod_ in &item.mods {
            let text = mod_.text.to_lowercase();
            if text.contains("to evasion rating") {
                flat += parse_flat_value(&mod_.text);
            } else if text.contains("increased evasion rating") {
                pct += parse_pct_value(&mod_.text);
            }
        }
    }

    (flat * (1.0 + pct / 100.0)) as u32
}

fn calculate_block(build: &BuildData) -> (f64, f64) {
    let mut block: f64 = 0.0;
    let mut spell_block: f64 = 0.0;

    for item in &build.items {
        for mod_ in &item.mods {
            let text = mod_.text.to_lowercase();
            if text.contains("chance to block attack damage") {
                block += parse_pct_value(&mod_.text);
            }
            if text.contains("chance to block spell damage") {
                spell_block += parse_pct_value(&mod_.text);
            }
        }
        // Check for shield base block (requires base item data)
        // TODO: load base block chance for shields
    }

    (block.min(75.0) / 100.0, spell_block.min(75.0) / 100.0)
}

fn calculate_life_regen(build: &BuildData) -> (f64, f64) {
    let mut flat_regen: f64 = 0.0;
    let mut pct_regen: f64 = 0.0;

    for item in &build.items {
        for mod_ in &item.mods {
            let text = mod_.text.to_lowercase();
            if text.contains("life regenerated per second") {
                flat_regen += parse_flat_value(&mod_.text);
            } else if text.contains("of life regenerated per second") {
                pct_regen += parse_pct_value(&mod_.text);
            }
        }
    }

    (flat_regen, pct_regen)
}

fn calculate_ailment_immunity(build: &BuildData) -> AilmentImmunity {
    let mut immunity = AilmentImmunity::default();

    for item in &build.items {
        let item_name = item.name.to_lowercase();
        for mod_ in &item.mods {
            let text = mod_.text.to_lowercase();
            check_ailment_source(&mut immunity, &text, &item.name);
        }
        // Unique item name checks
        if item_name.contains("atziri's step") {
            immunity.freeze = true;
            immunity.freeze_source = Some(item.name.clone());
        }
        if item_name.contains("dream fragments") {
            immunity.freeze = true;
            immunity.freeze_source = Some(item.name.clone());
        }
    }

    immunity
}

fn check_ailment_source(immunity: &mut AilmentImmunity, text: &str, source: &str) {
    if text.contains("cannot be frozen") {
        immunity.freeze = true;
        immunity.freeze_source = Some(source.to_string());
    }
    if text.contains("cannot be shocked") {
        immunity.shock = true;
        immunity.shock_source = Some(source.to_string());
    }
    if text.contains("cannot be ignited") {
        immunity.ignite = true;
        immunity.ignite_source = Some(source.to_string());
    }
    if text.contains("cannot be bleeded") || text.contains("immunity to bleeding") {
        immunity.bleed = true;
        immunity.bleed_source = Some(source.to_string());
    }
    if text.contains("corrupted blood cannot be inflicted") {
        immunity.corrupted_blood = true;
        immunity.corrupted_blood_source = Some(source.to_string());
    }
    if text.contains("cannot be cursed") {
        immunity.curse_immune = true;
    }
}

fn calculate_effective_hp(
    life: u32,
    es: u32,
    res: &ResistanceProfile,
    armour_reduction: f64,
) -> EffectiveHP {
    let phys_reduction = formulas::armour_phys_reduction(
        calculate_armour_from_reduction(armour_reduction),
        REFERENCE_HIT,
    );

    let hp_pool = life + es;

    EffectiveHP {
        vs_physical: formulas::effective_hp(hp_pool as f64, phys_reduction) as u32,
        vs_elemental: formulas::effective_hp(
            hp_pool as f64,
            formulas::effective_resistance(res.fire, res.max_fire) as f64 / 100.0,
        ) as u32,
        vs_chaos: formulas::effective_hp(
            hp_pool as f64,
            formulas::effective_resistance(res.chaos, res.max_chaos).max(0) as f64 / 100.0,
        ) as u32,
    }
}

fn calculate_armour_from_reduction(reduction: f64) -> f64 {
    // Invert the armour formula to get armour from reduction
    // A = (reduction / (1 - reduction)) * 10 * REFERENCE_HIT
    if reduction >= 1.0 { return f64::MAX; }
    (reduction / (1.0 - reduction)) * 10.0 * REFERENCE_HIT
}

// ─── Parsing helpers ──────────────────────────────────────────────────────────

/// Parse the first integer from a mod text string.
/// "+52 to maximum Life" → 52
fn parse_flat_value(text: &str) -> f64 {
    text.split_whitespace()
        .find_map(|w| w.trim_start_matches('+').trim_end_matches('%').parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// Parse a percentage value from mod text.
/// "30% increased maximum Life" → 30.0
fn parse_pct_value(text: &str) -> f64 {
    parse_flat_value(text)
}

/// Parse resistance value, handles both flat (+30%) and signed.
fn parse_res_value(text: &str) -> i32 {
    parse_flat_value(text) as i32
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_flat_life_mod() {
        assert_eq!(parse_flat_value("+52 to maximum Life"), 52.0);
    }

    #[test]
    fn parse_pct_res() {
        assert_eq!(parse_pct_value("30% to Fire Resistance"), 30.0);
    }

    #[test]
    fn class_base_life_marauder_highest() {
        assert_eq!(class_base_life("Marauder"), 63.0);
    }

    #[test]
    fn class_base_life_shadow_lowest() {
        assert_eq!(class_base_life("Shadow"), 38.0);
    }

    #[test]
    fn class_base_life_ascendancy_inherits_from_base() {
        assert_eq!(class_base_life("Inquisitor"), class_base_life("Templar"));
        assert_eq!(class_base_life("Juggernaut"), class_base_life("Marauder"));
        assert_eq!(class_base_life("Deadeye"),    class_base_life("Ranger"));
    }

    #[test]
    fn class_base_life_case_insensitive() {
        assert_eq!(class_base_life("TEMPLAR"), class_base_life("templar"));
    }

    #[test]
    fn class_base_life_unknown_returns_default() {
        assert!(class_base_life("Unknown") > 0.0);
    }

    #[test]
    fn templar_at_90_has_more_base_life_than_shadow() {
        let mut templar = BuildData::default();
        templar.class_name = "Templar".to_string();
        templar.level = 90;
        let mut shadow = BuildData::default();
        shadow.class_name = "Shadow".to_string();
        shadow.level = 90;
        let t_life = calculate_life(&templar);
        let s_life = calculate_life(&shadow);
        assert!(t_life > s_life, "Templar should have more base life than Shadow at 90");
    }

    #[test]
    fn chaos_res_starts_negative() {
        let build = BuildData::default();
        let res = calculate_resistances(&build);
        assert_eq!(res.chaos, -60); // no gear → -60%
    }

    #[test]
    fn armour_reduction_formula_correct() {
        use crate::calculator::formulas::armour_phys_reduction;
        // 10,000 armour vs 1,000 hit = 50% reduction
        let r = armour_phys_reduction(10_000.0, 1_000.0);
        assert!((r - 0.5).abs() < 0.001);
    }
}

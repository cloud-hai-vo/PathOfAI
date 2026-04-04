/// Build archetype detector — rule-based, no ML.
/// See ALGORITHMS.md for the classification rules.
use crate::models::build::BuildData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Archetype {
    FireDoT,        // RF, Burning Arrow, Ignite
    ColdDoT,        // Vortex, Cold Snap, Frost Blades DoT
    LightningDoT,   // Arc, Ball Lightning
    PhysDoT,        // Lacerate, Puncture bleed
    PoisonDoT,      // Caustic Arrow, Cobra Lash
    HitAttack,      // Generic attack (non-DoT)
    HitSpell,       // Generic spell (non-DoT)
    Minion,         // Raise Spectre, Animate Guardian, SRS
    Totem,          // Any totem build
    Trap,           // Trap skills
    Mine,           // Mine skills
    Unknown,
}

impl Archetype {
    pub fn id(&self) -> &'static str {
        match self {
            Archetype::FireDoT => "fire_dot",
            Archetype::ColdDoT => "cold_dot",
            Archetype::LightningDoT => "lightning_dot",
            Archetype::PhysDoT => "phys_dot",
            Archetype::PoisonDoT => "poison_dot",
            Archetype::HitAttack => "hit_attack",
            Archetype::HitSpell => "hit_spell",
            Archetype::Minion => "minion",
            Archetype::Totem => "totem",
            Archetype::Trap => "trap",
            Archetype::Mine => "mine",
            Archetype::Unknown => "unknown",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Archetype::FireDoT => "Fire DoT",
            Archetype::ColdDoT => "Cold DoT",
            Archetype::LightningDoT => "Lightning DoT",
            Archetype::PhysDoT => "Phys DoT / Bleed",
            Archetype::PoisonDoT => "Poison DoT",
            Archetype::HitAttack => "Attack",
            Archetype::HitSpell => "Spell",
            Archetype::Minion => "Minion",
            Archetype::Totem => "Totem",
            Archetype::Trap => "Trap",
            Archetype::Mine => "Mine",
            Archetype::Unknown => "Unknown",
        }
    }

    /// Stat weights for item scoring — which stats matter most for this archetype.
    /// Returns a map of stat_id → weight (0.0 to 1.0).
    pub fn stat_weights(&self) -> &'static [(&'static str, f64)] {
        match self {
            Archetype::FireDoT => &[
                ("life", 1.0),
                ("fire_res", 0.9),
                ("fire_dot_multi", 1.0),
                ("burning_damage", 0.9),
                ("all_res", 0.8),
                ("chaos_res", 0.7),
                ("regen", 0.8),
                ("armour", 0.6),
            ],
            Archetype::HitAttack => &[
                ("life", 1.0),
                ("phys_damage", 0.9),
                ("attack_speed", 0.8),
                ("crit_chance", 0.7),
                ("crit_multi", 0.7),
                ("all_res", 0.8),
            ],
            Archetype::Minion => &[
                ("life", 0.8),
                ("minion_damage", 1.0),
                ("minion_life", 0.7),
                ("all_res", 0.8),
                ("chaos_res", 0.7),
            ],
            // Default weights for other archetypes
            _ => &[
                ("life", 1.0),
                ("all_res", 0.8),
                ("damage", 0.7),
            ],
        }
    }
}

/// Detect build archetype from equipped items, gems, and passive tree.
/// Rule-based — no ML. Port of prototypes/build-detector.js.
pub fn detect_archetype(build: &BuildData) -> Archetype {
    let main_skill = build.gems
        .iter()
        .find(|g| g.is_main_skill)
        .or_else(|| build.gems.first());

    let skill_name = main_skill
        .map(|g| g.skill.to_lowercase())
        .unwrap_or_default();

    // Check for DoT-specific skills first (most specific)
    if is_fire_dot(&skill_name) { return Archetype::FireDoT; }
    if is_cold_dot(&skill_name) { return Archetype::ColdDoT; }
    if is_poison_dot(&skill_name, build) { return Archetype::PoisonDoT; }
    if is_phys_dot(&skill_name) { return Archetype::PhysDoT; }

    // Playstyle-based detection
    if has_support(build, "trap") { return Archetype::Trap; }
    if has_support(build, "remote mine") || has_support(build, "blastchain mine") {
        return Archetype::Mine;
    }
    if has_support(build, "spell totem") || has_support(build, "ballista totem") {
        return Archetype::Totem;
    }

    // Minion detection from skill name
    if skill_name.contains("spectre") || skill_name.contains("zombie")
        || skill_name.contains("skeleton") || skill_name.contains("animate")
        || skill_name.contains("summon") {
        return Archetype::Minion;
    }

    // Attack vs spell (generic)
    if main_skill.map(|g| g.gems.iter().any(|gem| !gem.is_support && !is_spell(&gem.name))).unwrap_or(false) {
        return Archetype::HitAttack;
    }

    Archetype::HitSpell
}

fn is_fire_dot(skill: &str) -> bool {
    ["righteous fire", "burning arrow", "scorching ray",
     "flameblast", "ignite proliferation"].iter()
        .any(|s| skill.contains(s))
}

fn is_cold_dot(skill: &str) -> bool {
    ["vortex", "cold snap", "frost blades", "glacial cascade",
     "creeping frost"].iter()
        .any(|s| skill.contains(s))
}

fn is_phys_dot(skill: &str) -> bool {
    ["lacerate", "puncture", "flicker strike", "exsanguinate"].iter()
        .any(|s| skill.contains(s))
}

fn is_poison_dot(skill: &str, build: &BuildData) -> bool {
    ["caustic arrow", "cobra lash", "viper strike"].iter()
        .any(|s| skill.contains(s))
    || (has_support(build, "void manipulation") && skill.contains("concoction"))
}

fn has_support(build: &BuildData, support_name: &str) -> bool {
    build.gems.iter().any(|setup| {
        setup.gems.iter().any(|g| {
            g.is_support && g.name.to_lowercase().contains(support_name)
        })
    })
}

fn is_spell(skill: &str) -> bool {
    // Heuristic — spells don't have attack speed scaling
    let attack_skills = ["blade flurry", "cyclone", "lacerate", "reave",
                          "split arrow", "rain of arrows", "ice shot"];
    !attack_skills.iter().any(|s| skill.to_lowercase().contains(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rf_as_fire_dot() {
        let mut build = BuildData::default();
        build.gems.push(crate::models::build::GemSetup {
            skill: "Righteous Fire".to_string(),
            slot: "BodyArmour".to_string(),
            socket_colors: "RRRRRR".to_string(),
            gems: vec![],
            is_main_skill: true,
        });
        assert_eq!(detect_archetype(&build), Archetype::FireDoT);
    }
}

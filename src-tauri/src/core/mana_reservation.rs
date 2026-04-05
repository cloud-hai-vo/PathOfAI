/// mana_reservation.rs — Mana Reservation Engine (Algorithm 28).
///
/// Computes how much mana / life is reserved by auras and skills,
/// how much free mana remains, and whether the build is over-reserved.
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationSkill {
    pub name:                String,
    /// Base reservation amount. For % skills this is the percentage (e.g. 35.0 for 35%).
    /// For flat skills this is the mana cost (e.g. 50.0 for 50 flat mana).
    pub base_reservation:    f64,
    pub is_percentage:       bool,
    pub tags:                Vec<String>, // "aura", "herald", "banner", "curse", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerReservationStats {
    pub max_mana:                    u32,
    pub max_es:                      u32,
    /// Default 100. Sovereignty cluster adds +8 each. More = better.
    pub reservation_efficiency:      f64,
    /// Tag-specific increases (e.g. Hex Master on curse auras). Applied after efficiency.
    pub increased_mana_reservation:  f64,
    /// Reduced mana reservation (rare, from specific items). Applied after increases.
    pub reduced_mana_reservation:    f64,
    /// Mana cost of the main skill (used to check over-reservation).
    pub main_skill_mana_cost:        f64,
    /// If true, ES pool is added to the effective mana pool for reservation checks.
    pub has_eldritch_battery:        bool,
}

impl Default for PlayerReservationStats {
    fn default() -> Self {
        Self {
            max_mana: 1000,
            max_es: 0,
            reservation_efficiency: 100.0,
            increased_mana_reservation: 0.0,
            reduced_mana_reservation: 0.0,
            main_skill_mana_cost: 10.0,
            has_eldritch_battery: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillReservationDetail {
    pub name:                  String,
    pub base_reservation:      f64,
    pub effective_reservation: f64, // actual mana/es spent after efficiency
    pub is_percentage:         bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationResult {
    pub skills:                  Vec<SkillReservationDetail>,
    pub total_reserved:          f64,
    pub free_mana:               f64,
    pub effective_pool:          f64, // max_mana (+ max_es if Eldritch Battery)
    pub over_reserved:           bool,
    pub reservation_pct_of_pool: f64, // 0–100
}

// ─── Algorithm ────────────────────────────────────────────────────────────────

/// Calculate mana reservation for all skills in the build.
pub fn calculate_reservation(
    skills: &[ReservationSkill],
    player: &PlayerReservationStats,
) -> ReservationResult {
    let eff = player.reservation_efficiency.max(1.0); // guard divide-by-zero
    let max_mana = player.max_mana as f64;

    let skill_details: Vec<SkillReservationDetail> = skills.iter().map(|s| {
        let effective = if s.is_percentage {
            // Step 1: Apply reservation efficiency (divides the % cost)
            let pct_after_eff = s.base_reservation / (eff / 100.0);

            // Step 2: Apply increased/reduced mana reservation
            let increased = player.increased_mana_reservation;
            let reduced   = player.reduced_mana_reservation;
            let pct_final = pct_after_eff
                * (1.0 + increased / 100.0)
                * (1.0 - reduced  / 100.0);

            // Step 3: Convert to flat mana, rounding UP (PoE always rounds up)
            (max_mana * pct_final / 100.0).ceil()
        } else {
            // Flat reservations are unaffected by efficiency or % modifiers
            s.base_reservation
        };

        SkillReservationDetail {
            name:                  s.name.clone(),
            base_reservation:      s.base_reservation,
            effective_reservation: effective,
            is_percentage:         s.is_percentage,
        }
    }).collect();

    let total_reserved: f64 = skill_details.iter().map(|s| s.effective_reservation).sum();

    // Eldritch Battery adds ES to the effective pool
    let effective_pool = if player.has_eldritch_battery {
        max_mana + player.max_es as f64
    } else {
        max_mana
    };

    let free_mana = effective_pool - total_reserved;
    let over_reserved = free_mana < player.main_skill_mana_cost;
    let reservation_pct_of_pool = (total_reserved / effective_pool.max(1.0) * 100.0)
        .min(100.0);

    ReservationResult {
        skills: skill_details,
        total_reserved,
        free_mana,
        effective_pool,
        over_reserved,
        reservation_pct_of_pool,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn player(max_mana: u32) -> PlayerReservationStats {
        PlayerReservationStats {
            max_mana,
            ..Default::default()
        }
    }

    fn pct_skill(name: &str, pct: f64) -> ReservationSkill {
        ReservationSkill {
            name: name.to_string(),
            base_reservation: pct,
            is_percentage: true,
            tags: vec!["aura".to_string()],
        }
    }

    fn flat_skill(name: &str, cost: f64) -> ReservationSkill {
        ReservationSkill {
            name: name.to_string(),
            base_reservation: cost,
            is_percentage: false,
            tags: vec!["herald".to_string()],
        }
    }

    // ── Basic reservation ─────────────────────────────────────────────────��───

    #[test]
    fn no_skills_means_full_mana_free() {
        let result = calculate_reservation(&[], &player(1000));
        assert_eq!(result.total_reserved, 0.0);
        assert_eq!(result.free_mana, 1000.0);
        assert!(!result.over_reserved);
    }

    #[test]
    fn single_35pct_aura_on_1000_mana() {
        // 35% of 1000 = 350 (exact, no rounding needed)
        let result = calculate_reservation(&[pct_skill("Determination", 35.0)], &player(1000));
        assert_eq!(result.total_reserved, 350.0);
        assert_eq!(result.free_mana, 650.0);
    }

    #[test]
    fn flat_reservation_unaffected_by_efficiency() {
        let mut p = player(1000);
        p.reservation_efficiency = 150.0; // high efficiency
        let result = calculate_reservation(&[flat_skill("Enlighten Herald", 50.0)], &p);
        // Flat cost always exactly the base value
        assert_eq!(result.total_reserved, 50.0);
    }

    #[test]
    fn multiple_auras_sum_correctly() {
        // Determination (35%) + Anger (35%) + Purity of Fire (35%) = 105% of 1000 = 1050
        let skills = vec![
            pct_skill("Determination", 35.0),
            pct_skill("Anger", 35.0),
            pct_skill("Purity of Fire", 35.0),
        ];
        let result = calculate_reservation(&skills, &player(1000));
        assert_eq!(result.total_reserved, 1050.0);
        assert!(result.over_reserved, "three 35% auras should over-reserve 1000 mana");
    }

    // ── Efficiency ────────────────────────────────────────────────────────────

    #[test]
    fn sovereignty_cluster_reduces_cost() {
        // 120% efficiency: 35% / 1.2 = 29.17% → ceil(291.7) = 292
        let mut p = player(1000);
        p.reservation_efficiency = 120.0;
        let result = calculate_reservation(&[pct_skill("Determination", 35.0)], &p);
        // 1000 * (35 / 1.2) / 100 = 291.666... → ceil = 292
        assert_eq!(result.total_reserved, 292.0);
        assert!(result.total_reserved < 350.0, "efficiency should reduce cost");
    }

    #[test]
    fn reduced_efficiency_increases_cost() {
        // 50% efficiency: 35% / 0.5 = 70% → 700 mana
        let mut p = player(1000);
        p.reservation_efficiency = 50.0;
        let result = calculate_reservation(&[pct_skill("Anger", 35.0)], &p);
        assert_eq!(result.total_reserved, 700.0);
    }

    // ── Increased mana reservation ────────────────────────────────────────────

    #[test]
    fn increased_reservation_raises_cost() {
        // Hex Master: +30% increased mana reservation for curses
        // Base 50% curse aura at default efficiency: 500 mana
        // +30% increased: 500 * 1.3 = 650 mana
        let mut p = player(1000);
        p.increased_mana_reservation = 30.0;
        let result = calculate_reservation(&[pct_skill("Vulnerability", 50.0)], &p);
        assert_eq!(result.total_reserved, 650.0);
    }

    #[test]
    fn reduced_reservation_lowers_cost() {
        // -25% reduced reservation: 35% → 35 * (1 - 0.25) = 26.25% → 263 mana
        let mut p = player(1000);
        p.reduced_mana_reservation = 25.0;
        let result = calculate_reservation(&[pct_skill("Determination", 35.0)], &p);
        // 1000 * 35/100 * (1 - 0.25) = 262.5 → ceil = 263
        assert_eq!(result.total_reserved, 263.0);
    }

    // ── Rounding ─────────────────────────────────────────────────────────────

    #[test]
    fn reservation_rounds_up_like_poe() {
        // 37.5% of 1000 = 375.0 (exact)
        // 37.5% of 301 = 112.875 → ceil = 113
        let result = calculate_reservation(&[pct_skill("Grace", 37.5)], &player(301));
        assert_eq!(result.total_reserved, 113.0);
    }

    // ── Over-reservation check ────────────────────────────────────────────────

    #[test]
    fn over_reserved_when_free_mana_less_than_main_skill_cost() {
        let mut p = player(1000);
        p.main_skill_mana_cost = 100.0;
        // Reserve 950 → 50 free, but skill costs 100
        let skills = vec![pct_skill("Determination", 95.0)];
        let result = calculate_reservation(&skills, &p);
        assert!(result.over_reserved);
    }

    #[test]
    fn not_over_reserved_with_enough_free_mana() {
        let mut p = player(1000);
        p.main_skill_mana_cost = 50.0;
        let skills = vec![pct_skill("Determination", 35.0)]; // 350 reserved, 650 free
        let result = calculate_reservation(&skills, &p);
        assert!(!result.over_reserved);
    }

    // ── Eldritch Battery ──────────────────────────────────────────────────────

    #[test]
    fn eldritch_battery_adds_es_to_pool() {
        let mut p = player(100);
        p.max_es = 5000;
        p.has_eldritch_battery = true;
        p.main_skill_mana_cost = 10.0;

        // Without EB: 100 mana. Reserve 35% = 35 mana, free = 65.
        // With EB:    5100 effective pool. Reserve 35% of MANA (35) = 35, free = 5065.
        // Note: EB doesn't change how % skills are calculated (still % of max_mana),
        // but the effective pool for "over reserved" check includes ES.
        let result = calculate_reservation(&[pct_skill("Determination", 35.0)], &p);
        assert_eq!(result.effective_pool, 5100.0);
        assert!(!result.over_reserved);
    }

    // ── Summary stats ─────────────────────────────────────────────────────────

    #[test]
    fn reservation_pct_of_pool_correct() {
        // 35% of 1000 = 350, so 35% of pool reserved
        let result = calculate_reservation(&[pct_skill("Determination", 35.0)], &player(1000));
        assert!((result.reservation_pct_of_pool - 35.0).abs() < 0.1);
    }

    #[test]
    fn skill_details_include_effective_reservation() {
        let result = calculate_reservation(&[pct_skill("Anger", 35.0)], &player(1000));
        assert_eq!(result.skills.len(), 1);
        assert_eq!(result.skills[0].name, "Anger");
        assert_eq!(result.skills[0].effective_reservation, 350.0);
    }

    #[test]
    fn free_mana_never_exceeds_effective_pool() {
        let result = calculate_reservation(&[], &player(500));
        assert!(result.free_mana <= result.effective_pool);
    }

    #[test]
    fn over_reservation_percentage_capped_at_100() {
        // Reserve 2000 out of 1000 mana pool
        let skills = vec![
            pct_skill("A", 100.0),
            pct_skill("B", 100.0),
        ];
        let result = calculate_reservation(&skills, &player(1000));
        assert!(result.reservation_pct_of_pool <= 100.0);
    }
}

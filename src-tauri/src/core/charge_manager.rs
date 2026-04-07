/// charge_manager.rs — Charge Management (Algorithm 31).
///
/// Models Endurance, Frenzy, and Power charges: maximum counts, expiry timers,
/// per-tick decay, and stat bonuses per charge type.
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChargeType {
    Endurance = 0,
    Frenzy    = 1,
    Power     = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargeConfig {
    pub max_endurance:           u8,   // default 3
    pub max_frenzy:              u8,
    pub max_power:               u8,
    /// Seconds each charge type lasts after the last gain (base 10s).
    pub endurance_duration_secs: f64,
    pub frenzy_duration_secs:    f64,
    pub power_duration_secs:     f64,
}

impl Default for ChargeConfig {
    fn default() -> Self {
        Self {
            max_endurance:           3,
            max_frenzy:              3,
            max_power:               3,
            endurance_duration_secs: 10.0,
            frenzy_duration_secs:    10.0,
            power_duration_secs:     10.0,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChargeState {
    pub counts:    [u8; 3],   // [endurance, frenzy, power]
    pub expiry_ms: [u32; 3],  // ms remaining until next charge is lost per type
}

impl ChargeState {
    pub fn endurance(&self) -> u8 { self.counts[0] }
    pub fn frenzy(&self)    -> u8 { self.counts[1] }
    pub fn power(&self)     -> u8 { self.counts[2] }
}

/// Aggregate stat bonuses from the current charge counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChargeBonuses {
    pub physical_damage_reduction_pct: f64, // from endurance
    pub all_elemental_resistances:     f64, // from endurance
    pub increased_attack_speed:        f64, // from frenzy
    pub increased_cast_speed:          f64, // from frenzy
    pub increased_damage:              f64, // from frenzy
    pub increased_crit_chance:         f64, // from power
}

// ─── Per-charge stat bonuses ──────────────────────────────────────────────────

const ENDURANCE_PHYS_REDUCTION_PER: f64 = 4.0;   // 4% phys reduction per charge
const ENDURANCE_ELE_RES_PER:        f64 = 4.0;   // 4% all ele res per charge
const FRENZY_ATTACK_SPEED_PER:      f64 = 4.0;   // 4% increased attack speed per charge
const FRENZY_CAST_SPEED_PER:        f64 = 4.0;   // 4% increased cast speed per charge
const FRENZY_DAMAGE_PER:            f64 = 4.0;   // 4% increased damage per charge
const POWER_CRIT_CHANCE_PER:        f64 = 40.0;  // 40% increased crit chance per charge

// ─── Core functions ───────────────────────────────────────────────────────────

/// Add charges of a given type, respecting max, resetting the expiry timer.
pub fn gain_charge(
    state:  &mut ChargeState,
    config: &ChargeConfig,
    kind:   ChargeType,
    count:  u8,
) {
    let idx = kind as usize;
    let max = [config.max_endurance, config.max_frenzy, config.max_power][idx];
    state.counts[idx] = state.counts[idx].saturating_add(count).min(max);

    // Reset expiry timer to the full duration
    let duration_ms = (duration_secs(kind, config) * 1000.0) as u32;
    state.expiry_ms[idx] = duration_ms;
}

/// Advance charge timers by `dt_ms` milliseconds, decaying charges as they expire.
pub fn tick_charges(state: &mut ChargeState, config: &ChargeConfig, dt_ms: u32) {
    for i in 0..3 {
        if state.counts[i] == 0 { continue; }

        if state.expiry_ms[i] <= dt_ms {
            state.counts[i] -= 1;
            // Reset timer for the next charge in the stack
            let kind = match i { 0 => ChargeType::Endurance, 1 => ChargeType::Frenzy, _ => ChargeType::Power };
            state.expiry_ms[i] = (duration_secs(kind, config) * 1000.0) as u32;
        } else {
            state.expiry_ms[i] -= dt_ms;
        }
    }
}

/// Calculate stat bonuses from current charge counts.
pub fn charge_bonuses(state: &ChargeState) -> ChargeBonuses {
    let e = state.endurance() as f64;
    let f = state.frenzy()    as f64;
    let p = state.power()     as f64;
    ChargeBonuses {
        physical_damage_reduction_pct: e * ENDURANCE_PHYS_REDUCTION_PER,
        all_elemental_resistances:     e * ENDURANCE_ELE_RES_PER,
        increased_attack_speed:        f * FRENZY_ATTACK_SPEED_PER,
        increased_cast_speed:          f * FRENZY_CAST_SPEED_PER,
        increased_damage:              f * FRENZY_DAMAGE_PER,
        increased_crit_chance:         p * POWER_CRIT_CHANCE_PER,
    }
}

fn duration_secs(kind: ChargeType, config: &ChargeConfig) -> f64 {
    match kind {
        ChargeType::Endurance => config.endurance_duration_secs,
        ChargeType::Frenzy    => config.frenzy_duration_secs,
        ChargeType::Power     => config.power_duration_secs,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_state() -> ChargeState { ChargeState::default() }
    fn cfg() -> ChargeConfig { ChargeConfig::default() }

    // ── gain_charge ───────────────────────────────────────────────────────────

    #[test]
    fn gain_endurance_increases_count() {
        let mut s = default_state();
        gain_charge(&mut s, &cfg(), ChargeType::Endurance, 1);
        assert_eq!(s.endurance(), 1);
    }

    #[test]
    fn gain_is_capped_at_max() {
        let mut s = default_state();
        gain_charge(&mut s, &cfg(), ChargeType::Frenzy, 10);
        assert_eq!(s.frenzy(), 3, "capped at max_frenzy = 3");
    }

    #[test]
    fn gain_resets_expiry_timer() {
        let mut s = default_state();
        gain_charge(&mut s, &cfg(), ChargeType::Power, 1);
        assert_eq!(s.expiry_ms[2], 10_000, "10s = 10000ms expiry");
    }

    #[test]
    fn gain_multiple_types_independently() {
        let mut s = default_state();
        gain_charge(&mut s, &cfg(), ChargeType::Endurance, 2);
        gain_charge(&mut s, &cfg(), ChargeType::Power, 1);
        assert_eq!(s.endurance(), 2);
        assert_eq!(s.frenzy(),    0);
        assert_eq!(s.power(),     1);
    }

    #[test]
    fn gain_with_extended_max() {
        let mut c = cfg();
        c.max_endurance = 7;
        let mut s = default_state();
        gain_charge(&mut s, &c, ChargeType::Endurance, 7);
        assert_eq!(s.endurance(), 7);
    }

    // ── tick_charges ──────────────────────────────────────────────────────────

    #[test]
    fn charges_dont_decay_before_expiry() {
        let mut s = default_state();
        gain_charge(&mut s, &cfg(), ChargeType::Endurance, 3);
        tick_charges(&mut s, &cfg(), 5_000); // 5s of 10s elapsed
        assert_eq!(s.endurance(), 3, "should still have 3 after 5s");
    }

    #[test]
    fn one_charge_lost_at_expiry() {
        let mut s = default_state();
        gain_charge(&mut s, &cfg(), ChargeType::Frenzy, 2);
        tick_charges(&mut s, &cfg(), 10_001); // just past 10s
        assert_eq!(s.frenzy(), 1, "one charge expires per interval");
    }

    #[test]
    fn all_charges_eventually_decay() {
        let mut s = default_state();
        gain_charge(&mut s, &cfg(), ChargeType::Power, 3);
        // Tick 3 times — each loses one charge
        for _ in 0..3 {
            tick_charges(&mut s, &cfg(), 10_001);
        }
        assert_eq!(s.power(), 0, "all 3 charges should expire");
    }

    #[test]
    fn no_charges_means_no_decay() {
        let mut s = default_state();
        tick_charges(&mut s, &cfg(), 60_000);
        assert_eq!(s.counts, [0, 0, 0]);
    }

    // ── charge_bonuses ────────────────────────────────────────────────────────

    #[test]
    fn three_endurance_gives_12_pct_phys_reduction() {
        let mut s = default_state();
        s.counts[0] = 3;
        let b = charge_bonuses(&s);
        assert!((b.physical_damage_reduction_pct - 12.0).abs() < 0.01);
        assert!((b.all_elemental_resistances - 12.0).abs() < 0.01);
    }

    #[test]
    fn three_frenzy_gives_12_pct_each() {
        let mut s = default_state();
        s.counts[1] = 3;
        let b = charge_bonuses(&s);
        assert!((b.increased_attack_speed - 12.0).abs() < 0.01);
        assert!((b.increased_cast_speed   - 12.0).abs() < 0.01);
        assert!((b.increased_damage       - 12.0).abs() < 0.01);
    }

    #[test]
    fn three_power_gives_120_pct_crit() {
        let mut s = default_state();
        s.counts[2] = 3;
        let b = charge_bonuses(&s);
        assert!((b.increased_crit_chance - 120.0).abs() < 0.01);
    }

    #[test]
    fn no_charges_means_zero_bonuses() {
        let s = default_state();
        let b = charge_bonuses(&s);
        assert_eq!(b.physical_damage_reduction_pct, 0.0);
        assert_eq!(b.increased_crit_chance, 0.0);
    }

    #[test]
    fn mixed_charges_all_bonuses_apply() {
        let mut s = default_state();
        s.counts[0] = 1; // 1 endurance
        s.counts[1] = 2; // 2 frenzy
        s.counts[2] = 3; // 3 power
        let b = charge_bonuses(&s);
        assert!((b.physical_damage_reduction_pct - 4.0).abs() < 0.01);
        assert!((b.increased_damage - 8.0).abs() < 0.01);
        assert!((b.increased_crit_chance - 120.0).abs() < 0.01);
    }
}

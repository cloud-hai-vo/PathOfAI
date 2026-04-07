/// es_recharge.rs — Energy Shield Recharge (Algorithm 27).
///
/// Models the ES recharge delay timer, recharge rate, and special interactions:
/// Eldritch Battery, Ghost Reaver, CI, Ghost Dance.
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────────────────────

/// Player-level ES recharge configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsRechargeConfig {
    pub max_es:                       f64,  // maximum energy shield
    /// Flat increase to recharge rate: "% increased ES Recharge Rate"
    pub increased_recharge_rate_pct:  f64,
    /// Flat reduction to recharge delay in percent
    pub reduced_recharge_delay_pct:   f64,
    /// "Energy Shield Recharge begins immediately" (e.g., Vaal Discipline)
    pub recharge_begins_immediately:  bool,
    /// Ghost Reaver keystone — disables ES recharge
    pub has_ghost_reaver:             bool,
    /// Ghost Dance keystone — delay = 1.0 - shrouds * 0.33
    pub has_ghost_dance:              bool,
    /// Number of Ghost Shrouds (0-3) for Ghost Dance interaction
    pub ghost_shrouds:                u8,
}

impl Default for EsRechargeConfig {
    fn default() -> Self {
        Self {
            max_es:                      1000.0,
            increased_recharge_rate_pct: 0.0,
            reduced_recharge_delay_pct:  0.0,
            recharge_begins_immediately: false,
            has_ghost_reaver:            false,
            has_ghost_dance:             false,
            ghost_shrouds:               0,
        }
    }
}

/// Mutable per-frame ES recharge state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsRechargeState {
    pub es:             f64,  // current ES
    pub recharge_timer: f64,  // seconds since last ES damage
    pub recharging:     bool, // is recharge active
}

impl EsRechargeState {
    pub fn full(config: &EsRechargeConfig) -> Self {
        Self { es: config.max_es, recharge_timer: 0.0, recharging: false }
    }
}

/// Result of simulating one tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsTickResult {
    pub es:         f64,
    pub recharging: bool,
    /// ES recovered this tick (0 if not recharging or already full)
    pub recovered:  f64,
}

// ─── Core helpers ─────────────────────────────────────────────────────────────

/// Effective recharge delay in seconds.
pub fn recharge_delay(config: &EsRechargeConfig) -> f64 {
    if config.has_ghost_reaver {
        // Ghost Reaver disables recharge entirely — return ∞ sentinel
        return f64::MAX;
    }
    if config.recharge_begins_immediately {
        return 0.0;
    }
    if config.has_ghost_dance {
        let shrouds = config.ghost_shrouds.min(3) as f64;
        return (1.0 - shrouds * 0.33).max(0.0);
    }
    2.0 * (1.0 - config.reduced_recharge_delay_pct / 100.0).max(0.0)
}

/// Effective recharge rate in ES/second.
pub fn recharge_rate_per_second(config: &EsRechargeConfig) -> f64 {
    if config.has_ghost_reaver {
        return 0.0;
    }
    config.max_es * 0.33 * (1.0 + config.increased_recharge_rate_pct / 100.0)
}

// ─── Per-tick update ──────────────────────────────────────────────────────────

/// Advance ES recharge state by `dt` seconds.
///
/// `es_damaged_this_tick` should be `true` if the player took ES damage during this tick.
pub fn tick_es_recharge(
    state:               &mut EsRechargeState,
    config:              &EsRechargeConfig,
    dt:                  f64,
    es_damaged_this_tick: bool,
) -> EsTickResult {
    if es_damaged_this_tick {
        state.recharge_timer = 0.0;
        state.recharging     = false;
    } else {
        state.recharge_timer += dt;
        let delay = recharge_delay(config);
        if !state.recharging && state.recharge_timer >= delay {
            state.recharging = true;
        }
    }

    let mut recovered = 0.0;
    if state.recharging && state.es < config.max_es {
        let rate = recharge_rate_per_second(config);
        let gain = (rate * dt).max(0.0);
        let new_es = (state.es + gain).min(config.max_es);
        recovered = new_es - state.es;
        state.es = new_es;
    }

    EsTickResult {
        es:         state.es,
        recharging: state.recharging,
        recovered,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> EsRechargeConfig { EsRechargeConfig::default() }

    fn state_at(es: f64) -> EsRechargeState {
        EsRechargeState { es, recharge_timer: 0.0, recharging: false }
    }

    // ── recharge_delay ────────────────────────────────────────────────────────

    #[test]
    fn default_delay_is_2s() {
        assert!((recharge_delay(&cfg()) - 2.0).abs() < 0.001);
    }

    #[test]
    fn immediate_flag_gives_zero_delay() {
        let mut c = cfg();
        c.recharge_begins_immediately = true;
        assert_eq!(recharge_delay(&c), 0.0);
    }

    #[test]
    fn ghost_dance_0_shrouds_gives_1s_delay() {
        let mut c = cfg();
        c.has_ghost_dance = true;
        c.ghost_shrouds   = 0;
        assert!((recharge_delay(&c) - 1.0).abs() < 0.001);
    }

    #[test]
    fn ghost_dance_3_shrouds_gives_zero_delay() {
        let mut c = cfg();
        c.has_ghost_dance = true;
        c.ghost_shrouds   = 3;
        // 1.0 - 3 * 0.33 ≈ 0.01 (effectively zero delay, float arithmetic)
        assert!(recharge_delay(&c) < 0.02);
    }

    #[test]
    fn ghost_reaver_returns_infinite_delay() {
        let mut c = cfg();
        c.has_ghost_reaver = true;
        assert_eq!(recharge_delay(&c), f64::MAX);
    }

    // ── recharge_rate ─────────────────────────────────────────────────────────

    #[test]
    fn base_rate_is_33_pct_of_max_es() {
        let c = cfg(); // max_es = 1000
        assert!((recharge_rate_per_second(&c) - 330.0).abs() < 0.01);
    }

    #[test]
    fn increased_rate_applies_multiplicatively() {
        let mut c = cfg();
        c.increased_recharge_rate_pct = 100.0; // +100% = 2× rate
        assert!((recharge_rate_per_second(&c) - 660.0).abs() < 0.01);
    }

    #[test]
    fn ghost_reaver_rate_is_zero() {
        let mut c = cfg();
        c.has_ghost_reaver = true;
        assert_eq!(recharge_rate_per_second(&c), 0.0);
    }

    // ── tick_es_recharge ──────────────────────────────────────────────────────

    #[test]
    fn no_recharge_before_delay_expires() {
        let mut s = state_at(500.0);
        // 1 second tick, no damage — delay = 2s, recharge hasn't started yet
        let r = tick_es_recharge(&mut s, &cfg(), 1.0, false);
        assert_eq!(r.recovered, 0.0);
        assert!(!r.recharging);
        assert_eq!(s.es, 500.0);
    }

    #[test]
    fn recharge_starts_after_delay() {
        let mut s = state_at(500.0);
        // First tick: 2.0s passes without damage — delay met
        let r = tick_es_recharge(&mut s, &cfg(), 2.0, false);
        assert!(r.recharging);
        // Should have recovered 330 * 2.0 = 660 → but capped at 500 recovery means 1000 total
        // 500 + 660 = 1160 → capped at 1000
        assert!((r.es - 1000.0).abs() < 0.01);
    }

    #[test]
    fn damage_resets_recharge_timer() {
        let mut s = state_at(500.0);
        // Start recharging
        tick_es_recharge(&mut s, &cfg(), 2.0, false);
        assert!(s.recharging);

        // Take damage — resets
        let r = tick_es_recharge(&mut s, &cfg(), 0.1, true);
        assert!(!r.recharging);
        assert_eq!(s.recharge_timer, 0.0);
    }

    #[test]
    fn recharge_does_not_exceed_max_es() {
        let mut s = state_at(990.0);
        s.recharge_timer = 2.0;
        s.recharging     = true;
        // 1 second at 330/s would overshoot 1000
        let r = tick_es_recharge(&mut s, &cfg(), 1.0, false);
        assert!((r.es - 1000.0).abs() < 0.01);
        assert!((r.recovered - 10.0).abs() < 0.01);
    }

    #[test]
    fn already_full_es_yields_zero_recovery() {
        let mut s = EsRechargeState::full(&cfg());
        s.recharging     = true;
        s.recharge_timer = 2.0;
        let r = tick_es_recharge(&mut s, &cfg(), 1.0, false);
        assert_eq!(r.recovered, 0.0);
        assert_eq!(r.es, 1000.0);
    }

    #[test]
    fn immediate_recharge_starts_on_first_tick() {
        let mut c = cfg();
        c.recharge_begins_immediately = true;
        let mut s = state_at(500.0);
        let r = tick_es_recharge(&mut s, &c, 0.1, false);
        assert!(r.recharging);
        assert!(r.recovered > 0.0);
    }

    #[test]
    fn ghost_reaver_prevents_recharge() {
        let mut c = cfg();
        c.has_ghost_reaver = true;
        let mut s = state_at(200.0);
        // Even after a very long delay
        let r = tick_es_recharge(&mut s, &c, 100.0, false);
        assert!(!r.recharging);
        assert_eq!(r.recovered, 0.0);
        assert_eq!(r.es, 200.0);
    }

    #[test]
    fn partial_recovery_over_multiple_ticks() {
        let mut s = state_at(0.0);
        // Start recharging after 2s delay
        tick_es_recharge(&mut s, &cfg(), 2.0, false); // delay met, recovers too
        // Reset to known state for partial test
        s.es = 0.0;
        s.recharging = true;
        // 0.5s tick: 330 * 0.5 = 165 ES
        let r = tick_es_recharge(&mut s, &cfg(), 0.5, false);
        assert!((r.recovered - 165.0).abs() < 0.01);
        assert!((r.es - 165.0).abs() < 0.01);
    }
}

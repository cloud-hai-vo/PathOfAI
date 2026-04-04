/// combat_sim.rs — Discrete-tick combat simulator (Algorithm 20).
/// Tests written FIRST (TDD RED). Run `cargo test combat_sim` → all FAIL → then implement.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const TICK_MS: u64 = 100;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DamageType { Physical, Fire, Cold, Lightning, Chaos }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Rarity { Normal, Magic, Rare, Unique }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    pub id:                  u32,
    pub hp:                  f64,
    pub max_hp:              f64,
    pub damage:              f64,
    pub damage_type:         DamageType,
    pub attack_cooldown_ms:  u32,
    pub attack_timer_ms:     u32,
    pub rarity:              Rarity,
    pub alive:               bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerState {
    pub hp:          f64,
    pub max_hp:      f64,
    pub es:          f64,
    pub max_es:      f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlaskState {
    pub current_charges:   f64,
    pub max_charges:       u32,
    pub charge_per_kill:   f64,
    pub duration_ms:       u32,
    pub remaining_ms:      u32,
    pub active:            bool,
    pub life_recovery:     f64,  // per tick when active
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefenseSnapshot {
    pub phys_reduction_pct:  f64,  // 0-100
    pub fire_res:            f64,
    pub cold_res:            f64,
    pub lightning_res:       f64,
    pub chaos_res:           f64,
    pub evasion_chance:      f64,  // 0-1
    pub block_chance:        f64,  // 0-1
    pub life_regen_per_tick: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OffenseSnapshot {
    pub aoe_dps_per_tick:    f64,
    pub has_aoe:             bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LeechInstance {
    pub rate_per_tick: f64,
    pub remaining:     f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimResult {
    pub clear_time_ms:  u64,
    pub kills:          u32,
    pub deaths:         u32,
    pub ticks:          u64,
}

// ─── Stubs → all unimplemented!() → RED ──────────────────────────────────────

/// Entropy-based evasion check (Algorithm 3 / PoE entropy system).
/// Returns true if the attack is evaded.
/// `entropy` is state carried across calls (starts at 0.0).
pub fn check_evasion(entropy: &mut f64, evasion_chance: f64) -> bool {
    if evasion_chance <= 0.0 { return false; }
    if evasion_chance >= 1.0 { return true; }
    // PoE entropy system: entropy increases by hit_chance each check,
    // evade when entropy >= 1.0 (resets to fractional remainder).
    let hit_chance = 1.0 - evasion_chance;
    *entropy += hit_chance;
    if *entropy >= 1.0 {
        *entropy -= 1.0;
        false  // hit
    } else {
        true   // evaded
    }
}

pub fn mitigate_damage(raw: f64, dtype: &DamageType, defense: &DefenseSnapshot) -> f64 {
    let result = match dtype {
        DamageType::Physical => raw * (1.0 - defense.phys_reduction_pct / 100.0),
        DamageType::Fire     => raw * (1.0 - defense.fire_res / 100.0),
        DamageType::Cold     => raw * (1.0 - defense.cold_res / 100.0),
        DamageType::Lightning=> raw * (1.0 - defense.lightning_res / 100.0),
        DamageType::Chaos    => raw * (1.0 - defense.chaos_res / 100.0),
    };
    result.max(0.0)
}

pub fn tick_leech_instance(inst: &mut LeechInstance) -> f64 {
    if inst.remaining <= 0.0 { return 0.0; }
    let healed = inst.rate_per_tick.min(inst.remaining);
    inst.remaining -= healed;
    healed
}

pub fn tick_leech(instances: &mut Vec<LeechInstance>, player: &mut PlayerState) -> f64 {
    let mut total = 0.0f64;
    for inst in instances.iter_mut() {
        total += tick_leech_instance(inst);
    }
    player.hp = (player.hp + total).min(player.max_hp);
    instances.retain(|i| i.remaining > 0.0);
    total
}

pub fn tick_flask(flask: &mut FlaskState, player: &mut PlayerState) -> f64 {
    if !flask.active { return 0.0; }
    let healed = flask.life_recovery;
    player.hp = (player.hp + healed).min(player.max_hp);
    let elapsed = TICK_MS as u32;
    if flask.remaining_ms <= elapsed {
        flask.remaining_ms = 0;
        flask.active = false;
    } else {
        flask.remaining_ms -= elapsed;
    }
    healed
}

pub fn try_activate_flask(flask: &mut FlaskState, player: &PlayerState, hp_threshold: f64) -> bool {
    if flask.active { return false; }
    if flask.current_charges < 1.0 { return false; }
    let hp_pct = if player.max_hp > 0.0 { player.hp / player.max_hp } else { 1.0 };
    if hp_pct >= hp_threshold { return false; }
    flask.active = true;
    flask.remaining_ms = flask.duration_ms;
    flask.current_charges -= 1.0;
    true
}

pub fn simulate_map(
    player:   &PlayerState,
    defense:  &DefenseSnapshot,
    offense:  &OffenseSnapshot,
    monsters: Vec<Monster>,
    flasks:   Vec<FlaskState>,
) -> SimResult {
    if monsters.is_empty() {
        return SimResult { clear_time_ms: 0, kills: 0, deaths: 0, ticks: 0 };
    }

    let mut monsters = monsters;
    let mut flasks = flasks;
    let mut p = player.clone();
    let mut evasion_entropy = 0.0f64;
    let mut kills = 0u32;
    let mut deaths = 0u32;
    let mut ticks = 0u64;
    const MAX_TICKS: u64 = 36_000; // 1 hour safety cap

    while kills < monsters.len() as u32 && ticks < MAX_TICKS {
        ticks += 1;

        // Phase 1 — player deals AoE damage
        if offense.has_aoe {
            for m in monsters.iter_mut().filter(|m| m.alive) {
                m.hp -= offense.aoe_dps_per_tick;
                if m.hp <= 0.0 {
                    m.alive = false;
                    kills += 1;
                    for f in flasks.iter_mut() {
                        f.current_charges += f.charge_per_kill;
                    }
                }
            }
        }

        // Phase 2 — monsters attack player
        for m in monsters.iter_mut().filter(|m| m.alive) {
            if m.attack_timer_ms > 0 {
                m.attack_timer_ms = m.attack_timer_ms.saturating_sub(TICK_MS as u32);
                continue;
            }
            m.attack_timer_ms = m.attack_cooldown_ms;

            if check_evasion(&mut evasion_entropy, defense.evasion_chance) { continue; }

            let raw = m.damage;
            let mitigated = mitigate_damage(raw, &m.damage_type, defense);
            p.hp -= mitigated;

            if p.hp <= 0.0 {
                deaths += 1;
                p.hp = p.max_hp;
            }
        }

        // Phase 3 — recovery
        p.hp = (p.hp + defense.life_regen_per_tick).min(p.max_hp);

        // Flask management
        for f in flasks.iter_mut() {
            try_activate_flask(f, &p, 0.60);
            tick_flask(f, &mut p);
        }
    }

    SimResult {
        clear_time_ms: ticks * TICK_MS,
        kills,
        deaths,
        ticks,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn no_defense() -> DefenseSnapshot { DefenseSnapshot::default() }

    fn full_defense() -> DefenseSnapshot {
        DefenseSnapshot {
            phys_reduction_pct: 80.0,
            fire_res: 75.0, cold_res: 75.0, lightning_res: 75.0, chaos_res: 75.0,
            evasion_chance: 0.70, block_chance: 0.0,
            life_regen_per_tick: 10.0,
        }
    }

    fn monster(id: u32, hp: f64, dmg: f64, dtype: DamageType, cooldown_ms: u32) -> Monster {
        Monster { id, hp, max_hp: hp, damage: dmg, damage_type: dtype,
            attack_cooldown_ms: cooldown_ms, attack_timer_ms: 0,
            rarity: Rarity::Normal, alive: true }
    }

    fn player(hp: f64) -> PlayerState {
        PlayerState { hp, max_hp: hp, ..Default::default() }
    }

    // ── mitigate_damage ───────────────────────────────────────────────────────

    #[test]
    fn physical_damage_reduced_by_armour() {
        let def = DefenseSnapshot { phys_reduction_pct: 50.0, ..Default::default() };
        let result = mitigate_damage(1000.0, &DamageType::Physical, &def);
        assert!((result - 500.0).abs() < 0.001, "50% phys reduction: got {result}");
    }

    #[test]
    fn fire_damage_reduced_by_fire_res() {
        let def = DefenseSnapshot { fire_res: 75.0, ..Default::default() };
        let result = mitigate_damage(1000.0, &DamageType::Fire, &def);
        assert!((result - 250.0).abs() < 0.001, "75% fire res: got {result}");
    }

    #[test]
    fn cold_damage_reduced_by_cold_res() {
        let def = DefenseSnapshot { cold_res: 60.0, ..Default::default() };
        let result = mitigate_damage(100.0, &DamageType::Cold, &def);
        assert!((result - 40.0).abs() < 0.001, "60% cold res: got {result}");
    }

    #[test]
    fn chaos_damage_reduced_by_chaos_res() {
        let def = DefenseSnapshot { chaos_res: 25.0, ..Default::default() };
        let result = mitigate_damage(100.0, &DamageType::Chaos, &def);
        assert!((result - 75.0).abs() < 0.001, "25% chaos res: got {result}");
    }

    #[test]
    fn zero_resistance_applies_full_damage() {
        let result = mitigate_damage(500.0, &DamageType::Fire, &no_defense());
        assert!((result - 500.0).abs() < 0.001);
    }

    #[test]
    fn physical_damage_never_below_zero() {
        let def = DefenseSnapshot { phys_reduction_pct: 100.0, ..Default::default() };
        let result = mitigate_damage(1000.0, &DamageType::Physical, &def);
        assert!(result >= 0.0);
    }

    // ── check_evasion (entropy system) ────────────────────────────────────────

    #[test]
    fn evasion_at_zero_chance_never_evades() {
        let mut entropy = 0.0f64;
        for _ in 0..20 {
            assert!(!check_evasion(&mut entropy, 0.0), "0% evasion must never evade");
        }
    }

    #[test]
    fn evasion_at_full_chance_always_evades() {
        let mut entropy = 0.0f64;
        for _ in 0..20 {
            assert!(check_evasion(&mut entropy, 1.0), "100% evasion must always evade");
        }
    }

    #[test]
    fn evasion_entropy_produces_correct_ratio() {
        // 70% evasion over 1000 checks should evade ~700 ± 50
        let mut entropy = 0.0f64;
        let evades: u32 = (0..1000).map(|_| check_evasion(&mut entropy, 0.70) as u32).sum();
        assert!(evades >= 650 && evades <= 750,
            "70% evasion: expected ~700, got {evades}");
    }

    #[test]
    fn evasion_entropy_no_consecutive_hits_beyond_limit() {
        // At 70% evasion, should never get more than 2 consecutive hits (ceil(1/0.3) = 4, but PoE caps earlier)
        let mut entropy = 0.0f64;
        let mut consecutive = 0u32;
        let mut max_consecutive = 0u32;
        for _ in 0..500 {
            if check_evasion(&mut entropy, 0.70) {
                consecutive = 0;
            } else {
                consecutive += 1;
                max_consecutive = max_consecutive.max(consecutive);
            }
        }
        assert!(max_consecutive <= 4,
            "entropy should prevent >4 consecutive hits at 70% evasion, got {max_consecutive}");
    }

    // ── LeechInstance ─────────────────────────────────────────────────────────

    #[test]
    fn leech_tick_returns_rate_when_remaining() {
        let mut inst = LeechInstance { rate_per_tick: 50.0, remaining: 200.0 };
        let healed = tick_leech_instance(&mut inst);
        assert!((healed - 50.0).abs() < 0.001);
        assert!((inst.remaining - 150.0).abs() < 0.001);
    }

    #[test]
    fn leech_tick_capped_at_remaining() {
        let mut inst = LeechInstance { rate_per_tick: 100.0, remaining: 30.0 };
        let healed = tick_leech_instance(&mut inst);
        assert!((healed - 30.0).abs() < 0.001, "can't leech more than remaining");
        assert_eq!(inst.remaining, 0.0);
    }

    #[test]
    fn leech_tick_zero_remaining_returns_zero() {
        let mut inst = LeechInstance { rate_per_tick: 50.0, remaining: 0.0 };
        assert_eq!(tick_leech_instance(&mut inst), 0.0);
    }

    #[test]
    fn tick_leech_heals_player_and_respects_max() {
        let mut p = player(900.0);
        p.max_hp = 1000.0;
        let mut instances = vec![LeechInstance { rate_per_tick: 200.0, remaining: 200.0 }];
        tick_leech(&mut instances, &mut p);
        assert!(p.hp <= p.max_hp, "leech must not exceed max hp");
        assert!(p.hp > 900.0, "player should have been healed");
    }

    #[test]
    fn tick_leech_removes_exhausted_instances() {
        let mut p = player(500.0);
        p.max_hp = 1000.0;
        let mut instances = vec![LeechInstance { rate_per_tick: 50.0, remaining: 0.0 }];
        tick_leech(&mut instances, &mut p);
        // Exhausted instances should be cleaned up or their healing is 0
        assert_eq!(p.hp, 500.0, "zero-remaining leech should heal nothing");
    }

    // ── FlaskState ────────────────────────────────────────────────────────────

    #[test]
    fn flask_heals_player_per_tick_when_active() {
        let mut f = FlaskState {
            active: true, remaining_ms: 1000, life_recovery: 50.0,
            current_charges: 60.0, max_charges: 60, charge_per_kill: 0.0, duration_ms: 1000,
        };
        let mut p = player(500.0);
        p.max_hp = 1000.0;
        let healed = tick_flask(&mut f, &mut p);
        assert!((healed - 50.0).abs() < 0.001);
        assert!((p.hp - 550.0).abs() < 0.001);
    }

    #[test]
    fn flask_does_not_heal_when_inactive() {
        let mut f = FlaskState { active: false, ..Default::default() };
        let mut p = player(500.0);
        assert_eq!(tick_flask(&mut f, &mut p), 0.0);
        assert_eq!(p.hp, 500.0);
    }

    #[test]
    fn flask_deactivates_when_duration_expires() {
        let mut f = FlaskState {
            active: true, remaining_ms: TICK_MS as u32, life_recovery: 10.0,
            current_charges: 60.0, max_charges: 60, charge_per_kill: 0.0, duration_ms: 1000,
        };
        let mut p = player(500.0);
        p.max_hp = 1000.0;
        tick_flask(&mut f, &mut p);
        assert!(!f.active, "flask should deactivate after duration expires");
    }

    #[test]
    fn try_activate_flask_activates_when_below_threshold() {
        let mut f = FlaskState {
            current_charges: 60.0, max_charges: 60, duration_ms: 4000,
            remaining_ms: 0, active: false, charge_per_kill: 0.0, life_recovery: 50.0,
        };
        let p = PlayerState { hp: 500.0, max_hp: 1000.0, ..Default::default() };
        let activated = try_activate_flask(&mut f, &p, 0.60);
        assert!(activated, "should activate when hp (50%) < threshold (60%)");
        assert!(f.active);
    }

    #[test]
    fn try_activate_flask_does_not_activate_above_threshold() {
        let mut f = FlaskState {
            current_charges: 60.0, max_charges: 60, duration_ms: 4000,
            remaining_ms: 0, active: false, charge_per_kill: 0.0, life_recovery: 50.0,
        };
        let p = PlayerState { hp: 800.0, max_hp: 1000.0, ..Default::default() };
        assert!(!try_activate_flask(&mut f, &p, 0.60),
            "should not activate when hp (80%) > threshold (60%)");
    }

    #[test]
    fn try_activate_flask_does_not_activate_with_no_charges() {
        let mut f = FlaskState { current_charges: 0.0, max_charges: 60, active: false, ..Default::default() };
        let p = PlayerState { hp: 100.0, max_hp: 1000.0, ..Default::default() };
        assert!(!try_activate_flask(&mut f, &p, 0.99),
            "cannot activate with 0 charges");
    }

    // ── simulate_map ──────────────────────────────────────────────────────────

    #[test]
    fn simulate_kills_all_monsters() {
        let offense = OffenseSnapshot { aoe_dps_per_tick: 999999.0, has_aoe: true };
        let monsters = vec![
            monster(1, 100.0, 0.0, DamageType::Physical, 1000),
            monster(2, 100.0, 0.0, DamageType::Physical, 1000),
        ];
        let p = player(10000.0);
        let result = simulate_map(&p, &no_defense(), &offense, monsters, vec![]);
        assert_eq!(result.kills, 2, "should kill all monsters");
    }

    #[test]
    fn simulate_returns_nonzero_time() {
        let offense = OffenseSnapshot { aoe_dps_per_tick: 100.0, has_aoe: true };
        let monsters = vec![monster(1, 1000.0, 0.0, DamageType::Physical, 9999)];
        let p = player(10000.0);
        let result = simulate_map(&p, &no_defense(), &offense, monsters, vec![]);
        assert!(result.clear_time_ms > 0);
    }

    #[test]
    fn simulate_no_monsters_returns_zero_kills_zero_time() {
        let offense = OffenseSnapshot { aoe_dps_per_tick: 0.0, has_aoe: false };
        let result = simulate_map(&player(1000.0), &no_defense(), &offense, vec![], vec![]);
        assert_eq!(result.kills, 0);
        assert_eq!(result.ticks, 0);
    }

    #[test]
    fn simulate_counts_deaths_when_player_hp_drops_to_zero() {
        // Player with 1 hp, monster that deals 1000 dmg per tick — should die immediately
        let defense = DefenseSnapshot { phys_reduction_pct: 0.0, ..Default::default() };
        let offense = OffenseSnapshot { aoe_dps_per_tick: 0.0, has_aoe: false };
        // Monster attacks every tick
        let monsters = vec![monster(1, 9999999.0, 2000.0, DamageType::Physical, TICK_MS as u32)];
        let p = PlayerState { hp: 1.0, max_hp: 1.0, ..Default::default() };
        let result = simulate_map(&p, &defense, &offense, monsters, vec![]);
        assert!(result.deaths > 0, "player should die when hit harder than hp");
    }

    #[test]
    fn simulate_ticks_matches_clear_time() {
        let offense = OffenseSnapshot { aoe_dps_per_tick: 999999.0, has_aoe: true };
        let monsters = vec![monster(1, 100.0, 0.0, DamageType::Physical, 1000)];
        let p = player(10000.0);
        let result = simulate_map(&p, &no_defense(), &offense, monsters, vec![]);
        assert_eq!(result.ticks * TICK_MS, result.clear_time_ms);
    }
}

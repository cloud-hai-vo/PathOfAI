# Path of AI — Algorithm Reference

> **Purpose:** Comprehensive specification of every computer science algorithm in
> the engine. Each section is self-contained and implementable without guessing.
> PoE game formulas live in [ENGINE-DESIGN.md](ENGINE-DESIGN.md) — this doc covers
> the **algorithmic strategies** that use those formulas.

## Table of Contents

1. [Modifier Aggregation Pipeline](#1-modifier-aggregation-pipeline)
2. [Damage Conversion Graph Resolver](#2-damage-conversion-graph-resolver)
3. [Entropy-Based Evasion Counter](#3-entropy-based-evasion-counter)
4. [Leech Instance Manager](#4-leech-instance-manager)
5. [Flask Uptime Model](#5-flask-uptime-model)
6. [Build Archetype Classifier](#6-build-archetype-classifier)
7. [Item Scoring — Archetype-Adaptive Weighted Scoring](#7-item-scoring--archetype-adaptive-weighted-scoring)
8. [Issue Detector — Constraint Violation Scanner](#8-issue-detector--constraint-violation-scanner)
9. [Sensitivity Analysis — Numerical Partial Derivatives](#9-sensitivity-analysis--numerical-partial-derivatives)
10. [Pareto-Optimal Upgrade Ranking](#10-pareto-optimal-upgrade-ranking)
11. [Budget-Constrained Upgrade Optimizer (Knapsack)](#11-budget-constrained-upgrade-optimizer-knapsack)
12. [Multi-Slot Constraint Solver](#12-multi-slot-constraint-solver)
13. [Passive Tree Pathfinding — Modified Dijkstra](#13-passive-tree-pathfinding--modified-dijkstra)
14. [Passive Tree Respec Optimizer](#14-passive-tree-respec-optimizer)
15. [Crafting Probability Engine](#15-crafting-probability-engine)
16. [Monte Carlo Craft Simulator](#16-monte-carlo-craft-simulator)
17. [Build Similarity & Collaborative Filtering](#17-build-similarity--collaborative-filtering)
18. [Intent Classifier — Query Router](#18-intent-classifier--query-router)
19. [Template Response Generator](#19-template-response-generator)
20. [Combat Simulation Engine](#20-combat-simulation-engine)
21. [Market Price Cache & Circuit Breaker](#21-market-price-cache--circuit-breaker)
22. [PoB XML Parser — Streaming State Machine](#22-pob-xml-parser--streaming-state-machine)
23. [Mod Text Parser — Pattern Matching Recognizer](#23-mod-text-parser--pattern-matching-recognizer)
24. [Change Detection — Hash-Based Lazy Recalculation](#24-change-detection--hash-based-lazy-recalculation)
25. [Fast Estimation Engine — Pre-Computed Impact Tables](#25-fast-estimation-engine--pre-computed-impact-tables)
26. [Ailment Mechanics — Ignite/Chill/Freeze/Shock/Poison/Bleed](#26-ailment-mechanics)
27. [Energy Shield Recharge](#27-energy-shield-recharge)
28. [Mana Reservation Engine](#28-mana-reservation-engine)
29. [Clipboard Item Parser — Ctrl+C Instant Analysis](#29-clipboard-item-parser)
30. [Stat Requirement Checker](#30-stat-requirement-checker)
31. [Charge Management — Endurance / Frenzy / Power](#31-charge-management)
32. [Playstyle Classifier](#32-playstyle-classifier)
33. [Change History Manager — Undo / Redo / Apply Suggestion / Revert](#33-change-history-manager)
34. [OAuth Token Lifecycle — Refresh / Reimport / Conflict Resolution](#34-oauth-token-lifecycle)
35. [Session Persistence & Auto-Save](#35-session-persistence--auto-save)
36. [Database Init & Migration](#36-database-init--migration)
37. [OAuth PKCE Authorization Flow](#37-oauth-pkce-authorization-flow)
38. [Stash Tab Processor](#38-stash-tab-processor)
39. [Map Mod Danger Scorer](#39-map-mod-danger-scorer)
40. [Build Share Code Codec](#40-build-share-code-codec)
41. [Client.txt Log Parser — Map Run Tracker](#41-clienttxt-log-parser--map-run-tracker)
42. [Seer Network Architecture](#42-seer-network-architecture)
43. [Vendor Recipe Detector](#43-vendor-recipe-detector)
44. [Portable Storage & File Watcher](#44-portable-storage--file-watcher)
45. [PoE Character Fetch Pipeline](#45-poe-character-fetch-pipeline)
46. [PoB Write-Back Engine](#46-pob-write-back-engine)
47. [Craft Suggestion Ranker & Trade Search](#47-craft-suggestion-ranker--trade-search)
48. [Top-N Passive Node Recommender](#48-top-n-passive-node-recommender)
49. [Build Comparator](#49-build-comparator)
50. [Price Alert Manager](#50-price-alert-manager)
51. [Item Image Resolver](#51-item-image-resolver)
52. [Buy Timing Advisor & Craft-vs-Buy](#52-buy-timing-advisor--craft-vs-buy)
53. [Map Run & Wealth Accumulator](#53-map-run--wealth-accumulator)
54. [Cloud AI Connection Manager](#54-cloud-ai-connection-manager)

---

## 1. Modifier Aggregation Pipeline

### Problem

A PoE build has hundreds of modifiers from items (9+ slots × 6 mods), passive tree
(~120 nodes × 1-3 stats each), gem links (6+ gems × supports), ascendancy, config
flags, flasks, and jewels. Before any calculation, ALL modifiers must be collected,
categorized, and merged into a single `ModPool` that the offense/defense calculators
consume.

### Algorithm

**Two-pass aggregation with tag-aware routing.**

```
Pass 1: COLLECT (linear scan over all sources)
  For each source in [items, tree, gems, ascendancy, jewels, config, flasks]:
    For each modifier on this source:
      1. Identify modifier SCOPE:
         - Local (on weapon/armour only) vs Global
         - Player vs Minion vs Both
         - Conditional vs Unconditional
      2. Identify modifier TYPE:
         - Flat added (e.g., "+50 to maximum life")
         - Increased/Reduced (e.g., "40% increased fire damage")
         - More/Less (e.g., "59% more damage with burning")
         - Conversion (e.g., "50% of phys converted to fire")
         - Gained-as-extra (e.g., "gain 20% of phys as extra fire")
         - Special (e.g., "your crits deal no extra damage" — Elemental Overload)
      3. Identify modifier TAGS (damage types it applies to):
         - fire, cold, lightning, chaos, physical
         - attack, spell, DoT, ailment, area, projectile
         - Multiple tags = applies if ANY match (OR logic for damage tags)
      4. Push to appropriate bucket in ModPool

Pass 2: TRANSFORM (apply special keystones that remap modifier categories)
  For each active transformation:
    Crown of Eyes:     copy spell damage → attack damage (×1.5 value)
    Iron Will:         strength bonus → spell damage
    Iron Grip:         strength bonus → projectile attack damage
    Spiritual Aid:     minion damage → player damage
    Battlemage:        weapon flat damage → spell flat damage
    Precise Technique: no crit, but accuracy → more damage

  These transformations DUPLICATE modifiers — the original stays AND the copy
  is added. They run BEFORE the main calculation.
```

### Data Structure

```rust
pub struct ModPool {
    // Flat damage by type (summed per type)
    pub flat_added: HashMap<DamageType, (f64, f64)>,  // (min, max) per type

    // Increased/Reduced — additive within each tag group
    // Key = stat tag (e.g., "fire_damage", "attack_speed", "life")
    pub increased: HashMap<StatTag, f64>,  // sum of all %increased

    // More/Less — each is a separate multiplier
    pub more_multipliers: HashMap<StatTag, Vec<f64>>,  // list per tag

    // Conversion chains
    pub conversions: Vec<ConversionMod>,  // (from_type, to_type, percent)

    // Gained as extra
    pub gained_as_extra: Vec<GainedAsExtra>,  // (from_type, to_type, percent)

    // Conditionals (only active under certain build states)
    pub conditional: Vec<(Condition, ModEntry)>,

    // Special flags
    pub flags: HashSet<SpecialFlag>,  // EO, RT, CI, EB, VP, etc.
}

/// Condition — how conditional mods are resolved before calculation
///
/// For the CALCULATOR (static snapshot), we use a BuildConfig that contains the
/// player's assumptions (boss fight? recently killed? using flask?). Conditionals
/// are resolved ONCE per calculation, not per-tick.
///
/// For the COMBAT SIMULATOR, conditionals are re-evaluated every tick.
pub enum Condition {
    // Always-active (config-based) — user sets these in PoB-style config toggles
    EnemyIsBoss,              // "against bosses" mods
    IsMoving,                 // "while moving" mods
    RecentlyKilled,           // "if you've killed recently" (15s window, assume true for mapping)
    IsFlasking,               // "while flask is active"
    FullLife,                 // "while at full life"
    LowLife,                  // player.life < max_life * 0.35
    LowMana,                  // player.mana < max_mana * 0.35
    HasEnduranceCharge(u8),   // "per endurance charge" → multiply value × current_charges
    HasFrenzyCharge(u8),
    HasPowerCharge(u8),
    EnemyChilled,             // "against chilled enemies" — assume true if build chills
    EnemyPoisoned,            // "against poisoned enemies"
    EnemyCursed,              // "against cursed enemies" — assume true if build curses
    InHeraldOfXEffect,        // "while affected by Herald of X"
    ChannellingSkill,         // "while channelling"
    OnHit,                    // "on hit" — always true for hit-based builds
    UniqueCondition(String),  // fallback for unusual mod text — treated as always-on with UI warning
}

/// Resolution rule:
///   1. Check BuildConfig (user's configured toggle state)
///   2. If toggle exists: use it directly
///   3. If no toggle: apply DEFAULT_ASSUMPTION (see below)
///   4. Mod is included in pool if condition resolves to true

const DEFAULT_ASSUMPTIONS: &[(Condition, bool)] = &[
    // For mapping (default context):
    (Condition::RecentlyKilled, true),   // safe assumption for map clearing
    (Condition::EnemyIsBoss, false),     // show mapping stats by default
    (Condition::IsMoving, false),        // conservative
    (Condition::IsFlasking, true),       // assume flasks are up
    (Condition::FullLife, false),        // conservative for on-full-life mods
    (Condition::LowLife, false),         // only true for LL builds
    (Condition::EnemyChilled, true),     // if build has chill source
    (Condition::EnemyCursed, true),      // if build has curse
];
```

### Key Rules

1. **Local vs Global:** A mod on a weapon saying "+20% increased Physical Damage"
   is LOCAL — it only scales that weapon's base damage, not global phys. The same
   mod on a ring is GLOBAL. Detection: check `mod.source_slot` and `mod.is_local`
   flag from game data.

2. **Additive ceiling for conversion:** If multiple sources convert phys to fire
   (e.g., 50% from gem + 40% from item = 90%), they are additive up to 100% cap
   per source type. If total conversion from phys exceeds 100%, all conversions
   are scaled proportionally.

3. **Modifier application order matters:** Flat → Conversion → Increased → More
   → Crit → Speed. The aggregation pipeline collects modifiers into the right
   buckets so the calculator can apply them in order.

### Complexity

- Time: O(M) where M = total modifier count (~500-1000 per build)
- Space: O(M) for the pool
- No sorting needed — just bucketing

### Edge Cases

- Empty modifier (value 0) → skip, don't add
- Negative "increased" (reduced) → add as negative number, sum can go below 0
- Local mods on weapon swap → only include active weapon set
- "Nearby enemies" mods → treat as always-on for calculation, flag in UI
- **Max resistance mods** are a distinct category — they raise the cap, not the current value:
  ```
  // Collected in Pass 1 alongside regular mods:
  ModType::MaxResistance(element, amount):
    mod_pool.max_res[element] += amount  // e.g., +4% max fire from Rise of the Phoenix

  // Applied in calculator BEFORE capping current resistance:
  fn calculate_resistance(element: Element, pool: &ModPool) -> (f64, f64) {
    let max_res = 75.0 + pool.max_res[element].clamp(0.0, 15.0)  // +0 to +15% cap
    // Capped at 90% absolute ceiling (can't go above 90% even with many sources)
    let max_res = max_res.min(90.0)
    let current_res = pool.base_res[element] + pool.increased_res[element]
    let capped_res = current_res.min(max_res).max(-100.0)  // chaos has no cap
    (capped_res, max_res)
  }

  // Common max-res sources:
  //   Rise of the Phoenix (shield):        +8% max fire res
  //   Purity of Fire (aura, level 20):     +4% max fire res
  //   Purity of Ice  (aura, level 20):     +4% max cold res
  //   Purity of Lightning (aura, level 20):+4% max lightning res
  //   The Wise Oak (flask, balanced res):  +10% max for balanced element
  //   Taste of Hate (flask):               converts phys → cold taken, +10% max cold
  //   Mahuxotl's Machination (shield):     +8% max all res, CI required
  ```

### CalcResult — Canonical Output Struct

All algorithms that consume calculation results (Issue Detector, Sensitivity Analysis,
Fast Estimation, etc.) operate on `CalcResult`. Every field referenced elsewhere in
this document must exist here.

```rust
pub struct CalcResult {
    pub offense: OffenseResult,
    pub defense: DefenseResult,
}

pub struct OffenseResult {
    /// Raw single-target DPS before any enemy resistance is applied.
    /// This is what PoB displays and what most users recognise as "their DPS".
    /// Formula: (hit_dps + dot_dps) with player's own damage modifiers only.
    pub total_dps: f64,

    /// Effective DPS after applying 40% physical / 0% elemental (standard endgame
    /// monster resistances). Used for build comparison — a fairer number than raw DPS
    /// when comparing physical vs elemental builds.
    /// Formula: total_dps adjusted per damage type × (1 - monster_res[type])
    pub effective_dps_vs_map: f64,

    /// Effective DPS against bosses (75% less elemental/chaos from Shaper/Elder,
    /// 40% physical). The number that matters for "can this build kill Uber Maven?".
    /// Shown in boss-context UI panels; hidden by default to avoid confusing new players.
    pub effective_dps_vs_boss: f64,

    pub aoe_dps: f64,              // area damage per second (raw)
    pub hit_dps: f64,              // hit-based share of total_dps (no DoT)
    pub dot_dps: f64,              // DoT-only share of total_dps (Ignite, Poison, Bleed, etc.)
    pub aoe_radius: f64,           // effective AoE radius in units
    pub single_target_cooldown_ms: u32,
    pub crit_chance: f64,          // 0.0-1.0
    pub crit_multiplier: f64,      // e.g., 1.5 = 150%
    pub hit_chance: f64,           // 0.0-1.0 (accuracy-based)
    pub ailment_chance: f64,       // chance to apply primary ailment
}

pub struct DefenseResult {
    // Resistances (capped at max_res per element)
    pub fire_res: f64,
    pub cold_res: f64,
    pub lightning_res: f64,
    pub chaos_res: f64,            // NOT capped — can be negative
    pub max_fire_res: f64,         // cap (default 75%, raised by max-res mods)
    pub max_cold_res: f64,
    pub max_lightning_res: f64,

    // Hit pool
    pub life: f64,
    pub max_life: f64,
    pub es: f64,
    pub max_es: f64,
    pub ward: f64,                 // ward value (0 unless ward items equipped)
    pub ehp_phys: f64,             // effective HP vs physical (life × mitigation)
    pub ehp_fire: f64,

    // Mitigation layers
    pub phys_reduction_pct: f64,   // from armour (against standard 5K hit)
    pub block_chance: f64,         // 0.0-1.0
    pub spell_block_chance: f64,
    pub evasion_chance: f64,       // 0.0-1.0 (against standard attacker)
    pub spell_suppression_chance: f64, // 0.0-1.0

    // Recovery
    pub life_regen_per_second: f64,
    pub es_regen_per_second: f64,
    pub life_leech_rate: f64,      // estimated sustained leech /s
    pub life_regen_net: f64,       // regen minus degen (negative = dying)

    // Ailment immunity flags
    pub freeze_immune: bool,
    pub chill_immune: bool,
    pub shock_immune: bool,
    pub ignite_immune: bool,
    pub bleed_immune: bool,
    pub poison_immune: bool,
    pub curse_immune: bool,
    pub stun_immune: bool,

    // Defense layer flags (for issue detection)
    pub has_armour: bool,          // armour > 500
    pub has_evasion: bool,         // evasion > 500
    pub has_es: bool,              // max_es > 500
    pub has_ward: bool,            // ward > 0
    pub has_guard_skill: bool,     // Molten Shell / Steelskin / Arcane Cloak equipped

    // Mana
    pub mana_reservation_percent: f64,  // 0.0-100.0
    pub free_mana: f64,
}
```

**Armour reduction formula** — `phys_reduction(raw_damage)` is damage-dependent:

```rust
impl DefenseResult {
    /// PoE armour formula: damage reduction % = armour / (armour + 5 × raw_damage)
    /// Capped at 90% maximum reduction.
    /// The 5× multiplier means armour efficiency DECREASES with larger hits.
    /// Example: 10,000 armour vs 1,000 damage → 10000/(10000+5000) = 66.7%
    ///          10,000 armour vs 5,000 damage → 10000/(10000+25000) = 28.6%
    pub fn phys_reduction(&self, raw_damage: f64) -> f64 {
        if self.armour <= 0.0 { return 0.0; }
        let reduction = self.armour / (self.armour + 5.0 * raw_damage);
        reduction.min(0.90)  // 90% cap
    }

    /// For the static stat panel (no incoming damage known), use a reference hit of 5,000.
    /// This is the PoE convention for displaying "physical damage reduction" %.
    pub fn phys_reduction_display(&self) -> f64 {
        self.phys_reduction(5000.0)
    }
}
// Note: `phys_reduction_pct` in the struct stores the display value (vs 5K hit).
// The combat simulator always calls phys_reduction(actual_raw) for accuracy.
```

**Note:** `overcap(element)` used in Issue Detector is a method:
```rust
impl DefenseResult {
    pub fn overcap(&self, elem: Element) -> f64 {
        match elem {
            Fire      => (self.fire_res - 75.0).max(0.0),
            Cold      => (self.cold_res - 75.0).max(0.0),
            Lightning => (self.lightning_res - 75.0).max(0.0),
        }
    }
}
```

---

## 2. Damage Conversion Graph Resolver

### Problem

PoE damage conversion follows a strict DAG (directed acyclic graph):

```
physical → lightning → cold → fire → chaos
```

Conversion is one-way only. Multiple conversion sources of the same type are
additive (capped at 100%). Converted damage inherits modifiers from BOTH the
source type AND the destination type. "Gained as extra" is NOT conversion — it
creates additional damage without removing the original.

### Algorithm

**Topological-order resolution with modifier inheritance tracking.**

```
Input:
  base_damage: HashMap<DamageType, (f64, f64)>  // min/max per type
  conversions: Vec<(DamageType, DamageType, f64)>  // (from, to, percent)
  gained_extra: Vec<(DamageType, DamageType, f64)>  // (from, to, percent)
  increased: HashMap<DamageType, f64>  // %increased per type
  more: HashMap<DamageType, Vec<f64>>  // more multipliers per type

Output:
  final_damage: HashMap<DamageType, f64>  // total damage per type after all scaling

Algorithm:
  // Step 1: Normalize conversions (cap at 100% per source type)
  for each source_type in [physical, lightning, cold, fire]:
    total = sum of all conversions FROM source_type
    if total > 1.0:
      scale_factor = 1.0 / total
      for each conversion FROM source_type:
        conversion.percent *= scale_factor

  // Step 2: Process in topological order (phys first, chaos last)
  // Track which types each damage "came from" for modifier inheritance
  type_order = [Physical, Lightning, Cold, Fire, Chaos]

  // damage_pool[type] = base damage of that type BEFORE increased/more
  // inherited_tags[type] = set of source types this damage was converted from
  damage_pool: HashMap<DamageType, f64>
  inherited_tags: HashMap<DamageType, HashSet<DamageType>>

  for type in type_order:
    damage_pool[type] = base_damage[type]
    inherited_tags[type] = {type}  // always inherits own type

  for source in type_order:
    // Apply conversions FROM this source to downstream types
    remaining = 1.0
    for each (_, dest, pct) in conversions where from == source:
      converted_amount = damage_pool[source] * pct
      damage_pool[dest] += converted_amount
      inherited_tags[dest] = inherited_tags[dest] ∪ inherited_tags[source]
      remaining -= pct

    // Apply gained-as-extra FROM this source (does NOT reduce source)
    for each (_, dest, pct) in gained_extra where from == source:
      extra_amount = damage_pool[source] * pct
      damage_pool[dest] += extra_amount
      inherited_tags[dest] = inherited_tags[dest] ∪ inherited_tags[source]

    // Reduce source by converted amount
    damage_pool[source] *= remaining

  // Step 3: Apply increased/more using inherited tags
  for type in type_order:
    // Collect ALL applicable %increased (from own type + all source types)
    total_increased = 0.0
    for tag in inherited_tags[type]:
      total_increased += increased[tag]
    // Also add generic "damage" increased (applies to everything)
    total_increased += increased[Generic]

    damage_pool[type] *= (1.0 + total_increased / 100.0)

    // Apply more multipliers (same inheritance rule)
    for tag in inherited_tags[type]:
      for m in more[tag]:
        damage_pool[type] *= (1.0 + m / 100.0)

  return damage_pool
```

### Why This Is Tricky

The modifier inheritance rule is the #1 source of calculator bugs. Example:

- Player has 100 physical damage, 50% converted to fire.
- Player has "+100% increased physical damage" and "+100% increased fire damage".
- **Wrong:** 50 phys × 2.0 = 100 phys, 50 fire × 2.0 = 100 fire. Total: 200.
- **Right:** 50 phys × 2.0 = 100 phys, 50 fire × (1 + 1.0 + 1.0) = 150 fire. Total: 250.
- The fire damage was converted FROM phys, so it inherits phys modifiers too.

### Complexity

- Time: O(T² × C) where T = 5 damage types, C = conversion count. Effectively O(1).
- Space: O(T) = O(1)

### Test Cases

```
Test 1: No conversion
  100 phys, 0 conversions → 100 phys
  
Test 2: Simple 50% phys → fire
  100 phys, 50% phys→fire → 50 phys + 50 fire
  
Test 3: Over-cap conversion (60% fire + 60% cold = 120% → scaled to 50/50)
  100 phys, 60% phys→fire, 60% phys→cold → 0 phys + 50 fire + 50 cold
  
Test 4: Chain conversion (phys → light → cold → fire)
  100 phys, 100% phys→light, 100% light→cold, 100% cold→fire → 100 fire
  With "+100% inc phys": 200 fire (phys tag inherited through entire chain)
  
Test 5: Gained-as-extra (non-destructive)
  100 phys, gain 20% as fire → 100 phys + 20 fire

Test 6: Mixed conversion + gained-as-extra
  100 phys, 50% phys→fire, gain 20% phys as fire → 50 phys + 70 fire
  (gained-as-extra applies to original amount before conversion reduces it)
```

---

## 3. Entropy-Based Evasion Counter

### Problem

PoE evasion does NOT use pure RNG. It uses an **entropy counter** system that
guarantees evenly-spaced hits and misses. If you have 70% evasion, you won't
randomly get hit 5 times in a row — the entropy system ensures roughly 3 evades
per 10 attacks, distributed evenly.

This matters for the combat simulator — we can't just roll random().

### Evasion Chance Formula

```
fn calculate_evasion_chance(evasion: f64, accuracy: f64) -> f64 {
  // PoE formula: attacker's accuracy vs defender's evasion
  // Base chance to hit = Accuracy / (Accuracy + (Evasion / 4)^0.8)
  // Clamped to [5%, 95%] (attacker always has 5% min chance to hit)
  let hit_chance = accuracy / (accuracy + (evasion / 4.0).powf(0.8));
  let hit_chance = hit_chance.clamp(0.05, 0.95);
  1.0 - hit_chance  // evasion chance = 1 - chance to hit
}
// Evasion is clamped to [5%, 95%] as well (defender always has min 5% chance to evade)
```

**Note:** Evasion only applies to **attack hits**. Spells bypass evasion entirely.
For spell mitigation, see Spell Suppression below.

### Spell Suppression

A parallel mechanic for spells: when a spell hits, there is a `spell_suppression_chance`
(default 0%, max 100% from tree/gear) of suppressing 50% of spell damage (or 100% with
"Glancing Blows"). This is NOT the entropy system — it uses independent RNG per spell hit.

```
On each incoming spell:
  if random() < player.spell_suppression_chance:
    suppress_factor = if player.has_glancing_blows { 1.0 } else { 0.5 }
    damage *= (1.0 - suppress_factor)  // 50% or 100% of spell damage suppressed
```

The combat simulator maintains separate suppression tracking from evasion entropy.

### Algorithm

```
State:
  entropy: f64 = 0.0  // accumulator, range [0, 100)

On each incoming attack:
  evasion_chance = calculate_evasion_chance(player.evasion, attacker.accuracy)
  // evasion_chance is in [5%, 95%] — PoE floor and cap

  entropy += evasion_chance * 100.0

  if entropy >= 100.0:
    entropy -= 100.0
    RESULT: EVADED (attack misses)
  else:
    RESULT: HIT (attack lands)
```

### Why Not Pure RNG

With 70% evasion and pure RNG:
- P(hit 3 times in a row) = 0.3³ = 2.7% — rare but happens
- P(hit 5 times in a row) = 0.3⁵ = 0.24% — RIP in hardcore

With entropy counter:
- After 2 consecutive hits, the counter has accumulated 140+ → next attack MUST evade
- Maximum consecutive hits ≈ ceil(100 / (evasion_chance × 100))
- At 70% evasion: max 2 consecutive hits, then guaranteed evade

### Integration Point

The combat simulator (Algorithm 20) maintains one entropy counter per enemy source.
Each enemy type has its own counter. Counters reset when changing zones.

### Complexity

- O(1) per attack check
- O(1) state (one f64 counter per attacker)

---

## 4. Leech Instance Manager

### Problem

PoE leech creates individual "leech instances" that each restore life/mana/ES over
time. There's a per-instance rate cap AND a total leech rate cap. Multiple instances
stack but the total rate is capped.

### Algorithm

```
State:
  instances: Vec<LeechInstance>  // active instances
  total_rate_cap: f64           // 20% of max life per second (default)
  per_instance_cap: f64         // 10% of max life per second (default)

struct LeechInstance {
  remaining: f64,      // remaining amount to leech
  rate: f64,           // rate per second (capped by per_instance_cap)
  start_time: f64,
}

On hit that leeches:
  leech_amount = hit_damage * leech_percent  // e.g., 2% of damage
  rate = min(leech_amount / duration, per_instance_cap)
  instances.push(LeechInstance { remaining: leech_amount, rate, start_time: now })

Per tick (e.g., every 100ms):
  // Calculate total leech this tick
  total_leech_this_tick = 0.0
  tick_duration = 0.1  // 100ms

  for instance in &mut instances:
    contribution = instance.rate * tick_duration
    contribution = min(contribution, instance.remaining)
    total_leech_this_tick += contribution

  // Cap total rate
  max_per_tick = total_rate_cap * tick_duration
  if total_leech_this_tick > max_per_tick:
    total_leech_this_tick = max_per_tick

  // Apply to life
  player.life += total_leech_this_tick
  player.life = min(player.life, player.max_life)

  // Consume from instances (proportionally if capped)
  if total_leech_this_tick < sum_of_contributions:
    ratio = total_leech_this_tick / sum_of_contributions
    for instance in &mut instances:
      instance.remaining -= instance_contribution * ratio
  else:
    for instance in &mut instances:
      instance.remaining -= instance_contribution

  // Remove exhausted instances
  instances.retain(|i| i.remaining > 0.001)

  // Remove instances when at full life (leech stops at full)
  if player.life >= player.max_life:
    instances.clear()  // unless Petrified Blood or similar
```

### Special Cases

| Keystone | Effect on Leech |
|----------|----------------|
| **Vaal Pact** | Double total rate cap (40%), but disable regen |
| **Ghost Reaver** | Life leech applies to ES instead |
| **Zealot's Oath** | Life regen → ES regen (doesn't affect leech) |
| **Petrified Blood** | Leech continues even at "full" life (overleech) |

### Steady-State Estimate (For Non-Simulation Use)

For the calculator (not combat sim), we estimate sustained leech rate:

```
effective_leech_rate = min(
  hit_rate × damage_per_hit × leech_percent,  // incoming leech
  total_rate_cap                               // cap
)
```

This is the number shown in the defense panel as "Life Leech: X/s".

---

## 5. Flask Uptime Model

### Problem

Flask uptime varies dramatically between mapping (constant kills → constant charges)
and bossing (no kills → limited charges). We need to model this for both the defense
calculator (sustained mitigation) and combat sim (real-time flask management).

### Algorithm — Steady-State Model (Calculator)

```
Input:
  flask: FlaskData  // charges, cost, duration, charge_gain_per_kill
  context: FightContext  // mapping or bossing

Output:
  uptime: f64  // 0.0-1.0, fraction of time flask is active

For MAPPING (kill-sustained):
  kills_per_second ≈ 5.0  // typical map clearing rate
  charges_per_second = kills_per_second * flask.charge_per_kill
  
  // How many uses does the flask support?
  uses_from_full = flask.max_charges / flask.use_cost
  seconds_to_refill = flask.use_cost / charges_per_second
  
  // If refill time < duration, flask is always up
  if seconds_to_refill <= flask.duration:
    uptime = 1.0
  else:
    uptime = flask.duration / (flask.duration + seconds_to_refill)

For BOSSING (charge-starved):
  // Most bosses have no adds → 0 kill-based charges
  // Some flasks gain charges on crit/hit (Pathfinder, enchant, mods)
  charges_per_second = flask.charge_on_hit * player.hit_rate
                     + flask.charge_on_crit * player.hit_rate * player.crit_chance
                     + flask.innate_charge_rate  // Pathfinder: 3 charges/3s

  if charges_per_second <= 0:
    // Fixed uses only (from pre-filled charges)
    total_uptime_seconds = uses_from_full * flask.duration
    fight_duration = estimate_boss_fight_time(player.dps, boss.hp)
    uptime = min(total_uptime_seconds / fight_duration, 1.0)
  else:
    seconds_to_refill = flask.use_cost / charges_per_second
    uptime = flask.duration / (flask.duration + seconds_to_refill)
```

### Algorithm — Discrete Event Model (Combat Sim)

The combat simulator tracks exact flask state per tick:

```
struct FlaskState {
  current_charges: f64,
  active: bool,
  remaining_duration_ms: u32,
}

per tick:
  // Charge gain from kills this tick
  state.current_charges += kills_this_tick * flask.charge_per_kill
  state.current_charges = min(state.current_charges, flask.max_charges)

  // Check if flask should be activated (AI logic)
  should_activate = !state.active
    && state.current_charges >= flask.use_cost
    && flask_needed(player, flask.type)  // e.g., life flask when < 60% HP

  if should_activate:
    state.current_charges -= flask.use_cost
    state.active = true
    state.remaining_duration_ms = flask.duration_ms

  // Tick active flask
  if state.active:
    apply_flask_effect(player, flask)
    state.remaining_duration_ms -= tick_ms
    if state.remaining_duration_ms <= 0:
      state.active = false
```

### Flask Activation AI (for combat sim)

```
fn flask_needed(player: &PlayerState, flask_type: FlaskType) -> bool {
  match flask_type {
    Life    => player.life_percent < 0.60,
    Mana    => player.mana_percent < 0.30,
    Utility => true,  // use on cooldown (granite, jade, etc.)
    Unique  => true,  // build-defining flasks → always up
  }
}
```

---

## 6. Build Archetype Classifier

### Data Structure Definitions

```rust
/// GemDatabase — trait injected into the classifier (hexagonal architecture).
/// Implemented by: JsonGemDatabase (prod), MockGemDatabase (tests).
pub trait GemDatabase: Send + Sync {
    /// Return the set of mechanic tags for a gem ID.
    /// Tags include: "minion", "attack", "spell", "damage_over_time", "burning",
    ///               "poison", "bleed", "projectile", "aoe", "totem", "trap", "mine",
    ///               "trigger", "channelling", "melee", "bow", "warcry", "brand"
    fn get_tags(&self, gem_id: &str) -> &[&'static str];

    /// Return the primary damage element for a gem.
    fn get_element(&self, gem_id: &str) -> DamageElement;

    /// Return true if this gem is a damage-dealing active skill (not a support/aura/utility).
    fn is_damage_skill(&self, gem_id: &str) -> bool;

    /// Return the trigger mechanism if the gem is triggered (CoC, CwC, Spellslinger, etc.)
    fn get_trigger(&self, gem_id: &str) -> Option<TriggerType>;
}

/// ARCHETYPE_DATABASE — O(1) direct lookup for known skill+ascendancy pairs.
/// This fires BEFORE the rule-based inference tree (Algorithm 6 Step 4) as an
/// optimization: 80% of popular builds match a direct key.
///
/// Key format: "{gem_id}+{ascendancy_class}"  (both lowercase, underscores)
/// Example:    "righteous_fire+inquisitor" → Archetype::FireDotRF
///
/// Populated from: game data + community meta analysis, updated per league.
/// Storage: static HashMap compiled into the binary (not loaded from disk).
static ARCHETYPE_DATABASE: phf::Map<&'static str, Archetype> = phf::phf_map! {
    "righteous_fire+inquisitor"         => Archetype::FireDotRF,
    "righteous_fire+chieftain"          => Archetype::FireDotRF,
    "ice_nova+assassin"                 => Archetype::CoCIceNova,     // CоC via Assassin
    "cyclone+slayer"                    => Archetype::AttackGeneric,
    "summon_skeletons+necromancer"      => Archetype::MinionSkeleton,
    "summon_spectres+necromancer"       => Archetype::MinionSpectre,
    "summon_raging_spirit+necromancer"  => Archetype::MinionSRS,
    "vortex+occultist"                  => Archetype::ColdDoT,
    "boneshatter+juggernaut"            => Archetype::Boneshatter,
    "lightning_arrow+deadeye"           => Archetype::AttackCrit,
    "tornado_shot+deadeye"              => Archetype::AttackCrit,
    "blade_vortex+assassin"             => Archetype::BladeVortex,
    "caustic_arrow+pathfinder"          => Archetype::CausticArrow,
    "seismic_trap+saboteur"             => Archetype::SeismicTrap,
    "absolution+necromancer"            => Archetype::MinionAbsolution,
    // ... ~200 more entries for all popular skill+ascendancy combinations
};
```

### Problem

Given a `BuildData`, determine the build archetype (e.g., `fire_dot`, `attack_crit`,
`cold_dot`, `minion`, `coc_ice_nova`). The archetype determines stat weights for
scoring, suggestion priorities, and Seer response context.

### Algorithm — Rule-Based Decision Tree

No ML. A decision tree with explicit rules based on skill gem tags, ascendancy,
and configured skills.

```
Input:
  build: BuildData  // items, gems, tree, ascendancy

Output:
  archetype: Archetype
  main_skill: GemId
  dps_type: DpsType  // Attack, Spell, DoT, Minion, Trigger

// === FAST PATH: direct lookup for known skill+ascendancy pairs (O(1)) ===
  // Covers ~80% of popular builds without running the full inference tree.
  // Key = "{gem_id}+{ascendancy}", e.g. "righteous_fire+inquisitor"
  let fast_key = format!("{}+{}",
    build.config_main_skill.to_lowercase(),
    build.ascendancy.to_lowercase().replace(' ', "_"));
  if let Some(&archetype) = ARCHETYPE_DATABASE.get(fast_key.as_str()) {
    return ClassifyResult { archetype,
      main_skill: build.config_main_skill,
      dps_type: archetype.canonical_dps_type() }
  }
  // === SLOW PATH: rule-based inference for unknown/hybrid builds ===

Step 1: IDENTIFY MAIN SKILL
  // The main skill is the one in the highest-linked group that deals damage.
  // If user set an active skill in PoB config, use that.
  // Otherwise, infer:
  
  candidates = []
  for each socket_group in build.socket_groups:
    active_gem = socket_group.gems.find(|g| g.is_active_skill && g.is_damage_skill)
    if active_gem:
      link_count = socket_group.gems.len()
      candidates.push((active_gem, link_count))
  
  // Sort by link count descending, break ties by gem level
  candidates.sort_by(|a, b| b.links.cmp(&a.links).then(b.gem_level.cmp(&a.gem_level)))
  main_skill = candidates[0]

Step 2: DETERMINE DPS TYPE from main skill tags
  tags = gem_db.get_tags(main_skill.id)
  
  if "minion" in tags:
    dps_type = Minion
  else if main_skill is triggered (CoC, CWC, Spellslinger):
    dps_type = Trigger
  else if "damage_over_time" in tags or "burning" in tags or "poison" in tags:
    dps_type = DoT
  else if "attack" in tags:
    dps_type = Attack
  else if "spell" in tags:
    dps_type = Spell

Step 3: DETERMINE DAMAGE ELEMENT
  // Check skill gem damage types + conversion from tree/items
  if skill deals fire or has fire conversion:
    element = Fire
  else if cold: element = Cold
  else if lightning: element = Lightning
  else if chaos/poison: element = Chaos
  else: element = Physical

Step 4: CLASSIFY ARCHETYPE using decision table
  archetype = match (dps_type, element, main_skill, ascendancy):
    // --- DoT archetypes ---
    (DoT, Fire, "righteous_fire", _)         → FireDotRF
    (DoT, Fire, _, _)                        → FireDotGeneric
    (DoT, Cold, _, _)                        → ColdDoT
    (DoT, Chaos, _, _) if has_poison_stacks  → PoisonDoT
    (DoT, Physical, _, _) if has_bleed       → BleedDoT

    // --- Attack archetypes ---
    (Attack, _, _, _) if crit_chance > 30%   → AttackCrit
    (Attack, _, _, _) if has_resolute_tech   → AttackRT
    (Attack, _, _, "Champion")               → AttackChampion
    (Attack, _, _, _)                        → AttackGeneric

    // --- Spell archetypes ---
    (Spell, _, _, _) if crit_chance > 30%    → SpellCrit
    (Spell, _, _, _)                         → SpellGeneric

    // --- Minion archetypes ---
    (Minion, _, "raise_spectre", _)          → MinionSpectre
    (Minion, _, "summon_skeleton", _)         → MinionSkeleton
    (Minion, _, _, "Necromancer")            → MinionGeneric
    (Minion, _, _, _)                        → MinionGeneric

    // --- Trigger archetypes ---
    (Trigger, Cold, _, _) if has_coc         → CoCIceNova
    (Trigger, _, _, _) if has_cwc            → CWC
    (Trigger, _, _, _)                       → TriggerGeneric

    // --- Channel archetypes ---
    (Spell, _, "blade_vortex", _)            → BladeVortex
    (Spell, Chaos, "dark_pact", _)           → DarkPact
    (DoT, Chaos, "caustic_arrow", _)         → CausticArrow
    (DoT, Chaos, "blight", _)               → BlightDoT

    // --- Trap/Mine archetypes ---
    (_, _, _, _) if has_traps && main_skill has "trap" tag    → TrapGeneric
    (_, _, _, _) if has_mines && main_skill has "mine" tag    → MineGeneric
    (_, Physical, "seismic_trap", _)         → SeismicTrap
    (_, Chaos, "exsanguinate", _) if has_trap → ChainableTrap

    // --- Ward build ---
    (_, _, _, _) if build.primary_defense == Ward
                 && build.ward >= 1000        → WardLoop

    // --- Impending Doom trigger ---
    (Spell, Chaos, _, _) if has_impending_doom → ImpendingDoom

    // --- Summon archetypes (extended) ---
    (Minion, _, "summon_raging_spirit", _)   → MinionSRS
    (Minion, Physical, "animate_weapon", _)  → MinionAnimateWeapon
    (Minion, _, "absolution", _)             → MinionAbsolution
    (Minion, _, _, "Guardian") if has_agressive_minions → MinionGuardian

    // --- Warcry / Melee ---
    (Attack, Physical, _, _) if has_exerted_attacks → WarcryMelee
    (Attack, Physical, "boneshatter", _)     → Boneshatter

    // --- Fallback ---
    _                                        → Generic
```

### Why Not ML

- Deterministic: same build always gets same archetype
- Debuggable: we can trace exactly why a build was classified as X
- Maintainable: new archetype = add one rule, not retrain a model
- No training data needed
- 100% accuracy for known archetypes (versus ~95% for a classifier)

### Test Cases

```
RF Inquisitor (main skill RF, fire DoT) → FireDotRF
Boneshatter Juggernaut (attack, phys, exerted, RT) → Boneshatter
Ice Nova Assassin (CoC triggered, cold, high crit) → CoCIceNova
Summon Raging Spirit Necro (minion, fire) → MinionSRS
Vortex Occultist (cold DoT) → ColdDoT
Lightning Arrow Deadeye (attack, lightning, crit) → AttackCrit
Poison Blade Vortex Assassin (DoT, chaos, poison) → PoisonDoT
Seismic Trap Saboteur (trap, physical) → SeismicTrap
Caustic Arrow Pathfinder (DoT, chaos, CA) → CausticArrow
Ward Loop Inquisitor (ward ≥ 1000, primary defense=ward) → WardLoop
Absolution Necromancer (minion, absolution skill) → MinionAbsolution
Impending Doom Occultist (spell, chaos, doom trigger) → ImpendingDoom
```

---

## 7. Item Scoring — Archetype-Adaptive Weighted Scoring

### Data Structures

```rust
/// Weight vector for one archetype. Keyed by StatType enum.
/// Values are dimensionless importance weights — calibrated so that
/// T1 of the most-important stat contributes ~30 raw score points,
/// and T1 of a resist contributes ~5 points. Normalised to 0-100 by
/// expected_perfect_score() (see below).
pub type ArchetypeWeights = HashMap<StatType, f64>;

/// Global lookup: archetype → its weight vector.
/// Loaded once at startup from `data/archetype_weights.json`.
/// Updated per-league if meta shifts (via background update system).
pub struct ArchetypeWeightTable {
    weights: HashMap<Archetype, ArchetypeWeights>,
}

impl ArchetypeWeightTable {
    /// O(1) lookup. Returns the weight for a stat in this archetype.
    /// Returns 0.0 for stats not in the table (dead mods).
    pub fn get(&self, archetype: Archetype, stat: StatType) -> f64 {
        self.weights
            .get(&archetype)
            .and_then(|w| w.get(&stat))
            .copied()
            .unwrap_or(0.0)
    }
}

/// expected_perfect_score — cached, not recomputed per item.
/// Key: (Slot, Archetype, ItemLevel bucket [0,60,75,82,86]).
/// Populated lazily on first access for each (slot, archetype) pair.
/// Invalidated only when ARCHETYPE_WEIGHTS data changes (league update).
static PERFECT_SCORE_CACHE: OnceLock<DashMap<(EquipSlot, Archetype, u8), f64>> =
    OnceLock::new();

pub fn expected_perfect_score(slot: EquipSlot, archetype: Archetype, ilvl: u8) -> f64 {
    let ilvl_bucket = match ilvl { 0..=59 => 0, 60..=74 => 60, 75..=81 => 75,
                                   82..=85 => 82, _ => 86 };
    let cache = PERFECT_SCORE_CACHE.get_or_init(DashMap::new);
    *cache.entry((slot, archetype, ilvl_bucket)).or_insert_with(|| {
        // Sum the top-3 prefix + top-3 suffix weights at T1 max value
        let possible = mod_db.available_for(slot.base_tags(), ilvl_bucket);
        let prefixes: f64 = possible.iter().filter(|m| m.is_prefix)
            .map(|m| ARCHETYPE_WEIGHTS.get(archetype, m.stat_type))
            .sorted_by(|a,b| b.partial_cmp(a).unwrap()).take(3).sum();
        let suffixes: f64 = possible.iter().filter(|m| m.is_suffix)
            .map(|m| ARCHETYPE_WEIGHTS.get(archetype, m.stat_type))
            .sorted_by(|a,b| b.partial_cmp(a).unwrap()).take(3).sum();
        prefixes + suffixes
    })
}
```

### Problem

Score each equipped item 0-100 so the player sees which slot is weakest. The score
must reflect how good the item is **for this specific build**, not generically.

### Algorithm

```
Input:
  item: Item           // the equipped item with parsed mods
  archetype: Archetype // from Algorithm 6
  slot: EquipSlot      // helmet, ring1, etc.
  mod_db: ModDatabase  // tier info for every mod

Output:
  score: f64  // 0-100

Algorithm:
  weights = ARCHETYPE_WEIGHTS[archetype]
  
  // 1. Score each mod on the item
  raw_score = 0.0
  for mod in item.explicit_mods:
    // Look up what tier this mod value corresponds to
    tier = mod_db.get_tier(mod.stat_id, mod.value, item.base_type, item.ilvl)
    
    // Tier factor: T1 = full weight, T5+ = almost nothing
    tier_factor = match tier.rank:
      1 => 1.00
      2 => 0.85
      3 => 0.65
      4 => 0.40
      5 => 0.20
      6 => 0.10
      _ => 0.05
    
    // Value factor: where in the tier range is this roll?
    // T1 life is 80-89 → a roll of 85 is (85-80)/(89-80) = 0.56
    value_factor = (mod.value - tier.min) / (tier.max - tier.min)
    value_factor = value_factor.clamp(0.0, 1.0)
    
    // Weight from archetype (how important is this stat?)
    stat_weight = weights.get(mod.stat_type).unwrap_or(0.0)
    
    // Combined mod score
    mod_score = stat_weight * tier_factor * (0.5 + 0.5 * value_factor)
    raw_score += mod_score
  
  // 2. Bonus for implicit mods (base type value)
  for mod in item.implicit_mods:
    stat_weight = weights.get(mod.stat_type).unwrap_or(0.0)
    raw_score += stat_weight * 0.3  // implicits worth ~30% of an explicit
  
  // 3. Bonus for open affixes (potential for benchcraft)
  open_prefixes = 3 - item.prefix_count
  open_suffixes = 3 - item.suffix_count
  if open_prefixes > 0: raw_score += 2.0  // open prefix = benchcraft opportunity
  if open_suffixes > 0: raw_score += 2.0
  
  // 4. Penalty for dead mods (mods with 0 weight for this archetype)
  dead_mods = item.explicit_mods.count(|m| weights.get(m.stat_type) == Some(0.0))
  raw_score -= dead_mods * 3.0  // each dead mod = wasted affix slot
  
  // 5. Normalize to 0-100
  // expected_max = theoretical perfect item for this slot and archetype
  expected_max = expected_perfect_score(slot, archetype, item.ilvl)
  score = (raw_score / expected_max * 100.0).clamp(0.0, 100.0)
  
  return score.round()
```

### Archetype Weight Tables

Each archetype has a weight vector. Here's the structure (values from ENGINE-DESIGN.md):

```
Stat Type              | fire_dot | attack_crit | cold_dot | minion | spell_crit | poison_dot | bleed_dot | ward_loop
-----------------------|----------|-------------|----------|--------|------------|------------|-----------|----------
flat_life              |     1.2  |        1.0  |     1.2  |    1.0 |       1.0  |       1.5  |      1.5  |      0.0
percent_life           |     4.0  |        3.0  |     4.0  |    3.5 |       3.0  |       5.0  |      5.0  |      0.0
flat_es                |     0.0  |        0.0  |     0.0  |    0.0 |       2.0  |       0.0  |      0.0  |      0.0
percent_es             |     0.0  |        0.0  |     0.0  |    0.0 |       4.0  |       0.0  |      0.0  |      0.0
ward                   |     0.0  |        0.0  |     0.0  |    0.0 |       0.0  |       0.0  |      0.0  |     25.0
fire_dot_multi         |    15.0  |        0.0  |     0.0  |    0.0 |       0.0  |       0.0  |      0.0  |      0.0
cold_dot_multi         |     0.0  |        0.0  |    15.0  |    0.0 |       0.0  |       0.0  |      0.0  |      0.0
chaos_dot_multi        |     0.0  |        0.0  |     0.0  |    0.0 |       0.0  |      15.0  |      0.0  |      0.0
phys_dot_multi         |     0.0  |        0.0  |     0.0  |    0.0 |       0.0  |       0.0  |     15.0  |      0.0
fire_damage            |     8.0  |        0.0  |     0.0  |    0.0 |       0.0  |       0.0  |      0.0  |      0.0
cold_damage            |     0.0  |        0.0  |     8.0  |    0.0 |       0.0  |       0.0  |      0.0  |      0.0
chaos_damage           |     0.0  |        0.0  |     0.0  |    0.0 |       0.0  |       8.0  |      0.0  |      0.0
spell_damage           |     0.0  |        0.0  |     0.0  |    0.0 |       7.0  |       0.0  |      0.0  |      0.0
flat_physical          |     0.0  |       10.0  |     0.0  |    0.0 |       0.0  |       0.0  |      8.0  |      0.0
attack_speed           |     0.0  |        8.0  |     0.0  |    0.0 |       0.0  |       0.0  |      6.0  |      0.0
cast_speed             |     0.0  |        0.0  |     0.0  |    0.0 |       5.0  |       3.0  |      0.0  |      0.0
crit_chance            |     0.0  |        6.0  |     0.0  |    0.0 |       6.0  |       0.0  |      0.0  |      0.0
crit_multi             |     0.0  |        5.0  |     0.0  |    0.0 |       5.0  |       0.0  |      0.0  |      0.0
poison_chance          |     0.0  |        0.0  |     0.0  |    0.0 |       0.0  |       6.0  |      0.0  |      0.0
bleed_chance           |     0.0  |        0.0  |     0.0  |    0.0 |       0.0  |       0.0  |      6.0  |      0.0
gem_level_fire         |    20.0  |        0.0  |     0.0  |    0.0 |       0.0  |       0.0  |      0.0  |      0.0
gem_level_cold         |     0.0  |        0.0  |    20.0  |    0.0 |       0.0  |       0.0  |      0.0  |      0.0
gem_level_active       |     0.0  |        0.0  |     0.0  |    0.0 |      18.0  |      10.0  |     10.0  |      0.0
gem_level_minion       |     0.0  |        0.0  |     0.0  |   20.0 |       0.0  |       0.0  |      0.0  |      0.0
minion_damage          |     0.0  |        0.0  |     0.0  |   12.0 |       0.0  |       0.0  |      0.0  |      0.0
fire_resistance        |     0.3  |        0.3  |     0.3  |    0.3 |       0.3  |       0.3  |      0.3  |      0.5
cold_resistance        |     0.3  |        0.3  |     0.3  |    0.3 |       0.3  |       0.3  |      0.3  |      0.5
lightning_resistance   |     0.3  |        0.3  |     0.3  |    0.3 |       0.3  |       0.3  |      0.3  |      0.5
chaos_resistance       |     0.8  |        0.8  |     0.8  |    0.8 |       0.8  |       0.8  |      0.8  |      1.0
movement_speed         |     3.0  |        3.0  |     3.0  |    3.0 |       3.0  |       3.0  |      3.0  |      3.0
armour (global)        |     0.05 |        0.05 |     0.05 |   0.05 |      0.05  |      0.05  |     0.05  |      0.0
spell_suppression      |     1.5  |        1.5  |     1.5  |    1.5 |       1.5  |       1.5  |      1.5  |      0.0
```

**Weight calibration:** Weights are set so that a T1 roll of the most important
stat (e.g., fire_dot_multi for RF, ward for ward_loop) contributes roughly the same
score as 2-3 T1 rolls of medium-importance stats (e.g., life + resist). This ensures
the scoring reflects actual build impact.

**Ward loop note:** Ward items score `ward` stat at weight 25 because ward value
*directly* determines survivability — a single high ward item can be more impactful
than 500 flat life. Resists still matter but armor/evasion are irrelevant.

### Expected Perfect Score Calculation

```
expected_perfect_score(slot, archetype, ilvl):
  // What would a BiS item look like for this slot?
  // Sum the top 3 prefix weights + top 3 suffix weights × T1 factor
  possible_mods = mod_db.available_for(slot.base_tags, ilvl)
  prefix_mods = possible_mods.filter(|m| m.is_prefix).sort_by_weight(archetype).take(3)
  suffix_mods = possible_mods.filter(|m| m.is_suffix).sort_by_weight(archetype).take(3)
  
  max = 0.0
  for m in prefix_mods + suffix_mods:
    max += weights[m.stat_type] * 1.0 * 1.0  // T1 tier_factor × max value_factor
  return max
```

---

## 8. Issue Detector — Constraint Violation Scanner

### Problem

Scan a build for problems the player might not notice: uncapped resists, missing
ailment immunity, stat requirements not met, low chaos res, etc.

### Algorithm

A series of constraint checks, each producing an Issue with severity.

```
Input:
  build: BuildData
  calc: CalcResult  // pre-computed stats

Output:
  issues: Vec<Issue>

struct Issue {
  severity: Severity,  // Critical, Warning, Info
  category: IssueCategory,
  message: String,
  remedy: String,
}

Checks (ordered by severity):

// === CRITICAL: Build is broken ===
if calc.defense.fire_res < 75:
  issues.push(Critical, "Fire resistance uncapped at {fire_res}%")
if calc.defense.cold_res < 75:
  issues.push(Critical, "Cold resistance uncapped")
if calc.defense.lightning_res < 75:
  issues.push(Critical, "Lightning resistance uncapped")
if any stat_requirement not met by total stats:
  issues.push(Critical, "Cannot equip {item}: needs {req} {stat}, you have {actual}")
if build.life < 3000 && build.level >= 70:
  issues.push(Critical, "Life dangerously low at {life}")

// === WARNING: Significant weakness ===
// Chaos resistance has NO hard cap (unlike elemental 75%). Soft thresholds:
//   < 0%  → taking amplified chaos damage — dangerous in endgame content
//   0-50% → vulnerable to chaos damage sources (Sirus, Maven, Vaal zones)
//   50%+  → reasonably protected for most content
//   75%   → fully capped (achievable but usually requires gear sacrifices)
// We warn at <0% (critical danger) and also flag 0-50% as a soft warning.
if calc.defense.chaos_res < 0:
  issues.push(Warning, "Negative chaos resistance ({chaos_res}%) — taking amplified chaos damage")
if calc.defense.chaos_res >= 0 && calc.defense.chaos_res < 50:
  issues.push(Info, "Low chaos resistance ({chaos_res}%) — vulnerable to Sirus, Maven, Vaal zones")
if calc.defense.overcap(fire) < 10 || overcap(cold) < 10 || overcap(light) < 10:
  issues.push(Warning, "Low resist overcap — Ele Weakness maps strip to {stripped_value}%")
if !calc.defense.freeze_immune:
  issues.push(Warning, "Not freeze immune — dangerous in all content")
if !calc.defense.stun_immune && build.is_ci:
  issues.push(Warning, "CI without stun immunity — lethal stun locks")
if !calc.defense.shock_immune:
  issues.push(Warning, "Shock vulnerable — up to 50% more damage taken")
if !calc.defense.bleed_immune && dps_type == Attack:
  issues.push(Warning, "No bleed immunity — corrupted blood kills you")
if calc.defense.life_regen_net < 0 && main_skill == "righteous_fire":
  issues.push(Warning, "RF degen exceeds regen — you are dying to your own skill")
if open_benchcraft_slots > 0:
  issues.push(Warning, "{slot} has open {prefix/suffix} — free benchcraft available")
if !calc.defense.has_guard_skill:
  issues.push(Warning, "No guard skill (Molten Shell/Steelskin/Arcane Cloak) — burst mitigation gap")
if !build.has_movement_skill:
  issues.push(Warning, "No movement skill equipped — dangerous for boss mechanics")

// === Ward-specific checks ===
if archetype == WardLoop:
  if calc.defense.ward < 1000:
    issues.push(Critical, "Ward loop requires ≥1000 ward — currently {ward}")
  if !build.has_unique("Olroth's Resolve"):
    issues.push(Critical, "Ward loop requires Olroth's Resolve flask")

// === Mana sustainability ===
if build.mana_cost_per_second > calc.mana_regen_per_second * 1.1
   && !build.has_mana_flask_on_cooldown:
  issues.push(Warning, "Mana unsustainable — cost {cost}/s exceeds regen {regen}/s")
if calc.mana_reservation_percent > 95:
  issues.push(Critical, "Mana over-reserved ({pct}%) — cannot cast main skill")
if calc.mana_reservation_percent > 85:
  issues.push(Warning, "Very high mana reservation ({pct}%) — no buffer for spells")

// === Defense layer checks ===
let defense_layers = [calc.defense.has_armour, calc.defense.has_evasion,
                      calc.defense.has_es, calc.defense.has_ward]
if defense_layers.iter().all(|d| !d):
  issues.push(Critical, "No primary defense layer — pure life build with no mitigation")
if !build.is_ci && calc.defense.life < 2000 && calc.defense.max_es < 3000:
  issues.push(Critical, "Combined HP pool too low (life: {life}, ES: {es})")

// === Resist over-cap check (for map mods) ===
// Always use calc.defense.overcap() — never access calc.fire_res directly.
// calc.defense contains capped values; raw resistances live in ModPool.
let min_overcap = [
    calc.defense.overcap(Element::Fire),
    calc.defense.overcap(Element::Cold),
    calc.defense.overcap(Element::Lightning),
  ].iter().cloned().fold(f64::MAX, f64::min)
if min_overcap < 15.0:
  issues.push(Warning,
    "Only {min_overcap}% resist overcap — Ele Weakness curse will uncap {element} res")

// === INFO: Optimization opportunity ===
if any gem is level < 20 && gem.max_level == 20:
  issues.push(Info, "{gem} is only level {level} — level 20 gives +X%")
if any gem is 20/20 and not corrupted:
  issues.push(Info, "{gem} is 20/20 — corrupt for 21/20 chance (+X%)")
if unallocated_passive_points > 0:
  issues.push(Info, "{N} unallocated passive points")
if jewel_socket_reachable && empty:
  issues.push(Info, "Empty jewel socket reachable in {N} points")
if archetype == AttackCrit || archetype == SpellCrit:
  if build.has_power_charge_passive && build.current_power_charges == 0:
    issues.push(Info, "No power charge generation — power charge nodes allocated but charges never gained")
if !calc.defense.freeze_immune && !calc.defense.chill_immune:
  // Info only — many builds skip ailment immunity intentionally (softcore mapping)
  issues.push(Info, "No freeze/chill immunity — consider Brine King pantheon for bossing")
if !calc.defense.shock_immune:
  issues.push(Info, "No shock immunity — consider Arakaali pantheon or Stormshroud unique")
```

### Severity Levels

| Severity | Meaning | UI Treatment |
|----------|---------|-------------|
| Critical | Build is broken or will die instantly | Red highlight, shown first |
| Warning | Significant weakness, should fix | Orange, shown in issues panel |
| Info | Optimization opportunity, not urgent | Gray, collapsed by default |

---

## 9. Sensitivity Analysis — Numerical Partial Derivatives

### Problem

For a given build, which stats have the highest marginal impact on DPS? This tells
the player what to prioritize on gear.

### Algorithm

Compute the numerical partial derivative of DPS with respect to each stat by
perturbing it slightly and measuring the change.

```
Input:
  build: BuildData
  calc: PathCalcEngine

Output:
  sensitivities: Vec<(StatType, f64)>  // sorted by DPS impact descending

Algorithm:
  base_dps = calc.calculate(build).offense.total_dps
  DELTA = 0.10  // 10% perturbation

  for stat in ALL_RELEVANT_STATS:
    // Create modified build with +10% of this stat
    modified = build.clone()
    
    match stat:
      FlatLife:      modified.add_flat(life, current_flat_life * DELTA)
      PercentLife:   modified.add_increased(life, 10.0)  // +10% increased
      FireDotMulti:  modified.add_dot_multi(fire, 10.0)
      AttackSpeed:   modified.add_increased(attack_speed, 10.0)
      CritChance:    modified.add_increased(crit_chance, 10.0)
      CritMulti:     modified.add_flat(crit_multi, 15.0)  // +15% multi
      GemLevel:      modified.add_gem_level(main_skill, 1)
      // Gem level DPS gain is non-linear — varies by current level.
      // Use the per-level multiplier table rather than a flat perturbation:
      //   Level 17 → 18: +10% base damage
      //   Level 18 → 19: +11%
      //   Level 19 → 20: +12%
      //   Level 20 → 21: +13%  (corruption required)
      //   Level 21 → 22: +14%
      //   Level 22 → 23: +15%
      //   Level 23 → 24: +16%
      // (from game datamined per-gem scaling tables; varies slightly per gem)
      // For sensitivity display: report as "gem_level_gain_pct_at_current_level"
      // so users see "RF level 20→21 gives +13% DPS" not a generic perturbation.
      GemLevel:      modified.add_gem_level(main_skill, 1)

      // ... etc

    new_dps = calc.calculate(&modified).offense.total_dps
    delta_dps = (new_dps - base_dps) / base_dps * 100.0  // % change

    sensitivities.push((stat, delta_dps))

  sensitivities.sort_by(|a, b| b.1.partial_cmp(&a.1))

  // Also calculate diminishing returns indicator
  // If a stat already has high total, additional % is worth less
  for (stat, _) in &mut sensitivities:
    current_total = build.total_increased(stat)
    diminishing = if current_total > 300.0 { "⚠ diminishing" }
                  else if current_total > 200.0 { "moderate returns" }
                  else { "high returns" }
    stat.diminishing_note = diminishing
```

### Output Example

```
Stat Sensitivity for RF Inquisitor:
  1. Fire DoT Multiplier: +10% → +5.6% DPS [high returns, you have 180%]
  2. Gem Level (+1 fire): → +4.2% DPS [high returns]
  3. Increased Fire Damage: +10% → +1.9% DPS [⚠ diminishing, you have 420%]
  4. Movement Speed: +10% → +0% DPS [but affects clear speed]
```

### Complexity

- Time: O(S × T_calc) where S = ~15 stats, T_calc = ~50ms. Total: ~750ms.
- Cache: Run once on build load, invalidate on build change.
- Parallelizable: each stat perturbation is independent → tokio::spawn all 15.

---

## 10. Pareto-Optimal Upgrade Ranking

### Problem

Given N upgrade suggestions, each with multiple objective values (DPS gain, life
gain, resist change, cost), find the set of **non-dominated** solutions (Pareto
frontier). No single ranking captures all trade-offs.

### Algorithm

```
Input:
  suggestions: Vec<Suggestion>
  // Each has: dps_change, life_change, resist_change, cost, risk_level

Output:
  frontier: Vec<RankedSuggestion>  // non-dominated solutions
  dominated: Vec<RankedSuggestion>  // dominated solutions (still shown, ranked lower)

Algorithm:
  frontier = []
  dominated = []

  for s in suggestions:
    // Check if s is dominated by ANY other suggestion
    is_dominated = suggestions.iter().any(|other| {
      other != s
      && other.dps_change >= s.dps_change
      && other.life_change >= s.life_change
      && other.resist_change >= s.resist_change
      && other.cost <= s.cost
      // Strictly better in at least one dimension
      && (other.dps_change > s.dps_change
          || other.life_change > s.life_change
          || other.resist_change > s.resist_change
          || other.cost < s.cost)
    })

    if is_dominated:
      dominated.push(s)
    else:
      // Label what this suggestion is best for
      s.best_for = determine_label(s, suggestions)
      frontier.push(s)

  // Sort frontier by user-selected criterion (default: DPS per divine)
  // UI provides sort toggle: [Best Value] [Max DPS] [Survival] [Cheapest]

fn determine_label(s, all):
  if s.cost == 0:                    return "Free"
  if s == max_by(dps_per_divine):    return "Best Value"
  if s == max_by(dps_change):        return "Max DPS"
  if s == max_by(life_change):       return "Max Survival"
  if s == min_by(cost):              return "Cheapest"
  if s unlocks a boss:               return "Boss Unlock"
  return "Balanced"
```

### Complexity

- Naive: O(N²) pairwise comparison. For N ≈ 50 suggestions, this is instant.
- If N ever grows large: use KD-tree for dominance checking, O(N log^(d-1) N).

---

## 11. Budget-Constrained Upgrade Optimizer (Knapsack)

### Problem

Player has B divines. Which combination of upgrades maximizes total DPS gain
without exceeding budget? Additional constraints: one upgrade per slot, must
maintain resist caps.

### Algorithm — 0/1 Knapsack with Slot Constraints

```
Input:
  upgrades: Vec<Upgrade>  // each has: slot, cost (integer divines), dps_gain, life_gain
  budget: u32             // total divine orbs available
  build: BuildData        // for constraint validation

Output:
  selected: Vec<Upgrade>  // optimal subset within budget

Algorithm:
  // Group upgrades by slot (only one per slot)
  slots: HashMap<Slot, Vec<Upgrade>> = group_by_slot(upgrades)
  
  // Convert to grouped knapsack: at most 1 item from each group
  // dp[w] = (max_dps_gain, selection_mask)
  dp = vec![(0.0, vec![]); budget + 1]

  for (slot, slot_upgrades) in slots:
    // Process each slot group
    // Iterate budget in reverse to avoid using same slot twice
    new_dp = dp.clone()
    
    for upgrade in slot_upgrades:
      cost = upgrade.cost as usize
      for w in (cost..=budget as usize).rev():
        candidate_gain = dp[w - cost].0 + upgrade.dps_gain
        if candidate_gain > new_dp[w].0:
          // Validate: does this combination break resists?
          let mut test_selection = dp[w - cost].1.clone()
          test_selection.push(upgrade.clone())
          
          if validate_resist_caps(build, &test_selection):
            new_dp[w] = (candidate_gain, test_selection)
    
    dp = new_dp

  selected = dp[budget as usize].1
```

### Constraint Validation

Before accepting a combination, verify:
```
fn validate_resist_caps(build: &BuildData, upgrades: &[Upgrade]) -> bool {
  // Simulate applying all upgrades
  let modified = build.clone()
  for u in upgrades:
    modified.swap_item(u.slot, u.new_item)
  
  let calc = calculator.calculate(&modified)
  calc.fire_res >= 75 && calc.cold_res >= 75 && calc.lightning_res >= 75
}
```

### Complexity

- Time: O(S × U × B) where S = slots (~10), U = upgrades per slot (~5), B = budget
- For typical values: 10 × 5 × 50 = 2,500 operations. Instant.
- With constraint validation: each validation is O(M) for modifier aggregation,
  but only called for improving candidates, so amortized cost is low.

### Output Example

```
Budget: 20 divine
Optimal set:
  Ring 2 craft (3d): +15.3% DPS, +350 life
  Gem corruptions ×5 (5d): +12% DPS
  Cluster jewel (10d): +8% DPS, +200 life
  Total: 18d spent, +35.3% DPS, +550 life
  Remaining: 2d unspent

Alternative (survival-focused):
  Ring 2 with chaos res (3d): +5% DPS, +350 life, +35% chaos res
  Boots upgrade (8d): +500 life, +ailment immunity
  Aegis Aurora (18d): OVER BUDGET — need 7 more divines
```

---

## 12. Multi-Slot Constraint Solver

### Problem

The knapsack optimizer treats each slot independently. But in PoE, slots are
**interdependent**: changing a ring affects total resists, which may require
compensating on another slot. Truly optimal solutions require considering
cross-slot interactions.

### Algorithm — Greedy Hill Climbing with Constraint Repair

Full constraint optimization across all slots is combinatorially explosive.
We use a practical greedy approach with repair.

```
Input:
  build: BuildData
  candidate_upgrades: HashMap<Slot, Vec<Upgrade>>
  budget: u32
  constraints: [fire_res >= 75, cold_res >= 75, light_res >= 75, chaos_res >= -60]

Output:
  upgrade_plan: Vec<(Slot, Upgrade)>  // ordered steps

Algorithm:
  plan = []
  remaining_budget = budget
  current_build = build.clone()

  loop:
    // Find the single best upgrade we can afford right now
    best = None
    best_score = 0.0

    for (slot, upgrades) in candidate_upgrades:
      if slot already in plan: continue  // one upgrade per slot
      for upgrade in upgrades:
        if upgrade.cost > remaining_budget: continue
        
        // Simulate this upgrade
        test_build = current_build.with_swap(slot, upgrade)
        test_calc = calculator.calculate(&test_build)
        
        // Check constraints
        if !meets_constraints(test_calc, constraints):
          // Try REPAIR: can we fix the constraint by benchcrafting?
          repaired = attempt_repair(test_build, constraints)
          if repaired.is_none(): continue  // can't fix → skip
          test_build = repaired
          test_calc = calculator.calculate(&test_build)
        
        // Score: weighted combination of DPS gain + life gain
        score = (test_calc.dps - current_dps) / upgrade.cost.max(1) as f64
        if score > best_score:
          best = Some((slot, upgrade))
          best_score = score
    
    if best.is_none(): break  // no more improvements possible
    
    // Apply best upgrade
    let (slot, upgrade) = best.unwrap()
    plan.push((slot, upgrade))
    current_build = current_build.with_swap(slot, upgrade)
    remaining_budget -= upgrade.cost
    current_dps = calculator.calculate(&current_build).offense.total_dps
  
  return plan

fn attempt_repair(build: &BuildData, constraints: &[Constraint]) -> Option<BuildData> {
  // If resists are uncapped, try adding resist benchcrafts to open affixes
  let calc = calculator.calculate(&build)
  
  if calc.fire_res < 75:
    // Find an item with open suffix → benchcraft fire res
    for item in build.items:
      if item.has_open_suffix():
        let repaired = build.add_benchcraft(item.slot, "fire_res_benchcraft")
        if calculator.calculate(&repaired).fire_res >= 75:
          return Some(repaired)
  // ... similar for cold, lightning
  
  None  // can't repair
}
```

### Why Not Brute Force

With 9 slots × 5 options each = 5⁹ ≈ 2 million combinations. Feasible to brute
force, BUT each combination requires a full calculator pass (~50ms) = 100,000
seconds. Too slow.

Greedy hill climbing with ~50 iterations × ~50 candidates × 50ms = ~125 seconds.
Still slow for interactive use, but we can:
1. Use Tier 1 fast estimates for initial filtering
2. Only full-calc the top 5 candidates per iteration
3. Cache calculator results for unchanged builds

Result: ~2-5 seconds for a full multi-slot optimization.

---

## 13. Passive Tree Pathfinding — Modified Dijkstra

### Problem

The PoE passive tree is a graph with ~1,300 nodes. Given the player's current
allocation, find the optimal path to a target node. "Optimal" means minimizing
total cost while considering that travel nodes have value too.

### Data Structure

```rust
struct PassiveTree {
    nodes: HashMap<NodeId, PassiveNode>,
    edges: HashMap<NodeId, Vec<NodeId>>,  // adjacency list
}

struct PassiveNode {
    id: NodeId,
    name: String,
    stats: Vec<Modifier>,     // what this node gives
    is_notable: bool,
    is_keystone: bool,
    is_mastery: bool,
    is_jewel_socket: bool,
    class_start: Option<Class>,
}
```

### Algorithm — A* with Opportunity Cost Heuristic

```
Input:
  tree: PassiveTree
  allocated: HashSet<NodeId>  // currently allocated nodes
  target: NodeId              // goal node
  build: BuildData
  calc: PathCalcEngine

Output:
  path: Vec<NodeId>      // nodes to allocate (in order)
  cost: usize            // passive points required
  value: f64             // total stat value gained along the path

Algorithm:
  // Pre-compute average value of unallocated reachable nodes
  avg_value = average_node_value(tree, allocated, build, calc)

  // A* search from allocated frontier to target
  // f(n) = g(n) + h(n)
  // g(n) = actual cost (points) to reach n from allocated set
  // h(n) = heuristic estimate of remaining cost to target
  //        We use graph distance (BFS hop count) as admissible heuristic

  open_set: BinaryHeap<(Reverse<OrderedFloat<f64>>, NodeId)>  // min-heap by f
  g_score: HashMap<NodeId, f64>
  came_from: HashMap<NodeId, NodeId>

  // Initialize: all allocated nodes have g=0
  for node in allocated:
    g_score[node] = 0.0
    f = 0.0 + heuristic(node, target)
    open_set.push((Reverse(OrderedFloat(f)), node))

  // Pre-compute BFS distances from target (for heuristic)
  bfs_dist = bfs_from(tree, target)

  while let Some((_, current)) = open_set.pop():
    if current == target:
      return reconstruct_path(came_from, target)

    for neighbor in tree.edges[current]:
      if allocated.contains(neighbor): continue  // already taken

      // Cost to take this node = 1 point
      // But we offset by the VALUE of the node (negative cost if valuable)
      node_value = calc.node_value(build, neighbor)
      edge_cost = 1.0 - (node_value / avg_value).min(0.8)
      // Floor at 0.2 — even the best node still "costs" something
      // This ensures we don't path through long chains of mediocre nodes
      // just because they have slight value

      tentative_g = g_score[current] + edge_cost
      
      if tentative_g < g_score.get(neighbor).unwrap_or(f64::MAX):
        came_from[neighbor] = current
        g_score[neighbor] = tentative_g
        f = tentative_g + bfs_dist[neighbor] as f64 * 0.2  // heuristic weight
        open_set.push((Reverse(OrderedFloat(f)), neighbor))

  // Target unreachable
  return None
```

### Node Value Calculation

```
fn node_value(calc: &PathCalcEngine, build: &BuildData, node: NodeId) -> f64 {
  // Quick estimate using stat weights (don't run full calc per node)
  let weights = ARCHETYPE_WEIGHTS[build.archetype]
  let value = 0.0
  for stat in tree.nodes[node].stats:
    value += stat.value * weights.get(stat.type).unwrap_or(0.0)
  value
}
```

### Complexity

- Time: O((V + E) log V) where V ≈ 1300, E ≈ 1600. ~5ms.
- BFS pre-computation: O(V + E). ~1ms.
- Total: ~6ms per path query.

---

## 14. Passive Tree Respec Optimizer

### Problem

Find allocated nodes that are inefficient (contribute little) and suggest respec.
Also find shorter alternative paths between important nodes.

### Algorithm

Two sub-algorithms:

### 14a: Inefficient Node Detection

```
Input:
  tree: PassiveTree
  allocated: HashSet<NodeId>
  build: BuildData
  calc: PathCalcEngine

Output:
  inefficient: Vec<(NodeId, f64)>  // node and its value

Algorithm:
  // For each allocated node, check if removing it would disconnect the tree
  // If NOT a bridge node, check its value
  
  bridges = find_bridge_nodes(tree, allocated)  // Tarjan's algorithm
  
  for node in allocated:
    if node in bridges: continue  // can't remove, would disconnect tree
    if node.is_keystone: continue  // keystones are intentional
    if node.is_class_start: continue  // can't unallocate starting node

    // What does this node contribute?
    value = calc.node_value(build, node)
    
    if value < threshold:  // threshold = 0.5 × average_node_value
      inefficient.push((node, value))
  
  inefficient.sort_by(|a, b| a.1.partial_cmp(&b.1))
```

### 14b: Path Shortening

```
For each pair of important nodes (keystones, notables) that are both allocated:
  current_path = find_path_in_allocated_tree(node_a, node_b)
  shortest_path = dijkstra(tree, node_a, node_b)  // ignoring current allocation
  
  if shortest_path.len() < current_path.len():
    savings = current_path.len() - shortest_path.len()
    // But we need to check: does the new path pass through worse nodes?
    old_value = sum(node_value for node in current_path)
    new_value = sum(node_value for node in shortest_path)
    
    if new_value >= old_value * 0.8:  // new path isn't much worse in stats
      suggest_respec(current_path, shortest_path, savings)
```

### Articulation Point Detection (Tarjan's)

We need **articulation points** (nodes whose removal disconnects the subgraph), NOT
bridge edges. These are different: a bridge is an edge; an articulation point is a node.

```
fn find_articulation_points(
  tree: &PassiveTree,
  allocated: &HashSet<NodeId>,
) -> HashSet<NodeId> {
  // Articulation point conditions:
  //   Root node:     is an articulation point if it has ≥ 2 DFS children
  //   Non-root node: is an articulation point if any child c has low[c] >= disc[node]
  //                  (the subtree rooted at c cannot reach above node without going through node)

  let mut disc = HashMap::<NodeId, u32>::new();
  let mut low  = HashMap::<NodeId, u32>::new();
  let mut parent = HashMap::<NodeId, Option<NodeId>>::new();
  let mut art_points = HashSet::<NodeId>::new();
  let mut timer = 0u32;

  // Pick any allocated node as root
  let root = *allocated.iter().next().unwrap();

  fn dfs(
    node: NodeId, par: Option<NodeId>,
    tree: &PassiveTree, allocated: &HashSet<NodeId>,
    disc: &mut HashMap<NodeId, u32>, low: &mut HashMap<NodeId, u32>,
    art_points: &mut HashSet<NodeId>, timer: &mut u32, root: NodeId,
  ) {
    disc.insert(node, *timer);
    low.insert(node, *timer);
    *timer += 1;
    let mut child_count = 0u32;

    for &neighbor in &tree.edges[&node] {
      if !allocated.contains(&neighbor) { continue; }
      if Some(neighbor) == par { continue; }  // skip direct parent edge

      if !disc.contains_key(&neighbor) {
        child_count += 1;
        dfs(neighbor, Some(node), tree, allocated, disc, low, art_points, timer, root);

        // Propagate low value up
        let low_neighbor = low[&neighbor];
        let l = low.entry(node).or_insert(u32::MAX);
        *l = (*l).min(low_neighbor);

        // Articulation point check:
        if node == root && child_count > 1 {
          art_points.insert(node);  // root with 2+ DFS children
        }
        if node != root && low_neighbor >= disc[&node] {
          art_points.insert(node);  // non-root: subtree can't bypass node
        }
      } else {
        // Back edge: update low with disc of already-visited neighbor
        let disc_neighbor = disc[&neighbor];
        let l = low.entry(node).or_insert(u32::MAX);
        *l = (*l).min(disc_neighbor);
      }
    }
  }

  dfs(root, None, tree, allocated, &mut disc, &mut low, &mut art_points, &mut timer, root);
  art_points
}
```

**Why this matters:** The previous condition `low[neighbor] > disc[node]` detects bridge
*edges*, which is a stricter condition. Two nodes could both be non-bridge-edge-adjacent but
one is still an articulation point (e.g., a node with two children that can't bypass it).
Using the wrong condition would allow suggesting removal of nodes that would disconnect the tree.

---

## 15. Crafting Probability Engine

### Problem

Given a base item, item level, and crafting method, calculate the exact probability
of hitting a target set of mods.

### Algorithm — Weighted Sampling Without Replacement

PoE mod rolling works as follows:
1. Determine number of affixes (prefixes: 1-3, suffixes: 1-3)
2. For each affix slot, roll from the remaining mod pool weighted by spawn weight
3. A mod can only appear once per item (no duplicates)
4. Some mods are mutually exclusive (same mod group)

```
Input:
  base: BaseType
  ilvl: u8
  influence: Option<Influence>
  method: CraftMethod  // Chaos, Fossil(vec), Essence(type), Harvest(type)
  target_mods: Vec<ModRequirement>  // what we want to hit
  mod_db: ModWeightDB

Output:
  probability: f64          // per attempt
  expected_attempts: u32
  expected_cost: f64        // in divine equivalents

Algorithm:
  // Step 1: Build the mod pool for this base + ilvl + influence
  pool = mod_db.get_eligible_mods(base, ilvl, influence)
  // pool = Vec<(ModId, spawn_weight, is_prefix, mod_group)>

  // Step 2: Apply crafting method modifiers
  match method:
    Fossil(fossils):
      for fossil in fossils:
        for (mod_id, weight) in pool:
          // Each fossil has tag multipliers
          // Pristine Fossil: "life" tag ×10, "defences" tag ×0 (blocked)
          for (tag, mult) in fossil.tag_multipliers:
            if mod_has_tag(mod_id, tag):
              weight *= mult
          // Weight of 0 = blocked (removed from pool)
      pool.retain(|m| m.weight > 0)
    
    Essence(essence_type):
      // One mod is guaranteed (prefix or suffix)
      guaranteed = essence_type.guaranteed_mod
      // Remove all mods in the same group as guaranteed from pool
      pool.retain(|m| m.group != guaranteed.group)
      // Remaining affixes are rolled normally
      // Effective: 2 random prefixes + 3 random suffixes (or vice versa)
    
    Chaos:
      // No modifications to pool

    Harvest(harvest_type):
      // Harvest crafts are targeted: "Augment Fire", "Remove-Add Cold", etc.
      match harvest_type:
        Augment(tag):
          // Add one random mod with given tag (if open prefix/suffix exists)
          // Pool = all mods with this tag, filtered by available prefix/suffix
          pool.retain(|m| m.has_tag(tag))
          // P(hit target) = target_weight / pool_total_weight
          return single_mod_probability(target_mods, pool)

        RemoveAdd(tag):
          // Removes any existing mod with given tag, adds a new one
          // First: which existing mods have this tag? (one is removed randomly)
          // Then: roll a new mod from same-tag pool
          // P(remove the right one) × P(add target) — two-step probability
          return remove_add_probability(build.current_item, target_mods, pool, tag)

        Augment(NonMatchingTag):
          // "Aug fire" when you already have all desired fire mods — targets empty slot
          pool.retain(|m| m.has_tag(tag) && !already_on_item(build.current_item, m))
          return single_mod_probability(target_mods, pool)

    Eldritch(exarch_mods, eater_mods):
      // Eldritch currency adds implicits from Searing Exarch or Eater of Worlds
      // Base items get a primary + secondary implicit per eldritch influence type
      // The pool for Exarch implicits is separate from Eater implicits
      // Eldritch Chaos Orb: randomizes the matching eldritch implicit tiers
      // Eldritch Exaltation: upgrades one implicit to next tier
      exarch_pool = mod_db.get_eldritch_mods(base, InfluenceType::Exarch, ilvl)
      eater_pool  = mod_db.get_eldritch_mods(base, InfluenceType::Eater, ilvl)
      // probability same as Chaos for target tier but using eldritch pools
      // Note: eldritch mods cannot be benchcrafted over; they occupy implicit slots

    Recombinator(item_a, item_b):
      // Recombinator: combine two items of same base type
      // Each mod on either item has 50% chance to appear on the output
      // Output has same affix count constraints (max 3 prefix, 3 suffix)
      // Fractured mods on either item are always preserved on output
      // Strategy: if target mod is on item_a, P(appears) ≈ 0.5 (per mod)
      // For multiple target mods across both items: independence assumption
      p_all_targets = 1.0
      for target in target_mods:
        p_source = if item_a.has_mod(target) { 0.5 } else { 0.0 }
               + if item_b.has_mod(target) && !item_a.has_mod(target) { 0.5 } else { 0.0 }
        // If same mod on both: still ~50% (mod is shared, appears as one)
        p_all_targets *= p_source
      // Additional constraint: total output mods ≤ 6 (prefixes + suffixes)
      // If the combination would exceed 6, some mods are dropped randomly
      // Approximate correction factor:
      total_candidate_mods = count_unique_mods(item_a, item_b)
      if total_candidate_mods > 6:
        p_all_targets *= 0.6  // rough adjustment for over-full combinations

    AwakenerOrb(donor_item, target_item):
      // Destroys donor item, transfers one random influenced mod to target
      // Target keeps its own mods + gets one donor mod (donor influence applies)
      // Target must have open affix in same category as donated mod
      donor_influenced_mods = donor_item.mods.filter(|m| m.is_influenced)
      p_hit_target_mod = target_weight / donor_influenced_mods.sum(weight)
      // P(target has open slot for donated mod) depends on current item state

    Chaos:
      // No modifications to pool

  // Step 3: Calculate probability of hitting ALL target mods
  // We need to find: P(all target mods appear in a random 3-prefix + 3-suffix item)
  
  prefix_pool = pool.filter(is_prefix)
  suffix_pool = pool.filter(is_suffix)
  prefix_total_weight = prefix_pool.sum(weight)
  suffix_total_weight = suffix_pool.sum(weight)
  
  target_prefixes = target_mods.filter(is_prefix)
  target_suffixes = target_mods.filter(is_suffix)
  
  // For each target mod: probability it appears in 3 rolls from the pool
  // P(mod appears in 3 rolls) = 1 - P(mod doesn't appear in any of 3 rolls)
  // P(mod doesn't appear in roll k) = 1 - (mod_weight / remaining_total_weight)
  // Rolls are WITHOUT replacement, so this is hypergeometric-like
  
  // Exact calculation for small target sets (1-3 mods):
  prob = exact_probability(target_prefixes, prefix_pool, 3)
       * exact_probability(target_suffixes, suffix_pool, 3)
  
  // Account for tier requirements (e.g., "T1 life" not just "any life")
  for target in target_mods:
    if target.min_tier > 1:
      // Only count weight for tiers at or above target
      tier_weight = sum(weight for tier in mod.tiers if tier.rank <= target.min_tier)
      total_mod_weight = sum(weight for tier in mod.tiers)
      prob *= tier_weight / total_mod_weight

  expected_attempts = ceil(1.0 / prob)
  expected_cost = expected_attempts * method.cost_per_attempt()

fn exact_probability(targets: &[Mod], pool: &[Mod], num_rolls: u32) -> f64:
  // For 0 targets: probability = 1.0
  if targets.is_empty(): return 1.0
  
  // For 1 target in 3 rolls (without replacement):
  // P = 1 - Π(1 - w_target / (W - sum_of_previously_removed_weights))
  // Approximation (very accurate for large pools):
  // P ≈ 1 - ((W - w_target) / W)^num_rolls
  
  // For multiple targets: inclusion-exclusion or direct computation
  // Since targets are usually 1-3 mods, direct computation is fast
  
  total_weight = pool.sum(weight)
  p = 1.0
  
  for target in targets:
    // P(this target appears in at least one of num_rolls slots)
    // Given other targets already consumed some slots
    available_rolls = num_rolls - targets_already_placed
    p_miss = ((total_weight - target.weight) / total_weight).powi(available_rolls)
    p *= 1.0 - p_miss
    total_weight -= target.weight  // this mod is now placed, remove from pool
    available_rolls -= 1
  
  return p
```

### Complexity

- Pool construction: O(P) where P = mod pool size (~200)
- Probability calculation: O(T × P) where T = target count (~1-4)
- Total: O(P) per query. < 1ms.

---

## 16. Monte Carlo Craft Simulator

### Problem

Beyond single-attempt probability, we need to answer: "How many attempts until I
get a good-enough item? When should I stop crafting and buy instead?"

### Algorithm

```
Input:
  base: BaseType
  method: CraftMethod
  target_score: f64       // item score threshold to accept
  buy_price: f64          // market price for equivalent item
  max_budget: f64         // maximum divines to spend
  simulations: u32        // 10_000 for accuracy

Output:
  strategy: CraftStrategy

struct CraftStrategy {
  expected_cost: f64,
  median_cost: f64,
  success_rate: f64,       // % of sims that hit target within budget
  percentile_90_cost: f64, // unlucky case (90th percentile)
  recommendation: Recommendation,  // Craft, Buy, or CraftThenBuy
  optimal_stop: u32,       // attempt number where expected value peaks
}

Algorithm:
  results: Vec<SimResult> = vec![]

  for _ in 0..simulations:
    total_cost = 0.0
    best_score = 0.0
    attempts = 0

    loop:
      // Simulate one craft attempt
      item = simulate_craft(base, method, mod_db)
      score = score_item(item, archetype, mod_db)  // Algorithm 7
      attempts += 1
      total_cost += method.cost_per_attempt()

      if score > best_score:
        best_score = score

      // Stopping conditions
      if best_score >= target_score:
        results.push(SimResult::Success(total_cost, attempts))
        break
      if total_cost >= buy_price:
        results.push(SimResult::ShouldHaveBought(total_cost, attempts, best_score))
        break
      if total_cost >= max_budget:
        results.push(SimResult::BudgetExhausted(total_cost, attempts, best_score))
        break

  // Analyze results
  successes = results.filter(is_success)
  success_rate = successes.len() / results.len()
  
  costs = successes.map(|r| r.cost).sorted()
  expected_cost = costs.mean()
  median_cost = costs[costs.len() / 2]
  percentile_90 = costs[costs.len() * 9 / 10]

  // Recommendation
  recommendation = if expected_cost < buy_price * 0.7:
    Craft  // crafting is significantly cheaper
  else if expected_cost > buy_price * 1.3:
    Buy  // buying is significantly cheaper
  else:
    CraftThenBuy  // try N crafts, buy if unlucky

  // Optimal stopping point: attempt where E[value] peaks
  // (beyond this point, you're throwing money away)
  optimal_stop = find_optimal_stop(results)
  
fn simulate_craft(base: &BaseType, method: &CraftMethod, mod_db: &ModWeightDB) -> Item:
  pool = build_pool(base, method, mod_db)
  
  // PoE rolls prefixes and suffixes INDEPENDENTLY, each with its own distribution.
  // Rarity-spawned rare items (chaos orb reroll on a rare):
  //   Prefixes: P(1) ≈ 8%, P(2) ≈ 58%, P(3) ≈ 34%
  //   Suffixes: P(1) ≈ 8%, P(2) ≈ 58%, P(3) ≈ 34%
  // The TOTAL affix count (4-6) is NOT a single roll — it's the sum of two independent rolls.
  // A 6-mod item requires P(3 prefix) × P(3 suffix) ≈ 0.34 × 0.34 ≈ 12%.
  // A 4-mod item: P(2 prefix) × P(2 suffix) ≈ 0.58 × 0.58 ≈ 34% (most common).
  // Note: magic items use different distributions (always 1-2 mods total).
  num_prefixes = weighted_random([(1, 0.08), (2, 0.58), (3, 0.34)])
  num_suffixes = weighted_random([(1, 0.08), (2, 0.58), (3, 0.34)])
  
  prefixes = sample_without_replacement(pool.prefixes, num_prefixes)
  suffixes = sample_without_replacement(pool.suffixes, num_suffixes)
  
  // For each mod, roll a value within its tier range
  for mod in prefixes + suffixes:
    mod.value = uniform_random(mod.tier.min, mod.tier.max)
  
  Item { prefixes, suffixes }

fn find_optimal_stop(results: &[SimResult]) -> u32:
  // For each attempt number N, calculate:
  // E[value at stop N] = P(success by N) × target_value + P(fail by N) × best_so_far_value
  // The optimal N maximizes E[value] - cost(N)
  // Simple: find N where marginal improvement < marginal cost
  for n in 1..100:
    success_by_n = results.filter(|r| r.attempts <= n && r.is_success).len() / results.len()
    marginal_success = success_by_n - success_by_(n-1)
    marginal_cost = method.cost_per_attempt()
    marginal_value = marginal_success * (buy_price - cost_so_far)
    
    if marginal_value < marginal_cost:
      return n  // stop here, further attempts not worth it
```

### Output Example

```
Craft: Essence of Anger on Opal Ring (target: 70+ score for RF)
  Success rate: 32% per attempt
  Expected cost: 2.1 divine (median: 1.5 divine)
  90th percentile: 5.0 divine (unlucky)
  Market price: 5 divine
  Recommendation: CRAFT (saves ~3 divine on average)
  Optimal stop: 6 attempts (if no good result by then, buy)
```

---

## 17. Build Similarity & Collaborative Filtering

### Problem

Given a player's build, find "similar" builds on poe.ninja and learn from what
top players equip. This powers the "90% of top RF Inquisitors use X" suggestions.

### Algorithm — Feature Vector Cosine Similarity

### 17a: Build Feature Vector

```
Encode a build as a fixed-length numerical vector:

fn build_to_vector(build: &BuildData) -> Vec<f64>:
  features = []
  
  // Ascendancy (one-hot, 19 ascendancies)
  for asc in ALL_ASCENDANCIES:
    features.push(if build.ascendancy == asc { 1.0 } else { 0.0 })
  
  // Main skill (one-hot, ~200 active skills)
  for skill in ALL_ACTIVE_SKILLS:
    features.push(if build.main_skill == skill { 1.0 } else { 0.0 })
  
  // Key stats (normalized to 0-1 range)
  features.push(build.dps / 10_000_000.0)          // normalize to 10M DPS
  features.push(build.life / 10_000.0)              // normalize to 10K life
  features.push(build.es / 10_000.0)
  features.push(build.armour / 100_000.0)
  features.push(build.evasion / 100_000.0)
  
  // Key unique items (binary: has/doesn't have)
  for unique_name in TOP_100_UNIQUES:
    features.push(if build.has_unique(unique_name) { 1.0 } else { 0.0 })
  
  // Keystones (binary)
  for keystone in ALL_KEYSTONES:
    features.push(if build.has_keystone(keystone) { 1.0 } else { 0.0 })
  
  features
```

### 17b: Similarity Search

```
fn find_similar(build: &BuildData, top_builds: &[TopBuild], k: usize) -> Vec<TopBuild>:
  query_vec = build_to_vector(build)
  
  // Pre-filter: same ascendancy + same main skill (mandatory)
  candidates = top_builds.filter(|b| 
    b.ascendancy == build.ascendancy && b.main_skill == build.main_skill
  )
  
  // Cosine similarity on remaining features
  scored = candidates.iter().map(|b| {
    let b_vec = build_to_vector(b)
    let similarity = cosine_similarity(&query_vec, &b_vec)
    (b, similarity)
  })
  
  // Return top K most similar
  scored.sort_by(|a, b| b.1.partial_cmp(&a.1))
  scored.take(k).map(|(b, _)| b).collect()

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64:
  let dot = a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>()
  let norm_a = a.iter().map(|x| x * x).sum::<f64>().sqrt()
  let norm_b = b.iter().map(|x| x * x).sum::<f64>().sqrt()
  if norm_a == 0.0 || norm_b == 0.0 { return 0.0 }
  dot / (norm_a * norm_b)
```

### 17c: Collaborative Suggestions

```
fn collaborative_suggestions(build: &BuildData, similar: &[TopBuild]) -> Vec<Suggestion>:
  suggestions = []
  
  for slot in EQUIPMENT_SLOTS:
    // Count what similar builds use in this slot
    item_frequency: HashMap<String, usize> = HashMap::new()
    for b in similar:
      let item_key = if b.item_at(slot).is_unique:
        b.item_at(slot).name.clone()  // exact unique name
      else:
        b.item_at(slot).base.clone()  // base type for rares
      *item_frequency.entry(item_key).or_insert(0) += 1
    
    // If >50% use something different from player
    let most_common = item_frequency.max_by_value()
    let usage = most_common.count as f64 / similar.len() as f64
    
    if usage > 0.50 && most_common.key != build.item_at(slot).key:
      suggestions.push(Suggestion {
        slot,
        message: format!("{:.0}% of top players use {} (you: {})",
          usage * 100.0, most_common.key, build.item_at(slot).name),
        confidence: usage,
      })
  
  suggestions
```

### Data Source

- poe.ninja provides top ~15,000 builds per league via their API
- We fetch and cache this data daily (or on user request)
- Feature vectors are pre-computed and stored in SQLite for fast lookup

### Complexity

- Feature vector: O(F) where F = feature dimension (~400)
- Similarity search: O(C × F) where C = candidates (~500 after pre-filter)
- Total: ~1ms

---

## 18. Intent Classifier — Query Router

### Problem

Classify a user's natural language question into an intent category and route it
to the correct engine (Calculator, Knowledge Base, or Cloud AI).

### Algorithm — Priority-Ordered Regex with Entity Extraction

The full regex rule list is in ENGINE-DESIGN.md section 5b. Here we specify the
classification algorithm itself.

```
Input:
  query: String

Output:
  intent: Intent
  entities: HashMap<String, String>  // extracted values (slot, boss, gem, etc.)
  confidence: f64                    // 0.0-1.0
  engine: Engine                     // Calculator, KnowledgeBase, or CloudAI

Algorithm:

Step 1: NORMALIZE query
  query = query.to_lowercase()
  query = query.trim()
  query = expand_abbreviations(query)
  // "rf" → "righteous fire", "dps" → "damage per second",
  // "res" → "resistance", "ms" → "movement speed"

Step 2: EXTRACT ENTITIES before classification
  entities = {}
  
  // Slot mentions
  if regex("(ring|helmet|boots|gloves|belt|amulet|body|shield|weapon)\\s*(1|2)?"):
    entities["slot"] = match[0]
  
  // Boss mentions
  if regex("(shaper|elder|sirus|maven|uber|atziri|cortex)"):
    entities["boss"] = match[0]
  
  // Gem mentions
  if regex("(righteous fire|rf|cyclone|ice nova|spectre|\\w+ \\w+ support)"):
    entities["gem"] = match[0]
  
  // Number mentions (for budget)
  if regex("(\\d+)\\s*(div|divine|chaos|ex|exalt)"):
    entities["budget"] = match[0]
    entities["currency"] = match[1]

Step 3: CLASSIFY by priority-ordered regex rules
  for (pattern, intent) in INTENT_RULES:  // from ENGINE-DESIGN.md 5b
    if pattern.matches(query):
      matched_intent = intent
      break
  
  // If no match: Intent::OpenEnded
  if !matched_intent:
    matched_intent = Intent::OpenEnded

Step 4: APPLY PRECEDENCE RULES for multi-match ambiguity
  // See ENGINE-DESIGN.md for the 5 precedence rules:
  // 1. Defense > Offense
  // 2. Specific slot > General upgrade
  // 3. Crafting > Item
  // 4. Boss-specific > General defense
  // 5. Ambiguous → route to BOTH engines

Step 5: ROUTE to engine
  engine = match matched_intent:
    DpsCheck | DpsBreakdown | GemSwap | DefenseAnalysis | EhpCalc |
    ResistCheck | UpgradeRank | ItemScore | ItemCompare | TreeAdvice
      → Engine::Calculator
    
    CraftAdvice | BossMechanic | MapMod | GemInteraction | PriceCheck
      → Engine::KnowledgeBase
    
    BuildDesign | WhyQuestion | PatchAnalysis | OpenEnded
      → Engine::CloudAI

Step 6: CONFIDENCE scoring
  confidence = match:
    Exact entity match + clear intent pattern → 0.95
    Intent pattern match, no entity → 0.80
    Ambiguous (multiple patterns) → 0.60
    No pattern match (OpenEnded) → 0.40
```

### Abbreviation Dictionary

```
ABBREVIATIONS = {
  "rf"    → "righteous fire",
  "ea"    → "explosive arrow",
  "ls"    → "lightning strike",
  "ts"    → "tornado shot",
  "la"    → "lightning arrow",
  "dps"   → "damage per second",
  "res"   → "resistance",
  "ms"    → "movement speed",
  "as"    → "attack speed",
  "cs"    → "cast speed",
  "ehp"   → "effective hit points",
  "ci"    → "chaos inoculation",
  "ll"    → "low life",
  "eb"    → "eldritch battery",
  "mom"   → "mind over matter",
  "ee"    → "elemental equilibrium",
  "eo"    → "elemental overload",
  "rt"    → "resolute technique",
  "bis"   → "best in slot",
  "coc"   → "cast on crit",
  "cwc"   → "cast while channelling",
  "cwdt"  → "cast when damage taken",
  "div"   → "divine orb",
  "ex"    → "exalted orb",
}
```

---

## 19. Template Response Generator

### Problem

Convert calculation results into natural language responses with PoE-themed flavor,
WITHOUT using an AI model. Only Engine 3 (Cloud AI) queries use a language model.

### Algorithm — Template Selection + Variable Injection

```
Input:
  intent: Intent
  calc: CalcResult
  build: BuildData
  entities: HashMap<String, String>

Output:
  response: String

Algorithm:

Step 1: SELECT template based on intent + conditions
  templates = TEMPLATE_DB[intent]
  
  // Each template has a condition function
  // Select the FIRST template whose condition matches
  template = templates.iter().find(|t| (t.condition)(calc, build, entities))
  
  // Example conditions:
  // DpsCheck: "has dps" → show breakdown
  // DefenseAnalysis with issues: "has issues" → show issue list
  // UpgradeRank: "has suggestions" → show ranked list

Step 2: INJECT variables
  response = template.text.clone()
  
  // Replace {variable} placeholders with actual values
  response = response
    .replace("{total_dps}", &format_number(calc.offense.total_dps))
    .replace("{life}", &format_number(calc.defense.life))
    .replace("{fire_res}", &format!("{}%", calc.defense.fire_res))
    // ... all variables
  
  // Replace {list:X} with formatted lists
  response = response.replace("{issues_list}",
    &calc.defense.issues.iter()
      .enumerate()
      .map(|(i, issue)| format!("  {}. {}", i + 1, issue.message))
      .join("\n"))
  
  // Replace {conditional:X} blocks
  // {if:has_boss_entity}Boss readiness: ...{/if}
  response = process_conditionals(response, entities, calc)

Step 3: FORMAT numbers consistently
  // DPS: 2,841,057 → "2.84M" or "2,841,057" depending on magnitude
  // Life: 6453 → "6,453"
  // Percentages: always with % sign
  // Costs: "3 divine" or "340 chaos"
```

### Number Formatting Rules

```
fn format_dps(n: f64) -> String:
  if n >= 1_000_000: format!("{:.2}M", n / 1_000_000.0)
  else if n >= 1_000: format!("{:.1}K", n / 1_000.0)
  else: format!("{:.0}", n)

fn format_number(n: f64) -> String:
  // Add thousands separators
  let s = format!("{:.0}", n)
  // Insert commas: "6453" → "6,453"
  add_thousands_separators(s)

fn format_cost(divines: f64) -> String:
  if divines < 0.1: "free"
  else if divines < 1.0: format!("{:.0} chaos", divines * 170.0)
  else: format!("{:.1} divine", divines)
```

---

## 20. Combat Simulation Engine

### Problem

Simulate a map clear or boss fight with real PoE mechanics, producing a timeline
of events (kills, deaths, flask usage, guard triggers) and performance statistics.

### Architecture — Discrete Tick Simulation

```
Constants:
  TICK_MS = 100          // simulation resolution
  MAP_MONSTER_COUNT = 400 // typical T16 map
  PACK_SIZE = 8          // monsters per pack

State:
  struct SimState {
    player: PlayerState,
    monsters: Vec<Monster>,
    timer_ms: u64,
    kills: u32,
    deaths: u32,
    currency_dropped: f64,
    flasks: [FlaskState; 5],
    guard: GuardState,
    evasion_entropy: f64,
    leech_instances: Vec<LeechInstance>,
    event_log: Vec<SimEvent>,
  }

  struct PlayerState {
    hp: f64,
    max_hp: f64,
    es: f64,
    max_es: f64,
    position: Vec2,
    move_speed: f64,
    stun_remaining_ms: u32,
    buffs: HashMap<BuffId, u32>,  // buff → remaining ms
  }

  struct Monster {
    hp: f64,
    max_hp: f64,
    damage: f64,
    damage_type: DamageType,
    attack_cooldown_ms: u32,
    attack_timer_ms: u32,
    rarity: Rarity,  // Normal, Magic, Rare, Unique
    position: Vec2,
    alive: bool,
  }
```

### Main Loop

```
fn simulate_map(build: &BuildData, calc: &CalcResult, map: &MapConfig) -> SimResult:
  state = init_state(build, calc, map)
  
  while state.kills < map.total_monsters && state.timer_ms < MAX_TIME:
    tick(&mut state, build, calc, map)
    state.timer_ms += TICK_MS
  
  SimResult {
    clear_time_ms: state.timer_ms,
    kills: state.kills,
    deaths: state.deaths,
    currency: state.currency_dropped,
    dps_uptime: calculate_uptime(state.event_log),
    event_log: state.event_log,
  }

fn tick(state: &mut SimState, build: &BuildData, calc: &CalcResult, map: &MapConfig):
  // === PHASE 1: Player deals damage ===
  
  // AoE skills (RF) damage all monsters in radius
  if build.has_aoe_skill:
    for monster in state.monsters.iter_mut().filter(|m| m.alive):
      if distance(state.player.position, monster.position) <= calc.aoe_radius:
        damage = calc.offense.aoe_dps_per_tick
        damage = apply_map_mods(damage, map.mods)
        monster.hp -= damage
        state.event_log.push(DamageDealt(monster.id, damage))
  
  // Single-target skills (Fire Trap)
  if build.has_single_target && state.single_target_cooldown <= 0:
    if let Some(target) = find_priority_target(state):
      damage = calc.offense.single_target_dps_per_tick
      target.hp -= damage
      state.single_target_cooldown = calc.offense.single_target_cooldown_ms
      state.event_log.push(DamageDealt(target.id, damage))
  
  // Check for kills
  for monster in state.monsters.iter_mut():
    if monster.alive && monster.hp <= 0.0:
      monster.alive = false
      state.kills += 1
      state.currency_dropped += roll_currency_drop(map.tier, monster.rarity)
      // Flask charge gain on kill
      for flask in &mut state.flasks:
        flask.current_charges += flask.charge_per_kill
      state.event_log.push(MonsterKilled(monster.id))

  // === PHASE 2: Monsters deal damage to player ===
  for monster in state.monsters.iter_mut().filter(|m| m.alive):
    monster.attack_timer_ms -= TICK_MS as u32
    if monster.attack_timer_ms <= 0:
      monster.attack_timer_ms = monster.attack_cooldown_ms
      
      // Evasion check (Algorithm 3)
      if build.has_evasion:
        if check_evasion(&mut state.evasion_entropy, calc.defense.evasion_chance):
          state.event_log.push(Evaded(monster.id))
          continue  // evaded
      
      // Block check
      if random() < calc.defense.block_chance:
        state.event_log.push(Blocked(monster.id))
        continue  // blocked
      
      // Calculate damage
      raw_damage = monster.damage * map.monster_damage_mult
      mitigated = mitigate_damage(raw_damage, monster.damage_type, calc, state)
      
      // Guard skill absorption
      if state.guard.active:
        absorbed = min(mitigated * state.guard.absorb_percent, state.guard.remaining)
        mitigated -= absorbed
        state.guard.remaining -= absorbed
        if state.guard.remaining <= 0: state.guard.active = false
      
      // Apply damage
      state.player.hp -= mitigated
      state.event_log.push(DamageTaken(monster.id, mitigated))
      
      // CWDT trigger check
      state.cwdt_damage_accumulator += mitigated
      if state.cwdt_damage_accumulator >= calc.cwdt_threshold:
        state.cwdt_damage_accumulator = 0.0
        activate_guard(&mut state.guard, calc)
        state.event_log.push(GuardActivated)
      
      // Death check
      if state.player.hp <= 0.0:
        state.deaths += 1
        state.player.hp = state.player.max_hp
        state.event_log.push(PlayerDeath)

  // === PHASE 3: Recovery ===
  // Life regen
  state.player.hp += calc.defense.life_regen_per_tick
  state.player.hp = state.player.hp.min(state.player.max_hp)
  
  // Leech (Algorithm 4)
  tick_leech(&mut state.leech_instances, &mut state.player, TICK_MS)
  
  // Flask management (Algorithm 5)
  tick_flasks(&mut state.flasks, &mut state.player, state.kills)

  // === PHASE 4: Movement ===
  // Move player toward next pack
  if current_pack_cleared(state):
    state.player.position = move_toward(state.player.position, next_pack_position(),
      state.player.move_speed * TICK_MS as f64 / 1000.0)

fn mitigate_damage(raw: f64, dtype: DamageType, calc: &CalcResult, state: &SimState) -> f64:
  match dtype:
    Physical:
      let after_armour = raw * (1.0 - calc.defense.phys_reduction(raw))
      let after_endurance = after_armour * (1.0 - state.endurance_charges * 0.04)
      let after_fortify = if state.has_fortify { after_endurance * 0.80 } else { after_endurance }
      after_fortify
    Fire | Cold | Lightning:
      let res = calc.defense.resistance(dtype)
      raw * (1.0 - res / 100.0)
    Chaos:
      raw * (1.0 - calc.defense.chaos_res / 100.0)
```

### Boss Simulation Extension

Boss fights differ from map clearing:
- Bosses have scripted attack patterns (from boss database JSON)
- Phases with immunity windows
- Dodge mechanics based on move speed + telegraph time
- No kill-based flask charges

See COMBAT-SIMULATOR.md for the full boss fight specification.

### Complexity

- Per tick: O(M) where M = active monster count
- Map simulation: O(T × M) where T = total ticks (~1500 for 2.5 min map)
- At speed 10x: simulate ~15,000 ticks in 150 seconds real time
- "Skip to end": run all ticks without rendering, ~50ms

---

## 21. Market Price Cache & Circuit Breaker

### Problem

poe.ninja API calls are rate-limited and can be slow/down. We need caching with
staleness handling and a circuit breaker pattern to gracefully handle API failures.

### Algorithm — TTL Cache with Stale Fallback + Circuit Breaker

```
struct PriceCache {
  entries: HashMap<ItemKey, CacheEntry>,
  circuit_state: CircuitState,
  failure_count: u32,
  last_failure: Instant,
}

struct CacheEntry {
  price: f64,
  fetched_at: Instant,
  ttl: Duration,  // 5 minutes for active items, 1 hour for stable
}

enum CircuitState {
  Closed,     // normal operation, API calls allowed
  Open,       // API is down, use stale cache only
  HalfOpen,   // testing if API is back (allow one request)
}

fn get_price(cache: &mut PriceCache, item: &ItemKey) -> PriceResult:
  // 1. Check cache
  if let Some(entry) = cache.entries.get(item):
    if entry.fetched_at.elapsed() < entry.ttl:
      return PriceResult::Fresh(entry.price)
    // Stale but exists
    stale_price = Some(entry.price)
  else:
    stale_price = None

  // 2. Check circuit breaker
  match cache.circuit_state:
    Open:
      // API is down — don't even try
      if cache.last_failure.elapsed() > Duration::from_secs(30):
        cache.circuit_state = HalfOpen  // test one request
      else:
        return stale_price.map(PriceResult::Stale)
          .unwrap_or(PriceResult::Unavailable)
    
    HalfOpen:
      // Allow ONE request to test
      match fetch_from_api(item):
        Ok(price):
          cache.circuit_state = Closed  // API is back!
          cache.failure_count = 0
          cache.entries.insert(item, CacheEntry::new(price))
          return PriceResult::Fresh(price)
        Err(_):
          cache.circuit_state = Open  // still down
          cache.last_failure = Instant::now()
          return stale_price.map(PriceResult::Stale)
            .unwrap_or(PriceResult::Unavailable)
    
    Closed:
      // Normal operation — fetch from API
      match fetch_from_api(item):
        Ok(price):
          cache.entries.insert(item, CacheEntry::new(price))
          return PriceResult::Fresh(price)
        Err(_):
          cache.failure_count += 1
          if cache.failure_count >= 3:
            cache.circuit_state = Open  // trip the circuit
            cache.last_failure = Instant::now()
          return stale_price.map(PriceResult::Stale)
            .unwrap_or(PriceResult::Unavailable)

enum PriceResult {
  Fresh(f64),       // current price from API
  Stale(f64),       // expired cache entry (show with "⚠ price may be outdated")
  Unavailable,      // no cache + API down
}
```

### Batch Fetching

For efficiency, batch multiple price lookups into one API call:

```
fn fetch_prices_batch(items: &[ItemKey]) -> HashMap<ItemKey, f64>:
  // poe.ninja bulk endpoint: fetch all prices for a category
  // Categories: UniqueArmour, UniqueWeapon, UniqueAccessory, etc.
  // One request per category, returns ALL items in that category
  
  categories = items.iter().map(|i| i.category).collect::<HashSet<_>>()
  
  prices = HashMap::new()
  for category in categories:
    response = http_get(format!("https://poe.ninja/api/data/item?league={}&type={}", league, category))
    for item_data in response.lines:
      prices.insert(item_data.name, item_data.chaosValue / divine_ratio)
  
  prices
```

### TTL Strategy

| Item Type | TTL | Reason |
|-----------|-----|--------|
| Meta uniques (Mageblood, HH) | 5 min | Prices swing fast |
| Common uniques | 30 min | Relatively stable |
| Currency ratios | 15 min | Medium volatility |
| Div card prices | 1 hour | Slow to change |
| Fossil/essence prices | 1 hour | Stable |

### League Phase Detection

Prices have radically different volatility at different points in a league's life.
The cache uses this to adjust TTL dynamically and surface buy/sell advice.

```rust
pub enum LeaguePhase {
    LaunchFrenzy,    // day 0-3:  chaos orbs most valuable, uniques wildly overpriced
    CrashPeriod,     // day 4-7:  prices crashing daily, leveling gear cheap
    Stabilization,   // day 8-21: prices settling, best time for mid-tier upgrades
    PeakEconomy,     // day 22-42: mature economy, best endgame item prices
    LateLeague,      // day 43+:  player count dropping, niche items scarce
    Unknown,         // no league start date available
}

pub fn detect_league_phase(league_start: Option<DateTime<Utc>>) -> LeaguePhase {
    let Some(start) = league_start else { return LeaguePhase::Unknown };
    let days = (Utc::now() - start).num_days();
    match days {
        0..=3   => LeaguePhase::LaunchFrenzy,
        4..=7   => LeaguePhase::CrashPeriod,
        8..=21  => LeaguePhase::Stabilization,
        22..=42 => LeaguePhase::PeakEconomy,
        _       => LeaguePhase::LateLeague,
    }
}

/// Phase-aware TTL multiplier — reduces TTL in volatile early league,
/// relaxes it in stable mid-league when prices change slowly.
pub fn ttl_multiplier(phase: LeaguePhase) -> f64 {
    match phase {
        LeaguePhase::LaunchFrenzy  => 0.25,  // 5 min → 75s for meta uniques
        LeaguePhase::CrashPeriod   => 0.50,
        LeaguePhase::Stabilization => 1.00,  // standard TTL
        LeaguePhase::PeakEconomy   => 2.00,  // prices stable, cache longer
        LeaguePhase::LateLeague    => 3.00,  // very stable, fewer updates needed
        LeaguePhase::Unknown       => 1.00,
    }
}
```

### League Phase Buy Advice (surfaced in UI)

| Phase | Advice |
|-------|--------|
| **Launch Frenzy** | Sell valuable uniques immediately. Hoard chaos — they're at max value. |
| **Crash Period** | Buy leveling uniques now (90% cheaper than day 1). Save divines. |
| **Stabilization** | Best time for mid-tier upgrades (5–20 divine range). |
| **Peak Economy** | Buy endgame items — best prices in the league. Mirror-tier crafts available. |
| **Late League** | Niche items scarce (fewer sellers). Common items very cheap. |

This advice is shown in the price panel alongside each item's price, not as a
separate screen — e.g. "3.5 divine ⚠ Launch Frenzy: price may drop 60% in 3 days".

---

## 22. PoB XML Parser — Streaming State Machine

### Problem

Parse Path of Building XML export files into our `BuildData` struct. PoB files
can be 50-200KB and contain: Build info, Items, Skills, Tree, Config, Notes.

### Algorithm — Section-Based Streaming Parser

```
Input:
  xml: &str  // raw PoB XML

Output:
  BuildData  // parsed struct

Algorithm:
  We don't need a full DOM parser. PoB XML has a predictable section structure:

  <PathOfBuilding>
    <Build ...>          → class, level, ascendancy, bandit
    <Import .../>        → imported build metadata
    <Calcs ...>          → (IGNORED — we calculate ourselves)
    <Skills ...>         → socket groups with gems
    <Tree ...>           → passive tree spec (hashed node IDs)
    <Notes>...</Notes>   → user notes (ignored)
    <TreeView .../>      → UI state (ignored)
    <Items ...>          → all equipped + inventory items
    <Config ...>         → build configuration toggles
  </PathOfBuilding>

Parser state machine:
  state = Idle
  
  for event in xml_parser.events():
    match (state, event):
      (Idle, StartElement("Build")):
        state = ParsingBuild
        parse_build_attributes(attrs)  // level, class, ascendancy
      
      (Idle, StartElement("Skills")):
        state = ParsingSkills
      
      (ParsingSkills, StartElement("Skill")):
        // New socket group
        current_group = SocketGroup::new(attrs)  // label, enabled, slot
      
      (ParsingSkills, StartElement("Gem")):
        // Gem in current socket group
        gem = parse_gem(attrs)  // skillId, level, quality, enabled
        current_group.gems.push(gem)
      
      (ParsingSkills, EndElement("Skill")):
        build.socket_groups.push(current_group)
      
      (Idle, StartElement("Tree")):
        state = ParsingTree
      
      (ParsingTree, StartElement("Spec")):
        // Tree spec: list of allocated node IDs + mastery effect selections
        // Stored as: nodes="26059,63135,29994,..."
        //            masteryEffects="12345:67890,..."  (nodeId:effectId pairs)
        node_str = attrs.get("nodes")
        build.allocated_nodes = node_str.split(",")
          .filter(|s| !s.is_empty())
          .map(|s| s.parse::<u32>())
          .collect()

        // Masteries: each mastery cluster node can have one effect selected
        // The selected effect is stored as a separate stat (not in the node's base stats)
        // Format: "nodeId1:effectId1,nodeId2:effectId2,..."
        if let Some(mastery_str) = attrs.get("masteryEffects"):
          build.mastery_effects = mastery_str.split(",")
            .filter(|s| !s.is_empty())
            .filter_map(|pair| {
              let parts: Vec<&str> = pair.split(":").collect()
              if parts.len() == 2 {
                Some((parts[0].parse::<u32>().ok()?, parts[1].parse::<u32>().ok()?))
              } else { None }
            })
            .collect()
          // Mastery effects must be applied AFTER node aggregation in Algorithm 1
          // Each mastery effect is an additional stat entry (from mastery_db.get_effect(effectId))
      
      (Idle, StartElement("Items")):
        state = ParsingItems
      
      (ParsingItems, Text) if state == ParsingItems:
        // Items are stored as text blocks in PoB format, NOT XML attributes
        // Each item is a text block between CDATA markers or <Item> tags
        parse_item_text(text, &mut build.items)
      
      (Idle, StartElement("Config")):
        state = ParsingConfig
      
      (ParsingConfig, StartElement("Input")):
        // Config toggle: name="enemyIsBoss" boolean="true"
        config_key = attrs.get("name")
        config_val = attrs.get("boolean").or(attrs.get("number")).or(attrs.get("string"))
        build.config.insert(config_key, config_val)
      
      (_, EndElement("PathOfBuilding")):
        break  // done
      
      _: continue  // skip unknown elements
```

### PoB Item Text Format Parser

PoB stores items in a unique text format (NOT standard PoE item copy format):

```
Item format example:
  Rarity: RARE
  Glyph Crest
  Royal Burgonet
  Armour: 453
  Quality: 20
  Sockets: R-R-R-G
  LevelReq: 65
  Implicits: 1
  +42 to maximum Life
  +97 to maximum Life
  +41% to Fire Resistance
  Nearby Enemies have -9% to Fire Resistance
  {crafted}+25% to Cold Resistance

Parsing rules:
  Line 1: "Rarity: X" → Normal, Magic, Rare, Unique
  Line 2: Item name
  Line 3: Base type
  Lines with "X: Y": properties (Armour, ES, Quality, Sockets, LevelReq)
  "Implicits: N": next N lines are implicit mods
  Remaining lines: explicit mods
  Lines starting with "{crafted}": benchcrafted mods
  Lines starting with "{fractured}": fractured mods
```

### File Safety

```
Write operations (apply upgrade → save to PoB file):
  1. Read current file
  2. Parse to verify it's valid
  3. Apply modification
  4. Write to TEMP file (same directory, random suffix)
  5. Atomic rename: temp → target
  6. Keep backup: target → target.bak (last 50 backups, ring buffer)

This prevents data loss from crashes during write.
```

---

## 23. Mod Text Parser — Pattern Matching Recognizer

### Problem

Convert PoE modifier text strings like "+97 to maximum Life" into structured data:
`{ stat_id: "base_maximum_life", value: 97, type: "flat" }`.

### Algorithm — Regex Pattern Table with Capture Groups

```
Input:
  mod_text: &str  // e.g., "+97 to maximum Life"

Output:
  ParsedMod { stat_id, value, mod_type, tags }

Algorithm:
  // Pre-compiled regex patterns ordered by specificity (most specific first)
  PATTERNS = [
    // Flat added to stat
    (r"^\+?(\d+) to maximum (Life|Mana|Energy Shield)",
     |caps| ParsedMod {
       stat_id: format!("base_maximum_{}", caps[2].to_lowercase()),
       value: caps[1].parse(),
       mod_type: Flat,
       tags: vec![match caps[2] { "Life" => Life, "Mana" => Mana, _ => ES }],
     }),

    // Percentage increased
    (r"^(\d+)% increased (.*)",
     |caps| ParsedMod {
       stat_id: format!("increased_{}", normalize_stat_name(caps[2])),
       value: caps[1].parse(),
       mod_type: Increased,
       tags: infer_tags(caps[2]),
     }),

    // Resistance
    (r"^\+?(\d+)% to (Fire|Cold|Lightning|Chaos) Resistance",
     |caps| ParsedMod {
       stat_id: format!("{}_resistance", caps[2].to_lowercase()),
       value: caps[1].parse(),
       mod_type: Flat,
       tags: vec![Resistance],
     }),

    // All elemental resistances
    (r"^\+?(\d+)% to all Elemental Resistances",
     |caps| {
       // Expands to 3 separate mods
       // Return as: fire_res + cold_res + lightning_res, each with value
     }),

    // Adds damage to attacks
    (r"^Adds (\d+) to (\d+) (Physical|Fire|Cold|Lightning|Chaos) Damage to Attacks",
     |caps| ParsedMod {
       stat_id: format!("added_{}_damage_to_attacks", caps[3].to_lowercase()),
       value: (caps[1].parse::<f64>() + caps[2].parse::<f64>()) / 2.0,
       mod_type: FlatDamage,
       tags: vec![Attack, element_from(caps[3])],
     }),

    // More multiplier (from support gems)
    (r"^(\d+)% more (.+)",
     |caps| ParsedMod {
       stat_id: format!("more_{}", normalize_stat_name(caps[2])),
       value: caps[1].parse(),
       mod_type: More,
       tags: infer_tags(caps[2]),
     }),

    // Conversion
    (r"^(\d+)% of (Physical|Lightning|Cold|Fire) Damage Converted to (Lightning|Cold|Fire|Chaos) Damage",
     |caps| ParsedMod {
       stat_id: format!("{}_to_{}_conversion", caps[2].to_lowercase(), caps[3].to_lowercase()),
       value: caps[1].parse(),
       mod_type: Conversion,
       tags: vec![element_from(caps[2]), element_from(caps[3])],
     }),

    // Nearby enemies have -X% to Y resistance
    (r"^Nearby Enemies have (-?\d+)% to (Fire|Cold|Lightning) Resistance",
     |caps| ParsedMod {
       stat_id: format!("nearby_enemy_{}_resistance", caps[2].to_lowercase()),
       value: caps[1].parse(),  // negative value
       mod_type: EnemyModifier,
       tags: vec![element_from(caps[2])],
     }),

    // ... 200+ more patterns covering all PoE mod text formats
  ]

  for (pattern, parser) in PATTERNS:
    if let Some(caps) = pattern.captures(mod_text):
      return parser(caps)

  // Unrecognized mod → log warning, return Unknown
  log::warn!("Unrecognized mod: {}", mod_text)
  ParsedMod { stat_id: "unknown", value: 0.0, mod_type: Unknown, tags: vec![] }
```

### Stat Name Normalization

```
fn normalize_stat_name(raw: &str) -> String:
  raw.to_lowercase()
    .replace(" ", "_")
    .replace("'", "")
    // Common PoE → internal mappings:
    // "Fire Damage" → "fire_damage"
    // "Attack Speed" → "attack_speed"
    // "Critical Strike Chance" → "critical_strike_chance"
    // "Damage over Time Multiplier" → "dot_multiplier"
```

### Handling Compound Mods

Some mods have multiple effects in one line:
```
"+15% to Fire and Cold Resistances"
  → fire_resistance: 15, cold_resistance: 15

"Adds 10 to 20 Physical Damage to Attacks and Spells"
  → added_physical_damage_to_attacks: 15, added_physical_damage_to_spells: 15
```

These are expanded into multiple `ParsedMod` entries during parsing.

### Test Validation

Match our parser output against RePoE's canonical mod database to ensure every
stat_id we produce matches a known game stat.

---

## 24. Change Detection — Hash-Based Lazy Recalculation

### Problem

The full calculator takes ~50ms. We don't want to re-run it on every UI interaction.
Only re-calculate when something actually changed.

### Algorithm — Structural Hashing with Granular Invalidation

```
struct BuildHash {
  items_hash: u64,        // hash of all equipped items
  tree_hash: u64,         // hash of allocated nodes
  gems_hash: u64,         // hash of socket groups + gem levels
  config_hash: u64,       // hash of config toggles
  
  // Per-section hashes for granular invalidation
  per_slot_hash: HashMap<Slot, u64>,  // hash per item slot
}

struct CalcCache {
  last_hash: BuildHash,
  last_offense: Option<OffenseResult>,
  last_defense: Option<DefenseResult>,
}

fn should_recalculate(cache: &CalcCache, build: &BuildData) -> RecalcScope:
  new_hash = compute_hash(build)
  
  if new_hash == cache.last_hash:
    return RecalcScope::None  // nothing changed
  
  // Determine what changed
  if new_hash.items_hash != cache.last_hash.items_hash:
    // Which specific slot changed?
    changed_slots = []
    for (slot, hash) in new_hash.per_slot_hash:
      if hash != cache.last_hash.per_slot_hash[slot]:
        changed_slots.push(slot)
    
    // Item change affects both offense and defense
    return RecalcScope::Full
  
  if new_hash.gems_hash != cache.last_hash.gems_hash:
    // Gem change affects offense only (usually)
    return RecalcScope::OffenseOnly
  
  if new_hash.tree_hash != cache.last_hash.tree_hash:
    // Tree change affects both
    return RecalcScope::Full
  
  if new_hash.config_hash != cache.last_hash.config_hash:
    // Config change (enemy is boss, etc.) may affect offense or defense
    return RecalcScope::Full

fn compute_hash(build: &BuildData) -> BuildHash:
  // Use a fast non-cryptographic hash (FxHash or xxHash)
  BuildHash {
    items_hash: hash_items(&build.items),
    tree_hash: hash_tree(&build.allocated_nodes),
    gems_hash: hash_gems(&build.socket_groups),
    config_hash: hash_config(&build.config),
    per_slot_hash: build.items.iter()
      .map(|item| (item.slot, hash_item(item)))
      .collect(),
  }

fn hash_item(item: &Item) -> u64:
  let mut hasher = FxHasher::default()
  hasher.write(item.base.as_bytes())
  hasher.write(item.rarity as u8)
  for m in &item.mods:
    hasher.write(m.stat_id.as_bytes())
    hasher.write(&m.value.to_le_bytes())
  hasher.finish()
```

### Cache Invalidation Rules

| Change | Invalidates |
|--------|------------|
| Item swap/mod change | Offense + Defense |
| Gem level/swap | Offense only |
| Tree node allocate/respec | Offense + Defense |
| Config toggle | Offense + Defense |
| Flask change | Defense only |
| Market price update | Nothing (prices don't affect calc) |

---

## 25. Fast Estimation Engine — Pre-Computed Impact Tables

### Problem

When browsing items in trade or stash, we need instant (~1ms) DPS/life change
estimates. The full calculator at 50ms is too slow for hover-preview on a list
of 50 items.

### Algorithm — Linear Approximation via Pre-Computed Derivatives

On build load (or any full recalculation), we pre-compute the marginal impact
of each stat type. Then estimates are just multiplication.

```
struct ImpactTable {
  // Per stat type: how much does +1 of this stat change DPS/life?
  dps_per_unit: HashMap<StatType, f64>,
  life_per_unit: HashMap<StatType, f64>,
  
  // Built for a specific build state
  build_hash: u64,
}

fn build_impact_table(build: &BuildData, calc: &PathCalcEngine) -> ImpactTable:
  base = calc.calculate(build)
  table = ImpactTable::new()
  
  for stat in ESTIMABLE_STATS:
    // Add a small delta of this stat
    delta = match stat:
      FlatLife => 10.0          // +10 flat life
      PercentLife => 1.0        // +1% increased life
      FireDotMulti => 1.0       // +1% DoT multi
      FlatPhysDamage => 10.0    // +10 flat phys
      AttackSpeed => 1.0        // +1% attack speed
      CritChance => 1.0         // +1% crit chance
      CritMulti => 1.0          // +1% crit multi
      FireRes => 1.0            // +1% fire res
      // ... all stat types
    
    modified = build.add_stat(stat, delta)
    result = calc.calculate(&modified)
    
    table.dps_per_unit[stat] = (result.dps - base.dps) / delta
    table.life_per_unit[stat] = (result.life - base.life) / delta
  
  table.build_hash = compute_hash(build)
  table

fn estimate_item_impact(item: &Item, current_item: &Item, table: &ImpactTable) -> Estimate:
  dps_change = 0.0
  life_change = 0.0
  
  // Sum up contributions of new item's mods
  for m in item.mods:
    dps_change += m.value * table.dps_per_unit.get(m.stat_type).unwrap_or(0.0)
    life_change += m.value * table.life_per_unit.get(m.stat_type).unwrap_or(0.0)
  
  // Subtract contributions of current item's mods
  for m in current_item.mods:
    dps_change -= m.value * table.dps_per_unit.get(m.stat_type).unwrap_or(0.0)
    life_change -= m.value * table.life_per_unit.get(m.stat_type).unwrap_or(0.0)
  
  Estimate {
    dps_change,
    life_change,
    is_estimate: true,  // show "~" prefix in UI
  }
```

### Accuracy

Linear approximation is accurate when:
- The perturbation is small relative to current values ✓ (single item swap)
- The function is approximately linear in this region ✓ (for most stats)

Inaccurate when:
- Crit chance near breakpoints (getting to 100% effective crit is non-linear)
- Resist changes near cap (going from 74% → 76% is different from 74% → 74%)
- Conversion interactions (new item adds conversion → non-linear interaction)

For these edge cases, the UI shows "~" prefix and the full calculator runs when
the user clicks for details. The 500ms auto-upgrade rule in ENGINE-DESIGN.md
ensures users always get exact numbers for items they're seriously considering.

### Table Refresh

- Rebuild table on every full recalculation (item swap, tree change, etc.)
- Table is valid as long as build_hash matches
- Building the table: ~15 stats × 50ms = 750ms. Run in background on build load.
- Can parallelize: each stat perturbation is independent → tokio::spawn

---

## 29. Clipboard Item Parser

### Problem

When a player presses Ctrl+C over an item in Path of Exile, the game writes the
item data to the clipboard in a specific text format. The user can paste this into
Path of AI for instant analysis (score vs current gear, DPS/life impact, market value).
This format is different from both the PoB item text format (Algorithm 22) and the
poe.trade API format.

### PoE Clipboard Item Format

```
Item Class: Body Armours
Rarity: Rare
Glyph Crest
Astral Plate
--------
Quality: 20% (augmented)
Armour: 553 (augmented)
--------
Requirements:
Level: 62
Str: 180
--------
Sockets: R-R-R-G B B
--------
Item Level: 86
--------
+92 to maximum Life
+38% to Fire Resistance
+41% to Cold Resistance
+35% to Lightning Resistance
15% increased Movement Speed
--------
Note: ~price 1 divine
```

### Algorithm — Line-by-Line State Machine

```
Input:
  clipboard_text: String  // raw clipboard content from Ctrl+C in PoE

Output:
  Result<ClipboardItem, ParseError>

struct ClipboardItem {
  rarity: Rarity,
  name: String,
  base_type: String,
  item_class: ItemClass,
  item_level: u8,
  quality: u8,
  sockets: String,          // e.g., "R-R-R-G B B"
  requirements: StatReqs,
  properties: ItemProperties,  // armour, es, evasion, ward, dps (for weapons)
  implicit_mods: Vec<String>,
  explicit_mods: Vec<String>,
  fractured_mods: Vec<String>,
  corrupted: bool,
  note: Option<String>,     // trade price note if set
}

Algorithm:
  sections = clipboard_text.split("--------\n")
  // PoE uses "--------" as section separators

  // Section 0: Item Class and Rarity + Name + Base
  header_lines = sections[0].lines()
  item_class = parse_item_class(header_lines[0])  // "Item Class: Body Armours"
  rarity     = parse_rarity(header_lines[1])      // "Rarity: Rare"
  name       = header_lines[2].trim()             // item name
  base_type  = if rarity != Unique && rarity != Normal {
    header_lines[3].trim()  // second line is base type for Rare/Magic
  } else {
    header_lines[2].trim()  // Normal/Unique: name IS base type (or unique uses name)
  }

  // Section 1: Properties (Armour, ES, APS, etc.)
  properties = parse_properties(sections[1])
  quality    = extract_quality(sections[1])  // "Quality: 20%"

  // Section 2: Requirements
  requirements = parse_requirements(sections[2])

  // Section 3: Sockets
  sockets = parse_sockets(sections[3])

  // Section 4: Item Level
  item_level = parse_item_level(sections[4])  // "Item Level: 86"

  // Section 5+: Mods (variable — depends on rarity and implicit count)
  // Implicit mods are in a section BEFORE explicit mods
  // Fractured mods are suffixed with "(fractured)"
  // Corrupted is indicated by a single "Corrupted" line in its own section
  state = ParsingImplicits
  for section in sections[5..]:
    if section.trim() == "Corrupted":
      corrupted = true
      continue
    if section.starts_with("Note:"):
      note = Some(section[6..].trim())
      continue
    for line in section.lines():
      if line.ends_with("(fractured)"):
        fractured_mods.push(line.trim_end_matches("(fractured)").trim())
      else if state == ParsingImplicits:
        implicit_mods.push(line.trim())
      else:
        explicit_mods.push(line.trim())
    state = ParsingExplicits  // after first section of mods, rest are explicit

  // Parse each mod text using Algorithm 23 (Mod Text Parser)
  parsed_mods = explicit_mods.iter()
    .filter_map(|text| mod_text_parser.parse(text).ok())
    .collect()
```

### Integration: Instant Analysis Flow

```
User pastes clipboard content
  ↓
[29] Clipboard Item Parser → ClipboardItem
  ↓
[23] Mod Text Parser (per mod line) → Vec<ParsedMod>
  ↓
[25] Fast Estimation Engine → Estimate { dps_change, life_change }
  ↓
[21] Market Price Cache → price of this item type
  ↓
Display:
  "Ring 2 → Clipboard Ring: +8.3% DPS, +120 life | ~3.5 divine market value"
```

### Weapon DPS Parsing

For weapons, the clipboard includes calculated DPS directly:
```
Physical Damage: 150-200 (augmented)
Elemental Damage: 50-100 Fire (augmented)
Attacks per Second: 1.50 (augmented)

// Parse weapon DPS:
phys_avg = (150 + 200) / 2.0 = 175
fire_avg = (50 + 100) / 2.0 = 75
aps = 1.50
weapon_phys_dps = phys_avg * aps = 262.5
weapon_elem_dps = fire_avg * aps = 112.5
// These feed into the attack DPS calculator
```

### Complexity

- O(L) where L = lines in clipboard text (~10-50 lines). < 1ms.

---

## 30. Stat Requirement Checker

### Problem

PoE items have attribute requirements (Strength, Dexterity, Intelligence). If the
player's total attributes don't meet an item's requirements, they cannot equip it.
Algorithm 8 references this check but the algorithm is never defined.

### Algorithm

```
Input:
  build: BuildData           // all equipped items, passive tree, gems
  candidate_item: Option<Item>  // item being considered for equip (None = check current)

Output:
  deficiencies: Vec<StatDeficiency>

struct StatDeficiency {
  stat: Attribute,        // Str, Dex, Int
  required: u32,
  available: u32,
  shortfall: i32,         // negative = deficient
  blocking_item: ItemSlot,  // which item imposes this requirement
}

Algorithm:

Step 1: CALCULATE TOTAL AVAILABLE ATTRIBUTES
  // Attributes come from: items, passive tree, ascendancy, base class bonus
  // Note: some items provide attributes on the item ITSELF — those count for equipping
  // OTHER items (global attributes), but NOT for the item that provides them.

  base_str = 14  // base class str (varies by class)
  base_dex = 14
  base_int = 14

  total_str = base_str
  total_dex = base_dex
  total_int = base_int

  for item in build.equipped_items:
    for m in item.mods.filter(|m| m.stat_type == Attribute):
      match m.stat_type:
        Strength     => total_str += m.value as u32
        Dexterity    => total_dex += m.value as u32
        Intelligence => total_int += m.value as u32
        AllAttributes => { total_str += m.value; total_dex += m.value; total_int += m.value }

  for node in build.allocated_tree_nodes:
    // Same pattern — passive tree attribute nodes
    ...

Step 2: CHECK EACH ITEM'S REQUIREMENTS
  deficiencies = []

  items_to_check = build.equipped_items.clone()
  if let Some(candidate) = candidate_item:
    // Simulate swapping — remove the item in the target slot, add candidate
    items_to_check.replace(candidate.slot, candidate)

  for item in items_to_check:
    let reqs = item.requirements

    if reqs.str > 0 && total_str < reqs.str:
      deficiencies.push(StatDeficiency {
        stat: Strength,
        required: reqs.str,
        available: total_str,
        shortfall: total_str as i32 - reqs.str as i32,
        blocking_item: item.slot,
      })
    // Same for dex, int

Step 3: SORT by shortfall (most severe first)
  deficiencies.sort_by_key(|d| d.shortfall)

  // ALSO check gem requirements — gems have stat requirements too
  for gem in build.all_gems:
    if gem.requirements.str > total_str:
      deficiencies.push(StatDeficiency {
        stat: Strength, required: gem.requirements.str, available: total_str,
        shortfall: total_str as i32 - gem.requirements.str as i32,
        blocking_item: gem.socket_location.slot,
      })
```

### Attribute Annointment & Mastery Attributes

Some builds use attribute-granting mods on rings/amulets to meet requirements.
The "60 to 100 Strength" ring mod is common for meeting Eternal Sword requirements.
The checker should also flag when a swap would break attribute requirements for OTHER items.

```
// When evaluating candidate_item for slot X:
// Temporarily simulate the swap. If the new item gives LESS str/dex/int than current:
attribute_delta = candidate_item.attribute_bonus - current_item.attribute_bonus
if attribute_delta.str < 0:
  // Check if ANY remaining items would now be under-requirement
  for other_item in all_equipped.except(slot_x):
    if total_str + attribute_delta.str < other_item.requirements.str:
      deficiencies.push(...)  // swap breaks another item's requirement
```

### Complexity

- O(I × G) where I = items, G = gems. Typically < 1ms.

---

## 31. Charge Management

### Problem

PoE has three charge types — Endurance (red), Frenzy (green), and Power (blue).
Each has a maximum count (base 3, raised by tree/items), generation conditions,
expiry timer, and stat bonuses per charge. Charges feed into Algorithm 1's
conditional modifier evaluation (`HasEnduranceCharge`, etc.) and the combat
simulator. Without a clear spec, implementors guess at the decay and generation rules.

### Charge Properties

```rust
pub struct ChargeConfig {
    pub max_endurance: u8,   // base 3, +1 per "maximum endurance charges" node
    pub max_frenzy:    u8,
    pub max_power:     u8,
    pub endurance_duration_secs: f64,  // base 10s, extended by "charge duration" mods
    pub frenzy_duration_secs:    f64,  // base 10s
    pub power_duration_secs:     f64,  // base 10s
}

/// Stat bonuses PER CHARGE (additive, applied in modifier aggregation)
const ENDURANCE_BONUS_PER_CHARGE: &[(&str, f64)] = &[
    ("physical_damage_reduction_pct", 4.0),  // 4% phys reduction per charge
    ("all_elemental_resistances", 4.0),       // 4% to all ele res per charge
];

const FRENZY_BONUS_PER_CHARGE: &[(&str, f64)] = &[
    ("increased_attack_speed", 4.0),
    ("increased_cast_speed",   4.0),
    ("increased_damage",       4.0),
];

const POWER_BONUS_PER_CHARGE: &[(&str, f64)] = &[
    ("increased_critical_strike_chance", 40.0),  // 40% increased crit per charge
];
```

### Generation Algorithm

```
// Charge generation is event-driven (not tick-based)
// Each charge has an independent expiry timer that resets on gain

struct ChargeState {
  counts: [u8; 3],          // [endurance, frenzy, power]
  expiry_ms: [u32; 3],      // ms until next charge lost (per type)
}

On relevant event:
  EnduranceCharge:
    on_stun:      if player.has("gain_endurance_on_stun"):  gain(Endurance, 1)
    on_kill:      if player.has("endurance_charge_on_kill"): gain(Endurance, 1)
    warcry_exert: if warcry grants endurance charges: gain(Endurance, warcry.endurance_count)
    Juggernaut:   gain 1 Endurance every 2s in combat (Unrelenting notable)

  FrenzyCharge:
    on_hit:       if player.has("frenzy_charge_on_hit") && random() < hit_chance: gain(Frenzy, 1)
    on_kill:      if player.has("frenzy_charge_on_kill"):  gain(Frenzy, 1)
    Raider:       gain via Frenzy mechanics (ascendancy passives)

  PowerCharge:
    on_crit:      if player.has("power_charge_on_crit"):   gain(Power, 1)
    on_kill:      if player.has("power_charge_on_kill"):   gain(Power, 1)
    Power Siphon: gain power charges on kill (skill mechanic)

fn gain(charge_type: ChargeType, count: u8, state: &mut ChargeState, config: &ChargeConfig):
  let idx = charge_type as usize
  let max = [config.max_endurance, config.max_frenzy, config.max_power][idx]
  state.counts[idx] = (state.counts[idx] + count).min(max)
  // Reset expiry timer — charges expire from the LAST gain, not individually
  state.expiry_ms[idx] = (duration_secs(charge_type, config) * 1000.0) as u32

Per tick (dt_ms):
  for i in 0..3:
    if state.counts[i] > 0:
      if state.expiry_ms[i] <= dt_ms:
        state.counts[i] -= 1       // lose one charge
        state.expiry_ms[i] = (duration_secs(i, config) * 1000.0) as u32  // reset for next
      else:
        state.expiry_ms[i] -= dt_ms
```

### Integration with Modifier Aggregation

```
// In Algorithm 1 Pass 2 — conditional modifiers with charge counts:
for (condition, mod_entry) in &pool.conditional:
  let include = match condition:
    Condition::HasEnduranceCharge(n) =>
      current_charges.endurance >= n,  // "per endurance charge": n=1, stack manually
    Condition::HasFrenzyCharge(n)    => current_charges.frenzy >= n,
    Condition::HasPowerCharge(n)     => current_charges.power >= n,

// For "per charge" mods (e.g., "4% phys reduction per endurance charge"):
//   mod.value is the per-charge value
//   effective_value = mod.value * current_charges.endurance
//   This is applied as a flat modifier to the relevant stat
```

### Calculator vs Simulator

- **Calculator**: use `build.config.assumed_charges` (user sets endurance/frenzy/power count in UI)
- **Simulator**: charges evolve dynamically per tick via the generation algorithm above

### Complexity

- O(1) per gain/tick event
- O(C) where C = 3 charge types

---

## 32. Playstyle Classifier

### Problem

Beyond archetype (what skill, what damage type), builds have a *playstyle* —
how they interact with the game's mechanics. A Righteous Fire Inquisitor is
`FireDotRF` archetype but could be `immortal_tank` or `glass_cannon` playstyle
depending on defenses. Playstyle drives UI labels, AI coaching tone, and suggestion
priorities (e.g., don't suggest DPS upgrades to an immortal_tank build that already
one-shots content).

### Algorithm — Threshold-Based Trait Accumulation

```rust
pub enum Playstyle {
    ImmortalTank,    // max block + high regen — designed to facetank everything
    TankyFacetank,   // high armour + moderate block — durable but not immortal
    DodgeKite,       // high evasion — avoids hits rather than tanking them
    GlassCannon,     // very high DPS, minimal defenses
    SpeedRunner,     // high movement speed, maps fast, trades survivability
    SummonerBack,    // minion build — player stays behind minions
    Balanced,        // no extreme in either direction
}

pub struct PlaystyleTraits {
    pub style: Playstyle,
    pub traits: Vec<&'static str>,   // e.g., ["max_block", "high_regen"]
    pub description: &'static str,
}

fn classify_playstyle(calc: &CalcResult, build: &BuildData) -> PlaystyleTraits:
  let mut traits = vec![]

  // Defense traits
  if calc.defense.block_chance >= 0.75        { traits.push("max_block") }
  else if calc.defense.block_chance >= 0.50   { traits.push("moderate_block") }
  if calc.defense.phys_reduction_display() >= 60.0 { traits.push("high_armour") }
  if calc.defense.evasion_chance >= 0.60      { traits.push("high_evasion") }
  if calc.defense.life_regen_per_second >= 2000.0  { traits.push("high_regen") }
  if calc.defense.spell_suppression_chance >= 0.50 { traits.push("spell_suppression") }

  // Offense traits
  let dps = calc.offense.effective_dps_vs_map
  if dps >= 10_000_000.0 { traits.push("godlike_dps") }
  else if dps >= 5_000_000.0  { traits.push("high_dps") }
  else if dps >= 2_000_000.0  { traits.push("moderate_dps") }
  else                         { traits.push("low_dps") }

  // Movement trait
  if calc.defense.movement_speed_pct >= 30.0  { traits.push("high_ms") }

  // Minion trait
  if build.archetype.is_minion()              { traits.push("minion_commander") }

  // Classify overall playstyle from trait combination
  let style = if traits.contains(&"max_block") && traits.contains(&"high_regen") {
    Playstyle::ImmortalTank
  } else if traits.contains(&"high_armour") && traits.contains(&"moderate_block") {
    Playstyle::TankyFacetank
  } else if traits.contains(&"high_evasion") && !traits.contains(&"high_armour") {
    Playstyle::DodgeKite
  } else if traits.contains(&"high_dps") && !traits.contains(&"high_armour")
         && !traits.contains(&"high_evasion") {
    Playstyle::GlassCannon
  } else if traits.contains(&"high_ms") && traits.contains(&"high_dps") {
    Playstyle::SpeedRunner
  } else if traits.contains(&"minion_commander") {
    Playstyle::SummonerBack
  } else {
    Playstyle::Balanced
  }

  PlaystyleTraits { style, traits, description: PLAYSTYLE_DESCRIPTIONS[style] }
```

### How Playstyle Affects the UI

| Playstyle | Suggestion Priority | Coaching Tone |
|-----------|--------------------|--------------------|
| ImmortalTank | DPS upgrades first (defense is already solved) | "Your defenses are excellent — time to focus on damage." |
| GlassCannon | Defense upgrades first | "You have incredible damage but you're one-shot risk. Add a guard skill." |
| DodgeKite | Spell suppression + evasion improvements | "Your evasion is great vs attacks. Add spell suppression to handle spells." |
| SpeedRunner | Movement skill + flask uptime | "You're built for speed. Optimize flask uptime and AoE radius for clear." |
| SummonerBack | Minion survivability + aura coverage | "Your minions do the work. Improve their coverage and durability." |
| Balanced | Whichever stat has lowest score | Standard priority order. |

### Complexity

- O(T) where T = number of traits checked (~15 thresholds). < 1ms.

---

## 33. Change History Manager

### Problem

Every build modification — whether from the user editing in PoB, the file watcher
detecting a save, or Path of AI applying an upgrade suggestion — must be tracked so
the player can undo, redo, or revert to any earlier snapshot. Critically, the
"apply suggestion + simulate" flow requires a safe way to try a change, see the
simulated result, then commit or discard without side effects.

### Data Structures

```rust
pub struct ChangeHistory {
    /// Ring buffer of snapshots. Index 0 = oldest.
    snapshots: VecDeque<Snapshot>,
    /// Points to the "current" position in the ring. Everything after
    /// this index is the redo stack.
    cursor: usize,
    max_snapshots: usize,   // from config (default 50)
}

pub struct Snapshot {
    pub id:          Uuid,
    pub created_at:  DateTime<Utc>,
    pub description: String,        // "Ring 2 upgraded via AI suggestion"
    pub source:      ChangeSource,
    pub build_hash:  u64,           // fast equality check
    pub build_xml:   Option<Bytes>, // zlib-compressed PoB XML (None for OAuth builds)
    pub build_state: BuildData,     // structured — always present
    pub stats_before: SnapshotStats,
    pub stats_after:  SnapshotStats,  // populated after calc runs
}

pub struct SnapshotStats {
    pub total_dps:    f64,
    pub effective_hp: f64,
    pub fire_res:     f64,
    pub cold_res:     f64,
    pub lightning_res: f64,
    pub chaos_res:    f64,
    pub item_scores:  HashMap<EquipSlot, u8>,
}

pub enum ChangeSource {
    UserEdit,              // player changed something manually in PoB
    FileWatcher,           // file-watcher detected a PoB save
    AiSuggestion(Uuid),    // Path of AI applied suggestion with this ID
    OAuthSync,             // live character data refreshed from GGG
    ManualImport,          // user pasted share code or imported file
    Undo,                  // this snapshot was created by undoing
    Redo,
}
```

### Algorithm — Push / Undo / Redo / Revert

```rust
impl ChangeHistory {

    /// Record a new state. Clears the redo stack (anything after cursor).
    pub fn push(&mut self, mut snap: Snapshot) -> &Snapshot {
        // Trim redo stack: discard snapshots after cursor
        self.snapshots.truncate(self.cursor + 1);

        // Enforce ring-buffer limit
        if self.snapshots.len() >= self.max_snapshots {
            self.snapshots.pop_front();
        }

        snap.stats_before = self.current().map(|s| s.stats_after.clone())
            .unwrap_or_default();
        self.snapshots.push_back(snap);
        self.cursor = self.snapshots.len() - 1;
        self.snapshots.back().unwrap()
    }

    /// Undo: step back one snapshot, return it for re-analysis.
    pub fn undo(&mut self) -> Option<&Snapshot> {
        if self.cursor == 0 { return None; }
        self.cursor -= 1;
        Some(&self.snapshots[self.cursor])
    }

    /// Redo: step forward if the redo stack exists.
    pub fn redo(&mut self) -> Option<&Snapshot> {
        if self.cursor + 1 >= self.snapshots.len() { return None; }
        self.cursor += 1;
        Some(&self.snapshots[self.cursor])
    }

    /// Revert to ANY earlier snapshot by index (not just the previous one).
    /// Creates a NEW snapshot from the target state so the revert itself
    /// is undoable. The revert appears as an entry in the timeline.
    pub fn revert_to(&mut self, target_id: Uuid) -> Option<Snapshot> {
        let target = self.snapshots.iter().find(|s| s.id == target_id)?.clone();
        let revert_snap = Snapshot {
            id: Uuid::new_v4(),
            description: format!("Reverted to: {}", target.description),
            source: ChangeSource::Undo,
            build_state: target.build_state.clone(),
            build_xml:   target.build_xml.clone(),
            ..Snapshot::now()
        };
        self.push(revert_snap);
        Some(target)
    }

    pub fn current(&self) -> Option<&Snapshot> {
        self.snapshots.get(self.cursor)
    }

    /// Full timeline for UI — ordered oldest→newest, marks cursor position.
    pub fn timeline(&self) -> Vec<TimelineEntry> {
        self.snapshots.iter().enumerate().map(|(i, s)| TimelineEntry {
            snapshot: s,
            is_current: i == self.cursor,
            is_redo: i > self.cursor,
        }).collect()
    }
}
```

### Apply-Suggestion Flow (try → simulate → commit or discard)

The most important user flow: user sees an AI suggestion, clicks "Apply & Simulate",
views the result, then decides to keep or revert.

```
User clicks "Apply & Simulate" on a suggestion
  ↓
[1] SNAPSHOT current state (source = AiSuggestion(suggestion_id))
  history.push(current_snapshot)
  ↓
[2] APPLY suggestion to build in memory (do NOT write to disk yet)
  modified_build = apply_suggestion(current_build, suggestion)
  ↓
[3] RUN FULL CALCULATOR on modified build
  new_result = calc.calculate(&modified_build)
  ↓
[4] SHOW diff in UI
  diff = {
    dps_change:     new_result.offense.total_dps - old_result.offense.total_dps,
    dps_pct:        dps_change / old_dps * 100,
    life_change:    new_result.defense.max_life - old_result.defense.max_life,
    resist_changes: per-element delta,
    score_changes:  per-slot delta,
  }
  UI shows side-by-side before/after panel
  Buttons: [✅ Commit] [↩ Discard]
  ↓
IF user clicks Commit:
  [5a] WRITE to PoB file (atomic write, Algorithm 22)
       history.current().stats_after = new_stats  // update snapshot with actual results
  ↓
IF user clicks Discard:
  [5b] REVERT in memory (do NOT write to disk)
       history.undo()  // pop the suggestion snapshot
       UI returns to original state instantly
```

### Stats Diff Format

```rust
pub struct StatDiff {
    // Shown in the "Apply & Simulate" before/after panel
    pub dps_raw:    (f64, f64),    // (before, after)
    pub dps_pct:    f64,           // e.g. +12.4
    pub life:       (f64, f64),
    pub fire_res:   (f64, f64),
    pub cold_res:   (f64, f64),
    pub light_res:  (f64, f64),
    pub chaos_res:  (f64, f64),
    pub item_scores: HashMap<EquipSlot, (u8, u8)>,  // per slot (before, after)
    pub new_issues:  Vec<Issue>,    // issues introduced by this change
    pub fixed_issues: Vec<Issue>,   // issues resolved by this change
}
```

### Persistence

- Snapshots are stored in SQLite `build_snapshots` table (DATABASE.md schema)
- `build_xml` stored zlib-compressed (average 5-20KB per snapshot)
- Loaded into memory on app start, capped at `max_snapshots` (default 50)
- Snapshots older than 30 days auto-pruned on startup (configurable)

### Complexity

- push / undo / redo: O(1)
- revert_to: O(N) scan, N = snapshot count (max 50) → effectively O(1)
- diff computation: O(D) where D = changed stat count

---

## 34. OAuth Token Lifecycle

### Problem

PoE OAuth access tokens expire (typically 1 hour). The app must silently refresh
them without interrupting the user, handle revoked tokens gracefully, and on
"reimport" detect whether the live server data conflicts with any local edits.

### Token Storage

```rust
// Tokens are stored in the OS keychain (Tauri Stronghold plugin)
// NEVER written to disk in plaintext. NEVER logged.
pub struct StoredToken {
    pub access_token:  String,    // short-lived (1 hour)
    pub refresh_token: String,    // long-lived (can be weeks)
    pub expires_at:    DateTime<Utc>,
    pub scope:         Vec<String>,  // e.g. ["account:characters", "account:stashes"]
    pub account_name:  String,
}

// Keychain key: "pathofai_poe_token_{account_name}"
// Stored as JSON, encrypted by OS (macOS Keychain / Windows Credential Manager)
```

### Refresh Algorithm

```rust
/// Called before every API request. Proactively refreshes if token expires
/// within the next 5 minutes — avoids 401 errors during multi-request flows.
pub async fn ensure_valid_token(
    token: &mut StoredToken,
    http: &HttpClient,
) -> Result<&str, TokenError> {

    let expires_soon = token.expires_at < Utc::now() + Duration::minutes(5);

    if !expires_soon {
        return Ok(&token.access_token);  // still valid
    }

    // Attempt silent refresh using refresh_token
    let response = http.post("https://www.pathofexile.com/oauth/token")
        .form(&[
            ("grant_type",    "refresh_token"),
            ("refresh_token", &token.refresh_token),
            ("client_id",     CLIENT_ID),
        ])
        .send().await?;

    match response.status() {
        200 => {
            let new_token: TokenResponse = response.json().await?;
            token.access_token  = new_token.access_token;
            token.refresh_token = new_token.refresh_token.unwrap_or(token.refresh_token.clone());
            token.expires_at    = Utc::now() + Duration::seconds(new_token.expires_in);
            keychain::save(token)?;  // persist updated token
            Ok(&token.access_token)
        }
        400 | 401 => {
            // Refresh token itself is expired/revoked — user must re-authenticate
            keychain::delete(&token.account_name)?;
            Err(TokenError::RefreshExpired)  // UI shows "Reconnect PoE Account" prompt
        }
        429 => Err(TokenError::RateLimited),
        _   => Err(TokenError::Unknown(response.status())),
    }
}
```

### Reimport / Sync Flow

When the user clicks "Refresh Character" or the 5-minute auto-sync fires:

```
[1] ensure_valid_token() → get fresh access token
  ↓
[2] Fetch live character data from GGG API:
    GET /character/{name}         → equipped items
    GET /character/{name}/skills  → gem links
    GET /character/{name}/passives → passive tree
  ↓
[3] CONFLICT DETECTION
    server_hash = hash(server_data)
    local_hash  = history.current().build_hash

    if server_hash == local_hash:
      return Ok(NoChange)  // nothing to do

    if history.current().source == ChangeSource::AiSuggestion(_):
      // Player applied an AI suggestion locally but hasn't actually
      // changed the in-game character yet. Server data = OLDER state.
      // Don't overwrite the local suggestion — show a warning instead:
      emit ConflictWarning {
        local:  "You have local changes (AI suggestion applied)",
        server: "Server has older data",
        options: ["Keep local", "Pull from server", "Show diff"],
      }
    else:
      // Server has newer data (player made changes in-game)
      // Record the server pull as a new snapshot
      history.push(Snapshot {
        source: ChangeSource::OAuthSync,
        description: "Synced from PoE account",
        build_state: server_build_data,
        ..
      })
      run_full_analysis()
  ↓
[4] Emit 'character_updated' event to frontend → UI refreshes
```

### OAuth Scope Required

```
account:characters   — list and view character data
account:stashes      — view stash tab contents (for currency tracking)
account:profile      — account name, league membership
```

### Disconnect / Revoke

```rust
pub fn disconnect_account(account_name: &str) -> Result<()> {
    keychain::delete(account_name)?;
    db::delete_character_data(account_name)?;
    // Does NOT delete build snapshots — player keeps their history
    emit AccountDisconnected { account_name }
    Ok(())
}
```

### Complexity

- `ensure_valid_token`: O(1) check; O(1) network round-trip on refresh
- Conflict detection: O(1) hash compare

---

## 35. Session Persistence & Auto-Save

### Problem

If the app crashes or the user closes it mid-analysis, they should return to
exactly where they left off — same character, same active tab, same pending
suggestion. Without explicit auto-save, any in-memory state is lost.

### What Must Persist

```rust
/// Saved to `PathOfAI_Data/config/session.json` atomically every time it changes.
pub struct SessionState {
    // Which character/build is active
    pub active_build_id:        Option<String>,
    pub active_character_name:  Option<String>,  // if OAuth
    pub active_pob_path:        Option<PathBuf>, // if PoB file

    // UI state
    pub active_tab:     Tab,    // Overview, Items, Tree, Crafting, Arena, etc.
    pub active_slot:    Option<EquipSlot>,  // which slot is selected in Items tab

    // Pending state (not yet committed)
    pub pending_suggestion_id: Option<Uuid>,  // suggestion shown but not applied

    // Last successful analysis (cached for instant display on restart)
    pub last_calc_result:  Option<CalcResult>,    // serialized
    pub last_analysis_at:  Option<DateTime<Utc>>,

    // Window state
    pub window_x: i32,
    pub window_y: i32,
    pub window_w: u32,
    pub window_h: u32,
    pub window_maximized: bool,
}
```

### Auto-Save Algorithm

```rust
/// Every state change triggers an async save. Writes are debounced (100ms)
/// so rapid changes don't hammer disk. Uses atomic write (temp → rename).

pub struct AutoSave {
    pending: Arc<Mutex<Option<SessionState>>>,
    save_handle: Option<JoinHandle<()>>,
}

impl AutoSave {
    pub fn mark_dirty(&self, state: SessionState) {
        *self.pending.lock() = Some(state);
        // Debounce: if save task already running, it will pick up latest state
        if self.save_handle.is_none() || self.save_handle.as_ref().unwrap().is_finished() {
            self.save_handle = Some(tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                if let Some(state) = self.pending.lock().take() {
                    atomic_write("session.json", &serde_json::to_vec(&state).unwrap()).await;
                }
            }));
        }
    }
}

/// Trigger auto-save on every meaningful state change:
///   - Tab switch
///   - Character selection change
///   - Suggestion applied or discarded
///   - Analysis completed (cache the CalcResult)
///   - Window moved/resized
```

### Startup Recovery Algorithm

```
App starts
  ↓
[1] Load session.json (if it exists and is valid JSON)
    If missing or corrupted → use default SessionState (first-launch defaults)
  ↓
[2] Restore window position/size from session.window_*
  ↓
[3] If active_build_id is set:
    Load that build's most recent snapshot from SQLite
    Restore to that CalcResult (instant display — no recalculation needed)
    Show "Last session: RF Inquisitor, 2.8M DPS" in status bar
  ↓
[4] In background (non-blocking):
    If OAuth character: check for server updates (Algorithm 34)
    If PoB file: check if file modified since last_analysis_at
    Run fresh analysis if stale — update display when complete
  ↓
[5] If pending_suggestion_id is set:
    The app crashed while showing a suggestion
    Re-show the suggestion panel with a "You had a pending suggestion" banner
    Do NOT auto-apply it — wait for user to explicitly commit or discard
```

### Atomic Write Implementation

```rust
/// Write `data` to `path` atomically: write temp file → fsync → rename.
/// If the process dies between write and rename, original file is untouched.
async fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    tokio::fs::write(&tmp, data).await?;
    // fsync ensures data is on disk before rename
    let file = tokio::fs::OpenOptions::new().write(true).open(&tmp).await?;
    file.sync_all().await?;
    drop(file);
    tokio::fs::rename(tmp, path).await?;
    Ok(())
}
// Same pattern used for PoB file writes (Algorithm 22) and snapshot saves.
```

### Complexity

- `mark_dirty`: O(1), fully async — never blocks the UI thread
- Startup recovery: O(1) session read + O(S) snapshot load from SQLite

---

## 36. Database Init & Migration

### Problem

The SQLite database must be created on first launch, upgraded when the schema
changes between versions, and recovered if corrupted. Without a clear migration
system, schema changes break existing installations.

### Algorithm — Versioned Migration Runner

```rust
/// Run on every app startup. Idempotent: safe to run multiple times.
pub async fn init_database(db: &SqlitePool) -> Result<()> {

    // 1. Ensure the migrations table exists (always safe)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version     INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            applied_at  TEXT NOT NULL
        )"
    ).execute(db).await?;

    // 2. Find current schema version
    let current: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) FROM _migrations"
    ).fetch_one(db).await?;

    // 3. Apply all pending migrations in order
    let migrations: &[(i64, &str, &str)] = &[
        // (version, name, sql)
        (1, "initial_schema",   include_str!("migrations/001_initial.sql")),
        (2, "add_snapshots",    include_str!("migrations/002_snapshots.sql")),
        (3, "add_price_hist",   include_str!("migrations/003_price_history.sql")),
        (4, "add_session",      include_str!("migrations/004_session.sql")),
        (5, "add_league_data",  include_str!("migrations/005_league.sql")),
        // New migrations added here — version number must be monotonically increasing
    ];

    for &(version, name, sql) in migrations {
        if version <= current { continue; }

        // Run migration inside a transaction — fully applied or fully rolled back
        let mut tx = db.begin().await?;
        sqlx::query(sql).execute(&mut tx).await.map_err(|e| {
            MigrationError { version, name, cause: e }
        })?;
        sqlx::query(
            "INSERT INTO _migrations (version, name, applied_at) VALUES (?, ?, ?)"
        ).bind(version).bind(name).bind(Utc::now().to_rfc3339())
         .execute(&mut tx).await?;
        tx.commit().await?;

        log::info!("Applied migration {}: {}", version, name);
    }

    Ok(())
}
```

### Corruption Recovery

```rust
pub async fn open_database(path: &Path) -> Result<SqlitePool> {
    match SqlitePoolOptions::new().connect(path.to_str().unwrap()).await {
        Ok(pool) => {
            // Quick integrity check
            let ok: String = sqlx::query_scalar("PRAGMA integrity_check")
                .fetch_one(&pool).await?;
            if ok != "ok" {
                return Err(DbError::Corrupted);
            }
            Ok(pool)
        }
        Err(e) if is_corruption_error(&e) => {
            log::error!("Database corrupted: {:?}", e);
            // Move corrupted file aside, start fresh
            let backup_path = path.with_extension(
                format!("corrupted_{}.db", Utc::now().timestamp())
            );
            tokio::fs::rename(path, &backup_path).await?;
            log::warn!("Moved corrupted DB to {:?}. Starting fresh.", backup_path);
            // Re-open (creates new empty file)
            let pool = SqlitePoolOptions::new().connect(path.to_str().unwrap()).await?;
            init_database(&pool).await?;
            Ok(pool)
        }
        Err(e) => Err(e.into()),
    }
}
```

### WAL Mode & Performance Settings

```sql
-- Applied once at connection time (before migrations)
PRAGMA journal_mode = WAL;       -- Write-Ahead Logging: readers don't block writers
PRAGMA synchronous  = NORMAL;    -- Balanced durability (safe with WAL)
PRAGMA cache_size   = -32000;    -- 32MB page cache
PRAGMA foreign_keys = ON;        -- Enforce referential integrity
PRAGMA temp_store   = MEMORY;    -- Keep temp tables in RAM
```

### Data Retention Cleanup

```rust
/// Called once on startup after migrations, then daily via background timer.
pub async fn cleanup_old_data(db: &SqlitePool, config: &RetentionConfig) -> Result<()> {
    let cutoff = Utc::now() - Duration::days(config.snapshot_retention_days as i64);

    // Keep last N snapshots per build regardless of age
    sqlx::query(
        "DELETE FROM build_snapshots
         WHERE id NOT IN (
           SELECT id FROM build_snapshots b2
           WHERE b2.build_id = build_snapshots.build_id
           ORDER BY created_at DESC
           LIMIT ?
         ) AND created_at < ?"
    ).bind(config.max_snapshots_per_build)
     .bind(cutoff.to_rfc3339())
     .execute(db).await?;

    // Clean stale price cache entries
    sqlx::query("DELETE FROM price_cache WHERE fetched_at < ?")
        .bind((Utc::now() - Duration::minutes(config.price_cache_ttl_minutes as i64))
              .to_rfc3339())
        .execute(db).await?;

    // VACUUM if database grew significantly (run at most weekly)
    let last_vacuum: Option<String> = sqlx::query_scalar(
        "SELECT value FROM settings WHERE key = 'last_vacuum'"
    ).fetch_optional(db).await?;
    let should_vacuum = last_vacuum
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|t| Utc::now() - t > Duration::days(7))
        .unwrap_or(true);
    if should_vacuum {
        sqlx::query("VACUUM").execute(db).await?;
        sqlx::query("INSERT OR REPLACE INTO settings VALUES ('last_vacuum', ?)")
            .bind(Utc::now().to_rfc3339()).execute(db).await?;
    }
    Ok(())
}
```

### Complexity

- Init/migration: O(M) migrations, each O(1). Runs once at startup (~50ms first install, ~1ms subsequent)
- Corruption recovery: O(1) rename + O(M) re-init
- Cleanup: O(N) rows scanned; VACUUM O(size of DB)

---

## Algorithm Dependency Graph

```
Build Load
  ├→ [22] PoB XML Parser → BuildData
  ├→ [23] Mod Text Parser (called by parser)
  │
  ├→ [1] Modifier Aggregation Pipeline → ModPool
  │   ├→ [2] Damage Conversion Graph Resolver
  │   └→ Full Calculator (ENGINE-DESIGN.md)
  │       ├→ Offense: flat → increased → more → crit → speed → DoT
  │       └→ Defense: resist → armour → evasion → block → guard → EHP
  │
  ├→ [6] Build Archetype Classifier → Archetype
  │
  ├→ [24] Change Detection (hash current state)
  ├→ [25] Fast Estimation Engine (build impact tables)
  │
  ├→ [7] Item Scoring (per slot, using archetype weights)
  ├→ [8] Issue Detector (constraint violations)
  ├→ [9] Sensitivity Analysis (stat priorities)
  │
  ├→ Upgrade Suggestions:
  │   ├→ [10] Pareto-Optimal Ranking
  │   ├→ [11] Knapsack Optimizer (budget-constrained)
  │   ├→ [12] Multi-Slot Constraint Solver
  │   ├→ [15] Crafting Probability Engine
  │   ├→ [16] Monte Carlo Craft Simulator
  │   └→ [17] Build Similarity / Collaborative Filtering
  │
  ├→ Passive Tree:
  │   ├→ [13] Pathfinding (A* to target node)
  │   └→ [14] Respec Optimizer (inefficient node detection)
  │
  ├→ Query Handling:
  │   ├→ [18] Intent Classifier → route to engine
  │   └→ [19] Template Response Generator → natural language output
  │
  ├→ [20] Combat Simulation Engine
  │   ├→ [3] Entropy-Based Evasion Counter
  │   ├→ [4] Leech Instance Manager
  │   └→ [5] Flask Uptime Model
  │
  ├→ [21] Market Price Cache & Circuit Breaker
  │
  ├→ [26] Ailment Mechanics (called by combat sim + calc)
  ├→ [27] Energy Shield Recharge (called by combat sim + defense calc)
  ├→ [28] Mana Reservation Engine (called on build load + aura change)
  ├→ [29] Clipboard Item Parser (Ctrl+C → ClipboardItem → [23] + [25])
  ├→ [30] Stat Requirement Checker (called by [8] Issue Detector + upgrade eval)
  ├→ [31] Charge Management (feeds into modifier aggregation + combat sim)
  ├→ [32] Playstyle Classifier (called after [6] archetype, feeds UI labels)
  ├→ [33] Change History Manager (undo/redo/revert/apply-suggestion flow)
  ├→ [34] OAuth Token Lifecycle (token refresh, reimport, conflict resolution)
  │     └← [37] OAuth PKCE Auth Flow (initial login → tokens → feeds [34])
  ├→ [35] Session Persistence & Auto-Save (crash recovery, state reload)
  ├→ [36] Database Init & Migration (SQLite setup, schema versioning)
  │
  ├→ OAuth / Stash path:
  │   ├→ [37] OAuth PKCE Authorization Flow → StoredToken
  │   └→ [38] Stash Tab Processor → StashResult
  │       ├→ [7] Item Scoring (score each stash item)
  │       └→ [25] Fast Estimation (quick item scoring)
  │
  ├→ Map Mods path:
  │   └→ [39] Map Mod Danger Scorer → MapDangerResult
  │       ├→ [1] Modifier Aggregation (build archetype + defense stats)
  │       └→ Alg 26 (ailment immunity flags)
  │
  ├→ Share path:
  │   └→ [40] Build Share Code Codec (encode/decode SharePayload)
  │
  ├→ Map Stats path:
  │   └→ [41] Client.txt Log Parser → session run statistics
  │
  ├→ Seer Engine (routes here from [18] Intent Classifier):
  │   └→ [42] Seer Network Architecture
  │       ├→ ItemNet  → upgrade scores (calls [7] Item Scoring)
  │       ├→ BuildNet → archetype + issues (calls [6] + [8])
  │       ├→ TreeNet  → node efficiency (calls [13] Pathfinding)
  │       ├→ QueryNet → intent + entities (feeds [18])
  │       ├→ EmbedNet → RAG knowledge retrieval (calls [17] cosine sim)
  │       └→ ResponseGen → natural language (calls [19] Template Gen)
  │
  ├→ Currency path:
  │   └→ [43] Vendor Recipe Detector → RecipeAnalysis
  │       └→ [38] Stash Tab Processor (item pool)
  │
  ├→ Infrastructure:
  │   └→ [44] Portable Storage & File Watcher
  │       ├→ [44a] Directory Init (runs at startup before all subsystems)
  │       └→ [44b] PoB File Watcher (drives [22] PoB XML Parser on change)
  │
  ├→ Character / Account path:
  │   └→ [45] PoE Character Fetch Pipeline
  │       ├→ [34] OAuth Token Lifecycle (ensure_valid_token before each call)
  │       ├→ [35] Session Persistence (switch_character updates session)
  │       └→ [38] Stash Tab Processor (stash fetch after character load)
  │
  ├→ Write-back path:
  │   └→ [46] PoB Write-Back Engine
  │       ├→ [33] Change History Manager (snapshot before + after)
  │       └→ [35] atomic_write (same helper used here)
  │
  ├→ Craft / Trade path:
  │   ├→ [47] Craft Suggestion Ranker & Trade Search
  │   │   ├→ [15] Crafting Probability Engine (success rate per method)
  │   │   └→ [21] Price Cache (cost per attempt in divine)
  │   └→ [48] Top-N Passive Node Recommender
  │       ├→ [42] TreeNet (node scoring)
  │       └→ [7]  Item Scoring (archetype weights reused)
  │
  ├→ Comparison path:
  │   └→ [49] Build Comparator
  │       ├→ [17] Build Similarity (cosine sim for overlap %)
  │       └→ [21] Price Cache (poe.ninja API for top builds)
  │
  ├→ Alert path:
  │   └→ [50] Price Alert Manager
  │       └→ [21] Price Cache (check current price against threshold)
  │
  ├→ UI / Display path:
  │   ├→ [51] Item Image Resolver (CDN cache → disk → CDN → wiki)
  │   └→ [52] Buy Timing Advisor & Craft-vs-Buy
  │       ├→ [21] Price Cache & price_history table
  │       └→ [47] Craft Suggestion Ranker (reuses geometric_99th_percentile)
  │
  ├→ Analytics path:
  │   └→ [53] Map Run & Wealth Accumulator
  │       ├→ [41] Client.txt Log Parser (feeds CompletedRun events)
  │       └→ [38] Stash Tab Processor (feeds CurrencyTotal for wealth snapshots)
  │
  └→ AI provider path:
      └→ [54] Cloud AI Connection Manager
          └→ [42] Seer Network Architecture (cloud is the 3% fallback engine)
```

---

## 26. Ailment Mechanics

### Problem

PoE has six ailments — Ignite, Chill, Freeze, Shock, Poison, and Bleed. Each is
calculated and scaled differently. Many builds center on inflicting ailments
(Ignite Elementalist, Poison Assassin) or being immune to them. We need accurate
ailment calculation for both offense (how much damage they deal) and defense
(how much the player is affected when hit).

### Ailment Formulas

#### Ignite

```
// Ignite deals fire damage over time based on the igniting hit
// Only the HIGHEST ignite on target is active at a time (others queue behind it)

ignite_damage_per_second =
  hit.fire_damage
  × (0.5 / ignite_duration_seconds)           // spread over 4s base, so 12.5% per second
  × (1 + fire_dot_multi / 100.0)
  × (1 + increased_burning_damage / 100.0)

ignite_duration = 4.0 * (1 + increased_ignite_duration / 100.0) // seconds

// Threshold to ignite: Any fire hit ignites by default (100% chance if "always ignite" or
// random chance = player.ignite_chance, capped to [0%, 100%])
ignite_threshold = 0  // all hits above 0 damage can ignite if chance allows
```

#### Chill

```
// Chill reduces enemy action speed by up to 30%
// Magnitude scales with cold hit damage vs enemy max life

chill_effect_pct = (cold_hit_damage / target.max_life * 100.0).sqrt() * 10.0
// Example: 10% of max life cold hit → sqrt(10) * 10 = 31.6% → capped to 30%
chill_effect_pct = chill_effect_pct.clamp(5.0, 30.0)
// (5% minimum if chill is inflicted; max 30% without "more effect of chill" mods)

chill_effect_pct *= (1 + increased_chill_effect / 100.0)
chill_duration = 2.0 * (1 + increased_chill_duration / 100.0)  // seconds
```

#### Freeze

```
// Freeze requires a cold hit above the freeze threshold
// Freeze threshold = 0.15% of target max life (monsters), 350 for standard monsters

freeze_threshold = target.max_life * 0.0015  // 0.15% of max life
can_freeze = cold_hit_damage >= freeze_threshold

freeze_duration = (cold_hit_damage / freeze_threshold - 1.0)
                  * base_freeze_duration  // seconds
// Minimum freeze: 0.3s; maximum: 60s
freeze_duration = freeze_duration.clamp(0.3, 60.0)
```

#### Shock

```
// Shock increases damage taken by enemy by up to 50%
// Magnitude scales with lightning hit damage vs enemy max life

shock_effect_pct = (lightning_hit_damage / target.max_life * 100.0).sqrt() * 10.0
shock_effect_pct = shock_effect_pct.clamp(1.0, 50.0)
shock_effect_pct *= (1 + increased_shock_effect / 100.0)
shock_duration = 2.0 * (1 + increased_shock_duration / 100.0)

// Common: "always shocks" keystones (Shaper of Storms) set shock to minimum 15%
if player.has_always_shocks:
  shock_effect_pct = shock_effect_pct.max(15.0)
```

#### Poison

```
// Poison deals chaos damage over time per stack (stacks are independent, all active)
// Each poison stack: 10% of the hit's physical + chaos damage per second, over 2 seconds

poison_dps_per_stack =
  (hit.physical_damage + hit.chaos_damage) * 0.10
  × (1 + chaos_dot_multi / 100.0)
  × (1 + increased_poison_damage / 100.0)

poison_duration = 2.0 * (1 + increased_poison_duration / 100.0)

// Max stacks (for calculator estimate — exact count from hit rate):
max_poison_stacks ≈ player.hit_rate * player.poison_chance * poison_duration
total_poison_dps = max_poison_stacks * poison_dps_per_stack
```

#### Bleed

```
// Bleed: only from physical hits; at most one stack at a time unless "Crimson Dance"
// Base rate: 70% of the inflicting hit's physical damage over 5 seconds = 14%/s
// Moving target: 3× rate → 42%/s (same hit damage, faster delivery window)
//   NOTE: "moving" = the ENEMY is moving (not the player).
//   Bossing default: stationary rate. Mapping: 3× rate for most trash.
//   3× multiplier is BASE PoE mechanic, NOT an ascendancy — it's always there.

bleed_dps_stationary = hit.physical_damage * 0.14   // 70% over 5s → 14%/s
bleed_dps_moving     = hit.physical_damage * 0.42   // 3× rate when target moves

bleed_dps = if bleed_context.target_is_moving { bleed_dps_moving } else { bleed_dps_stationary }
bleed_dps *= (1.0 + player.physical_dot_multi / 100.0)
bleed_dps *= (1.0 + player.increased_bleed_damage / 100.0)

bleed_duration = 5.0 * (1.0 + player.increased_bleed_duration / 100.0)

// Crimson Dance keystone: allows up to 8 simultaneous independent bleed stacks
// Without it: only the single highest-damage bleed is active at a time
max_stacks = if player.has_crimson_dance { 8 } else { 1 }
active_bleed_stacks = player.bleed_stacks.min(max_stacks)
total_bleed_dps = bleed_dps * active_bleed_stacks as f64

// Calculator estimate for max sustainable stacks:
estimated_stacks = (player.hit_rate * player.bleed_chance * bleed_duration)
  .min(max_stacks as f64)
```

### Integration Points

- **Offense calculator**: sums DoT DPS including all active ailments for the total DPS figure
- **Defense calculator**: when player is hit, applies enemy ailments to player state
- **Combat simulator**: tracks ailment stacks per monster, applies per-tick damage

### Complexity

- O(1) per ailment calculation
- O(N) per tick where N = active stacks (usually < 50 for poison builds)

---

## 27. Energy Shield Recharge

### Problem

Energy Shield recharges automatically after not being damaged for a delay period.
This is separate from leech and regeneration and has its own start delay + rate.
CI (Chaos Inoculation) builds rely entirely on ES for survival. ES also has unique
interactions with Eldritch Battery, Ghost Reaver, and Zealot's Oath.

### Algorithm

```
State:
  es: f64              // current ES
  max_es: f64          // maximum ES
  recharge_timer: f64  // seconds since last ES damage. Recharge starts after delay
  recharging: bool     // is recharge currently active

Constants (modified by passives/gear):
  RECHARGE_DELAY = 2.0          // seconds of no ES damage before recharge begins
  RECHARGE_RATE  = max_es * 0.33 // 33% of max ES per second (base)
  // Modified by: "increased Energy Shield Recharge Rate"
  // Modified by: "Energy Shield Recharge begins immediately" (removes delay)

Per tick (dt = seconds since last tick):
  // Check if ES was damaged this tick
  if es_damaged_this_tick:
    recharge_timer = 0.0
    recharging = false
  else:
    recharge_timer += dt
    if recharge_timer >= recharge_delay:
      recharging = true

  if recharging && es < max_es:
    recharge_this_tick = recharge_rate_per_second * dt
    es = (es + recharge_this_tick).min(max_es)

fn recharge_delay() -> f64:
  if player.has_flask_mod("Energy Shield Recharge begins immediately"):
    0.0
  else if player.has_keystone("Ghost Dance"):
    1.0 - (player.ghost_shrouds as f64 * 0.33)  // 0 shrouds = 1s, 3 shrouds = 0s delay
  else:
    2.0 * (1.0 - player.reduced_recharge_delay / 100.0)

fn recharge_rate_per_second() -> f64:
  player.max_es * 0.33 * (1.0 + player.increased_es_recharge_rate / 100.0)
```

### Special Cases

| Mechanic | Effect |
|----------|--------|
| **Eldritch Battery** | ES pool is spent as mana for skills. ES still recharges normally. Skills costing ES bypass reservation. |
| **Ghost Reaver** | Life leech applies to ES instead. ES does NOT recharge (recharge disabled). |
| **Zealot's Oath** | Life regen applies to ES. Doesn't affect recharge. |
| **CI (Chaos Inoculation)** | Life = 1. Immune to chaos damage. ES is the only HP pool. Recharge is critical for sustain. |
| **Mind over Matter** | 30% of damage is taken from mana before life. Mana acts as a buffer, not ES. |
| **Low Life (35%)** | Threshold: player.es < player.max_es × 0.35. Used for Shav's/LL builds. |

### Complexity

- O(1) per tick

---

## 28. Mana Reservation Engine

### Problem

Many builds run 3-5 auras, each reserving a percentage of mana (or flat amount).
Reservation mods are additive within their own category but interact in non-obvious
ways. We need to compute: how much mana is reserved, how much is free, and what
happens when reservation efficiency changes (e.g., Sovereignty cluster jewel).

### Algorithm

```
Input:
  skills: Vec<ReservationSkill>  // each has: type (% or flat), amount, tag
  player: PlayerStats            // max_mana, max_life, reservation_efficiency, reduced_mana_res

Output:
  total_reserved_mana: f64
  free_mana: f64
  over_reserved: bool  // can't cast main skill

struct ReservationSkill {
  name: String,
  base_reservation: f64,   // e.g., 35.0 for 35% or 50.0 for 50 flat mana
  is_percentage: bool,
  tags: Vec<String>,       // "aura", "herald", "banner", etc.
}

Algorithm:

Step 1: Apply reservation efficiency to each skill
  // "Mana Reservation Efficiency" reduces the cost of % reservations
  // It is a SEPARATE multiplier from %increased mana reservation
  // efficiency of 100 = normal; 200 = half cost; 50 = double cost (bad)

  for skill in &mut skills:
    if skill.is_percentage:
      // Efficiency applies to % reservations only, not flat
      effective_reservation_pct = skill.base_reservation / (player.reservation_efficiency / 100.0)
      // E.g., 35% reservation with 120% efficiency → 35 / 1.2 = 29.17%

      // Apply increased/decreased mana reservation (additive pool)
      increased_res = player.increased_mana_reservation(skill.tags)
      effective_reservation_pct *= (1.0 + increased_res / 100.0)

      skill.effective_reservation = (player.max_mana * effective_reservation_pct / 100.0).ceil()
      // PoE always rounds UP for reservation costs
    else:
      // Flat reservations are not affected by efficiency or increased %
      skill.effective_reservation = skill.base_reservation

Step 2: Sum all reservations
  total_reserved = skills.iter().map(|s| s.effective_reservation).sum()
  free_mana = player.max_mana - total_reserved

Step 3: Check if main skill can fire
  main_skill_cost = player.main_skill_mana_cost
  over_reserved = free_mana < main_skill_cost

// Special: Eldritch Battery — ES used as mana pool too
if player.has_eldritch_battery:
  // Auras/heralds that reserve mana now reserve from ES+Mana combined pool
  effective_pool = player.max_mana + player.max_es
  free_resources = effective_pool - total_reserved - player.mana_spent_on_other
```

### Reservation Efficiency vs Reduced Reservation

These are two different stats — a common source of confusion:

| Stat | Example | Effect |
|------|---------|--------|
| **Mana Reservation Efficiency** | "Sovereignty" notable (+8%) | Stacks additively into the efficiency divisor. More = better. Default 100%. |
| **Reduced Mana Reservation** | Reduces a specific aura's reservation by X% | Old stat type (still exists on some items). Applied multiplicatively after efficiency. |
| **Increased Mana Reserved** | Hex Master + Blasphemy | Makes curse auras cost MORE. Applied as a multiplier. |

```
final_reservation = base_reservation
  / (reservation_efficiency / 100.0)    // efficiency divisor
  × (1.0 + increased_reservation / 100.0)  // tag-specific increases
  × (1.0 - reduced_reservation / 100.0)    // item/node reduced (rare)
```

### Common Reservation Budgets (Reference)

| Aura combination | ~Total reserved | Needed efficiency for free 100 mana |
|-----------------|----------------|--------------------------------------|
| Grace + Determination | 40% + 40% = 80% | Need ≈ 120% efficiency |
| Purity of Ele × 3 | 35% × 3 = 105% | Impossible without very high efficiency |
| Grace + Determination + Vitality (flat 30) | ~80% + 30 flat | Calculate separately |
| Herald of Ash + Ice | 25% + 25% = 50% | 100% efficiency fine |

### Complexity

- O(A) where A = number of active reservations (usually < 10)

---

## 37. OAuth PKCE Authorization Flow

### Problem

`start_poe_oauth` initiates a browser-based login. The PoE API uses OAuth 2.0 with
PKCE (Proof Key for Code Exchange) — a public-client flow that does not require a
client secret. Algorithm 34 covers *token refresh*; this covers the *initial login
handshake*.

### Algorithm

```rust
pub async fn start_oauth_flow(http: &HttpClient) -> Result<StoredToken, OAuthError> {

    // 1. Generate PKCE pair
    let code_verifier: String = {
        let bytes = rand::thread_rng().gen::<[u8; 32]>();
        BASE64URL_NO_PAD.encode(&bytes)   // 43 chars, URL-safe
    };
    let code_challenge: String = {
        let digest = Sha256::digest(code_verifier.as_bytes());
        BASE64URL_NO_PAD.encode(&digest)
    };

    // 2. Generate state (CSRF protection)
    let state: String = {
        let bytes = rand::thread_rng().gen::<[u8; 16]>();
        hex::encode(bytes)
    };

    // 3. Build authorization URL
    let auth_url = format!(
        "https://www.pathofexile.com/oauth/authorize\
         ?client_id={CLIENT_ID}\
         &response_type=code\
         &scope=account:profile account:characters account:stashes\
         &state={state}\
         &redirect_uri=http://localhost:{REDIRECT_PORT}/callback\
         &code_challenge={code_challenge}\
         &code_challenge_method=S256"
    );

    // 4. Start local redirect server BEFORE opening browser
    //    (browser may redirect faster than we can start listening)
    let (tx, rx) = oneshot::channel::<(String, String)>(); // (code, state)
    let server = start_redirect_server(REDIRECT_PORT, tx);

    // 5. Open browser
    tauri::api::shell::open(&auth_url, None)?;

    // 6. Wait for redirect (timeout: 5 minutes)
    let (code, returned_state) = tokio::time::timeout(
        Duration::from_secs(300), rx
    ).await.map_err(|_| OAuthError::Timeout)??;

    server.abort();

    // 7. Validate state (CSRF check)
    if returned_state != state {
        return Err(OAuthError::StateMismatch);
    }

    // 8. Exchange authorization code for tokens
    //    NOTE: send code_verifier (the original random string), NOT code_challenge
    let resp = http.post("https://www.pathofexile.com/oauth/token")
        .form(&[
            ("client_id",     CLIENT_ID),
            ("grant_type",    "authorization_code"),
            ("code",          &code),
            ("redirect_uri",  &format!("http://localhost:{REDIRECT_PORT}/callback")),
            ("code_verifier", &code_verifier),  // ← the verifier, not the challenge
        ])
        .send().await?;

    let token: OAuthTokenResponse = resp.json().await?;

    // 9. Persist to OS keychain (Tauri Stronghold)
    let stored = StoredToken {
        access_token:  token.access_token,
        refresh_token: token.refresh_token,
        expires_at:    Utc::now() + Duration::seconds(token.expires_in as i64),
        scope:         token.scope,
    };
    keychain::save("poe_token", &stored)?;

    Ok(stored)
}

/// Tiny single-request HTTP server that captures the OAuth redirect.
fn start_redirect_server(port: u16, tx: oneshot::Sender<(String, String)>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let listener = TcpListener::bind(format!("127.0.0.1:{port}")).await.unwrap();
        if let Ok((stream, _)) = listener.accept().await {
            // Read the GET /callback?code=XXX&state=YYY request
            let mut buf = [0u8; 4096];
            let _ = stream.readable().await;
            let n = stream.try_read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]);

            // Parse query params from first request line
            if let Some(params) = extract_query_params(&req) {
                let code  = params.get("code").cloned().unwrap_or_default();
                let state = params.get("state").cloned().unwrap_or_default();
                // Send 200 OK with a "you may close this tab" page
                let _ = write_response(&stream, 200, "<h1>Login successful. Return to Path of AI.</h1>").await;
                let _ = tx.send((code, state));
            }
        }
    })
}
```

### Security Notes

- `code_verifier` never leaves the local process — only `code_challenge` (its hash) goes to PoE
- `state` parameter prevents cross-site request forgery
- The redirect server binds to `127.0.0.1` only (not `0.0.0.0`)
- Tokens stored via Tauri Stronghold (OS keychain), not plain files

### Complexity

- O(1) — pure I/O wait; no significant computation

---

## 38. Stash Tab Processor

### Problem

`fetch_stash_tabs`, `fetch_stash_items`, `find_stash_upgrades`, and
`get_currency_totals` all require fetching data from the PoE stash API. The API is
rate-limited (45 requests per 60 seconds) and the stash can have 50+ tabs. Without
careful rate-limit management, the app will receive HTTP 429 errors.

### Rate-Limit Model

PoE API uses a sliding window: 45 requests per 60-second window, per endpoint.
We implement a token-bucket to stay within limits.

```rust
pub struct RateLimiter {
    tokens:          f64,          // current available tokens
    max_tokens:      f64,          // 45
    refill_rate:     f64,          // 45 / 60 = 0.75 tokens/second
    last_refill:     Instant,
    retry_after:     Option<Instant>,  // set when we receive a 429
}

impl RateLimiter {
    pub async fn acquire(&mut self) {
        // Refill tokens based on elapsed time
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = Instant::now();

        if let Some(retry_at) = self.retry_after {
            if Instant::now() < retry_at {
                tokio::time::sleep_until(retry_at.into()).await;
            }
            self.retry_after = None;
        }

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
        } else {
            // Wait until we have a token
            let wait_ms = ((1.0 - self.tokens) / self.refill_rate * 1000.0) as u64;
            tokio::time::sleep(Duration::from_millis(wait_ms)).await;
            self.tokens = 0.0;
        }
    }

    pub fn on_429(&mut self, retry_after_secs: u64) {
        self.retry_after = Some(Instant::now() + Duration::from_secs(retry_after_secs));
        self.tokens = 0.0;
    }
}
```

### Fetch Pipeline

```rust
pub async fn process_stash(
    league:    &str,
    token:     &str,
    equipped:  Option<&EquippedItems>,
    limiter:   &mut RateLimiter,
) -> Result<StashResult> {

    // Step 1: Fetch tab list
    limiter.acquire().await;
    let tabs: Vec<StashTab> = fetch_stash_tabs(league, token).await?;

    // Step 2: Fetch each tab's contents (skip bulk-currency and map tabs for now)
    let mut all_items: Vec<StashItem> = Vec::new();
    for tab in &tabs {
        limiter.acquire().await;
        let items = fetch_tab_items(league, tab.id, token).await?;
        all_items.extend(items);
    }

    // Step 3: Score each item (use Algorithm 7 fast-path via Algorithm 25)
    let mut scored: Vec<ScoredStashItem> = all_items.iter().map(|item| {
        let score = fast_estimate_score(item);
        ScoredStashItem { item: item.clone(), score }
    }).collect();
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Step 4: Detect upgrades vs current equipped gear
    let upgrades = if let Some(equipped) = equipped {
        find_stash_upgrades(&scored, equipped)
    } else { vec![] };

    // Step 5: Tally currency
    let currency = tally_currency(&all_items);

    Ok(StashResult { tabs, scored_items: scored, upgrades, currency })
}

/// Upgrade detection: stash item scores higher than currently equipped in same slot
fn find_stash_upgrades(stash: &[ScoredStashItem], equipped: &EquippedItems) -> Vec<StashUpgrade> {
    let mut upgrades = vec![];
    for item in stash {
        let slot = item.item.inferred_slot();
        let current_score = equipped.score_for_slot(slot);
        if item.score > current_score + 5.0 {   // 5-point threshold avoids noise
            upgrades.push(StashUpgrade {
                item:          item.item.clone(),
                slot,
                score_gain:    item.score - current_score,
                current_score,
                new_score:     item.score,
            });
        }
    }
    upgrades.sort_by(|a, b| b.score_gain.partial_cmp(&a.score_gain).unwrap());
    upgrades
}

/// Currency tally: convert all currency items to chaos equivalent
fn tally_currency(items: &[StashItem]) -> CurrencyTotal {
    let mut chaos_total = 0.0;
    let mut breakdown: HashMap<String, f64> = HashMap::new();

    for item in items {
        if item.frame_type == FrameType::Currency {
            let rate = CURRENCY_RATES.get(&item.type_line).copied().unwrap_or(0.0);
            let stack_value = rate * item.stack_size as f64;
            *breakdown.entry(item.type_line.clone()).or_insert(0.0) += stack_value;
            chaos_total += stack_value;
        }
    }

    CurrencyTotal {
        chaos_total,
        divine_total: chaos_total / DIVINE_RATE,
        breakdown,
    }
}
```

### Complexity

- Tab fetch: O(T) where T = number of tabs, limited by rate limiter
- Item scoring: O(I) where I = total items across all tabs
- Upgrade detection: O(I × S) where S = number of equipment slots (≤ 12)
- Currency tally: O(I) hash map accumulation

---

## 39. Map Mod Danger Scorer

### Problem

`analyze_map_mods` takes the current map's mod list and the active build and returns
a per-mod danger rating plus an overall "roll this map / do not run" verdict. Danger
is build-specific: "No Life Regeneration" is fatal for Righteous Fire, irrelevant for
a CoC build.

### Algorithm

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DangerLevel {
    Safe,       // No impact
    Minor,      // Slight disadvantage
    Moderate,   // Manageable with care
    Major,      // Significant threat — consider rerolling
    Critical,   // Likely death or build-bricking — skip or brick-check
}

pub struct ModDanger {
    pub mod_text:   String,
    pub level:      DangerLevel,
    pub reason:     String,
}

pub struct MapDangerResult {
    pub mods:           Vec<ModDanger>,
    pub worst:          DangerLevel,
    pub verdict:        &'static str,   // "Run", "Run carefully", "Reroll", "Skip"
    pub fatal_mods:     Vec<String>,    // critical-level mods
    pub total_score:    u32,            // 0-100 danger score
}

pub fn score_map_mods(mods: &[&str], build: &BuildData) -> MapDangerResult {
    let archetype  = &build.archetype;
    let defense    = &build.calc.defense;
    let offense    = &build.calc.offense;

    let mut dangers: Vec<ModDanger> = mods.iter().map(|&m| {
        score_single_mod(m, archetype, defense, offense)
    }).collect();

    // Sum weighted danger
    let total_score: u32 = dangers.iter().map(|d| d.level as u32 * 25).sum::<u32>().min(100);
    let worst = dangers.iter().map(|d| d.level).max().unwrap_or(DangerLevel::Safe);
    let fatal_mods = dangers.iter()
        .filter(|d| d.level == DangerLevel::Critical)
        .map(|d| d.mod_text.clone())
        .collect();

    let verdict = match worst {
        DangerLevel::Safe     => "Run",
        DangerLevel::Minor    => "Run",
        DangerLevel::Moderate => "Run carefully",
        DangerLevel::Major    => "Reroll",
        DangerLevel::Critical => "Skip",
    };

    MapDangerResult { mods: dangers, worst, verdict, fatal_mods, total_score }
}

fn score_single_mod(
    mod_text:  &str,
    archetype: &Archetype,
    defense:   &DefenseResult,
    offense:   &OffenseResult,
) -> ModDanger {
    use DangerLevel::*;

    // Normalise text for matching
    let m = mod_text.to_lowercase();

    let (level, reason) = if m.contains("no life regeneration") || m.contains("no mana regeneration") {
        // Fatal for RF (life regen = damage negation), bad for low-life
        if *archetype == Archetype::FireDotRF { (Critical, "RF requires life regen to survive") }
        else if defense.life_regen_per_sec > 500.0 { (Major, "Removes significant life recovery") }
        else { (Minor, "Low regen — minimal impact") }

    } else if m.contains("players cannot leech") || m.contains("no leech") {
        if offense.uses_leech { (Critical, "Build relies on leech for sustain") }
        else { (Safe, "Build does not use leech") }

    } else if m.contains("elemental reflect") {
        let ele_dps = offense.fire_dps + offense.cold_dps + offense.lightning_dps;
        if ele_dps > offense.total_dps * 0.3 { (Critical, "High ele damage — reflect will kill you") }
        else if ele_dps > 0.0 { (Major, "Some ele damage taken as reflect") }
        else { (Safe, "No elemental damage to reflect") }

    } else if m.contains("physical reflect") {
        let phys_pct = offense.physical_dps / offense.total_dps.max(1.0);
        if phys_pct > 0.7 { (Critical, "Mostly phys damage — reflect is lethal") }
        else if phys_pct > 0.2 { (Moderate, "Partial phys damage reflected") }
        else { (Safe, "No phys damage to reflect") }

    } else if m.contains("monsters are hexproof") {
        if build.uses_curses { (Major, "Curse-based damage or defense disabled") }
        else { (Minor, "No curses allocated") }

    } else if m.contains("minus maximum resistances") || m.contains("reduced maximum resistances") {
        // Parse percentage from mod text
        let penalty = parse_number(&m).unwrap_or(10) as f64;
        let min_overcap = [defense.fire_res, defense.cold_res, defense.lightning_res]
            .iter().map(|r| r - 75.0).fold(f64::MAX, f64::min);
        if min_overcap < penalty { (Critical, "Resistance drops below cap — update overcap") }
        else if min_overcap < penalty + 5.0 { (Major, "Overcap barely covers penalty") }
        else { (Minor, "Sufficient overcap") }

    } else if m.contains("cannot be stunned") {
        (Safe, "Monster property, not player")  // This is a PLAYER mod — player can't be stunned = good

    } else if m.contains("monsters deal x% extra damage as") {
        let extra = parse_number(&m).unwrap_or(25) as f64 / 100.0;
        // Evaluate by how much EHP it effectively removes
        if extra > 0.4 { (Major, "Very high extra elemental damage taken") }
        else { (Moderate, "Additional element on hits — check resistance") }

    } else if m.contains("blood magic") || m.contains("no mana") {
        if build.mana_pool < 50.0 { (Major, "Mana-dependent skills may break") }
        else { (Moderate, "All costs paid from life — monitor life pool") }

    } else if m.contains("players are cursed with enfeeble") {
        (Moderate, "Enfeeble reduces hit accuracy and damage — not fatal but significant")

    } else if m.contains("players are cursed with temporal chains") {
        if defense.freeze_immune && defense.chill_immune { (Minor, "Slowed action speed — immune to freeze so only slight disadvantage") }
        else { (Moderate, "Slow can be dangerous in dense maps") }

    } else if m.contains("ground effects") || m.contains("burning ground") {
        if defense.fire_res >= 75.0 { (Minor, "Capped fire res — minimal damage from burning ground") }
        else { (Moderate, "Uncapped fire res — burning ground is dangerous") }

    } else {
        (Safe, "No specific threat detected for this build")
    };

    ModDanger { mod_text: mod_text.to_string(), level, reason: reason.to_string() }
}

fn parse_number(text: &str) -> Option<u32> {
    text.split_whitespace()
        .find_map(|w| w.trim_end_matches('%').parse::<u32>().ok())
}
```

### Complexity

- O(M × R) where M = number of mods (typically 4-8), R = number of rule checks (O(1) per mod)
- Effectively O(M) — each mod checked in constant time

---

## 40. Build Share Code Codec

### Problem

`generate_share_code` and `import_share_code` need a compact, versioned, URL-safe
encoding for a build state. Requirements: URL-pasteable, self-describing version
field, forward-compatible, decodable without a network call.

### Encode Algorithm

```rust
/// Schema version embedded at byte 0. Increment when fields change.
const SHARE_CODE_VERSION: u8 = 1;
const SHARE_CODE_PREFIX:  &str = "pofai:";

#[derive(Serialize, Deserialize)]
pub struct SharePayload {
    pub version:     u8,                  // = SHARE_CODE_VERSION
    pub character:   CharacterSummary,
    pub items:       Vec<ItemSummary>,
    pub tree_nodes:  Vec<u32>,            // allocated node IDs
    pub gems:        Vec<GemSummary>,
    pub stats:       SnapshotStats,
    pub archetype:   Archetype,
}

pub fn encode_share_code(payload: &SharePayload) -> Result<String, CodecError> {
    // 1. Serialize to JSON (compact, no pretty-print)
    let json = serde_json::to_vec(payload)?;

    // 2. zlib-compress (deflate, level 6 — good compression/speed balance)
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(6));
    encoder.write_all(&json)?;
    let compressed = encoder.finish()?;

    // 3. Prepend version byte
    let mut data = vec![SHARE_CODE_VERSION];
    data.extend_from_slice(&compressed);

    // 4. BASE64URL encode (RFC 4648 §5, no padding — URL-safe)
    let encoded = BASE64URL_NO_PAD.encode(&data);

    Ok(format!("{SHARE_CODE_PREFIX}{encoded}"))
}
```

### Decode Algorithm

```rust
pub fn decode_share_code(code: &str) -> Result<SharePayload, CodecError> {
    // 1. Strip prefix
    let encoded = code.strip_prefix(SHARE_CODE_PREFIX)
        .ok_or(CodecError::MissingPrefix)?;

    // 2. BASE64URL decode
    let data = BASE64URL_NO_PAD.decode(encoded)
        .map_err(|_| CodecError::InvalidBase64)?;

    if data.is_empty() { return Err(CodecError::Empty); }

    // 3. Read version byte
    let version = data[0];
    let compressed = &data[1..];

    // 4. Dispatch to correct deserializer for this version
    match version {
        1 => {
            let mut decoder = ZlibDecoder::new(compressed);
            let mut json = Vec::new();
            decoder.read_to_end(&mut json).map_err(|_| CodecError::DecompressFailed)?;
            serde_json::from_slice::<SharePayload>(&json)
                .map_err(|e| CodecError::DeserializeFailed(e.to_string()))
        }
        v => Err(CodecError::UnknownVersion(v)),
    }
}
```

### Size Estimate

| Build complexity | JSON size | Compressed | Share code length |
|-----------------|-----------|------------|-------------------|
| Simple (50 nodes) | ~4 KB | ~800 B | ~1,100 chars |
| Complex (120 nodes + full items) | ~12 KB | ~2 KB | ~2,700 chars |

Codes fit in a tweet, URL param, or Discord message without truncation.

### Complexity

- Encode: O(N) where N = JSON size
- Decode: O(N) decompression + O(N) deserialization

---

## 41. Client.txt Log Parser — Map Run Tracker

### Problem

Path of Exile writes zone transition events to `Client.txt` in the game directory.
Parsing this file gives us act completion tracking, map run timers, and session
statistics without any game API calls. `watch_pob_directory` logic applies here:
watch for file growth, not file replacement.

### Zone Entry Detection

```rust
// Log line format (UTC timestamp since 3.x):
// 2026/04/03 18:42:13 948640906 b5f [INFO Client 1234] : You have entered Strand.

static ZONE_ENTRY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d{4}/\d{2}/\d{2} \d{2}:\d{2}:\d{2}) \d+ \w+ \[INFO Client \d+\] : You have entered (.+)\.")
        .unwrap()
});

pub struct ZoneEntry {
    pub timestamp: DateTime<Utc>,
    pub zone_name: String,
    pub zone_type: ZoneType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZoneType {
    ActZone { act: u8 },
    Town     { act: u8 },
    Map      { name: String, tier: Option<u8> },
    Hideout,
    LabTrial,
    Labyrinth,
    Unknown,
}
```

### Tail-Follow Algorithm

```rust
pub struct ClientLogWatcher {
    path:       PathBuf,
    file_pos:   u64,       // byte offset into file
    runs:       Vec<CompletedRun>,
    current:    Option<ActiveRun>,
}

pub struct ActiveRun {
    pub zone:       ZoneEntry,
    pub started_at: Instant,
}

pub struct CompletedRun {
    pub zone_name:       String,
    pub zone_type:       ZoneType,
    pub duration_secs:   f64,
    pub completed_at:    DateTime<Utc>,
}

impl ClientLogWatcher {
    /// Called by the file-change notifier (inotify / ReadDirectoryChangesW)
    pub fn poll(&mut self, emit: &dyn Fn(ZoneEvent)) -> Result<()> {
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(self.file_pos))?;

        let reader = BufReader::new(&file);
        for line in reader.lines() {
            let line = line?;
            if let Some(caps) = ZONE_ENTRY.captures(&line) {
                let timestamp = parse_poe_timestamp(&caps[1]);
                let zone_name = caps[2].to_string();
                let zone_type = classify_zone(&zone_name);

                // Close the previous run
                if let Some(prev) = self.current.take() {
                    if is_mappable(&prev.zone.zone_type) {
                        let run = CompletedRun {
                            zone_name:     prev.zone.zone_name.clone(),
                            zone_type:     prev.zone.zone_type.clone(),
                            duration_secs: prev.started_at.elapsed().as_secs_f64(),
                            completed_at:  timestamp,
                        };
                        self.runs.push(run.clone());
                        emit(ZoneEvent::RunCompleted(run));
                    }
                }

                // Start new run
                self.current = Some(ActiveRun {
                    zone:       ZoneEntry { timestamp, zone_name: zone_name.clone(), zone_type: zone_type.clone() },
                    started_at: Instant::now(),
                });
                emit(ZoneEvent::ZoneEntered { zone_name, zone_type });
            }
        }

        // Update file position
        self.file_pos = file.seek(SeekFrom::Current(0))?;
        Ok(())
    }

    pub fn session_stats(&self) -> SessionStats {
        let map_runs: Vec<_> = self.runs.iter()
            .filter(|r| matches!(r.zone_type, ZoneType::Map { .. }))
            .collect();

        SessionStats {
            total_runs:     map_runs.len(),
            total_time_s:   map_runs.iter().map(|r| r.duration_secs).sum(),
            avg_time_s:     if map_runs.is_empty() { 0.0 }
                            else { map_runs.iter().map(|r| r.duration_secs).sum::<f64>() / map_runs.len() as f64 },
            fastest_run:    map_runs.iter().min_by(|a, b| a.duration_secs.partial_cmp(&b.duration_secs).unwrap()).cloned(),
            slowest_run:    map_runs.iter().max_by(|a, b| a.duration_secs.partial_cmp(&b.duration_secs).unwrap()).cloned(),
        }
    }
}

/// Classify zone name against static lookup tables
fn classify_zone(name: &str) -> ZoneType {
    if let Some(&act) = ACT_TOWNS.get(name)       { return ZoneType::Town { act }; }
    if let Some(&act) = ACT_ZONES.get(name)        { return ZoneType::ActZone { act }; }
    if name.contains("Hideout")                    { return ZoneType::Hideout; }
    if name.contains("Trial of")                   { return ZoneType::LabTrial; }
    if name.contains("Aspirant's Plaza") || name.contains("Labyrinth") {
                                                     return ZoneType::Labyrinth; }
    if let Some(&tier) = MAP_TIERS.get(name)       { return ZoneType::Map { name: name.to_string(), tier: Some(tier) }; }
    // Unknown map (custom/unique map name)
    if name.ends_with(" Map")                      { return ZoneType::Map { name: name.to_string(), tier: None }; }
    ZoneType::Unknown
}
```

### Complexity

- O(L) per poll, where L = new log lines since last poll
- Amortized O(1) per second when idle (no zone changes)
- Startup seek: O(1) — jump directly to last known position

---

## 42. Seer Network Architecture

### Problem

ARCHITECTURE.md describes the Seer as a "3-engine system" (Calculator / Seer / Cloud).
`seer-engine.js` implements 5 neural networks + a ResponseGen. These are two levels of
the same design: **the "3 engines" describe routing strategy; the "5 networks" implement
the Seer engine**.

### The Two Levels Reconciled

```
USER QUERY
    │
    ▼
[Algorithm 18: Intent Classifier]
    │
    ├─ 85% → Calculator Engine ─── Algorithms 1-32 (pure math, no ML)
    │
    ├─ 12% → Seer Engine (local, 5 networks, <100ms) ─── THIS ALGORITHM
    │
    └─  3% → Cloud Engine ──────── External AI provider (Claude/GPT/etc.)
                                   (complex theory, edge cases, new content)
```

The Seer Engine IS the "Knowledge engine." The 5 networks run together to answer
queries that need PoE understanding beyond pure calculation.

### Network Architecture

```
Total bundle: ~50-80MB  |  Inference: <100ms on CPU  |  No GPU required

┌─────────────────────────────────────────────────────────────────────┐
│                        SEER ENGINE                                  │
│                                                                     │
│  Network 1: ItemNet  (~5MB)          Network 4: QueryNet (~15MB)    │
│  ─────────────────────────           ──────────────────────────     │
│  Feed-forward, 431-dim input         Transformer encoder            │
│  Layers: 431→256→128→64             6 layers, 256-dim hidden        │
│  Outputs:                            Vocabulary: ~8K PoE tokens     │
│    score(0-100)                      Outputs:                       │
│    stat_impact(8 dims)               intent (30+ classes)           │
│    upgrade_priority                  entities (item, slot, boss…)   │
│    price_estimate                    confidence                     │
│                                                                     │
│  Network 2: BuildNet (~8MB)          Network 5: EmbedNet (~10MB)    │
│  ─────────────────────────           ────────────────────────       │
│  Multi-task classifier               Sentence embedding model       │
│  Input: build stat vector (14+)      Input: text (query or chunk)   │
│  Layers: N→256→128                   Output: 128-dim L2-norm vector │
│  Outputs:                            Used for:                      │
│    archetype (softmax)               RAG: query → nearest chunks    │
│    issue_flags (10 sigmoid)          Collaborative filtering        │
│    content_readiness (10)            Build similarity (Alg 17)      │
│    evolution_path                                                   │
│                                                                     │
│  Network 3: TreeNet (~6MB)           ResponseGen (rule + template)  │
│  ─────────────────────────           ──────────────────────────     │
│  Graph neural network on             NOT generative — template fill │
│  the passive tree (1500 nodes)       Zero hallucination possible    │
│  Input: allocated nodes +            Variants per intent (~200)     │
│    build context                     PoE-flavored tone built in     │
│  Output:                             Variable injection from        │
│    node efficiency scores            network outputs                │
│    wasteful nodes                                                   │
│    top-N recommendations                                            │
└─────────────────────────────────────────────────────────────────────┘
```

### Request Flow Through Seer Engine

```rust
pub async fn seer_respond(query: &str, build: &BuildData) -> SeerResponse {

    // 1. QueryNet: understand the question
    let understood = query_net.understand(query);
    // → { intent: "suggest_upgrade", entities: { slot: "helmet" }, confidence: 0.91 }

    // 2. EmbedNet: retrieve relevant knowledge chunks (RAG)
    let knowledge = embed_net.search(&understood.intent_description(), 5);
    // → top-5 most relevant knowledge base chunks by cosine similarity

    // 3. Route to specialist network based on intent
    let structured_data: serde_json::Value = match understood.intent {
        Intent::SuggestUpgrade  => item_net.score_upgrades(build, understood.entities.slot),
        Intent::AnalyzeBuild    => build_net.analyze(build),
        Intent::OptimizeTree    => tree_net.analyze_allocation(&build.tree_nodes, build),
        Intent::CheckPrice      => /* delegates to Algorithm 21 (Market Cache) */
            market.get_price(understood.entities.item_name).await.into(),
        _ => build_net.analyze(build).into(),  // fallback
    };

    // 4. ResponseGen: build natural language response
    let text = response_gen.generate(understood.intent, &structured_data, &knowledge);

    SeerResponse {
        text,
        engine:     "seer",
        confidence: understood.confidence,
        data:       structured_data,   // structured data for UI widgets
    }
}
```

### Training Data Sources

| Network | Training source | Size |
|---------|----------------|------|
| ItemNet | poe.ninja trade data + item valuations | 500K+ items |
| BuildNet | poe.ninja character snapshots | 100K+ builds |
| TreeNet | poe.ninja passive tree allocations | 200K+ builds |
| QueryNet | PoE Q&A pairs (community + synthetic) | 50K+ pairs |
| EmbedNet | Contrastive pairs from PoE wiki + forum | 100K+ pairs |

### On-Disk Layout

```
PathOfAI_Data/model/
  seer.bin          ← all 5 networks' weights (quantized INT8, ~50-80MB)
  config.json       ← architecture hyperparameters per network
  vocab.json        ← QueryNet token vocabulary (~8K tokens)
  embeddings.bin    ← pre-computed EmbedNet vectors for knowledge base
```

Weights are loaded into `OnceLock<SeerWeights>` at first query; subsequent queries
reuse the cached weights (no repeated disk I/O).

### Complexity

- ItemNet forward pass: O(1) — fixed 431-dim input, 3 dense layers
- BuildNet forward pass: O(1) — fixed input size
- TreeNet analysis: O(N) where N = allocated nodes (typically 100-130)
- QueryNet tokenize + encode: O(L × T²) where L = sequence length (≤64), T = 6 layers — effectively O(1) for fixed max length
- EmbedNet search: O(K) where K = knowledge base chunks (pre-computed embeddings)
- Total per-query: <100ms on modern CPU

---

## 43. Vendor Recipe Detector

### Problem

PoE vendor recipes let you trade sets of items to vendors for specific currency. The
most important are the chaos and regal recipes (full rare item sets). In SSF especially,
these are the primary currency sources early in a league. The detector identifies which
recipe-eligible items are in the stash and highlights the missing piece(s).

### Full Rare Set Recipe Rules

```
CHAOS RECIPE:  all items ilvl 60-74, complete set, unidentified → 2× Chaos Orb
               OR any item identified   → 1× Chaos Orb
REGAL RECIPE:  all items ilvl 75+, complete set, unidentified → 1× Regal Orb

FULL SET = one item in each of these slots:
  Helmet, Chest, Gloves, Boots (1 each)
  Belt, Amulet (1 each)
  Ring × 2
  Weapon slot: one of:
    Two-handed weapon (counts as 2)
    One-handed weapon + Shield
    Two one-handed weapons
```

### Algorithm

```rust
#[derive(Debug, Default)]
pub struct RecipeSet {
    pub helmet:   Option<StashItem>,
    pub chest:    Option<StashItem>,
    pub gloves:   Option<StashItem>,
    pub boots:    Option<StashItem>,
    pub belt:     Option<StashItem>,
    pub amulet:   Option<StashItem>,
    pub rings:    Vec<StashItem>,      // need 2
    pub weapons:  Vec<StashItem>,      // one 2H or two 1H or 1H+Shield
    pub shield:   Option<StashItem>,
}

pub struct RecipeAnalysis {
    pub chaos_sets:     Vec<RecipeSet>,    // ilvl 60-74
    pub regal_sets:     Vec<RecipeSet>,    // ilvl 75+
    pub missing_slots:  Vec<EquipSlot>,    // what's needed to complete next set
    pub quality_recipes: Vec<QualityRecipe>,
}

pub fn detect_recipes(items: &[StashItem]) -> RecipeAnalysis {

    // ── RARE SET RECIPES ──────────────────────────────────
    let mut chaos_pool: HashMap<EquipSlot, Vec<&StashItem>> = HashMap::new();
    let mut regal_pool: HashMap<EquipSlot, Vec<&StashItem>> = HashMap::new();

    for item in items {
        if item.frame_type != FrameType::Rare { continue; }
        let slot = infer_slot(item);
        let pool = if item.ilvl >= 75 { &mut regal_pool }
                   else if item.ilvl >= 60 { &mut chaos_pool }
                   else { continue };
        pool.entry(slot).or_default().push(item);
    }

    let chaos_sets = build_sets(&chaos_pool);
    let regal_sets = build_sets(&regal_pool);

    // Determine what's missing for the NEXT set
    let missing_slots = find_missing_slots(&chaos_pool, &regal_pool);

    // ── QUALITY RECIPES ──────────────────────────────────
    let quality_recipes = detect_quality_recipes(items);

    RecipeAnalysis { chaos_sets, regal_sets, missing_slots, quality_recipes }
}

/// Build as many complete sets as possible from available items.
fn build_sets(pool: &HashMap<EquipSlot, Vec<&StashItem>>) -> Vec<RecipeSet> {
    let mut sets = vec![];
    let mut counts: HashMap<EquipSlot, usize> = pool.iter()
        .map(|(slot, items)| (*slot, items.len()))
        .collect();

    // Rings need 2, weapons: count 2H as 2 slots
    loop {
        let helmet  = *counts.get(&EquipSlot::Helmet).unwrap_or(&0);
        let chest   = *counts.get(&EquipSlot::Chest).unwrap_or(&0);
        let gloves  = *counts.get(&EquipSlot::Gloves).unwrap_or(&0);
        let boots   = *counts.get(&EquipSlot::Boots).unwrap_or(&0);
        let belt    = *counts.get(&EquipSlot::Belt).unwrap_or(&0);
        let amulet  = *counts.get(&EquipSlot::Amulet).unwrap_or(&0);
        let rings   = *counts.get(&EquipSlot::Ring).unwrap_or(&0);
        let weapons = weapon_slots_available(&counts);

        if helmet < 1 || chest < 1 || gloves < 1 || boots < 1
            || belt < 1 || amulet < 1 || rings < 2 || weapons < 2 {
            break;
        }

        // Consume one set
        *counts.get_mut(&EquipSlot::Helmet).unwrap() -= 1;
        *counts.get_mut(&EquipSlot::Chest).unwrap()  -= 1;
        *counts.get_mut(&EquipSlot::Gloves).unwrap() -= 1;
        *counts.get_mut(&EquipSlot::Boots).unwrap()  -= 1;
        *counts.get_mut(&EquipSlot::Belt).unwrap()   -= 1;
        *counts.get_mut(&EquipSlot::Amulet).unwrap() -= 1;
        *counts.get_mut(&EquipSlot::Ring).unwrap()   -= 2;
        consume_weapon_slots(&mut counts);

        sets.push(RecipeSet::default()); // simplified; real version fills item refs
    }
    sets
}

/// Quality recipes: 20× flasks → Glassblower, 40×20% gems → GCP, 40× maps → Cartographer
fn detect_quality_recipes(items: &[StashItem]) -> Vec<QualityRecipe> {
    let mut flask_qual_sum: u32 = 0;
    let mut gem_qual_sum:   u32 = 0;
    let mut map_qual_sum:   u32 = 0;

    for item in items {
        match item.item_class {
            ItemClass::Flask  => flask_qual_sum += item.quality as u32,
            ItemClass::Gem    => gem_qual_sum   += item.quality as u32,
            ItemClass::Map    => map_qual_sum   += item.quality as u32,
            _ => {}
        }
    }

    let mut recipes = vec![];
    if flask_qual_sum >= 40 {
        recipes.push(QualityRecipe { output: "Glassblower's Bauble", count: flask_qual_sum / 40 });
    }
    if gem_qual_sum >= 40 {
        recipes.push(QualityRecipe { output: "Gemcutter's Prism", count: gem_qual_sum / 40 });
    }
    if map_qual_sum >= 40 {
        recipes.push(QualityRecipe { output: "Cartographer's Chisel", count: map_qual_sum / 40 });
    }
    recipes
}
```

### Complexity

- O(I) where I = number of items in stash
- Set-building inner loop: O(S) where S = complete sets found (usually 0-10)

---

## 44. Portable Storage & File Watcher

### Problem

Two related infrastructure algorithms: (1) `PathOfAI_Data/` directory initialization
so the app is USB-portable with no AppData dependency; (2) `watch_pob_directory` IPC
command that watches for external PoB file changes and triggers re-analysis.

---

### 44a. Portable Storage Directory Init

```rust
/// Called once at startup before any other subsystem accesses disk.
pub fn init_storage(custom_path: Option<PathBuf>) -> Result<StoragePaths> {

    // 1. Determine root directory
    let root = if let Some(path) = custom_path {
        path
    } else if let Ok(env_path) = std::env::var("PATHOFAI_DATA") {
        PathBuf::from(env_path)
    } else {
        // Default: place data folder next to the executable
        std::env::current_exe()?
            .parent()
            .ok_or(StorageError::CannotDetermineExeDir)?
            .to_path_buf()
    };

    let data_dir = root.join("PathOfAI_Data");

    // 2. Create directory tree (idempotent — ok if already exists)
    let paths = StoragePaths::new(&data_dir);
    for dir in paths.all_directories() {
        std::fs::create_dir_all(dir)?;
    }

    // 3. Check for legacy AppData installation and offer migration
    if let Some(appdata) = legacy_appdata_path() {
        if appdata.exists() && !data_dir.exists() {
            log::info!("Found legacy AppData installation at {:?}", appdata);
            // Emit event to frontend: prompt user to migrate
            // (migration is a user-confirmed action, not automatic)
        }
    }

    Ok(paths)
}

pub struct StoragePaths {
    data_dir: PathBuf,
}

impl StoragePaths {
    // Config
    pub fn settings(&self)     -> PathBuf { self.data_dir.join("config/settings.json") }
    pub fn ai_providers(&self) -> PathBuf { self.data_dir.join("config/ai-providers.json") }
    pub fn alerts(&self)       -> PathBuf { self.data_dir.join("config/alerts.json") }

    // Cache
    pub fn price_cache(&self)  -> PathBuf { self.data_dir.join("cache/prices") }
    pub fn image_cache(&self)  -> PathBuf { self.data_dir.join("cache/images") }

    // Persistence
    pub fn database(&self)     -> PathBuf { self.data_dir.join("pathofai.db") }
    pub fn session(&self)      -> PathBuf { self.data_dir.join("config/session.json") }
    pub fn snapshots(&self)    -> PathBuf { self.data_dir.join("snapshots") }
    pub fn backups(&self)      -> PathBuf { self.data_dir.join("backups") }

    // Seer model
    pub fn model_weights(&self)  -> PathBuf { self.data_dir.join("model/seer.bin") }
    pub fn model_config(&self)   -> PathBuf { self.data_dir.join("model/config.json") }
    pub fn embeddings(&self)     -> PathBuf { self.data_dir.join("model/embeddings.bin") }

    // Knowledge base
    pub fn knowledge(&self)    -> PathBuf { self.data_dir.join("knowledge") }

    // Logs
    pub fn app_log(&self)      -> PathBuf { self.data_dir.join("logs/app.log") }

    pub fn all_directories(&self) -> Vec<&Path> {
        // Returns all dirs that must exist before the app starts
        // (files are created lazily; dirs must pre-exist)
        vec![
            &self.data_dir,
            // config/, cache/, cache/images/, cache/images/items, …,
            // snapshots/, backups/, knowledge/, model/, logs/
            // (full list in implementation)
        ]
    }
}

/*
Disk layout:
  PathOfAI.exe
  PathOfAI_Data/
    config/
      settings.json          ← app settings
      ai-providers.json      ← API keys (NOT tokens — tokens go to OS keychain)
      keybinds.json
      alerts.json            ← price alerts
      session.json           ← Algorithm 35
    cache/
      images/
        items/ gems/ flasks/ currency/ skills/   ← game art CDN cache
      prices/                ← poe.ninja price cache (Algorithm 21)
      manifest.json          ← cache freshness index
    backups/
      MyBuild_2026-04-02_14-30.xml  ← PoB file backups before writes
    snapshots/
      {build_id}/snapshot-NNN.json  ← Algorithm 33
    knowledge/
      items/ gems/ tree/ crafting/ builds/       ← Seer knowledge base
    model/
      seer.bin  config.json  vocab.json  embeddings.bin
    logs/
      app.log
    pathofai.db              ← SQLite (Algorithm 36)
*/
```

---

### 44b. PoB File Watcher & Build Sync

```rust
/// Watches a directory for *.pob or *.xml file changes.
/// On change: debounce → re-parse → emit build-changed event.
pub struct PobFileWatcher {
    watcher:    RecommendedWatcher,    // notify crate, platform-native
    watched:    Option<PathBuf>,
    debounce:   HashMap<PathBuf, Instant>,
    debounce_ms: u64,
}

impl PobFileWatcher {
    pub fn watch_directory(&mut self, path: PathBuf) -> Result<()> {
        if let Some(old) = &self.watched {
            self.watcher.unwatch(old)?;
        }
        self.watcher.watch(&path, RecursiveMode::NonRecursive)?;
        self.watched = Some(path);
        Ok(())
    }

    /// Called by the `notify` crate event callback.
    pub fn on_fs_event(&mut self, event: Event, emit: &AppHandle) {
        let path = match event.paths.first() {
            Some(p) if is_pob_file(p) => p.clone(),
            _ => return,
        };

        // Debounce: ignore events within 500ms of last event for same file
        let last = self.debounce.entry(path.clone()).or_insert(Instant::now());
        if last.elapsed().as_millis() < self.debounce_ms as u128 {
            *last = Instant::now();
            return;
        }
        *last = Instant::now();

        // Spawn re-analysis task (non-blocking)
        let path_clone = path.clone();
        let emit_clone = emit.clone();
        tokio::spawn(async move {
            match parse_and_analyze(&path_clone).await {
                Ok(result) => {
                    // Algorithm 24: check if content actually changed (hash)
                    if hash_changed(&path_clone, &result) {
                        emit_clone.emit_all("build-changed", &result).unwrap();
                    }
                }
                Err(e) => log::warn!("Re-parse failed for {:?}: {}", path_clone, e),
            }
        });
    }
}

fn is_pob_file(path: &Path) -> bool {
    matches!(path.extension().and_then(|e| e.to_str()), Some("xml" | "pob"))
}
```

### Platform Notes

| Platform | `notify` backend | Latency |
|----------|-----------------|---------|
| Windows | `ReadDirectoryChangesW` | ~5ms |
| macOS | FSEvents | ~10ms |
| Linux | inotify | ~1ms |

### Complexity

- `init_storage`: O(D) where D = number of directories to create (~20), one-time
- File watcher: O(1) per event (debounce check + spawn), O(P) per re-parse where P = PoB file size

---

## Complexity Summary

| # | Algorithm | Time Complexity | Typical Runtime |
|---|-----------|----------------|-----------------|
| 1 | Modifier Aggregation | O(M) | <5ms |
| 2 | Conversion Resolver | O(1) | <1ms |
| 3 | Evasion Counter | O(1) per check | <1μs |
| 4 | Leech Manager | O(I) per tick | <1ms |
| 5 | Flask Uptime | O(1) estimate, O(1) per tick | <1ms |
| 6 | Archetype Classifier | O(G) gems | <1ms |
| 7 | Item Scoring | O(M) mods per item | <1ms |
| 8 | Issue Detector | O(C) checks | <5ms |
| 9 | Sensitivity Analysis | O(S × T_calc) | ~750ms (parallelizable) |
| 10 | Pareto Ranking | O(N²) | <1ms for N≈50 |
| 11 | Knapsack Optimizer | O(S × U × B) | <1ms |
| 12 | Multi-Slot Solver | O(I × C × T_calc) | 2-5s |
| 13 | Tree Pathfinding | O((V+E) log V) | ~6ms |
| 14 | Respec Optimizer | O(V + E) Tarjan's | ~3ms |
| 15 | Craft Probability | O(P) pool size | <1ms |
| 16 | Monte Carlo Craft | O(10K × attempts) | ~100ms |
| 17 | Build Similarity | O(C × F) | ~1ms |
| 18 | Intent Classifier | O(R) regex rules | <1ms |
| 19 | Response Generator | O(T) template size | <1ms |
| 20 | Combat Simulation | O(T × M) ticks × monsters | 50ms (skip) to 150s (10x) |
| 21 | Price Cache | O(1) lookup | <1ms (cache hit) |
| 22 | PoB XML Parser | O(N) file size | ~10ms |
| 23 | Mod Text Parser | O(R) regex patterns | <1ms per mod |
| 24 | Change Detection | O(M) hash | <1ms |
| 25 | Fast Estimation | O(S) stats per estimate | <1ms |
| 26 | Ailment Mechanics | O(1) per ailment, O(N) per tick | <1ms |
| 27 | ES Recharge | O(1) per tick | <1μs |
| 28 | Mana Reservation | O(A) auras | <1ms |
| 29 | Clipboard Item Parser | O(L) lines | <1ms |
| 30 | Stat Requirement Checker | O(I × G) items × gems | <1ms |
| 31 | Charge Management | O(C) charge types | <1ms |
| 32 | Playstyle Classifier | O(1) lookup + O(T) traits | <1ms |
| 33 | Change History Manager | O(1) push/pop, O(D) diff | <5ms |
| 34 | OAuth Token Lifecycle | O(1) keychain read, O(1) refresh | <200ms (network) |
| 35 | Session Persistence | O(S) state fields | <10ms |
| 36 | Database Init & Migration | O(M) migrations | <50ms startup |
| 37 | OAuth PKCE Auth Flow | O(1) | ~1-5s (browser round-trip) |
| 38 | Stash Tab Processor | O(T + I) tabs + items | 2-10s (rate-limited) |
| 39 | Map Mod Danger Scorer | O(M) mods | <1ms |
| 40 | Build Share Code Codec | O(N) JSON size | <5ms encode, <10ms decode |
| 41 | Client.txt Log Parser | O(L) new log lines | <1ms per poll |
| 42 | Seer Network Architecture | O(1) fixed input + O(N) tree nodes | <100ms total |
| 43 | Vendor Recipe Detector | O(I) stash items | <5ms |
| 44 | Portable Storage & File Watcher | O(D) dirs init, O(1)/event | <50ms init |
| 45 | PoE Character Fetch Pipeline | O(C) characters | ~500ms (network) |
| 46 | PoB Write-Back Engine | O(N) XML nodes | <20ms |
| 47 | Craft Suggestion Ranker & Trade Search | O(M × T) methods × tiers | <5ms |
| 48 | Top-N Passive Node Recommender | O(U × P) unallocated × points | <10ms |
| 49 | Build Comparator | O(S) stats + O(T) tree nodes | <50ms |
| 50 | Price Alert Manager | O(A) active alerts per poll cycle | <100ms/cycle |
| 51 | Item Image Resolver | O(1) lookup, O(1) CDN fetch | <1ms (cache hit) |
| 52 | Buy Timing Advisor & Craft-vs-Buy | O(H) price history | <1ms |
| 53 | Map Run & Wealth Accumulator | O(R) runs + O(I) items | <10ms |
| 54 | Cloud AI Connection Manager | O(1) | ~200ms (network test) |

**Full build analysis pipeline (load → analyze → suggest):** ~1-2 seconds total.
**Interactive response (query → answer):** <500ms for Calculator/KB queries.
**Seer engine query:** <100ms (local), <2s (cloud fallback).

---

## 45. PoE Character Fetch Pipeline

### Problem

`load_character` and `switch_character` both require fetching live character data from
the PoE API after OAuth. The API has multiple endpoints (account, characters, items,
passive tree, stash) that must be called in order, with rate limiting and partial-
failure handling. The result feeds every downstream algorithm.

### Endpoints & Order

```
GET /api/account/profile              → account name, league
GET /api/account/characters           → list of all characters (name, class, level, league)
GET /api/character/{name}/items       → equipped items (9 slots + flasks + jewels)
GET /api/character/{name}/passives    → allocated passive nodes + jewel data + masteries
```

Rate limit: 45 requests / 60s per endpoint group. Characters and items share a group.

### Algorithm

```rust
pub async fn fetch_character(
    account:   &str,
    character: &str,
    token:     &StoredToken,
    http:      &RateLimitedClient,
    db:        &SqlitePool,
) -> Result<BuildData, FetchError> {

    // 1. Ensure token is valid (Algorithm 34)
    let access_token = ensure_valid_token(token, http).await?;
    let auth = format!("Bearer {access_token}");

    // 2. Fetch equipped items
    let items_resp: PoeItemsResponse = http
        .get(format!("{POE_API}/character/{character}/items"))
        .header("Authorization", &auth)
        .send().await?
        .json().await?;

    // 3. Fetch passive tree (separate endpoint, separate rate-limit bucket)
    let tree_resp: PoePassivesResponse = http
        .get(format!("{POE_API}/character/{character}/passives"))
        .header("Authorization", &auth)
        .send().await?
        .json().await?;

    // 4. Parse into domain types (Algorithm 22 handles XML; for OAuth we parse JSON)
    let build = BuildData {
        source:        BuildSource::OAuth { account: account.to_string(), character: character.to_string() },
        class:         items_resp.character.class,
        ascendancy:    items_resp.character.ascendancy_class,
        level:         items_resp.character.level,
        league:        items_resp.character.league,
        items:         parse_items(&items_resp.items),
        passive_nodes: tree_resp.hashes,
        masteries:     tree_resp.mastery_effects,
        jewels:        tree_resp.jewel_data,
    };

    // 5. Persist character metadata
    sqlx::query(
        "INSERT OR REPLACE INTO character_data
         (name, class_name, level, league, last_synced)
         VALUES (?, ?, ?, ?, ?)"
    ).bind(character).bind(&build.class)
     .bind(build.level).bind(&build.league)
     .bind(Utc::now().to_rfc3339())
     .execute(db).await?;

    Ok(build)
}

/// Switch active character: update session, invalidate calc cache, re-analyse.
pub async fn switch_character(
    character:  &str,
    app_state:  &AppState,
    db:         &SqlitePool,
) -> Result<AnalysisResult, FetchError> {

    // 1. Load last-known data from DB (instant, no network)
    let cached = sqlx::query_as::<_, CharacterRow>(
        "SELECT * FROM character_data WHERE name = ?"
    ).bind(character).fetch_optional(db).await?;

    // 2. Update session (Algorithm 35) immediately — UI responds at once
    app_state.session.write().active_character = character.to_string();
    auto_save(&app_state.session.read()).await;

    // 3. Fetch fresh data from API in background
    let build = fetch_character(
        &app_state.account, character,
        &app_state.token.read(), &app_state.http, db
    ).await?;

    // 4. Run full analysis pipeline
    let result = analyse_build(&build, app_state).await?;
    Ok(result)
}

/// Full character list for the account (used by character picker UI).
pub async fn list_characters(
    token: &StoredToken,
    http:  &RateLimitedClient,
) -> Result<Vec<PoeCharacter>, FetchError> {
    let access_token = ensure_valid_token(token, http).await?;
    let resp: PoeCharactersResponse = http
        .get(format!("{POE_API}/account/characters"))
        .header("Authorization", format!("Bearer {access_token}"))
        .send().await?
        .json().await?;
    Ok(resp.characters)
}
```

### Partial-Failure Handling

If the items endpoint succeeds but the passives endpoint fails (e.g., rate limited),
return the items result with `passive_nodes = vec![]` and mark `data_partial: true`
in the response. The frontend shows a warning banner and the passives tab shows a
"retry" button. Never block the items display waiting for passives.

### Complexity

- O(C) characters in list call; O(1) for single character fetch (2 API calls)
- DB writes: O(1)

---

## 46. PoB Write-Back Engine

### Problem

`apply_upgrade` modifies a PoB XML file in response to a suggestion (swap item, change
gem level, update tree nodes, change config). The file must be patched atomically:
backup first, validate the XML result, then replace — and the change must be recorded
in the Change History (Algorithm 33) so it can be undone.

### Operations

```rust
pub enum WriteOp {
    ReplaceItem   { slot: SlotName, item_set_id: u32, new_item_text: String },
    UpdateGem     { skill_label: String, gem_id: String, level: Option<u8>, quality: Option<u8>, enabled: Option<bool> },
    UpdateTree    { spec_index: usize, nodes: Vec<u32> },
    UpdateConfig  { name: String, value: ConfigValue },
}

#[derive(Clone)]
pub enum ConfigValue { Bool(bool), Number(f64), Str(String) }
```

### Algorithm

```rust
pub async fn apply_upgrade(
    op:        WriteOp,
    pob_path:  &Path,
    history:   &mut ChangeHistory,   // Algorithm 33
    db:        &SqlitePool,
) -> Result<ApplyResult, WriteError> {

    // 1. Snapshot BEFORE change (Algorithm 33)
    let before = read_build_state(pob_path).await?;
    let snapshot_id = history.push_snapshot(Snapshot {
        description:  op.description(),
        source:       ChangeSource::AiSuggestion(op.suggestion_id()),
        build_state:  before.clone(),
        stats_before: before.stats(),
        ..Default::default()
    });

    // 2. Check file lock (PoB may be saving simultaneously)
    if is_file_locked(pob_path) {
        return Err(WriteError::FileLocked);
    }

    // 3. Create backup in PathOfAI_Data/backups/
    let backup_path = create_backup(pob_path).await?;

    // 4. Parse XML
    let xml_bytes = tokio::fs::read(pob_path).await?;
    let mut doc = parse_xml(&xml_bytes)?;

    // 5. Apply the operation
    match &op {
        WriteOp::ReplaceItem { slot, item_set_id, new_item_text } => {
            replace_item_in_doc(&mut doc, slot, *item_set_id, new_item_text)?;
        }
        WriteOp::UpdateGem { skill_label, gem_id, level, quality, enabled } => {
            update_gem_in_doc(&mut doc, skill_label, gem_id, *level, *quality, *enabled)?;
        }
        WriteOp::UpdateTree { spec_index, nodes } => {
            update_tree_in_doc(&mut doc, *spec_index, nodes)?;
        }
        WriteOp::UpdateConfig { name, value } => {
            update_config_in_doc(&mut doc, name, value)?;
        }
    }

    // 6. Validate: re-serialise and parse to confirm well-formed XML
    let new_xml = serialise_xml(&doc)?;
    if let Err(e) = parse_xml(new_xml.as_bytes()) {
        // Restore backup — generated XML is malformed
        tokio::fs::copy(&backup_path, pob_path).await?;
        return Err(WriteError::InvalidXmlGenerated(e));
    }

    // 7. Atomic write (temp → fsync → rename)  [Algorithm 35 atomic_write]
    atomic_write(pob_path, new_xml.as_bytes()).await?;

    // 8. Snapshot AFTER change (complete the history entry)
    let after = read_build_state(pob_path).await?;
    history.complete_snapshot(snapshot_id, after.stats());

    Ok(ApplyResult {
        snapshot_id,
        backup_path,
        can_undo: true,
    })
}

/// DOM helpers ─────────────────────────────────────────────────────────────

fn replace_item_in_doc(doc: &mut XmlDoc, slot: &SlotName, item_set_id: u32, text: &str) -> Result<()> {
    let item_set = doc.find_item_set(item_set_id)
        .ok_or(WriteError::ItemSetNotFound(item_set_id))?;
    let target_slot = item_set.find_slot(slot)
        .ok_or(WriteError::SlotNotFound(slot.clone()))?;

    let new_id = doc.next_item_id();
    doc.items_section().append_item(new_id, text);
    target_slot.set_item_id(new_id);
    Ok(())
}

fn update_tree_in_doc(doc: &mut XmlDoc, spec_index: usize, nodes: &[u32]) -> Result<()> {
    let specs = doc.find_all_specs();
    let spec = specs.get_mut(spec_index)
        .ok_or(WriteError::SpecNotFound(spec_index))?;
    spec.set_nodes_attr(&nodes.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(","));
    Ok(())
}
```

### Complexity

- Backup + read: O(F) file size (~50-200KB for typical PoB files)
- XML parse + patch + re-serialise: O(N) XML nodes
- Atomic write: O(F)
- Total: <20ms for typical PoB files

---

## 47. Craft Suggestion Ranker & Trade Search

### Problem

`get_craft_suggestions` must return ranked crafting options for an item slot.
`search_upgrades` must construct a trade search URL. Algorithm 15 (Crafting
Probability Engine) computes success rates; this algorithm ranks methods by
expected cost-per-success and builds the trade filter.

### Craft Ranking Algorithm

```rust
pub struct CraftOption {
    pub method:         CraftMethod,      // Essence, Fossil, AltRegal, Harvest, Bench
    pub target_mods:    Vec<ModTarget>,
    pub avg_cost_div:   f64,
    pub worst_cost_div: f64,
    pub success_rate:   f64,             // probability per attempt
    pub avg_attempts:   f64,
    pub verdict:        CraftVerdict,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CraftVerdict {
    BestOption,          // Cheapest average cost
    SafeOption,          // Low worst-case variance (worst_cost < 2× avg)
    HighRisk,            // avg_cost ok but worst_cost >> avg
    NotWorthIt,          // avg_cost > buy_price
}

pub fn rank_craft_suggestions(
    slot:      EquipSlot,
    archetype: Archetype,
    budget:    f64,           // divine orbs
    prices:    &PriceCache,   // Algorithm 21
) -> Vec<CraftOption> {

    let target_mods = best_mods_for_slot(slot, archetype); // from archetype weight table
    let buy_price   = estimate_buy_price(slot, archetype, &target_mods, prices);

    let mut options: Vec<CraftOption> = CRAFT_METHODS.iter().filter_map(|method| {
        // Algorithm 15: compute expected attempts and cost
        let prob       = crafting_success_rate(method, &target_mods);
        if prob <= 0.0 { return None; }

        let avg_attempts = 1.0 / prob;
        let cost_per     = method_cost_per_attempt(method, prices);
        let avg_cost     = avg_attempts * cost_per;
        let worst_cost   = geometric_99th_percentile(prob) * cost_per; // 99th pct attempts

        if avg_cost > budget * 2.0 { return None; } // filter unaffordable

        let verdict = if avg_cost > buy_price {
            CraftVerdict::NotWorthIt
        } else if worst_cost > buy_price * 2.0 {
            CraftVerdict::HighRisk
        } else if worst_cost < avg_cost * 1.5 {
            CraftVerdict::SafeOption
        } else {
            CraftVerdict::BestOption
        };

        Some(CraftOption {
            method: method.clone(), target_mods: target_mods.clone(),
            avg_cost_div: avg_cost, worst_cost_div: worst_cost,
            success_rate: prob, avg_attempts, verdict,
        })
    }).collect();

    // Sort: BestOption first, then by avg_cost ascending
    options.sort_by(|a, b| {
        let rank = |v: &CraftVerdict| match v {
            CraftVerdict::BestOption  => 0,
            CraftVerdict::SafeOption  => 1,
            CraftVerdict::HighRisk    => 2,
            CraftVerdict::NotWorthIt  => 3,
        };
        rank(&a.verdict).cmp(&rank(&b.verdict))
            .then(a.avg_cost_div.partial_cmp(&b.avg_cost_div).unwrap())
    });

    options
}

/// 99th percentile attempts for geometric distribution: ceil(log(0.01) / log(1 - p))
fn geometric_99th_percentile(p: f64) -> f64 {
    (0.01_f64.ln() / (1.0 - p).ln()).ceil()
}
```

### Trade Search URL Builder

```rust
/// `search_upgrades` — builds a pathofexile.com/trade URL for the best upgrade.
pub fn build_trade_search_url(
    slot:      EquipSlot,
    archetype: Archetype,
    min_score: f64,           // only show items that beat current score by this margin
    league:    &str,
) -> String {
    let required_mods = must_have_mods(slot, archetype);    // top 2-3 mods from weight table
    let stat_filters  = mods_to_trade_filters(&required_mods, min_score);

    // Build the poe trade query JSON
    let query = TradeQuery {
        query: QuerySpec {
            filters: ItemFilters {
                type_filters: TypeFilter { filters: TypeFilters { rarity: Rarity::Rare }, disabled: false },
                ..Default::default()
            },
            stats: vec![StatGroup { r#type: "and".into(), filters: stat_filters }],
        },
        sort: SortSpec { price: "asc".into() },
    };

    let encoded = urlencoding::encode(&serde_json::to_string(&query).unwrap());
    format!("https://www.pathofexile.com/trade/search/{league}?q={encoded}")
}
```

### Complexity

- Rank: O(M × T) — M craft methods (≈8) × T target mod combinations
- Trade URL: O(1) — fixed number of stat filters

---

## 48. Top-N Passive Node Recommender

### Problem

`get_tree_analysis` returns the `nextBestPoints` field: the N unallocated passive nodes
that give the most value per point spent, reachable from the current allocation.
Algorithm 42 (TreeNet) scores individual nodes; this algorithm ranks them considering
path cost (travel nodes required to reach each candidate).

### Algorithm

```rust
pub struct NodeRecommendation {
    pub node_id:        u32,
    pub node_name:      String,
    pub stats:          Vec<StatLine>,
    pub value_score:    f64,       // stat value (archetype-weighted)
    pub path_cost:      u32,       // travel nodes needed to reach it
    pub efficiency:     f64,       // value_score / path_cost
    pub path:           Vec<u32>,  // node IDs of the path
}

pub fn recommend_next_points(
    allocated:    &HashSet<u32>,
    archetype:    Archetype,
    tree:         &PassiveTree,
    weights:      &ArchetypeWeightTable,   // Algorithm 7
    top_n:        usize,
) -> Vec<NodeRecommendation> {

    // 1. BFS from all allocated nodes to find reachable unallocated nodes
    //    and their shortest path costs
    let reachable = bfs_reachable(allocated, tree);
    // reachable: HashMap<node_id, (path_cost, path_vec)>

    // 2. Score each reachable node
    let mut candidates: Vec<NodeRecommendation> = reachable
        .into_iter()
        .filter(|(id, _)| !allocated.contains(id))
        .filter_map(|(node_id, (path_cost, path))| {
            let node = tree.get(node_id)?;
            if node.is_travel_node() { return None; } // skip pure travel nodes

            let value_score = stat_value(node, archetype, weights);
            if value_score <= 0.0 { return None; }

            Some(NodeRecommendation {
                node_id,
                node_name:   node.name.clone(),
                stats:       node.stats.clone(),
                value_score,
                path_cost:   path_cost as u32,
                efficiency:  value_score / path_cost.max(1) as f64,
                path,
            })
        })
        .collect();

    // 3. Sort by efficiency descending
    candidates.sort_by(|a, b| b.efficiency.partial_cmp(&a.efficiency).unwrap());
    candidates.truncate(top_n);
    candidates
}

/// BFS from ALL allocated nodes simultaneously (multi-source BFS).
/// Returns map of node_id → (distance, path).
fn bfs_reachable(
    allocated: &HashSet<u32>,
    tree:      &PassiveTree,
) -> HashMap<u32, (usize, Vec<u32>)> {
    let mut visited: HashMap<u32, (usize, Vec<u32>)> = HashMap::new();
    let mut queue: VecDeque<(u32, usize, Vec<u32>)> = VecDeque::new();

    // Seed: all currently allocated nodes at distance 0
    for &node_id in allocated {
        visited.insert(node_id, (0, vec![]));
        queue.push_back((node_id, 0, vec![]));
    }

    while let Some((node_id, dist, path)) = queue.pop_front() {
        for &neighbor in tree.neighbors(node_id) {
            if visited.contains_key(&neighbor) { continue; }
            let new_path = {
                let mut p = path.clone();
                p.push(neighbor);
                p
            };
            visited.insert(neighbor, (dist + 1, new_path.clone()));
            queue.push_back((neighbor, dist + 1, new_path));
        }
    }
    visited
}

fn stat_value(node: &PassiveNode, archetype: Archetype, weights: &ArchetypeWeightTable) -> f64 {
    node.stats.iter().map(|s| {
        let w = weights.get(archetype, s.stat_type);
        s.value.abs() as f64 * w
    }).sum()
}
```

### Complexity

- BFS: O(V + E) where V ≈ 1,500 nodes, E ≈ 4,000 edges → ~5ms
- Scoring: O(U) unallocated nodes within reach (typically 200-400)
- Total: O(V + E + U) ≈ <10ms

---

## 49. Build Comparator

### Problem

`compare_builds` produces a side-by-side stat diff between two builds loaded locally.
`compare_to_top` fetches the top-N builds for the same archetype from poe.ninja and
computes percentile ranks for each stat.

### Side-by-Side Comparator

```rust
pub struct BuildComparison {
    pub build_a:         BuildSummary,
    pub build_b:         BuildSummary,
    pub stat_diffs:      Vec<StatDiff>,
    pub tree_overlap_pct: f64,          // % of nodes shared
    pub missing_nodes:   Vec<u32>,      // in B but not A
    pub extra_nodes:     Vec<u32>,      // in A but not B
    pub popular_gems:    Vec<GemDiff>,  // gems B uses that A doesn't
}

pub struct StatDiff {
    pub stat:    StatType,
    pub value_a: f64,
    pub value_b: f64,
    pub delta:   f64,   // value_b - value_a
    pub delta_pct: f64, // percentage change
}

pub fn compare_builds(build_a: &BuildData, build_b: &BuildData) -> BuildComparison {
    let stats_a = build_a.calc.all_stats();
    let stats_b = build_b.calc.all_stats();

    let stat_diffs = StatType::all().iter().map(|&stat| {
        let a = stats_a.get(stat).unwrap_or(0.0);
        let b = stats_b.get(stat).unwrap_or(0.0);
        StatDiff {
            stat,
            value_a:   a,
            value_b:   b,
            delta:     b - a,
            delta_pct: if a != 0.0 { (b - a) / a * 100.0 } else { 0.0 },
        }
    }).collect();

    let nodes_a: HashSet<u32> = build_a.passive_nodes.iter().cloned().collect();
    let nodes_b: HashSet<u32> = build_b.passive_nodes.iter().cloned().collect();
    let shared  = nodes_a.intersection(&nodes_b).count();
    let union   = nodes_a.union(&nodes_b).count();

    BuildComparison {
        build_a:          build_a.summary(),
        build_b:          build_b.summary(),
        stat_diffs,
        tree_overlap_pct: if union > 0 { shared as f64 / union as f64 * 100.0 } else { 0.0 },
        missing_nodes:    nodes_b.difference(&nodes_a).cloned().collect(),
        extra_nodes:      nodes_a.difference(&nodes_b).cloned().collect(),
        popular_gems:     diff_gems(build_a, build_b),
    }
}
```

### Compare-to-Top (poe.ninja)

```rust
pub struct TopBuildComparison {
    pub our_build:    BuildSummary,
    pub sample_size:  usize,
    pub percentiles:  HashMap<StatType, Percentile>,
    pub missing_gems: Vec<String>,          // gems >30% of top builds use that we don't
    pub missing_nodes: Vec<u32>,            // nodes >40% of top builds allocate that we don't
}

pub struct Percentile {
    pub stat:      StatType,
    pub our_value: f64,
    pub rank:      f64,    // 0.0 = bottom, 1.0 = top
    pub p25:       f64,
    pub p50:       f64,
    pub p75:       f64,
}

pub async fn compare_to_top(
    build:     &BuildData,
    http:      &HttpClient,
    league:    &str,
) -> Result<TopBuildComparison, FetchError> {

    // 1. Fetch top builds for this archetype from poe.ninja builds endpoint
    let archetype_tag = build.archetype.poe_ninja_tag(); // e.g. "RighteousFire"
    let top_builds: Vec<NinjaBuildEntry> = http
        .get(format!("https://poe.ninja/api/data/builds?league={league}&class={archetype_tag}&limit=200"))
        .send().await?
        .json::<NinjaBuildsResponse>().await?
        .builds;

    // 2. Compute percentile rank for each key stat
    let our_stats = build.calc.all_stats();
    let mut percentiles = HashMap::new();

    for stat in KEY_STATS_FOR_COMPARISON {
        let our_val = our_stats.get(stat).unwrap_or(0.0);
        let mut all_vals: Vec<f64> = top_builds.iter()
            .filter_map(|b| b.stat(stat))
            .collect();
        all_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let rank = all_vals.partition_point(|&v| v < our_val) as f64
            / all_vals.len().max(1) as f64;

        percentiles.insert(stat, Percentile {
            stat, our_value: our_val, rank,
            p25: percentile_val(&all_vals, 0.25),
            p50: percentile_val(&all_vals, 0.50),
            p75: percentile_val(&all_vals, 0.75),
        });
    }

    // 3. Find popular gems we don't use
    let our_gems: HashSet<&str> = build.gems.iter().map(|g| g.id.as_str()).collect();
    let gem_freq = count_gem_frequency(&top_builds);
    let missing_gems = gem_freq.into_iter()
        .filter(|(gem, freq)| *freq > 0.30 && !our_gems.contains(gem.as_str()))
        .map(|(gem, _)| gem)
        .collect();

    // 4. Find popular nodes we don't allocate
    let our_nodes: HashSet<u32> = build.passive_nodes.iter().cloned().collect();
    let node_freq = count_node_frequency(&top_builds);
    let missing_nodes = node_freq.into_iter()
        .filter(|(node, freq)| *freq > 0.40 && !our_nodes.contains(node))
        .map(|(node, _)| node)
        .collect();

    Ok(TopBuildComparison {
        our_build:    build.summary(),
        sample_size:  top_builds.len(),
        percentiles,
        missing_gems,
        missing_nodes,
    })
}

fn percentile_val(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() { return 0.0; }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx]
}
```

### Complexity

- `compare_builds`: O(S) stats + O(T) tree nodes — <1ms
- `compare_to_top`: O(N × S) where N = top builds (≤200), S = stats — <50ms

---

## 50. Price Alert Manager

### Problem

`set_price_alert`, `list_price_alerts`, and `remove_price_alert` manage a persistent
alert list. A background poller checks active alerts against live prices (via Algorithm
21) and emits a `price-alert-triggered` Tauri event when conditions are met.

### Data Model (from DATABASE.md `alerts` table)

```rust
pub struct PriceAlert {
    pub id:            i64,
    pub alert_type:    AlertType,
    pub alert_name:    Option<String>,
    pub item_key:      Option<String>,
    pub threshold:     Option<f64>,
    pub comparison:    Comparison,       // Below | Above | ChangePercent
    pub notify_method: NotifyMethod,     // Popup | Sound | Silent
    pub active:        bool,
    pub created_at:    DateTime<Utc>,
    pub last_triggered: Option<DateTime<Utc>>,
}

pub enum AlertType { PriceDrop, Snipe, CurrencyRate }
pub enum Comparison { Below, Above, ChangePercent }
```

### CRUD Operations

```rust
pub async fn set_price_alert(alert: NewAlert, db: &SqlitePool) -> Result<PriceAlert> {
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO alerts (alert_type, alert_name, item_key, threshold, comparison,
                             notify_method, active, created_at)
         VALUES (?, ?, ?, ?, ?, ?, 1, ?)
         RETURNING id"
    ).bind(alert.alert_type.to_str())
     .bind(&alert.alert_name)
     .bind(&alert.item_key)
     .bind(alert.threshold)
     .bind(alert.comparison.to_str())
     .bind(alert.notify_method.to_str())
     .bind(Utc::now().to_rfc3339())
     .fetch_one(db).await?;
    load_alert(id, db).await
}

pub async fn list_price_alerts(db: &SqlitePool) -> Result<Vec<PriceAlert>> {
    sqlx::query_as::<_, AlertRow>("SELECT * FROM alerts WHERE active = 1 ORDER BY created_at DESC")
        .fetch_all(db).await.map(|rows| rows.into_iter().map(PriceAlert::from).collect())
}

pub async fn remove_price_alert(id: i64, db: &SqlitePool) -> Result<()> {
    sqlx::query("DELETE FROM alerts WHERE id = ?")
        .bind(id).execute(db).await?;
    Ok(())
}
```

### Background Polling Loop

```rust
/// Spawned once at app startup. Polls every POLL_INTERVAL seconds.
pub async fn alert_poll_loop(
    db:     SqlitePool,
    cache:  Arc<Mutex<PriceCache>>,   // Algorithm 21
    app:    AppHandle,
) {
    const POLL_INTERVAL: u64 = 60; // seconds

    loop {
        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL)).await;

        let alerts = match list_price_alerts(&db).await {
            Ok(a) => a,
            Err(e) => { log::warn!("Alert poll DB error: {e}"); continue; }
        };

        for alert in alerts {
            if let Some(triggered) = check_alert(&alert, &cache).await {
                // Persist trigger timestamp
                sqlx::query("UPDATE alerts SET last_triggered = ? WHERE id = ?")
                    .bind(Utc::now().to_rfc3339())
                    .bind(alert.id)
                    .execute(&db).await.ok();

                // Emit to frontend
                app.emit_all("price-alert-triggered", &triggered).ok();
            }
        }
    }
}

async fn check_alert(alert: &PriceAlert, cache: &Arc<Mutex<PriceCache>>) -> Option<TriggeredAlert> {
    let item_key = alert.item_key.as_ref()?;
    let price = cache.lock().await.get(item_key)?;

    let triggered = match alert.comparison {
        Comparison::Below         => price.divine <= alert.threshold?,
        Comparison::Above         => price.divine >= alert.threshold?,
        Comparison::ChangePercent => price.change_7d.abs() >= alert.threshold?,
    };

    if triggered {
        Some(TriggeredAlert {
            alert_id:  alert.id,
            item_key:  item_key.clone(),
            message:   format_trigger_message(alert, price),
            price:     price.divine,
        })
    } else {
        None
    }
}
```

### Complexity

- CRUD: O(1) per operation (indexed `alerts` table)
- Poll loop: O(A) active alerts per cycle, each O(1) cache lookup
- Full poll cycle: <100ms for typical A ≤ 20

---

## 51. Item Image Resolver

### Problem

Every item, gem, flask, currency, and skill icon displayed in the UI needs a game art
URL. Images come from the PoE CDN (`web.poecdn.com`). They must be cached on disk to
avoid re-downloading on every app launch and to work offline after first fetch.

### Resolution Cascade

```
1. In-memory cache (HashMap) → return immediately
2. Disk cache (PathOfAI_Data/cache/images/) → return file:// URL + warm memory cache
3. CDN URL from static lookup table (UNIQUE_ITEM_IMAGES / BASE_TYPE_IMAGES etc.)
4. CDN URL constructed from naming convention (fallback for unlisted items)
5. Wiki URL (poe.wiki.net/wiki/Special:FilePath/) → last resort for obscure items
6. Placeholder (grey outline icon) → shown while any async fetch is in flight
```

### Algorithm

```rust
const CDN_BASE:  &str = "https://web.poecdn.com/image";
const WIKI_BASE: &str = "https://www.poewiki.net/wiki/Special:FilePath";

pub struct ImageResolver {
    memory:    HashMap<String, ResolvedImage>,
    cache_dir: PathBuf,           // PathOfAI_Data/cache/images/
    http:      HttpClient,
    manifest:  ImageManifest,     // tracks which CDN URLs are cached locally
}

#[derive(Clone)]
pub enum ResolvedImage {
    LocalFile(PathBuf),    // disk-cached — use file:// URL
    CdnUrl(String),        // not cached yet — use CDN directly
    Placeholder,           // unknown item — show generic icon
}

pub fn resolve(&mut self, request: ImageRequest) -> ResolvedImage {
    let key = request.cache_key();

    // 1. Memory cache
    if let Some(img) = self.memory.get(&key) {
        return img.clone();
    }

    // 2. Disk cache
    let disk_path = self.cache_dir.join(&key).with_extension("png");
    if disk_path.exists() {
        let img = ResolvedImage::LocalFile(disk_path);
        self.memory.insert(key.clone(), img.clone());
        return img;
    }

    // 3-5. Resolve URL
    let cdn_url = self.resolve_url(&request);
    let img = ResolvedImage::CdnUrl(cdn_url.clone());
    self.memory.insert(key.clone(), img.clone());

    // 6. Kick off background download (non-blocking)
    let resolver = self.clone_for_download();
    tokio::spawn(async move {
        if let Ok(bytes) = resolver.http.get(&cdn_url).send().await
            .and_then(|r| r.bytes().await.map_err(Into::into))
        {
            let _ = tokio::fs::write(&disk_path, &bytes).await;
            resolver.manifest.record(&key, &cdn_url);
        }
    });

    img
}

fn resolve_url(&self, req: &ImageRequest) -> String {
    match req {
        ImageRequest::UniqueItem(name) => {
            UNIQUE_ITEM_IMAGES.get(name.as_str())
                .map(|p| format!("{CDN_BASE}/{p}"))
                .unwrap_or_else(|| {
                    let safe = urlencoding::encode(&name.replace(' ', "_"));
                    format!("{WIKI_BASE}/{safe}_inventory_icon.png")
                })
        }
        ImageRequest::BaseType(base, tags) => {
            BASE_TYPE_IMAGES.get(base.as_str())
                .map(|p| format!("{CDN_BASE}/{p}"))
                .unwrap_or_else(|| fallback_by_tag(tags))
        }
        ImageRequest::Gem(gem_id) => {
            GEM_IMAGES.get(gem_id.as_str())
                .map(|p| format!("{CDN_BASE}/{p}"))
                .unwrap_or_else(|| format!("{CDN_BASE}/Art/2DItems/Gems/{gem_id}.png"))
        }
        ImageRequest::Currency(name) => {
            CURRENCY_IMAGES.get(name.as_str())
                .map(|p| format!("{CDN_BASE}/{p}"))
                .unwrap_or(PLACEHOLDER_CURRENCY.to_string())
        }
        ImageRequest::SkillIcon(skill_id) => {
            format!("{CDN_BASE}/Art/2DArt/SkillIcons/{skill_id}.png")
        }
    }
}
```

### Manifest Tracking

```rust
/// Tracks which items have been cached so we can invalidate on league update.
pub struct ImageManifest {
    entries: HashMap<String, ManifestEntry>,
    path:    PathBuf,   // PathOfAI_Data/cache/manifest.json
}

pub struct ManifestEntry {
    pub cdn_url:   String,
    pub cached_at: DateTime<Utc>,
    pub file_size: u64,
}

impl ImageManifest {
    /// Called when a new league launches — stale CDN paths may have changed.
    pub fn invalidate_all(&mut self) {
        self.entries.clear();
        // Files on disk remain — they'll be re-validated on next resolve
    }
}
```

### Complexity

- Cache hit (memory): O(1)
- Cache hit (disk): O(1)
- Cache miss: O(1) URL lookup + async download (non-blocking)

---

## 52. Buy Timing Advisor & Craft-vs-Buy

### Problem

When a user views an item's price, the app should say more than just the current price:
it should recommend *when* to buy (based on trend + league phase) and whether to craft
instead of buy (expected craft cost vs buy price). Both are synthesised from
Algorithm 21 (price cache) and the league phase model.

### Buy Timing Algorithm

```rust
#[derive(Debug, Clone)]
pub enum BuyAction {
    Wait,           // price falling — don't buy yet
    BuySoon,        // nearing floor — enter in 2-3 days
    BuyNow,         // price rising or late-league supply squeeze
    BuyNowOrWait,   // sharp spike — could correct or continue
    BuyWhenReady,   // stable — timing doesn't matter
    Monitor,        // insufficient data
}

pub struct BuyRecommendation {
    pub action:       BuyAction,
    pub reason:       String,
    pub urgency:      Urgency,       // None | Low | Medium | High
    pub confidence:   Confidence,    // Low | Medium | High
    pub current_div:  f64,
    pub trend:        TrendDirection,
    pub change_7d:    f64,
    pub league_phase: LeaguePhase,
    pub sparkline:    Vec<f64>,      // 14-day price history
}

pub fn generate_buy_recommendation(
    item_key: &str,
    history:  &[PricePoint],         // from price_history table
    phase:    LeaguePhase,
) -> BuyRecommendation {

    // Calculate 7-day trend
    let recent = history.iter().rev().take(7).collect::<Vec<_>>();
    let (oldest, newest) = match (recent.last(), recent.first()) {
        (Some(o), Some(n)) => (o.price_divine, n.price_divine),
        _ => return BuyRecommendation::unknown(item_key),
    };
    let change_7d = if oldest > 0.0 { (newest - oldest) / oldest * 100.0 } else { 0.0 };

    let trend = match change_7d {
        c if c < -20.0 => TrendDirection::DroppingFast,
        c if c <  -5.0 => TrendDirection::DroppingSlow,
        c if c >  20.0 => TrendDirection::RisingFast,
        c if c >   5.0 => TrendDirection::RisingSlow,
        _              => TrendDirection::Stable,
    };

    let confidence = match recent.len() {
        n if n >= 5 => Confidence::High,
        n if n >= 3 => Confidence::Medium,
        _           => Confidence::Low,
    };

    // Decision matrix (phase × trend)
    let (action, urgency) = match (phase, &trend) {
        // Early league — prices crash daily regardless of trend
        (LeaguePhase::LaunchFrenzy | LeaguePhase::CrashPeriod, t)
            if *t != TrendDirection::RisingFast
            => (BuyAction::Wait, Urgency::None),

        // Dropping fast — always wait
        (_, TrendDirection::DroppingFast)
            => (BuyAction::Wait, Urgency::None),

        // Slow drop in stable phase — approaching floor
        (LeaguePhase::Stabilization | LeaguePhase::PeakEconomy | LeaguePhase::LateLeague,
         TrendDirection::DroppingSlow)
            => (BuyAction::BuySoon, Urgency::Low),

        // Rising in late league — supply squeeze
        (LeaguePhase::PeakEconomy | LeaguePhase::LateLeague, TrendDirection::RisingSlow)
            => (BuyAction::BuyNow, Urgency::High),

        // Sharp spike — uncertain
        (_, TrendDirection::RisingFast)
            => (BuyAction::BuyNowOrWait, Urgency::Medium),

        // Stable in mature phase — timing irrelevant
        (LeaguePhase::Stabilization | LeaguePhase::PeakEconomy | LeaguePhase::LateLeague,
         TrendDirection::Stable)
            => (BuyAction::BuyWhenReady, Urgency::None),

        _ => (BuyAction::Monitor, Urgency::None),
    };

    let reason = format_reason(&action, item_key, newest, change_7d, &phase);

    BuyRecommendation {
        action, reason, urgency, confidence,
        current_div: newest, trend, change_7d, league_phase: phase,
        sparkline: history.iter().rev().take(14).rev().map(|p| p.price_divine).collect(),
    }
}
```

### Craft-vs-Buy Algorithm

```rust
pub struct CraftVsBuy {
    pub buy_price_div:   f64,
    pub craft_avg_div:   f64,
    pub craft_worst_div: f64,
    pub recommendation:  CraftVsBuyVerdict,
    pub risk:            RiskLevel,
}

pub enum CraftVsBuyVerdict {
    Craft,          // avg craft cost < 70% of buy price
    CraftIfFlexible, // avg ok but variance is high
    Buy,            // crafting not cost-efficient
}

pub fn compare_craft_vs_buy(
    target_item: &ItemSpec,
    method:      CraftMethod,
    buy_price:   f64,
    prices:      &PriceCache,
) -> CraftVsBuy {

    // Algorithm 15: expected attempts, Algorithm 47: cost per attempt
    let prob         = crafting_success_rate(&method, &target_item.required_mods);
    let cost_per     = method_cost_per_attempt(&method, prices);
    let avg_attempts = if prob > 0.0 { 1.0 / prob } else { f64::INFINITY };
    let avg_cost     = avg_attempts * cost_per;
    let worst_cost   = geometric_99th_percentile(prob) * cost_per;

    let recommendation = if avg_cost < buy_price * 0.70 {
        CraftVsBuyVerdict::Craft
    } else if avg_cost < buy_price {
        CraftVsBuyVerdict::CraftIfFlexible
    } else {
        CraftVsBuyVerdict::Buy
    };

    let risk = if worst_cost > buy_price * 2.0 {
        RiskLevel::High   // worst case costs 2× buy price
    } else {
        RiskLevel::Low
    };

    CraftVsBuy { buy_price_div: buy_price, craft_avg_div: avg_cost, craft_worst_div: worst_cost, recommendation, risk }
}
```

### Complexity

- `generate_buy_recommendation`: O(H) where H = history points (≤14) — <1ms
- `compare_craft_vs_buy`: O(1) — fixed-size computation

---

## 53. Map Run & Wealth Accumulator

### Problem

Three DATABASE.md tables (`map_runs`, `wealth_snapshots`, `div_card_progress`) have
no algorithm for how data flows into them. The `session_stats` and `map_runs` tables
are fed by Algorithm 41 (Client.txt log parser). This algorithm defines how raw zone
events become DB rows, how wealth snapshots are triggered, and how div cards are
tallied from stash data.

### Map Run Persistence

```rust
/// Called by Algorithm 41 when a map run completes.
pub async fn record_map_run(
    run:      &CompletedRun,
    build_id: Option<&str>,
    db:       &SqlitePool,
) -> Result<()> {
    // Only record actual map runs (not act zones, hideouts, labs)
    if !matches!(run.zone_type, ZoneType::Map { .. }) { return Ok(()); }

    let tier = match &run.zone_type {
        ZoneType::Map { tier, .. } => tier.unwrap_or(0),
        _ => 0,
    };

    sqlx::query(
        "INSERT INTO map_runs
         (build_id, map_name, map_tier, clear_time_ms, deaths, started_at, finished_at)
         VALUES (?, ?, ?, ?, 0, ?, ?)"
    ).bind(build_id)
     .bind(&run.zone_name)
     .bind(tier as i64)
     .bind((run.duration_secs * 1000.0) as i64)
     .bind((run.completed_at - Duration::seconds(run.duration_secs as i64)).to_rfc3339())
     .bind(run.completed_at.to_rfc3339())
     .execute(db).await?;

    // Update rolling session stats
    update_session_stats(build_id, run, db).await?;
    Ok(())
}

async fn update_session_stats(build_id: Option<&str>, run: &CompletedRun, db: &SqlitePool) -> Result<()> {
    // Upsert: if no session row exists for today, create it; else accumulate
    sqlx::query(
        "INSERT INTO session_stats (build_id, maps_run, total_deaths, avg_clear_ms, best_clear_ms, session_start)
         VALUES (?, 1, 0, ?, ?, ?)
         ON CONFLICT(build_id, session_start) DO UPDATE SET
           maps_run    = maps_run + 1,
           avg_clear_ms = (avg_clear_ms * (maps_run) + excluded.avg_clear_ms) / (maps_run + 1),
           best_clear_ms = MIN(best_clear_ms, excluded.best_clear_ms)"
    ).bind(build_id)
     .bind((run.duration_secs * 1000.0) as i64)
     .bind((run.duration_secs * 1000.0) as i64)
     .bind(today_iso())
     .execute(db).await?;
    Ok(())
}
```

### Wealth Snapshot Trigger

Wealth snapshots are taken automatically every time the stash is fetched (Algorithm 38)
and the total value changes by more than 1 divine, or on a daily schedule.

```rust
pub async fn maybe_snapshot_wealth(
    currency: &CurrencyTotal,   // from Algorithm 38
    gear_val: f64,              // sum of equipped item prices
    db:       &SqlitePool,
) -> Result<()> {
    // Check last snapshot value
    let last: Option<f64> = sqlx::query_scalar(
        "SELECT total_divine FROM wealth_snapshots ORDER BY recorded_at DESC LIMIT 1"
    ).fetch_optional(db).await?;

    let new_total = currency.divine_total + gear_val;
    let should_snap = last.map(|l| (new_total - l).abs() > 1.0).unwrap_or(true);

    if should_snap {
        sqlx::query(
            "INSERT INTO wealth_snapshots (total_divine, currency_breakdown, stash_value, gear_value, recorded_at)
             VALUES (?, ?, ?, ?, ?)"
        ).bind(new_total)
         .bind(serde_json::to_string(&currency.breakdown)?)
         .bind(currency.divine_total)
         .bind(gear_val)
         .bind(Utc::now().to_rfc3339())
         .execute(db).await?;
    }
    Ok(())
}
```

### Div Card Accumulator

```rust
/// Called after stash fetch — updates div_card_progress from current stash contents.
pub async fn update_div_cards(stash_items: &[StashItem], db: &SqlitePool) -> Result<()> {
    // Count owned cards from stash
    let mut owned: HashMap<String, i64> = HashMap::new();
    for item in stash_items {
        if item.frame_type == FrameType::DivinationCard {
            *owned.entry(item.type_line.clone()).or_insert(0) += item.stack_size as i64;
        }
    }

    for (card_name, count) in owned {
        let required = DIV_CARD_DB.required(&card_name).unwrap_or(1);
        sqlx::query(
            "INSERT INTO div_card_progress (card_name, owned, required, reward, drop_locations, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(card_name) DO UPDATE SET owned = excluded.owned, updated_at = excluded.updated_at"
        ).bind(&card_name)
         .bind(count)
         .bind(required)
         .bind(DIV_CARD_DB.reward(&card_name).unwrap_or(""))
         .bind(serde_json::to_string(&DIV_CARD_DB.drop_locations(&card_name))?)
         .bind(Utc::now().to_rfc3339())
         .execute(db).await?;
    }
    Ok(())
}
```

### Complexity

- `record_map_run`: O(1) DB insert + O(1) upsert
- `maybe_snapshot_wealth`: O(1) — one SELECT + conditional INSERT
- `update_div_cards`: O(D) where D = divination card types found in stash (typically < 20)

---

## 54. Cloud AI Connection Manager

### Problem

`test_cloud_ai` verifies that a user-provided API key works before saving it.
`ask_seer` falls back to the cloud engine for ~3% of queries (Algorithm 42).
This algorithm manages provider selection, connection testing, API key storage, and
the fallback chain when the primary cloud provider is unavailable.

### Supported Providers

```rust
pub enum CloudProvider {
    Claude,      // api.anthropic.com — default
    Gpt4,        // api.openai.com
    Gemini,      // generativelanguage.googleapis.com
    Ollama,      // localhost:11434 — local, no API key needed
    OpenRouter,  // openrouter.ai — aggregator
}
```

### Connection Test

```rust
pub async fn test_cloud_ai(
    provider: CloudProvider,
    api_key:  &str,
    http:     &HttpClient,
) -> Result<ConnectionTestResult, TestError> {

    // Minimal probe — send smallest valid request to verify auth
    let probe_result = match provider {
        CloudProvider::Claude => {
            http.post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&serde_json::json!({
                    "model": "claude-haiku-4-5-20251001",
                    "max_tokens": 1,
                    "messages": [{"role": "user", "content": "Hi"}]
                }))
                .send().await
        }
        CloudProvider::Gpt4 => {
            http.post("https://api.openai.com/v1/chat/completions")
                .bearer_auth(api_key)
                .json(&serde_json::json!({
                    "model": "gpt-4o-mini",
                    "max_tokens": 1,
                    "messages": [{"role": "user", "content": "Hi"}]
                }))
                .send().await
        }
        CloudProvider::Ollama => {
            // Ollama: no key needed, just check if server is running
            http.get("http://localhost:11434/api/tags").send().await
        }
        // ... other providers
    };

    match probe_result {
        Ok(resp) if resp.status().is_success() => {
            // Save key to encrypted config (NOT to OS keychain — API keys are less sensitive than OAuth tokens)
            save_api_key(provider, api_key)?;
            Ok(ConnectionTestResult {
                provider,
                success: true,
                latency_ms: resp.elapsed_ms(),
                model_available: true,
            })
        }
        Ok(resp) if resp.status() == 401 => Err(TestError::InvalidKey),
        Ok(resp) if resp.status() == 429 => Err(TestError::RateLimited),
        Ok(resp) => Err(TestError::UnexpectedStatus(resp.status().as_u16())),
        Err(e) if e.is_timeout() => Err(TestError::Timeout),
        Err(e) => Err(TestError::Network(e.to_string())),
    }
}
```

### Fallback Chain

When Algorithm 42 decides a query needs the Cloud engine:

```rust
pub async fn cloud_query(
    prompt:    &str,
    providers: &[CloudProvider],   // ordered list from settings
    keys:      &ApiKeyStore,
    http:      &HttpClient,
) -> Result<String, CloudError> {

    for provider in providers {
        let key = match keys.get(provider) {
            Some(k) => k,
            None    => continue,   // provider not configured — try next
        };

        match send_cloud_request(provider, key, prompt, http).await {
            Ok(response)                    => return Ok(response),
            Err(CloudError::RateLimited)    => continue,  // try next provider
            Err(CloudError::Timeout)        => continue,
            Err(e)                          => return Err(e), // auth/network — stop
        }
    }

    Err(CloudError::AllProvidersFailed)
}
```

API key storage: saved to `PathOfAI_Data/config/ai-providers.json` (AES-256 encrypted
with a key derived from machine ID). Unlike OAuth tokens, API keys are user-typed
secrets that don't need OS keychain-level isolation.

### Complexity

- `test_cloud_ai`: O(1) — single HTTP probe, ~200ms
- `cloud_query`: O(P) providers in fallback chain (usually P = 1-2)

---

## Complexity Summary

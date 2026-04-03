# Path of AI — Engine Design & The Seer Architecture

## The Core Question

How do we make The Seer so accurate that users NEVER need Claude/GPT/Gemini?

**Answer: Don't use AI where math works.** 95% of PoE advice is deterministic calculation,
not creative reasoning. The Seer should be a **calculation engine with natural language output**,
not a language model trying to do math.

---

## 1. THE THREE-ENGINE ARCHITECTURE

```
User asks: "What should I upgrade next?"
           ↓
┌──────────────────────────────────────────────────────────┐
│  ENGINE 1: CALCULATOR (deterministic, 100% accurate)     │
│                                                          │
│  "Ring 2 is your worst slot (score 42/100).              │
│   Replacing with +80 life +fire DoT multi ring           │
│   gives exactly +15.3% DPS and +350 life."               │
│                                                          │
│  HOW: PoB Lua calc engine (exact same math as PoB)       │
│  ACCURACY: 100% — same engine, same result               │
│  SPEED: <500ms                                           │
│  WHEN TO USE: Any question involving DPS, life,          │
│  resist, score, ranking, comparison                      │
└──────────────────────────────────────────────────────────┘
           ↓ (if Calculator can answer, stop here — 85% of queries)
           ↓ (if needs game knowledge, continue to Engine 2)
┌──────────────────────────────────────────────────────────┐
│  ENGINE 2: KNOWLEDGE BASE (lookup, 99% accurate)         │
│                                                          │
│  "The best fossil combo for your helmet is               │
│   Pristine + Scorched (blocks bad mods, boosts           │
│   life + fire). Expected cost: 3 divine."                │
│                                                          │
│  HOW: Structured game data (poedb mod weights,           │
│  boss HP/damage, gem tags, vendor recipes, etc.)         │
│  + template response generation                          │
│  ACCURACY: 99% — data from game files, updated per patch │
│  SPEED: <100ms                                           │
│  WHEN TO USE: Crafting advice, boss mechanics,           │
│  gem interactions, mod explanations                      │
└──────────────────────────────────────────────────────────┘
           ↓ (if Knowledge Base can answer, stop here — 12% of queries)
           ↓ (if needs creative/open-ended reasoning, continue)
┌──────────────────────────────────────────────────────────┐
│  ENGINE 3: LANGUAGE MODEL (creative, ~90% accurate)      │
│                                                          │
│  "Design me a league starter that uses fire skills        │
│   and can do all content on 10 divine budget."           │
│                                                          │
│  HOW: Fine-tuned small LLM (Phi-3/Llama 3B) OR          │
│  cloud API (Claude/GPT) for complex creative queries     │
│  ACCURACY: ~90% (needs human judgment)                   │
│  SPEED: 1-5s                                             │
│  WHEN TO USE: Creative build design, "why" questions,    │
│  open-ended strategy, patch analysis                     │
└──────────────────────────────────────────────────────────┘

ROUTING: 85% → Engine 1 (free, instant, 100% accurate)
         12% → Engine 2 (free, fast, 99% accurate)
          3% → Engine 3 (optional, slower, needs model)

RESULT: 97% of queries answered WITHOUT any AI model.
        Users only need cloud AI for creative questions.
```

---

## 2. ENGINE 1: THE CALCULATOR (Dual Engine — Ours + PoB)

### Why This Is The Core

The #1 reason users want a build advisor is: **"What gives me more DPS/life?"**
This is pure math. Not opinion. Not creativity. Math.

### The Dual Calculator Strategy

We build **our own calculation engine in Rust** as the primary engine, and keep
PoB's Lua calc engine as an **optional secondary/verification** engine.

```
┌──────────────────────────────────────────────────────────┐
│  DUAL CALCULATOR ARCHITECTURE                            │
│                                                          │
│  ┌────────────────────────┐  ┌────────────────────────┐  │
│  │  PATH OF AI CALCULATOR │  │  PoB LUA CALCULATOR    │  │
│  │  (PRIMARY — Rust)      │  │  (SECONDARY — Lua)     │  │
│  │                        │  │                        │  │
│  │  • Written in Rust     │  │  • Embedded LuaJIT     │  │
│  │  • We own & maintain   │  │  • PoB community code  │  │
│  │  • Always available    │  │  • Optional (toggle)   │  │
│  │  • Fast (<100ms)       │  │  • Slower (<500ms)     │  │
│  │  • Updated by us       │  │  • Updated by PoB team │  │
│  │  • Full control        │  │  • Dependency risk     │  │
│  └───────────┬────────────┘  └───────────┬────────────┘  │
│              │                            │               │
│              └──────────┬─────────────────┘               │
│                         ▼                                 │
│              ┌──────────────────────┐                     │
│              │  COMPARISON VIEW     │                     │
│              │                      │                     │
│              │  Our calc:  2.84M    │                     │
│              │  PoB calc:  2.84M ✓  │                     │
│              │  Match: YES          │                     │
│              │                      │                     │
│              │  (or if different:)  │                     │
│              │  Our calc:  2.91M    │                     │
│              │  PoB calc:  2.84M    │                     │
│              │  Diff: +2.5%        │                     │
│              │  [Which to trust?]  │                     │
│              └──────────────────────┘                     │
└──────────────────────────────────────────────────────────┘
```

### Why Build Our Own (Not Just Embed PoB)

**Risks of depending entirely on PoB's Lua engine:**
- PoB could stop being updated (maintainers burnout, PoE 2 focus)
- PoB could change their internal API (breaks our integration)
- PoB could have bugs we can't fix (we don't control the code)
- LuaJIT adds complexity to our Rust build (FFI boundary)
- 50,000 lines of Lua = hard to audit, debug, or extend
- PoB calculates EVERYTHING — we only need specific parts

**Benefits of our own Rust engine:**
- We own it — fix bugs same day, not waiting for PoB PR review
- Rust is faster than Lua for math-heavy code
- We can optimize for our specific use cases (item comparison, what-if)
- We can add calculations PoB doesn't do (crafting probability, combat sim)
- No FFI boundary — pure Rust, easy to test and deploy
- Easier to update per patch (our formulas, our data, our timeline)

### Our Rust Calculator — What It Computes

```rust
/// The Path of AI calculation engine — written in Rust, we own it.
pub struct PathCalcEngine {
    mod_db: ModDatabase,      // mod tier data from poedb
    gem_db: GemDatabase,      // gem scaling data
    tree_db: TreeDatabase,    // passive node stats
    formulas: GameFormulas,   // PoE math formulas (from poewiki)
}

impl PathCalcEngine {
    /// Calculate all stats for a build
    pub fn calculate(&self, build: &BuildData) -> CalcResult {
        // 1. Aggregate all modifier sources
        let mods = ModAggregator::new()
            .add_tree(&build.tree, &self.tree_db)
            .add_items(&build.items, &self.mod_db)
            .add_gems(&build.skills, &self.gem_db)
            .add_config(&build.config)
            .build();

        // 2. Calculate offense
        let offense = OffenseCalc::new(&mods, &self.formulas)
            .base_damage()           // flat damage from gems + items
            .apply_increased()       // sum of all %increased (additive)
            .apply_more()            // chain of more multipliers
            .apply_conversion()      // phys → ele conversion chains
            .apply_crit()            // effective crit chance + multi
            .apply_dot()             // DoT multi, burning, poison, bleed
            .apply_speed()           // attack/cast speed
            .apply_accuracy()        // hit chance (for attacks)
            .apply_enemy_resists()   // penetration, exposure, curses
            .finalize();

        // 3. Calculate defense
        let defense = DefenseCalc::new(&mods, &self.formulas)
            .life_pool()             // base life × (1 + %inc life)
            .energy_shield()         // flat ES × (1 + %inc ES)
            .armour_reduction()      // PoE armour formula vs ref hit
            .evasion_chance()        // entropy-based evasion
            .block_chance()          // attack + spell block
            .resistances()           // ele + chaos, with overcap
            .max_resistances()       // above-75 sources
            .recovery()              // regen, leech, on-hit, flasks
            .guard_skills()          // Molten Shell, Steelskin uptime
            .ailment_immunity()      // freeze, shock, ignite, bleed, etc.
            .effective_hp()          // life × mitigation layers
            .finalize();

        CalcResult { offense, defense }
    }

    /// "What if I change X?" — our engine
    pub fn what_if(&self, build: &BuildData, change: &Change) -> CalcDiff {
        let before = self.calculate(build);
        let after = self.calculate(&build.apply(change));
        CalcDiff::compare(before, after)
    }
}
```

### The PoE Math Formulas (Our Implementation)

These are the core PoE formulas we implement in Rust. All are well-documented
on poewiki.net and verified against PoB's output.

```rust
pub struct GameFormulas;

impl GameFormulas {
    /// Armour physical damage reduction
    /// Source: poewiki.net/wiki/Armour
    /// Formula: reduction = Armour / (Armour + 5 × Damage)
    pub fn phys_reduction(armour: f64, damage: f64) -> f64 {
        if armour <= 0.0 || damage <= 0.0 { return 0.0; }
        (armour / (armour + 5.0 * damage) * 100.0).min(90.0)
    }

    /// Evasion chance (entropy-based)
    /// Source: poewiki.net/wiki/Evasion
    pub fn evasion_chance(evasion: f64, accuracy: f64) -> f64 {
        let chance = 1.0 - (accuracy / (accuracy + (evasion / 4.0).powf(0.8)));
        chance.max(0.05).min(0.95) // 5% floor, 95% cap
    }

    /// Effective crit chance
    /// Source: poewiki.net/wiki/Critical_strike
    ///
    /// PoE crit is a TWO-ROLL system:
    ///   1. Roll accuracy (can you hit?) — if miss, no crit possible
    ///   2. Roll crit chance (is this hit a crit?)
    ///   effective_crit = crit_chance × accuracy_factor
    ///
    /// Crit chance itself is: base × (1 + Σ increased_crit / 100)
    /// Capped at 100% AFTER all modifiers (not before accuracy)
    /// Diamond Flask: "lucky" crits = roll twice, take higher
    pub fn effective_crit(
        base_crit: f64,
        increased_crit: f64,
        hit_chance: f64,     // 0.0-1.0 (from accuracy calc)
        is_lucky: bool,      // Diamond Flask
    ) -> f64 {
        let raw_crit = (base_crit * (1.0 + increased_crit / 100.0) / 100.0).min(1.0);
        let crit_with_accuracy = raw_crit * hit_chance;
        if is_lucky {
            // Lucky: 1 - (1 - chance)^2 = probability of at least one success in 2 rolls
            1.0 - (1.0 - crit_with_accuracy).powi(2)
        } else {
            crit_with_accuracy
        }
    }

    /// DPS chain: base × (1 + Σincreased) × Πmore × dot_multi
    pub fn dot_dps(
        base: f64,
        increased_sum: f64,  // sum of all %increased (additive)
        more_chain: &[f64],  // each more multiplier
        dot_multi: f64,      // damage over time multiplier
    ) -> f64 {
        let mut dps = base * (1.0 + increased_sum / 100.0);
        for m in more_chain {
            dps *= 1.0 + m / 100.0;
        }
        dps * (1.0 + dot_multi / 100.0)
    }

    /// Life calculation: base_life × (1 + %increased_life / 100) + flat_life
    pub fn max_life(base: f64, flat_added: f64, percent_increased: f64) -> f64 {
        (base + flat_added) * (1.0 + percent_increased / 100.0)
    }

    /// Mana reservation
    /// Source: poewiki.net/wiki/Mana#Mana_reservation
    ///
    /// PoE reservation formula:
    ///   reserved = base_percent × (1 - Σ reduced_reservation / 100)
    ///                            × Π(1 - less_reservation / 100)
    ///
    /// "Reduced mana reservation" is ADDITIVE (sum all sources)
    /// "Mana reservation efficiency" uses: cost / (1 + efficiency_bonus)
    ///
    /// Example: 50% aura, 30% reduced reservation, 20% efficiency
    ///   = 50% × (1 - 0.30) × (1 / (1 + 0.20))
    ///   = 50% × 0.70 × 0.833
    ///   = 29.2% reserved
    pub fn mana_reserved(
        total_mana: f64,
        base_percent: f64,
        reduced_reservation: f64,  // sum of all "reduced mana reservation" (additive)
        efficiency_bonus: f64,     // "increased mana reservation efficiency" (0.0 = none)
    ) -> f64 {
        let after_reduced = base_percent * (1.0 - reduced_reservation / 100.0);
        let after_efficiency = after_reduced / (1.0 + efficiency_bonus / 100.0);
        total_mana * (after_efficiency / 100.0).max(0.0)
    }

    /// Damage conversion chain
    /// PoE conversion order: phys → lightning → cold → fire → chaos
    /// Multiple sources of same conversion are ADDITIVE (capped at 100%)
    /// Converted damage inherits ALL modifiers of both source and result types
    pub fn apply_conversion(
        base_phys: f64, 
        phys_to_lightning: f64, // 0.0-1.0
        phys_to_cold: f64,
        phys_to_fire: f64,
        lightning_to_cold: f64,
        cold_to_fire: f64,
    ) -> DamageByType {
        // Cap total conversion at 100%
        let total_phys_conv = (phys_to_lightning + phys_to_cold + phys_to_fire).min(1.0);
        let remaining_phys = base_phys * (1.0 - total_phys_conv);
        
        let as_lightning = base_phys * phys_to_lightning;
        let as_cold_from_phys = base_phys * phys_to_cold;
        let as_fire_from_phys = base_phys * phys_to_fire;
        
        // Second stage: lightning → cold
        let cold_from_lightning = as_lightning * lightning_to_cold;
        let remaining_lightning = as_lightning * (1.0 - lightning_to_cold);
        
        // Third stage: cold → fire
        let total_cold = as_cold_from_phys + cold_from_lightning;
        let fire_from_cold = total_cold * cold_to_fire;
        let remaining_cold = total_cold * (1.0 - cold_to_fire);
        
        let total_fire = as_fire_from_phys + fire_from_cold;
        
        DamageByType {
            physical: remaining_phys,
            lightning: remaining_lightning,
            cold: remaining_cold,
            fire: total_fire,
            chaos: 0.0, // chaos conversion is one-way terminal
        }
    }

    /// Guard skill effective absorption
    /// Molten Shell: absorbs 20% of armour as damage (75% of hit)
    /// Steelskin: absorbs flat amount
    /// Immortal Call: % phys reduction for duration
    /// Guard skill effective absorption
    /// Source: poewiki.net/wiki/Molten_Shell, /wiki/Steelskin, /wiki/Immortal_Call
    ///
    /// Guard skills absorb a PERCENTAGE of each incoming hit,
    /// up to a MAXIMUM total absorbed (the "shield" amount).
    ///
    /// Molten Shell: absorbs 75% of each hit, shield = 20% of armour
    ///   When hit for 5000 damage: absorbs min(5000 × 0.75, remaining_shield)
    ///   If shield is 7000 (from 35000 armour): absorbs 3750, you take 1250
    ///
    /// Steelskin: absorbs 70% of each hit, shield = flat amount per gem level
    /// Immortal Call: reduces phys damage taken by 25-35% for duration
    pub fn guard_absorption(guard_type: GuardSkill, armour: f64, level: u8) -> GuardResult {
        match guard_type {
            GuardSkill::MoltenShell => {
                let shield = armour * 0.20; // shield pool = 20% of armour
                // Each hit: absorb = min(hit × 0.75, remaining_shield)
                // Shield depletes as it absorbs damage
                GuardResult {
                    shield_amount: shield,    // total pool (e.g., 7000 from 35K armour)
                    absorb_percent: 0.75,     // absorbs 75% of each hit
                    duration_ms: 3000,
                }
            },
            GuardSkill::Steelskin => {
                let shield = 500.0 + level as f64 * 100.0; // flat shield per gem level
                GuardResult {
                    shield_amount: shield,
                    absorb_percent: 0.70,     // absorbs 70% of each hit
                    duration_ms: 1500,
                }
            },
            GuardSkill::ImmortalCall => {
                // Immortal Call doesn't have a shield pool — it's a flat reduction
                let phys_reduction = 25.0 + level as f64 * 0.5; // 25-35% at levels 1-20
                GuardResult {
                    shield_amount: 0.0,       // no shield pool
                    absorb_percent: phys_reduction / 100.0, // flat reduction
                    duration_ms: 1000 + level as u64 * 40,
                }
            },
        }
    }

    /// Conditional modifier check
    /// Many PoE mods only apply under specific conditions
    pub fn applies_condition(condition: &str, build_state: &BuildState) -> bool {
        match condition {
            "while_stationary" => !build_state.is_moving,
            "while_moving" => build_state.is_moving,
            "on_full_life" => build_state.life_percent >= 1.0,
            "on_low_life" => build_state.life_percent <= 0.35,
            "while_leeching" => build_state.is_leeching,
            "with_shield" => build_state.has_shield,
            "dual_wielding" => build_state.is_dual_wielding,
            "while_fortified" => build_state.has_fortify,
            "enemy_is_rare_or_unique" => build_state.enemy_is_boss,
            _ => true, // unknown conditions default to active (conservative)
        }
    }
}
```

### Edge Cases the Calculator Must Handle

These are common sources of calculation errors:

```
1. CONVERSION + DAMAGE INHERITANCE
   Physical damage converted to fire STILL benefits from "+% physical damage"
   AND "+% fire damage". Both apply. Many calculators get this wrong.

2. MORE vs INCREASED
   Increased is additive: 100% + 50% + 30% = 180% total → ×2.8
   More is multiplicative: ×1.5 × 1.3 = ×1.95
   Getting these mixed up = massive DPS error.

3. LOCAL vs GLOBAL MODS
   "+% increased Physical Damage" on a weapon is LOCAL (only that weapon)
   "+% increased Physical Damage" on a ring is GLOBAL (all damage)
   Must check mod source to determine scope.

4. MINION vs PLAYER STATS
   "Minions deal 50% increased damage" does NOT apply to player
   "You and your minions deal..." applies to BOTH
   Need separate modifier pools for minions.

5. RESERVATION EFFICIENCY
   "+10% reduced mana reservation" and "+20% increased reservation efficiency"
   are DIFFERENT mechanics with different math.
   Reservation efficiency: base_cost × (1 / (1 + efficiency_bonus))

6. DOT DOUBLE DIPPING (removed in 3.0, but legacy)
   Pre-3.0: ignite scaled from both hit damage mods AND dot mods
   Post-3.0: ignite only scales from dot-specific mods
   Must use post-3.0 rules.

7. ENEMY RESISTANCE REDUCTION
   Penetration only applies to HITS, not DoTs
   Exposure reduces enemy resistance (different mechanic)
   -resistance (like nearby enemies) is a third mechanic
   Each works differently. Don't combine them wrong.
```

### PoB Lua as Verification (Optional)

PoB Lua is NOT required — it's an optional verification layer.

```
Settings → Calculation Engine:

  ◉ Path of AI Calculator (default)
    Our Rust engine. Fast, accurate, always available.
    We maintain and update it ourselves.

  ☐ Enable PoB verification
    Also run PoB's Lua calc engine for comparison.
    Shows both results side-by-side.
    Useful for verifying our calculations match PoB.

When PoB verification is enabled:
  ┌──────────────────────────────────────────────┐
  │  DPS Calculation                              │
  │                                               │
  │  Path of AI:  2,841,057  ← primary           │
  │  PoB Lua:     2,841,057  ← verification      │
  │  Status:      ✓ MATCH                         │
  │                                               │
  │  (if different:)                              │
  │  Path of AI:  2,912,000  ← primary           │
  │  PoB Lua:     2,841,057  ← verification      │
  │  Diff:        +2.5%                           │
  │  Note: Difference may be due to rounding or   │
  │  different handling of conditional modifiers.  │
  │  [Report discrepancy]                         │
  └──────────────────────────────────────────────┘
```

### Three Calculation Tiers

```
TIER 1: INSTANT ESTIMATE (<10ms)
  For browsing and quick comparisons.
  Uses pre-computed impact tables per mod type per archetype.
  Shows "~estimated" label. Accuracy: 85-95%.

  Example: "+15% fire DoT multi on RF at 180% total"
           = (195/180) - 1 = +8.3% MORE ← instant math

TIER 2: OUR RUST CALC (<100ms) — DEFAULT
  For all suggestions, analysis, and detailed views.
  Runs our full Rust calculation engine.
  Shows result without label (this IS the primary engine).
  Accuracy: 99%+ (validated against PoB test suite).

TIER 3: POB LUA VERIFICATION (<500ms) — OPTIONAL
  For users who want to cross-check against PoB.
  Runs PoB's Lua engine via LuaJIT FFI.
  Shows "PoB: X" alongside our result.
  Accuracy: matches PoB exactly (it IS PoB).

USER EXPERIENCE:
  1. User sees suggestion with DPS from our Rust calc (default)
  2. If PoB verification enabled → also shows PoB number
  3. If numbers match → "✓ verified by PoB"
  4. If numbers differ → show both, let user choose
  5. User clicks "Apply to PoB" → writes XML
```

### Complete Calculation Coverage

Every PoE mechanic that affects DPS or survivability must be in our calculator.
Here are the formulas grouped by category. Source for all: poewiki.net.

#### Ailment Thresholds & Damage

```
FREEZE: Occurs when hit damage ≥ 5% of enemy max life
  Duration = 60ms × (damage / (5% × enemy_life))^0.7
  Capped at 5 seconds. Reduced by freeze reduction on enemy.
  Immune if enemy has "Cannot be Frozen" mod.

SHOCK: Occurs when hit damage ≥ 1% of enemy max life
  Effect = 5% + (damage / enemy_life) × 150%
  Capped at 50% (increases damage taken by up to 50%)
  Boss penalty: already has high life, so shock effect is lower.

IGNITE: Requires fire damage + ignite chance
  Base DPS = base_fire_hit × 0.9 × (1 + fire_dot_multi)
  Duration: 4 seconds (base)
  Only strongest ignite applies (no stacking)

POISON: Requires physical OR chaos damage + poison chance
  Base DPS = (phys + chaos) × 0.3 × (1 + dot_multi)
  Duration: 2 seconds (base). STACKS (all poisons deal damage)
  
BLEED: Requires physical damage + bleed chance
  Base DPS = phys × 0.7 × (1 + dot_multi)
  Duration: 5 seconds (base)
  Moving target: ×2.1 multiplier (Crimson Dance changes this)
```

#### Totem / Trap / Mine / Trigger DPS

```
TOTEM DPS:
  Totem uses player stats but has its OWN cast speed.
  Total DPS = single_totem_dps × totem_count × totem_uptime
  Totem uptime = 1.0 - (summon_time / (totem_duration + summon_time))
  Totem count: base 1, increased by tree nodes + items.

TRAP DPS:
  Trap throws per second = throw_rate (affected by trap speed)
  Single trap DPS = trap_damage × crit_multi_if_applicable
  Total DPS = traps_per_second × trap_damage × (1 + increased) × Π(more)
  Trap cooldown: some traps have cooldown → reduces effective DPS.

MINE DPS:
  Similar to traps but detonation chain matters.
  Detonation sequence: first mine → chain → each successive mine gets aura bonus.
  More mines in sequence = more damage per mine (up to cap).

CAST ON CRIT (CoC):
  Trigger rate = attacks_per_second × crit_chance × hit_chance
  BUT: 150ms internal cooldown on CoC → cap at 6.67 triggers/sec
  Effective spell DPS = spell_damage × min(trigger_rate, 1/0.15)
  Key: stack attack speed + crit to reach cooldown cap, then stack spell damage.

CAST WHILE CHANNELING (CWC):
  Trigger every 350ms while channeling → 2.86 triggers/sec
  Channel must not be interrupted.
  Effective DPS = spell_damage × 2.86

SPELLSLINGER:
  Triggers socketed spell when you attack with wand.
  Reserves mana (like aura). DPS = attack_rate × spell_damage.
  Limited by mana reservation (can't reserve too many skills).
```

#### Minion DPS Pipeline

```
Minion DPS is calculated separately from player DPS.
Minions have their OWN modifier pool:

1. Minion base damage (from gem level → monster level → base stats)
   - Spectre: depends on specific monster type + level
   - Zombie: depends on zombie gem level
   - Golem: depends on golem type + gem level
   - SRS (Summon Raging Spirit): depends on gem level

2. Minion modifiers:
   "Minions deal X% increased damage" → goes to MINION pool (not player)
   "You and your Minions deal X%" → goes to BOTH pools
   "+X to level of Minion Gems" → increases base damage (like +gem level)

3. Minion support gems:
   Minion Damage Support, Melee Physical Damage, etc.
   Each adds a MORE multiplier to the minion's DPS chain.

4. Total minion DPS:
   = Σ (per_minion_dps × minion_count)
   = Σ (base × (1 + Σincreased_minion) × Π(more_supports) × count)

IMPORTANT: Player "%increased fire damage" does NOT affect minion fire damage
UNLESS it says "you and your minions" or "minions deal % of your damage".
```

#### Impale & Wither Stacks

```
IMPALE:
  Each hit that impales stores 10% of the physical damage dealt.
  Next 5 hits against target deal that stored damage again.
  Effective DPS bonus = impale_chance × 0.10 × 5 = +50% phys DPS at 100% impale
  With impale effect modifiers: bonus = chance × (0.10 × (1 + effect)) × 5
  "Impale Effect" increases the 10% stored → huge DPS for physical builds.

WITHER STACKS:
  Each stack = 6% increased chaos damage taken by enemy (was 4%, changed)
  Max 15 stacks = 90% increased chaos damage taken
  Applied by: Wither skill (totem or self-cast), Withering Step, etc.
  Uptime depends on application method:
    Wither Totem: ~15 stacks in 3 seconds, maintained as long as totem lives
    Withering Step: instant max stacks but short duration

CALCULATION:
  Chaos DPS with Wither = base_chaos_dps × (1 + 0.06 × wither_stacks)
```

#### Charge Generation & Uptime

```
FRENZY CHARGES:
  +4% MORE attack/cast speed per charge (multiplicative)
  +4% MORE damage per charge
  Generation: on kill, on crit, Frenzy skill, etc.
  Duration: 10 seconds base (refreshed on gain)
  Uptime: boss fights ≈ depends on generation method
    "On kill" → 100% in maps, 0% on bosses (no kills)
    "On crit" → depends on crit chance + hit rate
    
POWER CHARGES:
  +40% increased critical strike chance per charge
  Generation: on kill, Power Charge on Crit support, etc.
  Same duration/uptime rules as frenzy.

ENDURANCE CHARGES:
  +4% physical damage reduction per charge
  +4% to all elemental resistances per charge
  Generation: Enduring Cry, on kill, CWDT Immortal Call consumes them.

UPTIME ESTIMATION:
  For map clearing: assume max charges (kills are constant)
  For boss fights: 
    "on kill" source = 0 charges (no kills)
    "on crit" source = ~80% uptime if crit chance > 50%
    "on hit" source = ~100% uptime if hit rate > 2/sec
    skill-generated = depends on cooldown + cast time
```

#### Curse Effectiveness on Bosses

```
CURSE EFFECT ON BOSSES:
  Normal enemies: 100% curse effectiveness
  Map bosses: 33% less curse effectiveness → curses apply at 67%
  Uber bosses: 33% less curse effectiveness (same as map bosses)
  Pinnacle bosses: some have additional curse reduction

FORMULA:
  effective_curse_value = base_value × (1 + increased_curse_effect) × boss_penalty
  boss_penalty = 0.67 for map/pinnacle bosses

EXAMPLE:
  Flammability: -44% fire resistance (gem level 20)
  On normal enemy: -44%
  On map boss: -44% × 0.67 = -29.5%
  With 50% increased curse effect: -44% × 1.5 × 0.67 = -44.2%

CURSE LIMIT:
  Default: 1 curse
  Additional curse from: tree nodes, Whispers of Doom, items
  If at limit: new curse replaces oldest
```

#### Flask Effect & Uptime

```
FLASK CHARGES:
  Gain: 1 charge per monster killed (base), +charge on crit for some
  Consume: use_cost per activation (varies by flask type)
  Max charges: varies (e.g., 40 for life flask, 60 for utility)
  
UPTIME CALCULATION:
  In maps (constant kills): ~100% for utility flasks
  On bosses: depends on:
    charges_per_second = monsters_killed × charge_gain_per_kill
    uses_per_flask = max_charges / use_cost
    uptime = flask_duration × (uses_per_flask / fight_duration)

FLASK EFFECTIVENESS:
  "%increased Flask Effect" → increases ALL flask stats
  Ruby Flask base: +50% fire resistance
  With 30% effect: +50% × 1.3 = +65% fire resistance
  
  Life flask recovery: base_recovery × (1 + increased_flask_effect)
```

#### Enemy Resistance Calculation (Complete)

```
ORDER OF OPERATIONS for enemy resistance:

1. Start with enemy base resistance
   Normal enemy: 0% all
   Act boss: varies (30-40%)
   Map boss: varies (30-40%)
   Shaper/Elder/Maven: 40% ele, 25% chaos

2. Apply FLAT resistance reduction (curses, exposure)
   Flammability: -44% fire res (×0.67 on bosses)
   Fire Exposure: -25% fire res (from skill/item)
   "Nearby enemies have -9% fire res": -9% flat
   These are ADDITIVE with each other.

3. Calculate: effective_res = base - Σ(flat_reductions)
   Example: 40% - 29.5% (curse) - 25% (exposure) - 9% (nearby) = -23.5%

4. Apply penetration (HITS ONLY — does NOT apply to DoTs)
   Fire Penetration Support: 37% at level 20
   "Ignores X% of enemy fire res"
   Final effective res for HITS = effective_res - penetration
   Example: -23.5% - 37% = -60.5% → enemy takes 160.5% fire HIT damage

5. For DoTs: penetration does NOT apply
   RF DPS uses effective_res WITHOUT penetration
   Example: -23.5% → enemy takes 123.5% fire DoT damage

BOTTOM CAP: Enemy resistance cannot go below -200%
```

#### Complete Offense Calculation Order (from PoB CalcOffence.lua)

Our calculator MUST follow this exact order. Source: PoB community fork.

```
STEP 1: FLAT DAMAGE (Base + Added)
  base_damage = weapon_base + skill_base + Σ(added_flat_damage_from_gear)
  
  Sources of added flat:
    - "Adds X-Y fire damage to attacks" (from rings, amulets, abyss jewels)
    - "Adds X-Y fire damage to spells" (from wands, sceptres)
    - Auras: Anger/Wrath/Hatred add flat damage
    - NOTE: added damage scales with damage effectiveness of the skill

STEP 2: DAMAGE CONVERSION (recursive chain)
  phys → lightning → cold → fire → chaos (one direction only)
  Multiple conversion sources are ADDITIVE (capped at 100% total per source type)
  Converted damage inherits mods from BOTH source and destination type

STEP 3: GAINED AS EXTRA DAMAGE (not conversion — it's additional)
  "Gain 20% of Physical Damage as Extra Fire Damage"
  This ADDS damage, doesn't convert. You keep the original + gain extra.
  Different from conversion! Both the original and extra are then scaled.

STEP 4: INCREASED / REDUCED (additive pool)
  damage × (1 + Σ(all_applicable_increased%) / 100)
  All "increased" mods for applicable types are summed, then applied once.
  Example: 420% total increased fire damage → ×5.2

STEP 5: MORE / LESS (multiplicative chain)
  damage × Π(1 + more_value / 100)
  Each "more" or "less" is a separate multiplier.
  Example: 59% more × 34% more × 29% more = ×1.59 × ×1.34 × ×1.29

STEP 6: CRITICAL STRIKE
  effective_crit_chance = base_crit × (1 + Σ increased_crit / 100)
  crit_multiplier = 150% base + Σ additional_multi
  damage_with_crit = damage × (1 + effective_crit × (crit_multi - 1))
  NOTE: Diamond Flask makes crit "lucky" (roll twice, take higher)

STEP 7: ACCURACY / HIT CHANCE (attacks only)
  hit_chance = attacker_accuracy / (attacker_accuracy + (defender_evasion/4)^0.8)
  Floor: 5%, Cap: 100%
  For spells: always 100% hit (no accuracy check)

STEP 8: DOUBLE / TRIPLE DAMAGE
  If "X% chance to deal double damage": 
    effective_multi = 1 + (chance/100) × 1.0  (double = 2×, so +100% when it procs)
  Some sources grant triple damage (rare).

STEP 9: PROJECTILE MECHANICS
  Projectile count: base + additional (GMP, LMP, Dying Sun, etc.)
  Shotgunning: if multiple projectiles can hit same target → ×projectile_count
  Fork/Chain/Pierce: affects clear, not single-target DPS usually
  Point Blank: +30% more at close range, -30% less at far

STEP 10: ATTACK/CAST SPEED
  hits_per_second = base_speed × (1 + Σ increased_speed / 100) × Π(more_speed)
  For attacks: affected by weapon speed + local attack speed
  For spells: base cast time × modifiers
  
STEP 11: DOT MULTIPLIER (for DoT skills)
  Applied AFTER all other scaling.
  dot_dps = hit_damage × dot_multiplier
  DoT multi is its OWN multiplier category (not increased, not more)

STEP 12: SPECIAL MECHANICS
  Unleash (Seals): stored seal count × damage per seal (burst)
  Spell Echo / Multistrike: repeats with damage scaling per repeat
  Cooldown recovery: affects trigger rate (CoC, CWC)
  Warcry power: based on nearby enemy count
```

#### Complete Defense Calculation Order (from PoB CalcDefence.lua)

```
STEP 1: RESISTANCES
  fire_res = Σ(all_fire_res_sources), capped at max_fire_res (default 75%)
  Same for cold, lightning, chaos
  Max res can be increased by: Rise of Phoenix, tree nodes, etc.
  Overcap = current - cap (buffer for -res curses)

STEP 2: EVASION / ARMOUR (with keystone conversions)
  If Iron Reflexes: evasion → armour (additive)
  Armour reduction = armour / (armour + 5 × incoming_damage)
  Evasion uses entropy system (not pure RNG)

STEP 3: ENERGY SHIELD
  ES = base_es × (1 + Σ increased_es / 100) × Π(more_es)
  ES recharge: starts after 2 seconds of not taking damage
  ES recharge rate: 33% of max ES per second (default)
  If Eldritch Battery: ES protects mana instead of life
  If CI (Chaos Inoculation): max life = 1, immune to chaos, ES is your HP

STEP 4: WARD
  Ward absorbs damage BEFORE ES and life
  Ward does NOT recharge — it refills between encounters
  Ward is consumed on hit (reduced by damage amount)

STEP 5: BLOCK / SPELL BLOCK / SPELL SUPPRESSION
  Block: X% chance to block 100% of hit damage
  Spell Block: same but for spells
  Spell Suppression: X% chance to reduce spell damage by 50%
  These are checked BEFORE damage is applied

STEP 6: MIND OVER MATTER (MoM)
  "X% of damage taken from mana before life"
  Default: 30% (from keystone)
  Can be increased by gear/tree
  Damage goes to mana first, overflow to life

STEP 7: DAMAGE TAKEN AS ANOTHER TYPE
  Lightning Coil: "30% of Physical Damage taken as Lightning Damage"
  Taste of Hate: "20% of Physical Damage taken as Cold Damage"
  The shifted damage is mitigated by the NEW type's resistance
  Original type's armour does NOT apply to shifted portion

STEP 8: DAMAGE REDUCTION MODIFIERS
  "X% reduced damage taken" — additive with other reduced
  "X% less damage taken" — multiplicative
  Fortify: "20% less damage taken from hits"
  Endurance charges: 4% phys reduction per charge

STEP 9: GUARD SKILL ABSORPTION
  Molten Shell: absorbs 75% of hit, up to 20% of armour value
  Steelskin: absorbs 70% of hit, up to flat amount
  Applied AFTER resistance, BEFORE life/ES pool

STEP 10: LIFE POOL
  life = (base_life_per_level × level + Σ flat_life) × (1 + Σ%increased_life/100)
  Recovery: regen, leech, life gain on hit, flasks
  Leech cap: 20% of max life per second (base, Vaal Pact doubles)

STEP 11: EFFECTIVE HP (vs specific hit)
  ehp = life_pool / (1 - total_mitigation)
  Where total_mitigation combines: resist, armour, block chance, guard, etc.
  Calculated per damage type (phys EHP ≠ ele EHP ≠ chaos EHP)
```

#### Special Modifier Transformations (from PoB)

PoB handles these special modifier conversions. We must too:
```
Crown of Eyes:     Spell damage → Attack damage (at 150% value)
Battlemage:        Weapon damage → Spell damage bonus
Spiritual Aid:     Minion damage → Player damage
Iron Will:         Strength → Spell damage
Iron Grip:         Strength → Projectile attack damage
Rigwald's Curse:   Claw damage → Unarmed damage
Projectile Speed:  → Bow damage / Area damage (some interactions)
Light Radius:      → Accuracy, Area, Damage (Replica Last Resort)
```

These are NOT simple additions — they transform entire modifier categories.
Our calculator needs a "modifier transformation pass" before the main calc.

#### Aura Effect Scaling

```
AURA EFFECT:
  Determination gives: 3000 base armour + X% more armour
  With 50% increased aura effect: 3000 × 1.5 = 4500 base armour

IMPORTANT:
  "Increased aura effect" does NOT increase mana reservation
  It only increases the STATS the aura provides.
  Reservation is a separate calculation.

STACKING:
  Multiple auras stack their effects.
  Determination (armour) + Grace (evasion) + Defiance Banner (armour+evasion)
  Each calculated independently, then summed.
```

#### Passive Tree Calculation & Advice

```
PASSIVE NODE VALUE:
  For each UNALLOCATED node within reach:
    1. Snapshot current build stats
    2. Allocate node (add its modifiers to pool)
    3. Re-run calculator → get new DPS, life, defenses
    4. Diff: ΔLife, ΔDPS, ΔResists, ΔArmour
    5. Score = weighted sum of deltas (using archetype weights)
    6. Cost = points to travel to this node

  Efficiency = Score / Cost
  "This node gives +340 life for 2 points (170 life per point)"

RANKING:
  Sort all reachable nodes by efficiency.
  Top 5 shown as "Next Best Points" in UI.

INEFFICIENT NODE DETECTION:
  For each ALLOCATED node:
    1. Remove node from tree
    2. Re-run calculator → get stats WITHOUT this node
    3. If stat loss < threshold → node is "inefficient"
    4. "Coordination gives +10 dex — you don't need dex → respec"

PATH OPTIMIZATION:
  For each keystone pair you've allocated:
    Try all possible paths between them.
    If shorter path exists → "Respec 3 nodes, save 2 points"
    Score saved points by what they could be spent on.

KEYSTONE ANALYSIS:
  For each keystone NOT allocated:
    Simulate taking it → full recalc
    "Elemental Overload: +0% DPS (you have no crit anyway) → good"
    "Point Blank: +22% DPS at close range, -30% at far → depends"

MASTERY ANALYSIS:
  For each mastery cluster where you have points:
    List available mastery options
    Calc impact of each → rank
    "Fire Mastery: +20% burning > +1 fire exposure for your build"
```

#### Jewel Analysis & Advice

```
REGULAR JEWELS:
  Score = sum of (mod_value × archetype_weight) for all 4 mods
  Compare to "expected jewel score" for this build
  "This jewel is 68/100 — a T1 life + fire DoT multi jewel would be 92/100"

JEWEL SOCKET VALUE:
  For each tree jewel socket:
    Cost = points to travel to socket
    Value = best available jewel score for this build
    Worth it? = Value > Cost × (average_point_value)
    "Socket near Scion costs 3 points — needs jewel worth 3+ points"

CLUSTER JEWELS:
  Evaluate by notable value:
    For each notable on the cluster:
      Simulate taking it → full recalc
      "Burning Bright: +8% DPS"
      "Blowback: +6% DPS"
    Total cluster value = sum of notable values - points spent
    Compare: cluster vs tree nodes of same point cost

TIMELESS JEWELS:
  Seed determines which keystones transform into what.
  For each transformed keystone within jewel radius:
    Simulate the transformed version → calc impact
    "Lethal Pride seed 14832: transforms Unwavering Stance into
     'X% increased fire damage' — worth +4% DPS"

WATCHER'S EYE:
  Based on active auras, rank all possible Watcher's Eye mods:
    For each aura you run:
      List all Watcher's Eye mods for that aura
      Calc impact of each → rank by DPS/defense value
    "Best: Fire DoT Multi while affected by Purity of Fire (+12% DPS)"
    "Best double: Fire DoT + Phys Reduction while Determination"
    Estimate price from poe.ninja for target combo

ANOINTMENT ADVICE:
  For amulet anointment:
    Simulate every anointable notable → full recalc
    Rank by impact
    Show oil cost for top 5
    "Best anoint: Whispers of Doom (2× Golden + Silver) = extra curse slot"
    Compare: anoint value vs pathing to same node on tree
    "Anointing Breath of Flames saves 4 passive points"
```

#### Expanded Conditional Modifiers (Complete List)

```
PLAYER STATE CONDITIONS:
  while_stationary          while_moving
  on_full_life              on_low_life (≤35%)
  on_full_energy_shield     on_low_mana (≤35%)
  while_leeching            while_leeching_energy_shield
  while_fortified           while_focused (Focus skill active)
  while_channelling         while_at_maximum_frenzy_charges
  while_at_maximum_power_charges   while_at_maximum_endurance_charges
  recently_killed            recently_been_hit
  recently_used_a_warcry     recently_used_a_travel_skill
  if_used_a_minion_skill_recently

EQUIPMENT CONDITIONS:
  with_shield               dual_wielding
  wielding_a_staff          wielding_a_sword
  wielding_a_mace           while_unarmed
  with_body_armour          without_body_armour

ENEMY CONDITIONS:
  enemy_is_rare_or_unique   enemy_is_full_life
  enemy_is_low_life         enemy_is_nearby
  enemy_is_chilled          enemy_is_frozen
  enemy_is_shocked          enemy_is_ignited
  enemy_is_bleeding         enemy_is_poisoned
  enemy_is_cursed           enemy_is_maimed
  enemy_is_hindered         enemy_is_intimidated

HANDLING UNCERTAIN CONDITIONS:
  For build planning, assume:
    - "while stationary" = ON (conservative, player stands still for DPS)
    - "recently killed" = ON for mapping, OFF for boss fights
    - "enemy is full life" = ON for first hit, OFF for sustained
    - "on full life" = depends on build (RF = never full life)
  
  Show in UI: "DPS (with conditions: X, Y, Z): 3.2M"
              "DPS (no conditions): 2.4M"
```

#### Fast Estimation Engine (Tier 1 — <10ms)

```
PURPOSE: Instant DPS/life change estimates while browsing items.
NOT used for final suggestions (those use full Rust calc).

DATA STRUCTURE:
  Pre-computed lookup tables per archetype:
  
  impact_tables = {
    "fire_dot": {
      "flat_life":     1.0,   // per point of flat life → +1.0 life
      "percent_life":  64.5,  // per 1% increased life → +64.5 life (at 6453 base)
      "fire_dot_multi": 0.56, // per 1% DoT multi → +0.56% MORE DPS
      "fire_res":      0.0,   // fire res above cap = 0 DPS (but defense value)
      "cold_res":      0.0,
      "movement_speed": 0.0,  // no DPS impact
      "+gem_level":    8.2,   // per +1 gem level → ~8.2% MORE DPS (RF specific)
    },
    "attack_crit": {
      "flat_phys":     12.3,
      "attack_speed":  varies, // depends on current AS
      "crit_chance":   varies, // depends on current crit
      "crit_multi":    varies,
      // ...
    }
  }

CALCULATION:
  estimated_dps_change = Σ(mod_value × impact_table[archetype][mod_type])
  
  FAST — just multiplication and sum, <1ms.
  ACCURATE for simple mods (life, resist, flat damage).
  INACCURATE for interactive mods (crit depends on current crit level).

WHEN TO UPDATE TABLES:
  - On build load (calc tables from current build stats)
  - On major stat change (recalc tables)
  - Tables are build-specific, not global

LABEL: Shows "~estimated" in UI. Never used for final "Apply" decisions.
```

#### Error Handling

```rust
/// Every calculation function returns Result, never panics
pub enum CalcError {
    ModNotFound(String),      // Mod ID not in our database
    DivisionByZero(String),   // 0/0 in some edge case
    IncompatibleVersion,      // PoB file from unsupported version
    MissingGameData(String),  // JSON data file not found/corrupt
    OverflowError(String),    // Number too large (bad mod stacking)
}

/// How to handle each error:
/// ModNotFound → skip mod, log warning, continue calc (don't crash)
/// DivisionByZero → return 0.0, log warning
/// IncompatibleVersion → show user-friendly error, suggest PoB update
/// MissingGameData → fallback to bundled data, show "data may be outdated"
/// OverflowError → cap at MAX_DPS (999,999,999), log warning
```

#### Performance Strategy

```
TARGET: <100ms for full build calculation

HOW TO ACHIEVE:
1. Single-pass modifier aggregation (don't scan items multiple times)
2. Lazy calculation (don't calc defense if only DPS changed)
3. Skip unchanged gems/items (hash-based change detection)
4. Parallel offense + defense calc (tokio::spawn for each)
5. Pre-compute archetype stat weights at build load time

IF CALC TAKES >500ms:
  → Show cached result with "recalculating..." spinner
  → Never block UI

PROFILING:
  Use criterion benchmarks (already in TESTING.md)
  Profile with cargo-flamegraph for hotspot detection
```

#### Formula Versioning & Patch Updates

```
Every formula function has a version comment:
  /// PoE 3.24: Armour formula unchanged since 3.0
  /// Source: poewiki.net/wiki/Armour (verified 2026-04-03)
  pub fn phys_reduction(armour: f64, damage: f64) -> f64 { ... }

PATCH UPDATE PROCESS:
  1. GGG releases patch notes
  2. Check: did any formula change? (usually listed in "Balance Changes")
  3. If yes: update formula + version comment + add regression test
  4. If no: just update data files (mod tiers, gem values, tree)
  5. Run full test suite against PoB to verify
  6. Publish data update via GitHub Releases

FORMULA CHANGE LOG:
  Keep CHANGELOG-FORMULAS.md tracking every formula change per patch.
  Example: "3.25: Armour formula now uses 10 × Damage instead of 5 × Damage"
```

### Building Our Calculator Incrementally

We don't need to implement ALL of PoB's 50,000 lines on day 1.
Build incrementally, validate against PoB at each step:

```
Sprint 1: Core formulas (Week 1-2)
  → Life/ES/Mana calculation
  → Resistance aggregation (flat + percent, with overcap)
  → Armour/Evasion/Block formulas
  → Validate: parse 50 builds, compare life/resists to PoB → must match

Sprint 2: Offense basics (Week 3-4)
  → Base damage + increased + more multiplier chain
  → DoT DPS (burning, bleed, poison)
  → Attack/cast speed
  → Validate: compare DPS for 50 DoT builds → must match ±1%

Sprint 3: Advanced offense (Week 5-6)
  → Crit calculation (chance, multi, effective crit)
  → Hit chance / accuracy
  → Damage conversion chains
  → Penetration / exposure
  → Validate: compare DPS for 50 hit builds → must match ±2%

Sprint 4: Advanced defense (Week 7-8)
  → Guard skill uptime (Molten Shell, Steelskin)
  → Recovery (regen, leech, on-hit)
  → Ailment immunity detection
  → EHP against specific boss hits
  → Validate: compare EHP for 50 builds → must match ±5%

Sprint 5: Edge cases (Week 9-10)
  → Minion builds (spectre, zombie, golem DPS)
  → Totem/trap/mine DPS
  → Trigger builds (CoC, CWC)
  → Aura effect stacking
  → Validate: all archetypes covered

At each sprint:
  - Run our calc on 50 builds from poe.ninja
  - Compare to PoB Lua output
  - Fix any discrepancies > 1%
  - Add regression test for every fix
```

### Why This Is Better Than PoB-Only

| Aspect | PoB Lua Only | Our Rust + PoB Backup |
|---|---|---|
| If PoB stops updating | **We're dead** | We continue independently |
| If PoB has a bug | Wait for community fix | Fix ourselves same day |
| Performance | Lua FFI overhead (~500ms) | Native Rust (~100ms) |
| Testing | Can't unit test Lua easily | Full Rust test suite |
| Extending | Must learn PoB's codebase | Add features in our code |
| New PoE mechanics | Wait for PoB to add | Add ourselves from patch notes |
| Build size | +LuaJIT runtime (~2MB) | Optional (0 if disabled) |
| Debugging | Lua stack traces, FFI issues | Rust error handling |
| User trust | "Same as PoB" | "Our calc, verified by PoB ✓" |

---

## 3. ENGINE 2: THE KNOWLEDGE BASE

### What It Contains

Every piece of PoE game knowledge that can be stored as structured data:

```
game-data/
  mods/
    mod-tiers.json          # Every mod, every tier, every value range
    mod-weights.json        # Spawn weights per base+ilvl+influence (from poedb)
    mod-tags.json           # Mod tag associations (fire, life, caster, etc.)
    fossil-multipliers.json # Fossil tag multiplier table
    essence-mods.json       # Guaranteed mods per essence tier
    harvest-crafts.json     # Harvest craft outcome pools
    eldritch-tiers.json     # Exarch/Eater implicit tiers
    influenced-mods.json    # Shaper/Elder/Conqueror exclusive mods
    veiled-mods.json        # Veiled mod pool per Syndicate member
    bench-crafts.json       # All benchcraft options with costs

  gems/
    active-gems.json        # All gems with tags, scaling, damage effectiveness
    support-gems.json       # Support gem applicability rules
    gem-interactions.json   # Special interactions (CoC, CWC, etc.)

  tree/
    passive-tree.json       # All nodes, connections, masteries
    cluster-jewels.json     # Cluster jewel notable pools
    timeless-jewels.json    # Keystone transformations per jewel type

  bosses/
    boss-attacks.json       # Every boss attack: damage, type, speed, tells
    boss-phases.json        # Phase transitions, immunity windows
    monster-scaling.json    # HP/damage per area level

  maps/
    map-mods.json           # All map mod effects
    map-layouts.json        # Layout ratings per map (linear, open, etc.)
    atlas-nodes.json        # Atlas passive tree data

  economy/
    div-card-locations.json # Drop locations per card
    vendor-recipes.json     # All vendor recipes
    currency-ratios.json    # Base exchange rates (updated from poe.ninja)

  builds/
    archetype-weights.json  # Stat weights per build archetype
    common-setups.json      # Popular gem setups per skill
    progression-checklist.json # Per-phase build requirements
```

### How Knowledge Queries Work

```
User: "What fossil should I use for my helmet?"

Step 1: CLASSIFY QUERY
  Intent: crafting_advice
  Slot: helmet
  Build context: RF Inquisitor (fire DoT)

Step 2: LOOKUP DATA
  → Load helmet mod pool for Royal Burgonet ilvl 84
  → Load desired mods: T1 life, fire res, -fire res nearby
  → Load fossil multiplier tables
  → Calculate: which fossil combo maximizes desired mod probability?

Step 3: CALCULATE PROBABILITY
  Without fossils:
    P(T1 life AND fire res AND -fire res) = 0.3% per chaos
    Expected cost: ~330 chaos ≈ 4 divine

  Pristine + Scorched:
    Pristine: life weight ×10, blocks ES/evasion mods
    Scorched: fire weight ×10, blocks cold mods
    P(target combo) = 4.2% per resonator
    Expected cost: ~24 resonators ≈ 2.5 divine

Step 4: GENERATE RESPONSE (template)
  "For your helmet, use Pristine + Scorched fossil combo
   in a 2-socket resonator. This blocks bad mods and boosts
   life + fire weights. Expected cost: ~2.5 divine (24 attempts).
   Compared to chaos spam at ~4 divine, fossils save ~40%."

NO AI MODEL NEEDED — pure data lookup + probability math.
```

### Crafting Probability Engine

This is the Craft of Exile equivalent built into our app:

```rust
/// Calculate probability of hitting target mods with given method
pub fn craft_probability(
    base: &BaseType,
    ilvl: u8,
    target_mods: &[ModRequirement],
    method: CraftMethod,       // Chaos, Fossil, Essence, Harvest, etc.
    mod_weights: &ModWeightDB,
) -> CraftResult {
    // 1. Get full mod pool for this base + ilvl + influence
    let pool = mod_weights.get_pool(base, ilvl);

    // 2. Apply method modifiers
    let modified_pool = match method {
        CraftMethod::Fossil(fossils) => {
            pool.apply_fossil_multipliers(&fossils)
            // Pristine: life tags ×10, ES tags ×0
            // Scorched: fire tags ×10, cold tags ×0
        },
        CraftMethod::Essence(essence) => {
            pool.guarantee_mod(essence.guaranteed_mod())
            // One mod is locked, rest rolled normally
        },
        CraftMethod::Chaos => pool, // unmodified
        // ... other methods
    };

    // 3. Calculate probability of hitting ALL target mods
    let mut total_weight = modified_pool.total_prefix_weight();
    let mut prob = 1.0;

    for target in target_mods {
        let mod_weight = modified_pool.weight_for(target.mod_id);
        let tier_weight = modified_pool.tier_weight(target.mod_id, target.min_tier);
        prob *= tier_weight as f64 / total_weight as f64;
        total_weight -= mod_weight; // mod can't roll twice
    }

    // 4. Account for number of affixes (3 prefix + 3 suffix)
    // Probability increases with more rolls
    let attempts_prob = 1.0 - (1.0 - prob).powi(6); // 6 mod slots

    CraftResult {
        probability_per_attempt: attempts_prob,
        expected_attempts: (1.0 / attempts_prob).ceil() as u32,
        expected_cost: calculate_cost(method, expected_attempts),
        comparison: compare_to_market(target_mods, market_price),
    }
}
```

---

## 4. ENGINE 3: THE LANGUAGE MODEL (Optional)

### When It's Actually Needed

Only ~3% of user queries need creative reasoning:

| Query Type | Engine | Example |
|-----------|--------|---------|
| "What's my DPS?" | Calculator | Exact PoB calc |
| "Best helmet for RF?" | Knowledge | Mod pool lookup + scoring |
| "Why am I dying?" | Calculator + Knowledge | Check defenses + boss data |
| "Craft +1 fire amulet?" | Knowledge | Mod weights + probability |
| **"Design me a league starter"** | **Language Model** | Creative, open-ended |
| **"Explain damage conversion"** | **Language Model** | Educational, nuanced |
| **"Compare RF vs Cold DoT"** | **Language Model** | Multi-factor comparison |

### The Seer's Language Model

For the 3% that needs it, we have two options:

**Option A: Fine-tuned Small Model (offline, free)**
```
Model: Phi-3 Mini 3.8B (quantized Q4, ~2.3GB)
Fine-tuned on: 50K PoE Q&A pairs from builds + forums
Runs on: CPU (15-30 tok/s) or GPU (50-100 tok/s)
Quality: Good for PoE-specific questions, weak on novel reasoning
Cost: $0 per query
```

**Option B: Cloud API (online, paid, better)**
```
Model: Claude Sonnet 4 / GPT-4o
Fed with: Build context + knowledge base excerpts (RAG)
Quality: Excellent reasoning, may hallucinate PoE details
Cost: ~$0.01-0.05 per query
```

**Strategy: Use Calculator + KB as default (97% of queries).
Cloud API only when user asks creative/open-ended questions.**

### Context Injection — How We Feed Build Data to Cloud AI

When a query goes to Claude/GPT, we DON'T just send the raw question.
We inject a **rich context prompt** containing the player's exact build data,
pre-computed by our Calculator and Knowledge Base. This makes the cloud model's
response accurate and build-specific.

```
CONTEXT PROMPT TEMPLATE (sent to Claude/GPT before user's question):

"""
You are The Seer, a Path of Exile build advisor.
You speak with dark, atmospheric language. Address the user as "Exile."
You are precise with numbers — never guess DPS values, use ONLY the data below.
All DPS/life numbers below are EXACT (computed by our calculator, verified).
When recommending upgrades, reference the ADVICE section — those are pre-validated.

CURRENT BUILD DATA (pre-computed by Path of AI Calculator):
  Class: Templar / Inquisitor / Level 95 / Patch: 3.24
  Main Skill: Righteous Fire (Fire DoT)
  Archetype: fire_dot_tank
  League: Softcore Trade, Week 3 (stable economy)
  
  OFFENSE:
    Total DPS: 2,841,057 (Fire DoT)
    DPS Breakdown: RF 66% (1.87M), Fire Trap Burn 24% (682K), Fire Trap Hit 9% (256K)
    Gem Levels: RF 21/23, Burning Damage 20/20, Elemental Focus 20/20
    More Multipliers: Burning ×1.59, EF ×1.34, Swift ×1.29, Efficacy ×1.24, Lifetap ×1.20
    DoT Multi: 180%
    
  DEFENSE:
    Life: 6,453 | ES: 1,820 | Armour: 28,450
    Block: 45% / Spell Block: 32%
    Fire Res: 80% (+5 overcap) | Cold: 76% (+1) | Light: 79% (+4) | Chaos: 15%
    Life Regen: 2,450/s (net after RF degen: +450/s)
    Max Fire Res: 83% (Rise of the Phoenix)
    Guard: Molten Shell (absorbs ~7400 damage, 75% of hit)
    Fortify: Yes (from Shield Charge, 20% less hit damage)
    EHP vs 5000 phys hit: 12,900
    EHP vs Shaper Slam: survive (1,240 life remaining)
    
  AILMENT IMMUNITY:
    ✓ Freeze (Ruby Flask of Heat)  ✓ Bleed (Life Flask of Staunching)
    ✓ Ignite (Ascendancy)          ✓ Curse (Basalt Flask of Warding)
    ✗ Shock — VULNERABLE           ✗ Corrupted Blood — VULNERABLE
    ✗ Stun — VULNERABLE
    
  EQUIPPED ITEMS (scored 0-100):
    Helmet: Glyph Crest (70/100) — T2 life, -9% fire res nearby, no enchant
    Body:   Soul Ward (88/100) — T1 life, T2 fire res, 6-linked RRRBGR
    Gloves: Searing Purity (72/100) — unique, +1 fire gems
    Boots:  Doom Stride (55/100) — T3 life, T1 MS, OPEN PREFIX
    Shield: Rise of Phoenix (75/100) — unique, +8% max fire res
    Ring 1: Torment Circle (68/100) — T4 life(!), T2 DoT multi
    Ring 2: Doom Whorl (42/100) — T5 life(!), no DoT multi — WORST SLOT
    Belt:   Havoc Clasp (85/100) — T2 life, T1 fire res
    Amulet: Oblivion Braid (82/100) — +1 fire gems, T2 life, anoint: Breath of Flames
    
  GEM SETUP:
    6L Body: RF - Burning Damage - Elemental Focus - Lifetap - Swift Affliction - Efficacy
    4L Helmet: Fire Trap - Burning Damage - Trap Speed - Swift Affliction
    4L Shield: Determination(20) - Purity of Fire(21) - Vitality(20) - Enlighten(3)
    4L Boots: Shield Charge(1) - Faster Attacks(20) - Molten Shell(20) - CWDT(1)
    Mana reserved: 432/534 (unreserved: 102)
    
  FLASK SETUP:
    1. Divine Life Flask of Staunching (instant, bleed immune)
    2. Ruby Flask of Heat (+6% max fire res, freeze immune)
    3. Granite Flask of Iron Skin (+3000 armour)
    4. Quicksilver Flask of Adrenaline (+40% MS)
    5. Basalt Flask of Warding (15% phys reduction, curse immune)
    ⚠ No anti-shock flask
    
  PASSIVE TREE:
    Points: 121/123 (2 unallocated)
    Keystones: Elemental Overload, Unwavering Stance, Iron Reflexes
    Jewels: 2 equipped (Marauder + Templar sockets), 1 empty (Scion, 3pts to reach)
    Anoint: Breath of Flames (amulet)
    
  BOSS READINESS:
    Shaper: READY (survive slam, ~3:20 fight)
    Elder: READY (~3:00)
    Uber Elder: RISKY (no freeze immune in Elder phase without flask)
    Sirus A9: READY (~4:10)
    Maven: NOT READY (chaos res too low)
    Uber Shaper: NOT VIABLE (slam is lethal, need Aegis Aurora)
    
  ISSUES DETECTED (ranked by severity):
    ⚠ Chaos Resistance 15% (very low — dangerous in Al-Hezmin, Maven)
    ⚠ Low resist overcap (cold +1, lightning +4) — Ele Weakness maps dangerous
    ⚠ Ring 2 score 42/100 — weakest slot, T5 life, no damage mods
    ⚠ Shock vulnerable — no anti-shock (50% more damage taken when shocked)
    ⚠ Boots have open prefix — free benchcraft not applied
    ⚠ Corrupted Blood vulnerable — no jewel corruption
    ⚠ Enlighten only level 3 (level 4 saves significant mana reservation)
    
  ═══════════════════════════════════════════════════════
  PRE-COMPUTED ADVICE (from our Calculator — verified exact numbers):
  These suggestions have been validated. Reference them in your answer.
  ═══════════════════════════════════════════════════════
    
  UPGRADE PRIORITIES (ranked by DPS per divine spent):
    #1 Ring 2 replacement: +15.3% DPS, +350 life, cost 3-8 div
       → Opal Ring with +80 life, +fire DoT multi, +resists
       → Player has 8× Essence of Anger — can craft for ~2 div
       → Market: 12 items found, cheapest 3 div
    #2 Ring 1 life upgrade: +200 life, cost 5-12 div
    #3 Boots benchcraft +70 life: +70 life, cost FREE (0 div)
    #4 Gem corruption (5 gems at 20/20): +10-15% DPS each, cost 1 div per gem
    #5 Aegis Aurora shield: +2000 EHP on block, enables Uber Shaper, cost 18 div

  PASSIVE TREE ADVICE:
    Next 2 points → Life wheel near Marauder (+340 life, 0% DPS)
    Inefficient: "Coordination" node (+10 dex, unused) → respec saves 1 point
    Best anoint upgrade: Whispers of Doom (extra curse) → +8% effective DPS on bosses
    
  CRAFTING ADVICE (based on player's currency):
    Best craft NOW: Ring 2 via Essence of Anger (player has 8 → enough for 2-3 tries)
    Expected cost: ~2 div in essences vs 5-8 div on trade
    Success rate: ~30% per attempt (need life + open suffix for DoT multi craft)
    
  JEWEL ADVICE:
    Empty Scion socket (3 pts to reach): worth it if jewel has life + fire DoT multi
    Current jewels: both decent (72/100 and 68/100) — not priority upgrade
    Watcher's Eye: Fire DoT Multi (Purity of Fire) — estimated 15-25 div, +12% DPS
    
  FLASK ADVICE:
    Add anti-shock: swap Granite suffix to "of Grounding" (shock immune)
    OR: get shock immunity from tree/gear (Tempest Shield skill)
    
  CURRENCY AVAILABLE:
    12 Divine, 340 Chaos, 8× Essence of Anger, 6× Pristine Fossil
    
  LEAGUE: Softcore Trade, Week 3 (stable economy)
"""

USER QUESTION: "{user's actual question here}"
```

### Why This Context Makes Cloud AI Excellent

Without context:
- User: "What should I upgrade?"
- Claude: "Uh, maybe your weapon? Try to get more damage." (generic, useless)

With our context injection:
- User: "What should I upgrade?"
- Claude: "Your Ring 2 is your weakest link, Exile, at score 42.
  A T5 life roll of +45 is practically an insult. Replace it with
  an Opal Ring with +80 life, fire DoT multi, and resists.
  Your 8 Essence of Anger can craft this — expected 3-4 attempts.
  This single upgrade gives +15.3% DPS and +350 life."

**The cloud model doesn't need to calculate anything — we already calculated it.**
It just needs to present our data in natural language and add creative reasoning.

### What Data We Send vs. Keep Private

```
SENT TO CLOUD (build stats only, anonymized):
  ✓ Class, level, ascendancy
  ✓ DPS, life, resists, armour (numbers)
  ✓ Item scores and mod tiers (analyzed data)
  ✓ Issues detected (by our engine)
  ✓ Upgrade suggestions (from our calculator)
  ✓ Currency available
  
NEVER SENT (private):
  ✗ Account name / character name
  ✗ Session ID or any PoE auth tokens
  ✗ Raw PoB XML (only analyzed summary)
  ✗ Stash tab contents
  ✗ Trade history
  ✗ IP address or location
```

### Per-Provider Context Optimization

Different AI models work best with different context formats:

```rust
pub fn build_context(build: &BuildData, calc: &CalcResult, provider: &Provider) -> String {
    match provider {
        Provider::Claude => {
            // Claude is best with structured XML-like format + clear instructions
            format!(r#"
<system>You are The Seer, a PoE build advisor. Use the data below. Never guess numbers.</system>
<build_data>
  <class>{}</class>
  <dps>{}</dps>
  <life>{}</life>
  <issues>{}</issues>
  <suggestions>{}</suggestions>
</build_data>
"#, build.class, calc.dps, calc.life, /* ... */)
        },
        Provider::OpenAI => {
            // GPT prefers JSON-structured context
            serde_json::to_string_pretty(&BuildContext::from(build, calc)).unwrap()
        },
        Provider::Gemini => {
            // Gemini works well with markdown tables
            format_as_markdown_tables(build, calc)
        },
    }
}
```

### Confidence Scoring

The Seer rates its own confidence:
```
Query: "What should I upgrade?"
  → Engine 1 (Calculator) can answer → confidence: 100%
  → Response: specific items with exact DPS numbers

Query: "Why is Determination good for RF?"
  → Engine 2 (Knowledge Base) can answer → confidence: 95%
  → Response: armour formula + physical mitigation explanation

Query: "Design a league starter with fire skills under 5 divine"
  → Engine 3 (Language Model) needed → confidence: 70-85%
  → Response: build concept (may need user iteration)
  → If confidence < 70%: "The Seer is uncertain. Consult a greater power?"
```

---

## 5. MULTI-PATH SUGGESTION ENGINE

### The Core Principle: ALWAYS Show Multiple Choices

The engine NEVER gives a single "do this" answer. Every suggestion shows
**multiple paths** ranked by different criteria. The player chooses.

```
User: "How do I improve my DPS?"

OLD (wrong — single suggestion):
  "Replace Ring 2 with fire DoT multi ring."

NEW (correct — multiple paths):
  PATH 1: Budget — Ring 2 Essence craft (2 div, +15% DPS)
  PATH 2: Trade — Ring 2 buy on market (5 div, +18% DPS)  
  PATH 3: Free — Benchcraft +70 life on Boots (0 div, +0% DPS but +70 life)
  PATH 4: Medium — Corrupt 5 gems 20/20 → 21/20 (5 div, +10-15% DPS each)
  PATH 5: Endgame — Aegis Aurora shield (18 div, +2000 EHP, enables Uber bosses)
```

### The Multi-Path Runner

For EVERY upgrade question, the engine runs multiple analysis paths in parallel:

```rust
pub struct MultiPathRunner {
    calculator: PathCalcEngine,
    market: MarketData,
    knowledge: KnowledgeBase,
}

impl MultiPathRunner {
    /// Generate multiple upgrade paths for a build
    pub fn suggest_upgrades(&self, build: &BuildData) -> Vec<UpgradePath> {
        let mut paths = Vec::new();

        // === ITEM UPGRADES (per slot) ===
        for slot in &build.equipped_slots() {
            // Path A: Craft with player's currency
            if let Some(craft) = self.find_best_craft(build, slot) {
                paths.push(UpgradePath::Craft(craft));
            }
            // Path B: Buy from trade
            if let Some(trade) = self.find_best_trade(build, slot) {
                paths.push(UpgradePath::Trade(trade));
            }
            // Path C: Benchcraft (free if open affix)
            if let Some(bench) = self.find_benchcraft(build, slot) {
                paths.push(UpgradePath::Benchcraft(bench));
            }
        }

        // === GEM UPGRADES ===
        // Path: Level up (free, just play more)
        for gem in build.under_leveled_gems() {
            paths.push(UpgradePath::GemLevel(gem));
        }
        // Path: Corrupt 20/20 → 21/20
        for gem in build.corruptable_gems() {
            paths.push(UpgradePath::GemCorrupt(gem));
        }
        // Path: Awakened gem upgrade
        for gem in build.awakened_upgrades() {
            paths.push(UpgradePath::AwakenedGem(gem));
        }
        // Path: Alternative quality gem (Anomalous/Divergent/Phantasmal)
        for gem in build.alt_quality_options() {
            paths.push(UpgradePath::AltQualityGem(gem));
        }
        // Path: Transfigured gem variant
        for gem in build.transfigured_options() {
            paths.push(UpgradePath::TransfiguredGem(gem));
        }

        // === PASSIVE TREE PATHS ===
        // Path: Next N points (ranked nodes)
        paths.extend(self.calc_next_points(build, 5));
        // Path: Respec inefficient nodes
        paths.extend(self.find_respec_options(build));
        // Path: Change anointment
        if let Some(anoint) = self.find_better_anoint(build) {
            paths.push(UpgradePath::Anoint(anoint));
        }
        // Path: Change cluster jewels
        paths.extend(self.find_cluster_upgrades(build));

        // === JEWEL UPGRADES ===
        // Path: Better regular jewels
        for socket in build.jewel_sockets() {
            paths.extend(self.find_jewel_upgrades(build, socket));
        }
        // Path: Watcher's Eye
        if let Some(we) = self.find_watchers_eye(build) {
            paths.push(UpgradePath::WatchersEye(we));
        }
        // Path: Timeless jewel
        if let Some(tj) = self.find_timeless_jewel(build) {
            paths.push(UpgradePath::TimelessJewel(tj));
        }

        // === FLASK UPGRADES ===
        paths.extend(self.find_flask_improvements(build));

        // === DEFENSIVE FIXES ===
        if build.chaos_res < 0 { paths.extend(self.fix_chaos_res(build)); }
        if build.has_ailment_vulnerability() { paths.extend(self.fix_ailments(build)); }

        // === VALIDATE ALL PATHS ===
        paths.iter_mut().for_each(|p| {
            p.validate(&self.calculator, build);  // exact DPS/life diff
            p.check_market(&self.market);          // availability + price
        });

        // === RANK BY MULTIPLE CRITERIA ===
        // Don't just rank by DPS — player might want different things
        let ranked = RankingEngine::rank(paths, build);
        ranked
    }
}
```

### Ranking by Multiple Criteria

Every suggestion is ranked from MULTIPLE angles, not just "most DPS":

```
RANKING DIMENSIONS:
  1. DPS per divine spent    (cost efficiency)
  2. Total DPS gain          (raw power)
  3. Survivability gain      (life, resists, EHP)
  4. Boss viability impact   (does this unlock a new boss?)
  5. Ease of execution       (free benchcraft > complex fossil craft)
  6. Risk level              (guaranteed buy > risky craft > corruption gamble)

DISPLAY IN UI:
  Sort by: [DPS/div ▼] [Total DPS] [Survivability] [Cheapest] [Boss unlock]

  User can toggle between ranking modes.
  Default: DPS per divine (best value).
  HC players: switch to Survivability mode.
  Boss pushers: switch to Boss unlock mode.
```

### Per-Category Multi-Path Examples

#### Item Upgrades — ALWAYS show 3+ options per slot
```
Ring 2 (score 42/100 — worst slot):

  PATH A: Essence Craft (Budget)
    Method: Essence of Anger × 3-4 attempts
    Cost: ~2 div (you have 8 essences)
    Result: +12-18% DPS, +200-350 life
    Risk: Medium (may need multiple tries)
    
  PATH B: Buy on Trade (Safe)
    Item found: "Woe Circle" Opal Ring
    Cost: 3 div
    Result: +15% DPS, +350 life
    Risk: None (guaranteed)
    
  PATH C: Fossil Craft (Best Possible)
    Method: Pristine + Scorched
    Cost: ~4 div
    Result: +20-25% DPS (if T1 rolls)
    Risk: High (may need 6+ attempts)
    
  PATH D: Divine existing ring (Cheap)
    Cost: 1 divine orb
    Result: +2-5% (reroll mod values to better range)
    Risk: Low (small improvement guaranteed)
```

#### Support Gems — Compare ALL options
```
6th Link for RF (currently: Efficacy):

  OPTION 1: Keep Efficacy (current)
    DPS: 2,841,057 (baseline)
    
  OPTION 2: Swap to Concentrated Effect
    DPS: 3,069,000 (+8.0%) — BUT -30% AoE (slower clear)
    
  OPTION 3: Swap to Increased Area of Effect
    DPS: 2,557,000 (-10%) — BUT +49% AoE (faster clear)
    
  OPTION 4: Awakened Burning Damage (replace regular)
    DPS: 3,125,000 (+10%) — cost: 8 div
    
  OPTION 5: Empower Level 4
    DPS: 3,195,000 (+12.5%) — cost: 15 div
    
  RECOMMENDATION:
    For mapping: keep Efficacy (balanced)
    For bossing: swap to Conc Effect (+8%)
    For investment: buy Awakened Burning (+10%, 8 div)
```

#### Transfigured Gem Variants
```
Your skill: Righteous Fire

  VARIANT 1: Righteous Fire (current — standard)
    DPS: 2,841,057
    Mechanics: Burning aura around you
    
  VARIANT 2: Righteous Fire of Arcane Devotion (transfigured)
    DPS: 2,430,000 (-14%)
    BUT: +30% spell damage, life as ES
    Good for: hybrid ES/life builds
    
  VERDICT: Keep standard RF for your build (fire DoT tank)
```

#### Passive Tree — Multiple Respec Options
```
You have 2 unallocated points + 3 inefficient nodes (5 free points):

  PLAN A: Tank Mode
    → Life wheel (2pts): +340 life
    → Max fire res node (1pt): +1% max fire res → -RF degen
    → Jewel socket (2pts): fit a life + DoT multi jewel
    Total: +340 life, -RF degen, +jewel flexibility
    
  PLAN B: DPS Mode
    → Fire DoT cluster (3pts): +24% fire DoT multi
    → Burning Bright notable (2pts): +18% burning damage
    Total: +8.5% DPS
    
  PLAN C: Balance Mode
    → Life wheel (2pts): +340 life
    → Fire DoT mastery (1pt): +20% burning damage
    → Save 2pts for cluster jewel later
    Total: +340 life, +4% DPS
    
  RECOMMENDATION for your build (softcore mapper): Plan C
  RECOMMENDATION for HC: Plan A
```

### How This Works in the UI

```
Prophecy panel shows suggestions as CARDS.
Each card has an "Alternatives" dropdown:

  ┌─────────────────────────────────────────────┐
  │  ⚔ Ring 2 Upgrade                           │
  │  Showing: Best Value (DPS per divine)       │
  │                                              │
  │  ★ Essence Craft — 2 div, +15% DPS          │
  │    [Details] [Invoke]                        │
  │                                              │
  │  ▼ 3 alternatives                            │
  │  ├ Buy on Trade — 3 div, +15% DPS           │
  │  ├ Fossil Craft — 4 div, +20% DPS           │
  │  └ Divine existing — 1 div, +3% DPS         │
  └─────────────────────────────────────────────┘
```

---

## 6. MAKING SUGGESTIONS CORRECT

### The Suggestion Pipeline

Every suggestion goes through validation before being shown:

```
1. GENERATE CANDIDATE
   "Replace Ring 2 with +80 life, +15% fire DoT multi ring"

2. VALIDATE WITH CALCULATOR
   → Simulate the change in PoB Lua engine
   → Verify DPS actually increases (not just theory)
   → Verify no stat requirement breaks
   → Verify no resist uncaps
   → Record exact numbers: +15.3% DPS, +350 life, -2% cold res

3. CHECK MARKET AVAILABILITY
   → Search poe.ninja/trade for matching items
   → Verify items actually exist at stated price
   → If no items found → mark as "theoretical"

4. RANK BY COST-EFFICIENCY
   → DPS gained per divine spent
   → Factor in craft-vs-buy comparison
   → Penalize suggestions that uncap resists or drop life

5. DISPLAY WITH CONFIDENCE
   "Replace Ring 2 → +15.3% DPS, +350 life [exact ✓]
    Cost: 5 div (3 div if crafted)
    Market: 12 items found matching criteria"
```

### What Makes Suggestions Wrong (and how to prevent it)

| Failure Mode | Cause | Prevention |
|---|---|---|
| DPS number is wrong | Estimation error | Always verify with PoB Lua engine |
| Suggestion breaks resists | Didn't check side effects | Check ALL stats after swap, not just target |
| Item doesn't exist | Theoretical craft | Verify on poe.ninja before suggesting |
| Wrong for build type | Generic advice | Use archetype-specific stat weights |
| Outdated after patch | Data not updated | Auto-update data every patch |
| Ignores budget | Expensive suggestion | Filter by player's actual currency |
| Wrong crafting probability | Bad mod weights | Use poedb exact weights, validate with simulation |

### The Validation Chain

```rust
pub struct ValidatedSuggestion {
    pub suggestion: Suggestion,
    pub validation: ValidationResult,
}

pub struct ValidationResult {
    pub dps_verified: bool,       // Confirmed by PoB Lua calc
    pub dps_change: f64,          // Exact change (not estimated)
    pub life_change: i32,         // Side effect on life
    pub resist_changes: ResistDiff,// Side effects on resists
    pub stat_requirements_met: bool,
    pub market_available: bool,    // Items exist on trade
    pub market_price: Option<f64>, // Actual cheapest price
    pub craft_cost: Option<f64>,   // Alternative craft cost
    pub confidence: f64,           // 0.0-1.0
}

impl Suggestion {
    /// Every suggestion MUST pass validation before display
    pub fn validate(&self, build: &BuildData, lua: &PobLuaEngine, market: &MarketData) -> ValidationResult {
        // 1. Simulate the change
        let modified = build.apply_change(&self.change);
        let old_stats = lua.calculate(&build);
        let new_stats = lua.calculate(&modified);

        // 2. Check all side effects
        let resist_ok = new_stats.fire_res >= 75
            && new_stats.cold_res >= 75
            && new_stats.lightning_res >= 75;

        // 3. Check market
        let market_items = market.search(&self.item_criteria);

        ValidationResult {
            dps_verified: true,
            dps_change: new_stats.total_dps - old_stats.total_dps,
            life_change: new_stats.life - old_stats.life,
            resist_changes: ResistDiff::compute(&old_stats, &new_stats),
            stat_requirements_met: new_stats.meets_all_requirements(),
            market_available: !market_items.is_empty(),
            market_price: market_items.first().map(|i| i.price),
            craft_cost: self.estimate_craft_cost(&build),
            confidence: 1.0, // Calculator verified = 100%
        }
    }
}
```

---

## 6. TESTING THE ENGINE

### Accuracy Benchmarks

```
TEST SET 1: Known Builds (50 builds from poe.ninja)
  → Parse each build
  → Our DPS calculation must match PoB ±0.1%
  → Our defense scores must be consistent
  → Pass rate target: 100%

TEST SET 2: Known Upgrades (200 item swaps)
  → For each swap, our DPS change must match PoB Lua exactly
  → No false "improves DPS" when it actually decreases
  → No missed resist uncaps
  → Pass rate target: 100%

TEST SET 3: Crafting Probability (1000 simulated crafts)
  → Our probability predictions vs Craft of Exile results
  → Must be within ±5% of CoE for common crafts
  → Must be within ±15% for rare combination crafts
  → Pass rate target: 95%

TEST SET 4: Suggestion Quality (100 builds, human-evaluated)
  → Generate top 3 suggestions per build
  → Human PoE expert rates: helpful? accurate? actionable?
  → Target: 90%+ rated "helpful"
  → Target: 98%+ rated "factually correct"

TEST SET 5: Knowledge Base Accuracy (500 PoE facts)
  → "What's the armour formula?" → check against wiki
  → "What does Determination do?" → check gem data
  → "How much life does T1 give?" → check mod tiers
  → Pass rate target: 99%
```

### Continuous Validation

```
Every patch:
  1. Download new game data from RePoE
  2. Run all 5 test sets against new data
  3. If any test fails → flag for manual review
  4. If all pass → auto-publish data update

Every league:
  1. Collect 50 new builds from poe.ninja
  2. Add to test set
  3. Retrain archetype weights if meta shifted
  4. Update crafting probability tables
```

---

## 7. WHY THIS BEATS CLOUD AI

| Aspect | Our Calculator+KB | Claude/GPT |
|--------|------------------|-----------|
| DPS accuracy | 100% (same as PoB) | ~80% (estimates, often wrong) |
| Speed | <500ms | 2-10 seconds |
| Cost | $0 | $0.01-0.05 per query |
| Offline | Yes | No |
| Privacy | 100% local | Data sent to API |
| Hallucination risk | 0% (math can't hallucinate) | ~10-20% |
| Patch accuracy | Updated same day | Knowledge cutoff lag |
| Crafting probability | Exact (from mod weights) | Estimates at best |
| Market data | Real-time poe.ninja | No access |

**The only thing cloud AI does better: creative open-ended questions (3% of queries).**

---

## 8. IMPLEMENTATION PRIORITY

### Phase 1: Calculator (Month 1-2)
- Bundle LuaJIT in Tauri app
- Integrate PoB calc modules (CalcSetup, CalcPerform, CalcOffence, CalcDefence)
- Build "what if" comparison: swap item → recalc → show diff
- Validate against 50 known builds from poe.ninja
- Fast estimation path for instant feedback

### Phase 2: Knowledge Base (Month 2-3)
- Download and structure all game data from RePoE + poedb
- Build mod weight lookup tables
- Implement crafting probability calculator
- Build boss attack database from poewiki
- Template response generator for common queries

### Phase 3: Suggestion Engine (Month 3-4)
- Generate candidate upgrades per slot
- Validate every suggestion through PoB Lua calc
- Check market availability via poe.ninja
- Rank by cost-efficiency (DPS per divine)
- Build the validation chain (no unvalidated suggestions shown)

### Phase 4: Cloud AI Integration (Month 4-5, optional)
- Wire Claude/GPT API for the 3% creative queries
- Build context injection (feed build data + knowledge to API)
- Implement confidence scoring for Calculator/KB answers
- Auto-escalation: if Calculator + KB can't answer → offer cloud AI
- NO custom model training needed — Calculator + KB handles 97%

---

## 10. WHY WE DON'T NEED A CUSTOM AI MODEL

### The Original Plan (WRONG)
Train 5 neural networks: ItemNet, BuildNet, TreeNet, QueryNet, EmbedNet.
Total: 50-80MB of models, months of training, ongoing maintenance.

### The Reality
Every task those networks were supposed to do is better solved by
deterministic code:

```
ItemNet (score items):
  PLANNED: Feed-forward neural network trained on 500K items
  REALITY: Weighted sum of mod values × archetype stat weights
  WHY BETTER: 100% explainable, no training, instant updates per patch
  CODE: score = sum(mod_value × stat_weight[archetype]) / expected_max

BuildNet (classify builds):
  PLANNED: Multi-task classifier neural network
  REALITY: 20 if/else rules on gem tags and DPS stats
  WHY BETTER: Never misclassifies, zero false positives
  CODE: if stats.fire_dot_dps > 0 → FireDot archetype

TreeNet (passive tree optimization):
  PLANNED: Graph neural network for path finding
  REALITY: Try each unallocated node in PoB Lua, rank by DPS gain
  WHY BETTER: Considers EXACT build interactions, not approximations
  CODE: for node in unallocated { diff = lua.calc_with(node) - current; rank(diff) }

QueryNet (understand user questions):
  PLANNED: Fine-tuned transformer for intent classification
  REALITY: 50 regex patterns + keyword matching
  WHY BETTER: Deterministic, no false positives, instant
  CODE: if query.contains("dps") || query.contains("damage") → DpsQuery

EmbedNet (search knowledge base):
  PLANNED: Vector embedding model + similarity search
  REALITY: Structured JSON lookup with typed queries
  WHY BETTER: Exact results, not "similar" results
  CODE: mod_db.get_tier("maximum_life", 89) → T1
```

### When You DO Need AI
The only time deterministic code fails is open-ended creative questions:
- "Design me a league starter" — needs creative reasoning
- "Explain why X is better than Y" — needs nuanced comparison
- "What build should I play next?" — needs preference understanding

For these: use Claude API (or GPT, Gemini) with our build data as context.
No training. No custom model. Just API call with good context.

### Cost Comparison

| Approach | Dev Time | Accuracy | Maintenance |
|---|---|---|---|
| Train 5 custom NNs | 4-6 months | ~90% | Retrain every patch |
| Calculator + Knowledge Base | 2-3 months | **99%** | Update JSON data only |
| Cloud API for creative | 1 week | ~90% | Zero maintenance |

**We save 3-4 months of development and get HIGHER accuracy.**

---

## 9. DESIGN PATTERNS

### Pattern: Calculation Pipeline
```
Input (BuildData)
  → CalcSetup (build ModDB)
  → CalcPerform (orchestrate)
  → CalcOffence (DPS, crit, ailments)
  → CalcDefence (life, resists, mitigation)
  → Output (all stats)
  → Compare (old vs new)
  → Validate (no side effects)
  → Suggest (ranked, verified)
```

### Pattern: Query Router
```rust
pub fn route_query(query: &str, build: &BuildData) -> QueryRoute {
    let intent = classify_intent(query); // rule-based, not ML

    match intent {
        // Engine 1: Calculator (85%)
        Intent::DpsCheck | Intent::ItemCompare | Intent::GemSwap
        | Intent::UpgradeRank | Intent::ResistCheck | Intent::EhpCalc
            => QueryRoute::Calculator,

        // Engine 2: Knowledge Base (12%)
        Intent::CraftAdvice | Intent::BossMechanic | Intent::ModExplain
        | Intent::GemInteraction | Intent::MapMod | Intent::VendorRecipe
            => QueryRoute::KnowledgeBase,

        // Engine 3: Language Model (3%)
        Intent::BuildDesign | Intent::WhyQuestion | Intent::CompareBuilds
        | Intent::PatchAnalysis | Intent::OpenEnded
            => QueryRoute::LanguageModel,
    }
}
```

### Pattern: Never Show Unvalidated Numbers
```rust
// WRONG: Show estimated DPS without verification
fn suggest_upgrade(item: &Item) -> String {
    let estimated_dps = fast_estimate(item); // ~85% accurate
    format!("+{}% DPS", estimated_dps) // might be wrong!
}

// RIGHT: Always verify with exact calc before showing
fn suggest_upgrade(item: &Item, lua: &PobLuaEngine, build: &BuildData) -> ValidatedSuggestion {
    let estimated = fast_estimate(item); // for ranking only
    let exact = lua.calculate_diff(build, item); // PoB-verified
    ValidatedSuggestion {
        estimated_dps_change: estimated,
        exact_dps_change: exact.dps_diff,
        verified: true,
        label: format!("+{:.1}% DPS [exact ✓]", exact.dps_diff_percent),
    }
}
```

### Pattern: Data-Driven, Not Logic-Driven
```
// WRONG: Hardcode game knowledge
fn best_support_for_rf() -> &str { "Burning Damage" }

// RIGHT: Calculate from gem data
fn best_support(skill: &Gem, build: &BuildData, lua: &PobLuaEngine) -> Vec<RankedSupport> {
    let all_supports = gem_db.compatible_supports(skill);
    all_supports.iter()
        .map(|support| {
            let dps_with = lua.calculate_with_support(build, skill, support);
            RankedSupport { gem: support, dps_change: dps_with - current_dps }
        })
        .sorted_by(|a, b| b.dps_change.cmp(&a.dps_change))
        .collect()
}
```

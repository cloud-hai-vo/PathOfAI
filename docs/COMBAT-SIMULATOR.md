# Path of AI — Combat Simulator & Character Renderer

## Overview

The Combat Simulator ("The Arena") is a **visual simulation engine** that renders
the player's character fighting monsters and bosses — animated, with real game
mechanics, showing exactly how upgrades translate to faster kills and better survival.

This is NOT a static calculator. It's a **real-time animated battle** that plays
out like the actual game, using our calculation engine for all damage/defense math.

---

## 1. CHARACTER RENDERER

### Goal: 99% Like the In-Game Character

The character displayed in the center panel should look and move like the actual
PoE character, showing:

#### Visual Appearance
```
BODY:
  - Class-specific body shape (Templar is broad, Witch is slim, etc.)
  - Animated idle stance (slight breathing motion)
  - Equipment shown ON the character:
    → Helmet visible on head (shape changes by base type)
    → Body armour visible on torso
    → Gloves visible on hands
    → Boots visible on feet
    → Shield on left arm (if equipped)
    → Weapon in right hand (sceptre, sword, staff, bow, etc.)
  - Item rarity glow:
    → Unique items: orange glow
    → Rare items: yellow tint
    → Normal/magic: no glow

EFFECTS:
  - Active aura rings rotating around character (one per aura)
  - RF burning effect (fire particles around body if RF active)
  - Herald effects (ice/fire/thunder particles)
  - Buff icons floating above head
  - Fortify visual (golden shimmer on body)

RENDERING:
  - SVG for body silhouette + equipment outlines (scales cleanly)
  - CSS animations for idle motion, aura rotation, particle effects
  - Canvas overlay for particle systems (fire, ice, lightning)
  - Production: replace SVG with actual PoE character art via CDN
    (web.poecdn.com/image/Art/2DArt/Characters/)
```

#### Equipment Slots on Character Body
```
Each equipped item appears visually on the character:

  ┌──────── Helmet (on head) ────────┐
  │         visible shape             │
  │     ┌── Amulet (neck) ──┐       │
  │     │                    │       │
  ├─ Weapon ──── Body ── Shield ─────┤
  │  (right     Armour   (left       │
  │   hand)              hand)       │
  │     ├── Gloves (hands) ──┤       │
  │     │                    │       │
  │     ├── Belt (waist) ────┤       │
  │     │                    │       │
  │     ├── Ring1    Ring2 ──┤       │
  │     │                    │       │
  │     └── Boots (feet) ────┘       │
  └──────────────────────────────────┘

Click any slot → item tooltip in right panel (Flow #3)
Hover → glow effect + item name + score
```

---

## 2. MAP COMBAT SIMULATION

### What the Player Sees

When user clicks "Simulate Map" in The Arena panel:

```
┌───────────────────────────────────────────────────────────────┐
│                    THE ARENA — Strand T16                      │
│                                                               │
│  ┌─── Map Background (scrolling tiles) ──────────────────┐   │
│  │                                                        │   │
│  │    [Monster]  [Monster]  [Monster]                    │   │
│  │         ↓         ↓         ↓                         │   │
│  │     damage    damage    damage                        │   │
│  │     numbers   numbers   numbers                      │   │
│  │                                                        │   │
│  │              ┌──────────┐                             │   │
│  │              │ CHARACTER │  ← RF burning aura         │   │
│  │              │  walking  │  ← shield charge motion    │   │
│  │              └──────────┘                             │   │
│  │                                                        │   │
│  │    [Rare Monster]  ← HP bar above                     │   │
│  │    "Vaal Fallen"   ← takes 3 hits to kill             │   │
│  │    ████████░░ 65%  ← HP decreasing in real-time       │   │
│  │                                                        │   │
│  └────────────────────────────────────────────────────────┘   │
│                                                               │
│  Timer: 1:45 / ~2:30      Kills: 287/400      Deaths: 0      │
│  Currency Dropped: ~3.2 chaos   XP: 0.4% of level            │
│                                                               │
│  [Pause] [Speed: 2x ▼] [Stop]                                │
└───────────────────────────────────────────────────────────────┘
```

### How It Works Technically

```
MAP SIMULATION ENGINE:

1. GENERATE MAP:
   - Load map layout data (linear/open/branching)
   - Generate monster packs based on map tier:
     → Normal monsters: X HP, Y damage (from default_monster_stats.json)
     → Magic monsters: ×3 HP, ×1.5 damage
     → Rare monsters: ×10 HP, ×3 damage, 3 mods
     → Map boss: specific HP/damage from boss database
   - Apply map mods (if user selected any):
     → "40% more monster life" → all HP ×1.4
     → "-12% max res" → player takes more damage
     → "Monsters have 30% chance to avoid ailments"

2. SIMULATE COMBAT (per tick, 100ms):
   FOR each monster pack in map:
     // Player attacks
     player_dps = calculator.offense.total_dps
     damage_per_tick = player_dps * 0.1  // 100ms tick
     
     // RF burns everything in radius
     for monster in monsters_in_aoe(player_pos, rf_radius):
       monster.hp -= rf_dps_per_tick
       show_damage_number(rf_damage, monster.pos, color: fire)
     
     // Fire trap hits single target
     if fire_trap.cooldown_ready:
       fire_trap.throw(nearest_rare)
       target.hp -= fire_trap_damage
       show_damage_number(fire_trap_damage, target.pos, color: fire)
     
     // Monster attacks player
     for monster in monsters_attacking_player:
       hit_damage = monster.base_damage * monster.damage_mods
       mitigated = calculator.defense.mitigate(hit_damage, monster.damage_type)
       player.hp -= mitigated
       
       if player.hp <= 0:
         player.deaths += 1
         player.hp = player.max_hp  // respawn
         show_death_animation()
       
       // Guard skill triggers
       if mitigated > cwdt_threshold:
         activate_molten_shell()
         show_guard_animation()
       
       // Regen
       player.hp += player.regen_per_tick
       player.hp = min(player.hp, player.max_hp)
     
     // Movement
     player.move_towards(next_pack, player.move_speed)
     update_position_on_screen()
   
   // Pack cleared
   kills += pack.monster_count
   currency += random_currency_drop(map_tier)
   update_timer()
   update_kill_counter()

3. DISPLAY:
   - Character sprite moves across the map
   - Monsters appear ahead, die as RF burns them
   - Damage numbers float up (like in-game)
   - HP bars above rare/unique monsters
   - Currency icons drop from killed monsters
   - Timer counts up
   - Kill counter updates
   - Player HP bar at bottom shows damage taken + regen
```

### Simulation Parameters
```
Speed: [0.5x] [1x] [2x] [5x] [10x] [Skip to end]
  → 1x plays in real-time (2:30 for a full map)
  → 10x shows rapid combat (15 seconds to simulate full map)
  → Skip to end: instant result with stats

Pause: freeze simulation, inspect current state
  → hover any monster to see its remaining HP
  → hover player to see current buffs/debuffs

Map Mods: user can toggle map mods on/off to see impact
  → "Enable -max res" → watch player take more damage
  → "Enable no regen" → watch RF kill the character
```

---

## 3. BOSS FIGHT SIMULATION

### What the Player Sees

When user clicks "Simulate Shaper" in boss readiness:

```
┌───────────────────────────────────────────────────────────────┐
│              THE ARENA — Shaper (Phase 1/4)                   │
│                                                               │
│  ┌─── Boss Arena (circular platform) ───────────────────┐    │
│  │                                                       │    │
│  │                 ┌──────────┐                         │    │
│  │                 │  SHAPER  │ ← boss sprite           │    │
│  │                 │ casting  │ ← current animation      │    │
│  │                 └──────────┘                         │    │
│  │                                                       │    │
│  │    ████████████████████░░░░░░░░░░░░                  │    │
│  │    Shaper HP: 14,200,000 / 20,000,000 (71%)          │    │
│  │                                                       │    │
│  │              [Ball Lightning projectiles]              │    │
│  │                   ↓   ↓   ↓                          │    │
│  │              ┌──────────┐                             │    │
│  │              │ CHARACTER │  ← dodging                 │    │
│  │              │  casting  │  ← fire trap thrown        │    │
│  │              └──────────┘                             │    │
│  │                                                       │    │
│  │  ┌─ Player HP ──────────────────────────────────┐    │    │
│  │  │ ████████████████████████░░░░ 6,453 / 6,453   │    │    │
│  │  └──────────────────────────────────────────────┘    │    │
│  │                                                       │    │
│  └───────────────────────────────────────────────────────┘    │
│                                                               │
│  Phase: 1/4    Time: 0:48    Deaths: 0    DPS: 1.7M eff.     │
│                                                               │
│  ⚠ INCOMING: Shaper Slam (3.2s telegraph)                    │
│  Your survival: ✅ YES (1,240 life remaining with Molten Shell)│
│                                                               │
│  [Pause] [Speed: 2x ▼] [Skip Phase] [Stop]                   │
└───────────────────────────────────────────────────────────────┘
```

### Dodge Chance Formula

Boss attacks have a `dodge_chance_base` — the probability of dodging IF the
player has 100% move speed. Actual dodge chance scales with move speed and
telegraph duration:

```rust
fn calc_dodge_chance(attack: &BossAttack, player: &PlayerState) -> f64 {
    // Base chance assumes 100% move speed + full telegraph visibility
    let base = attack.dodge_chance_base;
    
    // Move speed factor: faster player = easier to dodge
    // At 0% MS: impossible to dodge. At 200% MS: 50% better than base.
    let ms_factor = (player.move_speed as f64 / 100.0).clamp(0.0, 2.0);
    
    // Telegraph factor: longer telegraph = easier to dodge
    // Minimum telegraph for dodging: 200ms (human reaction time)
    let telegraph_factor = if attack.telegraph_ms < 200 {
        0.1 // nearly impossible to dodge (instant attack)
    } else {
        (attack.telegraph_ms as f64 / 2000.0).clamp(0.3, 1.5) // normalize to 2s reference
    };
    
    // Stun penalty: stunned player can't dodge
    if player.stun_remaining_ms > 0 {
        return 0.0;
    }
    
    // Final dodge chance
    let dodge = base * ms_factor * telegraph_factor;
    dodge.clamp(0.0, 0.95) // cap at 95% (nothing is guaranteed)
}

// Examples:
// Shaper Slam (base 0.80, telegraph 3200ms, player 145% MS):
//   = 0.80 × (145/100) × (3200/2000) = 0.80 × 1.45 × 1.5 = 1.74 → capped at 0.95
//   → 95% dodge chance (easy to dodge with good MS)

// Shaper Ball Lightning (base 0.70, telegraph 500ms, player 145% MS):
//   = 0.70 × 1.45 × (500/2000) = 0.70 × 1.45 × 0.30 = 0.30
//   → 30% dodge chance (hard to dodge, low telegraph)

// Sirus Die Beam (dodge_requirement: "must_move_perpendicular"):
//   = 0.0 if player not moving, 0.90 if player IS moving perpendicular
//   → Special case: binary dodge (either you move or you don't)
```

### Boss AI Simulation

Each boss has a scripted attack pattern from our boss database:

```
SHAPER FIGHT PHASES:

Phase 1 (100% → 75% HP):
  BOSS AI LOOP:
    1. Idle for 2.0s
    2. ATTACK: Ball Lightning barrage (3 projectiles)
       → each deals 2000 cold damage
       → player dodges with 70% probability (based on move speed)
       → if hit: mitigate through cold res + armour → show damage taken
    3. Idle for 1.5s
    4. ATTACK: Melee slam (telegraph 3.2s)
       → deals 8000 physical damage
       → player CAN dodge (80% chance with good move speed)
       → if hit: Molten Shell absorbs 75% → show remaining life
       → if Molten Shell down: check if lethal → show ❌ or ✅
    5. Idle for 2.0s
    6. ATTACK: Golden beam (4 second channel)
       → deals 12000 cold damage over 4 seconds
       → player MUST dodge (moving breaks beam)
       → if not moving: die in 0.8 seconds
    7. REPEAT from step 1

  PLAYER DPS during phase:
    → RF burns constantly: 91,560/s × (1 - 0.40 boss_res) = 54,936/s effective
    → Fire Trap every 3.5s: 682,000 per trap × (1 - 0.40) = 409,200 per trap
    → DPS uptime: ~60% (rest is dodging mechanics)
    → Effective DPS: ~1,700,000
    → Time to clear 25% of 20M HP: 5M / 1.7M = ~2.9 seconds of DPS
    → With dodging overhead: ~45 seconds real time

Phase transition (75%):
  → Shaper becomes immune for 8 seconds
  → Zana shield phase (safe, no DPS possible)
  → Timer pauses

Phase 2-3 (75% → 25%):
  → Same as Phase 1 but adds:
    → Vortex ground (cold DOT zones to avoid)
    → Clone attack (two Shapers attacking simultaneously)

Phase 4 (25% → 0%):
  → Bullet hell phase (many projectiles)
  → Player regen tested heavily
  → Slam + beam + projectiles simultaneously
```

### Boss Attack Database Format
```json
// game-data/bosses/shaper.json
{
  "name": "The Shaper",
  "hp": 20000000,
  "phases": [
    {
      "name": "Phase 1",
      "hp_start": 1.0,
      "hp_end": 0.75,
      "attacks": [
        {
          "name": "Ball Lightning",
          "damage": 2000,
          "type": "cold",
          "count": 3,
          "cooldown_ms": 3500,
          "telegraph_ms": 500,
          "dodge_chance_base": 0.70,
          "description": "Three cold projectiles that track the player"
        },
        {
          "name": "Slam",
          "damage": 8000,
          "type": "physical",
          "cooldown_ms": 8000,
          "telegraph_ms": 3200,
          "dodge_chance_base": 0.80,
          "description": "Ground slam with golden circle indicator"
        },
        {
          "name": "Die Beam",
          "damage": 3000,
          "type": "cold",
          "duration_ms": 4000,
          "dps": true,
          "dodge_requirement": "must_move",
          "description": "Channeled beam — move to avoid"
        }
      ],
      "transition": {
        "immunity_duration_ms": 8000,
        "description": "Zana creates protective bubble"
      }
    }
  ],
  "uber_variant": {
    "hp_multiplier": 3.75,
    "damage_multiplier": 1.5,
    "description": "All attacks deal 50% more damage, 3.75x HP"
  }
}
```

---

## 4. UPGRADE PREVIEW SIMULATION

### The Key Feature: "See the Difference"

Before applying an upgrade, the player can **watch the simulation twice**:
once with current gear, once with the upgrade. Side by side.

```
┌────────────────────────────────────────────────────────────────┐
│  UPGRADE PREVIEW — Ring 2 → Woe Circle (Opal Ring)            │
│                                                                │
│  ┌─── BEFORE (current gear) ───┐ ┌─── AFTER (with upgrade) ──┐│
│  │                              │ │                            ││
│  │  Shaper Phase 1              │ │  Shaper Phase 1            ││
│  │  DPS: 1.7M effective         │ │  DPS: 2.0M effective       ││
│  │  Kill time: 45s              │ │  Kill time: 38s            ││
│  │                              │ │                            ││
│  │  [Sim running]               │ │  [Sim running]             ││
│  │  Timer: 0:32                 │ │  Timer: 0:27               ││
│  │  Deaths: 1                   │ │  Deaths: 0                 ││
│  │                              │ │                            ││
│  └──────────────────────────────┘ └────────────────────────────┘│
│                                                                │
│  DIFFERENCE:                                                   │
│  Fight time:  3:20 → 2:48  (saves 32 seconds)                 │
│  Deaths:      3 avg → 1 avg (66% fewer deaths)                │
│  Currency/hr: 4 div → 5.2 div (+30% farming efficiency)       │
│                                                                │
│  [Apply Upgrade] [Try Different Upgrade] [Close]               │
└────────────────────────────────────────────────────────────────┘
```

### Multi-Step Upgrade Path Preview

The Prophecy panel shows upgrade suggestions. Player can select MULTIPLE
upgrades and preview the CUMULATIVE effect:

```
┌────────────────────────────────────────────────────────────────┐
│  UPGRADE PATH BUILDER                                          │
│                                                                │
│  Select upgrades to apply (in order):                          │
│                                                                │
│  ☑ Step 1: Benchcraft +70 life on Boots (FREE)                │
│     → Life: 6,453 → 6,523 (+70)                               │
│                                                                │
│  ☑ Step 2: Replace Ring 2 with Woe Circle (3 div)             │
│     → DPS: 2.84M → 3.27M (+15.3%)                             │
│     → Life: 6,523 → 6,873 (+350)                              │
│                                                                │
│  ☑ Step 3: Corrupt 5 gems to 21/20 (5 div)                   │
│     → DPS: 3.27M → 3.60M (+10%)                               │
│                                                                │
│  ☐ Step 4: Aegis Aurora shield (18 div)                       │
│     → EHP: massive increase, Uber Shaper viable               │
│                                                                │
│  ════════════════════════════════════════                       │
│  CUMULATIVE RESULT (Steps 1-3):                                │
│  DPS:  2.84M → 3.60M (+26.8%)                                 │
│  Life: 6,453 → 6,873 (+420)                                   │
│  Cost: 8 divine total                                          │
│                                                                │
│  SIMULATION PREVIEW:                                           │
│  Shaper fight: 3:20 → 2:15 (saves 1:05)                       │
│  T16 clear: 2:30 → 1:55 (saves 35s)                           │
│  Currency/hr: 8 div → 11.2 div (+40%)                         │
│                                                                │
│  [▶ Watch Simulation] [Apply All Steps] [Apply Step by Step]   │
└────────────────────────────────────────────────────────────────┘

User clicks [▶ Watch Simulation]:
  → Plays boss fight with ALL selected upgrades applied
  → Shows the character with upgraded gear visually
  → Player can see the faster kill time in real-time

User clicks [Apply All Steps]:
  → Flow #5 (Apply Upgrade) for each step in order
  → Each step validated before applying
  → Show cumulative progress bar

User clicks [Apply Step by Step]:
  → Apply Step 1 only → re-analyze → show new suggestions
  → User can then apply Step 2 → re-analyze → etc.
  → Each step shows "Before → After" diff
```

---

## 4b. SIMULATION STATE MACHINE

### Combat Simulation States

```rust
enum SimState {
    Idle,                    // waiting to start
    Generating,              // creating map/boss arena
    Running {                // simulation playing
        tick: u32,
        speed: f32,          // 1.0 = real-time, 10.0 = fast forward
    },
    Paused { tick: u32 },    // user paused
    Complete(SimResult),     // finished — show results
}

enum EntityState {
    Alive { hp: f64, pos: Vec2, facing: Direction },
    Attacking { target_id: u32, animation_frame: u8 },
    Casting { skill_id: String, cast_time_remaining_ms: u32 },
    Moving { destination: Vec2, speed: f32 },
    TelegraphWarning { attack_name: String, time_remaining_ms: u32 },
    Dying { animation_frame: u8 },
    Dead,
    Immune,                  // boss phase transition
}
```

### Simulation Tick (Detailed)

```rust
/// Called every 100ms (10 ticks/second)
fn simulation_tick(state: &mut SimState, scene: &mut CombatScene) {
    // === PHASE 1: PLAYER ACTIONS (what player does this tick) ===
    
    // 1a. RF burns everything in radius (always active for RF builds)
    if scene.player.has_active_skill("RighteousFire") {
        for monster in scene.monsters_in_range(scene.player.pos, scene.player.rf_radius) {
            let rf_damage = scene.player.calc.rf_dps_per_tick; // pre-computed
            monster.take_damage(rf_damage, DamageType::Fire);
            scene.spawn_damage_number(rf_damage, monster.pos, Color::FIRE);
        }
    }
    
    // 1b. Active skills (Fire Trap, Blade Vortex, etc.)
    for skill in &mut scene.player.active_skills {
        if skill.cooldown_remaining <= 0 {
            let target = scene.find_target(skill.targeting); // nearest, aoe, etc.
            if let Some(target) = target {
                let damage = scene.player.calc.skill_damage(skill.id);
                target.take_damage(damage, skill.damage_type);
                scene.spawn_damage_number(damage, target.pos, skill.damage_type.color());
                skill.cooldown_remaining = skill.cooldown_ms;
                scene.spawn_effect(skill.visual_effect, target.pos);
            }
        }
        skill.cooldown_remaining -= 100; // tick = 100ms
    }
    
    // === PHASE 2: MONSTER ACTIONS ===
    
    for monster in &mut scene.monsters {
        match monster.state {
            EntityState::Alive { .. } => {
                // 2a. Check if monster should attack
                if monster.in_range_of(scene.player.pos, monster.attack_range) {
                    let hit_damage = monster.base_damage * monster.damage_mods;
                    let damage_type = monster.damage_type;
                    
                    // Player mitigates damage
                    let mitigated = scene.player.calc.mitigate(hit_damage, damage_type);
                    scene.player.hp -= mitigated;
                    scene.spawn_damage_number(mitigated, scene.player.pos, Color::WHITE);
                    
                    // CWDT check → trigger guard skill
                    if mitigated > scene.player.cwdt_threshold {
                        scene.player.activate_guard_skill();
                        scene.spawn_effect(Effect::MoltenShell, scene.player.pos);
                    }
                    
                    monster.attack_cooldown = monster.attack_speed_ms;
                } else {
                    // 2b. Move towards player
                    monster.move_towards(scene.player.pos, monster.move_speed);
                }
            },
            EntityState::Dying { frame } => {
                if frame >= 8 { monster.state = EntityState::Dead; }
                else { monster.state = EntityState::Dying { animation_frame: frame + 1 }; }
            },
            _ => {}
        }
        
        // Check if monster died
        if monster.hp <= 0.0 && !matches!(monster.state, EntityState::Dead | EntityState::Dying { .. }) {
            monster.state = EntityState::Dying { animation_frame: 0 };
            scene.stats.kills += 1;
            scene.stats.currency += monster.currency_drop();
        }
    }
    
    // === PHASE 3: PLAYER RECOVERY ===
    
    // 3a. Life regen
    scene.player.hp += scene.player.calc.regen_per_tick;
    scene.player.hp = scene.player.hp.min(scene.player.calc.max_life);
    
    // 3b. ES recharge (if not hit recently)
    if scene.player.last_hit_tick + 20 < state.tick() { // 2 seconds = 20 ticks
        scene.player.es += scene.player.calc.es_recharge_per_tick;
        scene.player.es = scene.player.es.min(scene.player.calc.max_es);
    }
    
    // 3c. Leech
    scene.player.hp += scene.player.calc.leech_per_tick;
    scene.player.hp = scene.player.hp.min(scene.player.calc.max_life);
    
    // 3d. Flask charges (gain on kill)
    for flask in &mut scene.player.flasks {
        flask.charges += scene.stats.kills_this_tick * flask.charge_on_kill;
    }
    
    // === PHASE 4: STUN CHECK (after damage taken) ===
    
    // PoE stun: if single hit > stun_threshold, player is stunned
    // stun_threshold = max_life × 0.12 (default, reduced by stun avoidance)
    if let Some(last_hit) = scene.player.last_hit_this_tick {
        let stun_threshold = scene.player.calc.max_life * 0.12
            * (1.0 - scene.player.calc.stun_avoidance);
        if last_hit > stun_threshold && !scene.player.calc.stun_immune {
            scene.player.stun_remaining_ms = 350; // base stun duration
            scene.spawn_effect(Effect::Stunned, scene.player.pos);
            // While stunned: player can't attack, can't dodge, can't move
            // This makes stun immunity CRITICAL for some builds
        }
    }
    
    // === PHASE 5: DEATH CHECK ===
    
    if scene.player.hp <= 0.0 {
        scene.stats.deaths += 1;
        scene.player.hp = scene.player.calc.max_life;
        scene.player.es = scene.player.calc.max_es;
        scene.stats.death_penalty_time += 5000; // 5 seconds respawn
        scene.spawn_effect(Effect::DeathExplosion, scene.player.pos);
    }
    
    // === PHASE 6: MOVEMENT ===
    
    // Player movement (if not stunned)
    if scene.player.stun_remaining_ms <= 0 && scene.current_pack_cleared() {
        scene.player.move_towards(scene.next_pack_position(), scene.player.move_speed);
    }
    scene.player.stun_remaining_ms = (scene.player.stun_remaining_ms - 100).max(0);
    
    // Monster movement (with pursuit range + disengagement)
    for monster in &mut scene.monsters {
        if matches!(monster.state, EntityState::Alive { .. }) {
            let dist = monster.pos.distance(scene.player.pos);
            if dist > monster.pursuit_range {
                // Monster too far — disengage (return to spawn point)
                monster.move_towards(monster.spawn_pos, monster.move_speed * 0.5);
            } else if dist > monster.attack_range {
                // Chase player
                monster.move_towards(scene.player.pos, monster.move_speed);
            }
            // If in attack_range → handled in Phase 2 (monster attacks)
        }
    }
    
    // === PHASE 6: CHECK COMPLETION ===
    
    if scene.all_monsters_dead() {
        *state = SimState::Complete(scene.stats.finalize());
    }
}
```

## 4c. ADDITIONAL BOSS SCHEMAS

### Sirus, Awakener of Worlds

```json
{
  "name": "Sirus, Awakener of Worlds",
  "hp": 25000000,
  "resistance": { "fire": 40, "cold": 40, "lightning": 40, "chaos": 25 },
  "phases": [
    {
      "name": "Phase 1",
      "hp_range": [1.0, 0.75],
      "attacks": [
        { "name": "Corridor Beams", "damage": 3000, "type": "physical", "cooldown_ms": 5000, "dodge_chance_base": 0.85, "telegraph_ms": 1500 },
        { "name": "Spinning Quad Beam", "damage": 2500, "type": "physical", "count": 4, "cooldown_ms": 8000, "dodge_requirement": "stand_between_beams" },
        { "name": "Clone Teleport", "damage": 0, "type": "none", "cooldown_ms": 12000, "description": "Teleports and creates clone" }
      ]
    },
    {
      "name": "Phase 4 (Final)",
      "hp_range": [0.25, 0.0],
      "attacks": [
        { "name": "Die Beam", "damage": 4000, "type": "physical", "dps": true, "duration_ms": 8000, "dodge_requirement": "must_move_perpendicular", "telegraph_ms": 800 },
        { "name": "Meteor Maze", "damage": 15000, "type": "physical", "dodge_requirement": "escape_maze_in_5s", "telegraph_ms": 2000 },
        { "name": "Everlasting Fire", "damage": 2000, "type": "fire", "dps": true, "duration_ms": 6000, "ground_degen": true }
      ]
    }
  ],
  "uber_variant": { "hp_multiplier": 4.0, "damage_multiplier": 1.5 }
}
```

### Maven, The Maven

```json
{
  "name": "The Maven",
  "hp": 30000000,
  "resistance": { "fire": 40, "cold": 40, "lightning": 40, "chaos": 30 },
  "phases": [
    {
      "name": "Phase 1",
      "hp_range": [1.0, 0.60],
      "attacks": [
        { "name": "Cascade of Pain", "damage": 3500, "type": "physical", "count": 8, "cooldown_ms": 6000, "dodge_chance_base": 0.60 },
        { "name": "Gravity Well", "damage": 0, "type": "none", "description": "Pulls player to center", "cooldown_ms": 15000 },
        { "name": "Brain Laser", "damage": 5000, "type": "fire", "dps": true, "duration_ms": 3000, "dodge_requirement": "must_move" }
      ]
    },
    {
      "name": "Memory Game",
      "hp_range": [0.60, 0.60],
      "special_mechanic": {
        "type": "memory_game",
        "description": "Player must stand on lit platforms in sequence",
        "damage_on_fail": 99999,
        "chaos_damage": true,
        "rounds": 3,
        "note": "Chaos res critical here — low chaos res = death on any mistake"
      }
    }
  ],
  "uber_variant": { "hp_multiplier": 3.5, "damage_multiplier": 1.6 }
}
```

### Elder

```json
{
  "name": "The Elder",
  "hp": 18000000,
  "resistance": { "fire": 40, "cold": 40, "lightning": 40, "chaos": 25 },
  "phases": [
    {
      "name": "Phase 1",
      "hp_range": [1.0, 0.75],
      "attacks": [
        { "name": "Tentacle Slam", "damage": 6000, "type": "physical", "cooldown_ms": 6000, "telegraph_ms": 2000, "dodge_chance_base": 0.75 },
        { "name": "Ice Spear Barrage", "damage": 1500, "type": "cold", "count": 12, "cooldown_ms": 4000, "dodge_chance_base": 0.50 },
        { "name": "Siphon", "damage": 2000, "type": "physical", "dps": true, "duration_ms": 5000, "description": "Drains player life, heals Elder" }
      ]
    },
    {
      "name": "Portal Phase",
      "hp_range": [0.75, 0.75],
      "special_mechanic": {
        "type": "add_phase",
        "description": "Elder spawns portals — must kill adds or they heal Elder",
        "add_count": 4,
        "add_hp": 500000
      }
    }
  ],
  "uber_variant": { "hp_multiplier": 4.0, "damage_multiplier": 1.5 }
}
```

---

## 5. RENDERING APPROACH — THREE TIERS

The character and combat rendering uses a tiered approach. Higher tiers
look better but require more resources. User can choose in Settings.

### Tier 1: 2D Sprite (MVP — Default)

```
TECHNOLOGY: HTML5 Canvas 2D + CSS animations
QUALITY: PoE-like but simplified (like poe.ninja build preview)
PERFORMANCE: Works on ANY PC, even integrated graphics
FILE SIZE: ~5-10MB of sprite assets

HOW IT WORKS:
  - Pre-rendered 2D sprite sheets for each class (7 classes × 4 directions)
  - Equipment overlays as separate sprite layers (helm, body, weapon, shield)
  - Walk cycle: 8-frame animation
  - Attack cycle: 6-frame animation
  - Effects (RF fire, aura rings): Canvas 2D particle system
  - Monsters: simplified sprite sheets (normal, magic, rare, boss variants)
  - Boss sprites: larger sprites with attack animations (8-12 frames per attack)

SPRITE SOURCE:
  - Extract from PoE game files via datamining tools
  - OR: commission custom pixel art (PoE style)
  - OR: use community sprite sheets (CC-licensed)
  - MVP: simple SVG silhouettes (what we have now, just animated)

LAYER STACK:
  Canvas Layer 0: Background (tiled map texture, scrolling)
  Canvas Layer 1: Ground effects (vortex, consecrated ground)
  Canvas Layer 2: Monsters (sprites + HP bars)
  Canvas Layer 3: Player character (sprite + equipment overlays)
  Canvas Layer 4: Projectiles + particles (fire, ice, lightning)
  HTML Overlay:   UI (HP bars, damage numbers, timer, controls)
```

### Tier 2: 2.5D Skeletal Animation (Post-MVP)

```
TECHNOLOGY: Spine 2D / PixiJS + Canvas 2D
QUALITY: Smooth skeletal animation like mobile ARPGs
PERFORMANCE: Needs decent GPU (integrated OK, no discrete required)
FILE SIZE: ~20-30MB of skeletal data + textures

HOW IT WORKS:
  - Spine 2D skeletal animation for character body
  - Skeleton has bones: head, torso, arms, legs, weapon_hand, shield_hand
  - Equipment skins swapped based on equipped items:
    → Each base type has a skin (plate helm, evasion helm, ES helm, etc.)
    → Rarity changes tint/glow
  - Walk, idle, attack, cast, death animations as Spine animations
  - Monsters also skeletal (reusable skeleton with different skins)
  - Boss fights: boss has own skeleton with phase-specific animations

LIBRARY: PixiJS (2D WebGL renderer, hardware-accelerated)
  - @pixi/spine for skeletal animation
  - 60fps WebGL rendering
  - Particle system for effects (RF fire, auras)
  - Works in Tauri webview (WebGL2 supported)
```

### Tier 3: 3D Rendered (Future — Ambitious)

```
TECHNOLOGY: Three.js + WebGL2 (or WebGPU when available)
QUALITY: Near in-game quality 3D characters
PERFORMANCE: Needs discrete GPU (GTX 1060+ / RX 580+)
FILE SIZE: ~100-200MB of 3D models + textures

HOW IT WORKS:
  - 3D character model per class (from PoE game files or custom)
  - Skeletal animation with IK (inverse kinematics)
  - Equipment as 3D mesh attachments (helm, armour, weapon, shield)
  - PBR materials for realistic lighting
  - Particle systems: fire (RF), ice, lightning, chaos, blood
  - Boss models: full 3D with phase animations
  - Camera: isometric angle matching PoE's camera

CHALLENGES IN TAURI WEBVIEW:
  - WebGL performance varies by OS webview:
    → Windows (WebView2/Edge): good WebGL2 support
    → macOS (WKWebView): good WebGL2
    → Linux (WebKitGTK): WebGL can be spotty
  - Known issue: Tauri + WebGL can have lag on Windows (GitHub issue #8020)
  - Mitigation: use OffscreenCanvas for rendering, keep UI on main thread
  - Alternative: render in Rust via wgpu, display as texture in webview

3D MODEL SOURCES:
  Option A: Extract from PoE game files (GGPK)
    → Legal gray area (GGG terms may prohibit)
    → Models are high-poly, need decimation
    → Textures are proprietary
  
  Option B: Custom 3D models (PoE-inspired style)
    → Commission from 3D artist (~$500-2000 per class)
    → We own the assets, no legal issues
    → Can match PoE's art style without using their assets
  
  Option C: Procedural / stylized (like poe.ninja's 3D viewer)
    → Low-poly stylized characters
    → Equipment as modular mesh pieces
    → Faster to create, easier to maintain
    → Community can contribute
```

### Tier 4: Native Rust Render Engine (wgpu — Our Own Engine)

```
TECHNOLOGY: wgpu (Rust GPU library) — Vulkan/Metal/DX12 native
QUALITY: Full GPU-accelerated rendering, no WebGL limitations
PERFORMANCE: Best possible — direct GPU access, no webview overhead
FILE SIZE: wgpu adds ~2-5MB to binary

WHY BUILD OUR OWN:
  - We're desktop-only — no need for browser compatibility
  - wgpu runs on Vulkan (Windows/Linux), Metal (macOS), DX12 (Windows)
  - No WebGL lag issues in Tauri webview
  - Full control over rendering pipeline
  - Can match PoE's isometric camera + lighting exactly
  - Rust ecosystem has mature 2D/3D rendering tools

HOW IT INTEGRATES WITH TAURI:

  ═══════════════════════════════════════════════════════
  CHOSEN APPROACH: Hybrid Window — wgpu surface INSIDE Tauri window
  ═══════════════════════════════════════════════════════

  Tauri v2 supports MULTIPLE SURFACES in the same window.
  The window is split into regions:
    - HTML/CSS webview regions (stats sidebar, right panel, HUD bar)
    - Native wgpu render region (character display area in center)

  Both coexist in the SAME WINDOW. No separate window needed.

  ┌──────────────────────────────────────────────────────────┐
  │  Header (HTML/CSS webview)                                │
  ├──────────┬──────────────────────────┬─────────────────────┤
  │          │                          │                     │
  │  Stats   │  ┌──────────────────┐   │   Right Panel       │
  │  Sidebar │  │  wgpu RENDER     │   │   (HTML/CSS)        │
  │          │  │  SURFACE         │   │                     │
  │ (HTML)   │  │                  │   │   Prophecy/          │
  │          │  │  Character +     │   │   Grimoire/          │
  │          │  │  Combat sim +    │   │   Forge/etc.         │
  │          │  │  GPU particles   │   │                     │
  │          │  │                  │   │                     │
  │          │  └──────────────────┘   │                     │
  │          │  Passive tree / scores  │                     │
  │          │  (HTML/CSS)             │                     │
  ├──────────┴──────────────────────────┴─────────────────────┤
  │  HUD Bar — Life/Mana orbs + Gem buttons (HTML/CSS)        │
  └──────────────────────────────────────────────────────────┘

  HOW THIS WORKS TECHNICALLY:

  1. Tauri creates the main window with its webview
  2. In Rust setup, we get the raw window handle
  3. We create a wgpu::Surface for JUST the center region
     (using set_viewport to limit render area)
  4. wgpu renders character + combat into that region
  5. HTML/CSS renders everything else around it
  6. Communication: Tauri events between webview ↔ wgpu renderer
     → Webview sends: "user clicked equipment slot at (x,y)"
     → Renderer sends: "combat simulation complete, results: {...}"

  WHY THIS IS THE RIGHT APPROACH:
  - Single window (not two separate windows)
  - Best performance for the render area (native GPU, not WebGL)
  - HTML/CSS stays for everything else (we keep all our CSS work)
  - Only the character/combat area uses GPU rendering
  - The render area is small (~400×400px) — even pixel copying
    would be fast (640KB/frame × 60fps = 38MB/s — trivial)
  
  FALLBACK for low-end PCs without GPU:
  - If wgpu fails to initialize → fall back to Canvas 2D in webview
  - Auto-detect on first launch
  - User can override: Settings → Rendering → [Native GPU] [Canvas 2D]

  IMPLEMENTATION:
  ```rust
  // In Tauri setup, create wgpu surface for center panel region
  fn setup_renderer(app: &AppHandle) -> Result<CombatRenderer, RenderError> {
      let window = app.get_webview_window("main")?;
      let raw_handle = window.raw_window_handle();
      
      // Create wgpu instance
      let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
          backends: wgpu::Backends::all(), // Vulkan + Metal + DX12
          ..Default::default()
      });
      
      // Create surface from window handle
      let surface = instance.create_surface_from_window(raw_handle)?;
      
      // Configure for our render region size
      let config = wgpu::SurfaceConfiguration {
          width: 400,   // center panel width
          height: 400,  // center panel height
          format: wgpu::TextureFormat::Bgra8UnormSrgb,
          present_mode: wgpu::PresentMode::Fifo, // vsync
          ..Default::default()
      };
      
      Ok(CombatRenderer::new(instance, surface, config))
  }
  ```

RENDERING CAPABILITIES WITH WGPU:
  - 2D sprite batching (thousands of sprites at 60fps)
  - Skeletal animation (bone system in Rust, GPU-transformed)
  - Particle systems (fire, ice, lightning — GPU compute shaders)
  - Post-processing (bloom, screen-space effects for RF glow)
  - Instanced rendering (100 monsters with one draw call)
  - Custom shaders (WGSL) for PoE-like visual effects
```

### Recommended Implementation Path

```
MVP (Sprint 6-7):
  → Canvas 2D in webview as placeholder
  → Animated SVG character + CSS particles
  → Proves the simulation math works
  → No GPU dependency for MVP

Sprint 8-9 (Month 4-5):
  → Replace center panel with native wgpu surface
  → Hybrid window: HTML/CSS around the edges, wgpu in the center
  → Character sprite rendering at 60fps
  → GPU particle systems (fire, ice, lightning)
  → Equipment visually shown on character

Sprint 10+ (Month 5-6):
  → Skeletal animation system (bone-based character)
  → Boss sprites with full attack animations
  → Map scrolling with tiled backgrounds
  → Post-processing effects (bloom for RF glow)

Post-MVP (Month 8+):
  → 3D character models (if community demand)
  → Isometric camera matching PoE's angle
  → PBR materials for equipment

User setting:
  Settings → Rendering:
    ◉ Native GPU (recommended — best quality)
    ○ Canvas 2D (fallback for PCs without GPU support)
```

### Performance Targets (All Tiers)
```
Simulation tick: 100ms (10 ticks/second) — game logic
Render frame: 
  Tier 1: 30fps target (Canvas 2D, CPU-based)
  Tier 2: 60fps target (PixiJS WebGL, GPU-accelerated)
  Tier 3: 60fps target (Three.js WebGL2/WebGPU)

Monster count on screen: up to 30
Particle count: up to 200 (Tier 1), 1000 (Tier 2), 5000 (Tier 3)
Damage numbers: up to 20 floating simultaneously

Auto-quality detection:
  → On first launch: benchmark GPU capability
  → If WebGL2 available + >30fps: default to Tier 2
  → If WebGL2 slow or unavailable: fallback to Tier 1
  → User can override in Settings
```

### Simulation Accuracy (Same for All Tiers)

```
The rendering tier is VISUAL ONLY — simulation accuracy is IDENTICAL
regardless of whether you see 2D sprites or 3D models.

The simulation uses our EXACT calculator engine for all math:
  - Player DPS: same numbers as shown in the DPS panel
  - Player defense: same mitigation as Defense panel
  - Monster HP/damage: from game data (default_monster_stats.json + boss database)
  - Hit/dodge probability: based on move speed + telegraph duration

What IS simulated (approximate):
  - Dodge chance (probabilistic, not player-skill-dependent)
  - Positioning (simplified — no actual pathfinding)
  - Buff uptime (assumed optimal flask/guard usage)
  - Boss attack selection (follows scripted pattern)

What is NOT simulated:
  - Player mechanical skill (dodging is probability-based)
  - Lag/desync (assumes perfect connection)
  - Actual loot drops (uses average values per map tier)
  - Party play (single player only for now)
```

---

## 6. BOSS DATABASE (Required for Simulation)

We need detailed attack data for each boss. Priority:

### Tier 1 (MVP — must have)
```
Bosses with full attack patterns + phases:
  - Shaper (4 phases, 3 attack types)
  - Elder (3 phases, 4 attack types)
  - Sirus A9 (4 phases, 5 attack types including Die Beam)
  - Maven (3 phases, memory game mechanic)
```

### Tier 2 (Post-MVP)
```
  - Uber Shaper, Uber Elder, Uber Sirus, Uber Maven
  - The Feared (all bosses simultaneously)
  - Cortex, Synthete bosses
  - Atziri, Uber Atziri
  - Breach lords (Chayula, Xoph, etc.)
```

### Tier 3 (Future)
```
  - Map bosses (per map)
  - Expedition bosses
  - Simulacrum wave 25-30 boss
  - Delve bosses
```

---

## 7. INTEGRATION WITH MULTI-STEP UPGRADE PATH

### How Prophecy + Arena Work Together

```
PROPHECY PANEL:
  Shows upgrade suggestions ranked by value
  Each suggestion has "▶ Preview" button
  Multiple can be selected for cumulative preview

  [Suggestion 1] ☑ Ring 2 → Woe Circle     [▶ Preview]
  [Suggestion 2] ☑ Boots benchcraft         [▶ Preview]
  [Suggestion 3] ☐ Gem corruption           [▶ Preview]
  
  [Preview Selected (2)] [Apply Selected (2)]

USER CLICKS [Preview Selected]:
  → Arena panel opens with split view:
     LEFT: current build simulation
     RIGHT: upgraded build simulation
  → Both play simultaneously
  → Difference stats shown below
  → Player can VISUALLY see the improvement

USER CLICKS [Apply Selected]:
  → Step-by-step application (Flow #5 for each)
  → After each step: re-analyze, update all panels
  → Show progress: "Step 1/2 applied. DPS: 2.84M → 3.12M"
  → After all steps: "All upgrades applied! Total: +26.8% DPS"
  → Power-up animation on character

FORESIGHT MODE:
  → User can select ALL upgrade steps (even expensive ones)
  → Preview shows "What your build looks like at the end"
  → Even if user can't afford it yet
  → Shows: "Total cost: 45 divine. You have: 12 divine."
  → "At your farming rate (8 div/hr): ~4 hours to afford all"
  → "Priority order: Step 1 (free) → Step 2 (3 div) → Step 3 (5 div) → ..."
```

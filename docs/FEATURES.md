# PoB Advisor — Complete Feature Specification

## CORE SYSTEMS

### 1. Two-Way PoB Sync Engine
- Auto-detect PoB install path (`%AppData%/Path of Building/`)
- Auto-detect PoB Community Fork vs original
- File watcher with debounce (wait 500ms after last change)
- XML parser for all PoB sections (Build, Tree, Items, Skills, Config, Calcs)
- XML writer with atomic writes (write temp → rename)
- Auto-backup before every write to `%AppData%/PoBAdvisor/backups/`
- Undo/redo history (last 50 changes)
- Preview diff before applying changes (side-by-side before/after)
- File lock detection (don't write while PoB is saving)
- Watch multiple build folders including subfolders
- Detect build file renames, deletes, and moves
- Import from PoB pastebin codes (paste URL → parse → create local build)
- Import from pathofexile.com public profiles
- Import from poe.ninja build pages
- Import from forum guide threads (parse gear/tree/gem sections)
- Import from YouTube video descriptions (detect PoB codes)
- **In-game item import: Ctrl+C item in PoE → paste into Path of AI → instant analysis**
  - Parse PoE's clipboard item format (same as PoB uses)
  - Show: mod tiers, score, DPS impact vs current item, market value
  - "Is this item an upgrade? YES — +8% DPS, +200 life over current Ring 2"
- Export build summary as shareable image
- Export build as Reddit/forum formatted post
- Short URL generation for sharing builds
- **Build share codes** — generate compact code that others can import
- **Build notes** — user can write notes per build (leveling plans, goals, etc.)

### 2. Character Visualization ("The Exile")

Interactive character model displayed in the center of the app, showing:

#### Equipped Gear on Character Body
- SVG character silhouette (Templar/Marauder/Witch/etc based on class)
- Equipment slots positioned on the body (helm on head, body on torso, etc)
- Hovering a slot shows item name + score
- Clicking a slot opens item tooltip in the detail panel
- Item rarity color reflected in slot border glow (gold = unique, yellow = rare)
- Gear changes animate (old item fades, new slides in)

#### Aura & Buff Effects
- Aura rings rotate around the character (one ring per active aura)
  - Determination: grey/blue ring, slow rotation
  - Purity of Fire: red ring, counter-rotation
  - Vitality: green pulsing ring
- Buff icons above character head (floating animation)
  - Shows all active buffs: RF, Molten Shell, Fortify, Onslaught, etc.
  - Hover to see buff name and stats
  - Active RF shows fire glow around character body
- Aura stacking visualization (3+ auras = brighter ambient glow)

#### Cosmetic MTX (via PoE OAuth)
- Sync character appearance from PoE account
- Show equipped cosmetic effects (wings, portals, weapon effects)
- Character portrait from PoE profile
- Requires PoE OAuth connection (Settings → Connect PoE Account)

#### Combat Animation Mode
- In Combat tab ("The Arena"), character animates:
  - Attack animation (shield charge / fire trap throw)
  - Damage numbers pop up on monster
  - Monster HP bar above monster
  - Character takes damage (life bar decreases, then regens)
  - Molten Shell activation on big hit
  - Death animation if simulated kill fails
- Boss encounters show boss sprite next to character
- Map clear shows rapid monster kills with currency drops

### 3. DPS Calculation Engine (Dual — Our Rust Calc + PoB Verification)

> See [ENGINE-DESIGN.md](ENGINE-DESIGN.md) for the full dual calculator architecture.
> Our Rust engine is PRIMARY. PoB Lua is OPTIONAL verification.

#### Fast Estimation (instant, <50ms)
Percentage-based approximation using mod impact database:

```
Mod Type                 Calculation Method
──────────────────────── ──────────────────────────────────────
Fire DoT Multiplier      More multiplier: (current + new) / current
+Gem Level               Lookup table per gem per level
% Increased Damage       Additive: (100 + current + new) / (100 + current)
+Flat Damage             base × attack_speed × multipliers
Attack/Cast Speed        Linear: (current + new) / current
Crit Chance              (new_effective_crit / old) × crit_portion_of_dps
Crit Multiplier          Linear if critting
Penetration              Enemy-res dependent calculation
+Level of Skill Gems     Per-gem scaling table
More multipliers         Exact: multiply into chain
Flat Life                value × (1 + %increased_life / 100)
% Increased Life         base_life × (value / 100)
Resistance               Binary: capped or not, overcap value
```

Shows "~estimated" label. Accuracy: 85-95% for most mods.

#### Exact Calculation (on demand, 200-500ms)
Bundle LuaJIT runtime (~500KB) with PoB's open source calc engine:
- CalcPerform.lua, CalcDefence.lua, CalcOffence.lua, CalcBreakdown.lua
- Feed modified XML → get exact stats back
- 100% accurate — same engine as PoB
- Used for "Calculate Exact" button and "Apply to PoB" preview

#### Impact Display
Every suggestion shows both:
```
Replace Ring 2
  DPS:  2,841,057 → 3,077,797 (+8.3%)     [exact ✓]
  Life: 6,453 → 6,551 (+98)                [exact ✓]
  Fire Res: 80% → 88% (+8% overcap)        [exact ✓]
  
  [Apply to PoB]  [Search Market]  [Undo]
```

#### Cost Efficiency Ranking
Every upgrade ranked by value:
```
#1 Boot benchcraft:  +70 life / 0 div = FREE
#2 Helmet enchant:   +8% DPS / 0 div = FREE (run lab)
#3 Gem corruption:   +10% DPS / 1 div = 10% per div
#4 Ring 2 replace:   +8.3% DPS / 3 div = 2.8% per div
#5 Amulet upgrade:   +15% DPS / 12 div = 1.25% per div
```

### 3. Build Analysis Engine
- Parse all PlayerStat values
- Calculate effective HP (life × mitigation layers)
- Track DPS breakdown by damage type
- Detect build archetype automatically (attack/spell/dot/minion/totem/trap/mine/brand/channeling)
- Score build on 0-100 scale across categories (defense, offense, utility, completeness)
- Detect CI (Chaos Inoculation), LL (Low Life), EB (Eldritch Battery), MoM (Mind over Matter)
- Detect Blood Magic, Reserved Life builds
- Damage conversion chain validation

### 4. Game Mode Awareness
- Detect HC/Hardcore mode — apply stricter defense thresholds (death = character gone)
  - HC minimum: 6000+ life, all resists overcapped, ailment immune, stun immune
  - Flag ANY one-shot risk as critical
- Detect SSF mode — disable trade suggestions, focus on self-found crafting paths and drop targeting
  - Suggest div card farming for target uniques
  - Suggest vendor recipes
  - Suggest crafting paths using available currency only
- Detect Ruthless mode — adjust item economy expectations
- Private league detection
- PoE 1 vs PoE 2 support/detection and separate mod databases

---

## DEFENSIVE ANALYSIS

### Resistance Checker
- Elemental resist cap detection (75% base, higher with max res)
- Max resist tracking (+max fire from Rise of Phoenix, Purity, etc.)
- Overcap calculation (how much buffer for -res curses)
- Chaos resist warning tiers: <0 danger, 0-50 okay, 50-75 good, 75 capped
- Curse penalty awareness (-24% ele res from Elemental Weakness in maps)
- Exposure/penetration vulnerability detection
- "You need X more cold res to survive Elemental Weakness maps"
- Map mod "-max res" impact calculation
- "With -12% max res map mod, your RF degen increases by X%"

### Effective HP Calculator
- Raw life pool
- Energy shield (hybrid or CI detection)
- Armour-based physical reduction (with PoE diminishing returns formula)
- Evasion chance + entropy calculation
- Block/spell block contribution
- Guard skill uptime estimation (Molten Shell, Steelskin, Immortal Call)
- Fortify detection and uptime
- Petrified Blood / Progenesis interactions
- Ward calculation
- Wind Dancer / Kintsugi dodge layer
- "Your effective physical HP vs a 5000 hit = X"
- "Your effective elemental HP vs Shaper slam = X"
- "Your effective chaos HP vs Al-Hezmin = X"
- Damage taken as X% calculations (Lightning Coil, Taste of Hate)

### Ailment Immunity Checklist
```
✅ Freeze immune (source: Brine King + boot craft)
❌ Shock — VULNERABLE (no source detected)
  Fix: Tempest Shield / boot craft / ascendancy
✅ Ignite immune (source: ascendancy)
❌ Bleed — VULNERABLE (need corrupted blood jewel)
  Fix: Corrupted Blood immune jewel corrupt
❌ Poison — not immune (low priority for your build)
✅ Corrupted Blood — jewel detected
❌ Stun — no stun immunity (dangerous for RF)
  Fix: Unwavering Stance / Brine King / boot implicit
❌ Curse immune — no source detected
  Fix: Curse immunity flask suffix / Atziri's Reflection
❌ Hinder — no source detected
❌ Maim — no source detected
```

### Damage Taken Simulation
- Simulate specific boss hits against your defenses
- "Shaper Slam (physical): you survive with 1240 life remaining"
- "Sirus Die Beam (multi-hit): you die in 0.8 seconds"
- "Maven Memory Game failure: lethal"
- "Uber Elder tentacle slam: you die"
- "The Feared (all bosses): survival chance 60%"
- Compare pre/post upgrade survivability
- Factor in flasks, guard skills, ascendancy, pantheon
- "With Molten Shell active: you survive Shaper Slam with 4200 remaining"

### Combat Simulator ("The Arena")

Full fight simulation engine that models real combat scenarios — not just
single-hit survival, but actual fight duration, DPS check phases, and
how upgrades translate to faster/safer kills.

#### Map Monster Combat
```
Current Build vs T16 Rare Monster (4M life, 30% res):
  ┌──────────────────────────────────────────────────┐
  │  YOUR DPS:  2.84M  │  MONSTER HP: 4,000,000     │
  │                                                  │
  │  ████████████████████░░░░░░░░  2.84M / 4M       │
  │                                                  │
  │  Time to Kill:  1.6 seconds                      │
  │  Hits to Kill:  3 hits (fire trap) + RF burn     │
  │                                                  │
  │  WITH Ring 2 Upgrade (+15% DPS → 3.27M):         │
  │  Time to Kill:  1.4 seconds  (▼ 0.2s faster)    │
  │  Hits to Kill:  2 hits + RF burn                 │
  │                                                  │
  │  WITH Full Upgrade Path (+34% DPS → 3.82M):      │
  │  Time to Kill:  1.1 seconds  (▼ 0.5s faster)    │
  │  Hits to Kill:  2 hits + RF burn                 │
  └──────────────────────────────────────────────────┘
```

#### Boss Fight Simulation
```
═══════════════════════════════════════════════════
  BOSS ARENA: Shaper (20M HP, 4 phases)
═══════════════════════════════════════════════════

  Phase 1 (100% → 75%):
    Your DPS: 2.84M (effective after 40% boss res)
    Time: 1.76M effective → ~2.8s per phase
    Danger: Slam (survive ✅), Beam (dodge required)

  Phase 2-3 (75% → 25%): Same + Zana bubble phase
    Added time for immunity phases: +15s total

  Phase 4 (25% → 0%):
    Bullet hell + clone phase
    Your regen: 2450/s → can tank minor hits ✅
    One-shot risk: Slam = survive, Double slam = DIE ❌

  ┌──────────────────────────────────────────────┐
  │  TOTAL FIGHT ESTIMATE                        │
  │                                              │
  │  Current build:  ~3 min 20s  (3 deaths avg)  │
  │  With upgrades:  ~2 min 30s  (1 death avg)   │
  │  Top RF builds:  ~1 min 45s  (deathless)     │
  │                                              │
  │  BOTTLENECK: DPS (phase transitions slow)    │
  │  FIX: Ring 2 + Amulet upgrade = -40s         │
  └──────────────────────────────────────────────┘
```

#### Uber Boss Simulation
```
═══════════════════════════════════════════════════
  BOSS ARENA: Uber Shaper (75M HP, 50% more dmg)
═══════════════════════════════════════════════════

  ❌ NOT VIABLE — estimated 8+ deaths

  Reasons:
  1. Slam: LETHAL (deals 12,400, you have 6,453 life + 4,200 Molten Shell)
     → Need: Aegis Aurora or +max res
  2. DPS too low for phase burn
     → 1.7M effective = ~11s per phase (get hit too many times)
  3. Beam: LETHAL in 0.4s (Uber does 50% more)

  MINIMUM UPGRADES NEEDED:
  - Aegis Aurora (+ES on block = survive slam)    15-25 div
  - +1 max fire res from tree                     2 passive points
  - DPS to 4M+ (ring + amulet + gem corrupt)      20-30 div
  → After upgrades: ~5 min fight, 0-2 deaths (viable!)
```

#### Map Clear Speed Estimate
```
  MAP: Strand T16 (linear, ~400 monsters)

  Current build (2.84M DPS, 45% MS):
    Clear style: Walk + RF burns → Fire Trap rares
    Est. clear time: ~2 min 30s
    Monsters per second: ~2.7
    Currency/hour: ~8 div (with atlas strategy)

  With +30% MS boots + DPS upgrade:
    Est. clear time: ~1 min 50s  (▼ 40s faster)
    Monsters per second: ~3.6
    Currency/hour: ~11 div  (+37% more efficient)

  Comparison to meta builds:
    Top RF Inquis: ~1 min 30s  (you're 67% slower)
    Lightning Arrow: ~0 min 50s (different class)
```

#### Simulation Parameters
- Monster HP scaled by map tier (T1 = 100%, T16 = 1200%)
- Boss HP from known values (Shaper 20M, Maven 30M, Uber Shaper 75M, etc.)
- Boss resistance applied (40% base, -exposure, -curses)
- Phase transition immunity windows factored in
- Player movement speed for clear time estimation
- Death penalty: 10% XP loss + portal re-entry time
- Factor in flask uptime, guard skill uptime, buff uptime
- Compare current vs upgraded vs top-build performance

#### How Combat Simulation Uses Upgrades
Every upgrade suggestion gets a "fight impact" metric:
```
  Ring 2 Upgrade (3-8 div):
    DPS:          +15% → 2.84M → 3.27M
    Shaper fight:  3:20 → 2:55  (saves 25 seconds)
    T16 clear:     2:30 → 2:15  (saves 15 seconds)
    Deaths/Shaper: 3 avg → 2 avg

  Aegis Aurora (15-25 div):
    Survivability: +massive (ES on block = 2000+/hit)
    Shaper fight:  3:20 → 3:10  (only -10s, but 0 deaths)
    Deaths/Shaper: 3 avg → 0 avg
    Uber viable:   NO → YES
```

### Recovery Analysis
- Life regen per second (critical for RF)
- Life regen vs RF degen balance
- Net regen calculation after all degens
- Leech rate and cap (20% default, Vaal Pact doubles)
- Life gain on hit sources and effectiveness
- Flask recovery per use and over time
- ES recharge rate and delay
- "Your net regen is +450/s — safe margin for RF"
- "Adding Vitality would push regen to +620/s"
- "During no-regen maps: you cannot sustain RF — DANGER"
- Regen breakpoints for RF/Death Aura/etc.

---

## OFFENSIVE ANALYSIS

### DPS Breakdown
- Total DPS by damage type (physical, fire, cold, lightning, chaos)
- Total DoT DPS by type (ignite, bleed, poison, burning, cold dot)
- DPS per gem link analysis (which setup contributes most)
- DPS with/without flasks (realistic vs tooltip)
- DPS with/without conditional buffs (onslaught, frenzy charges, power charges)
- Boss DPS vs map clear DPS
- Single target vs AoE damage ratio
- Damage over time tracking (ignite, bleed, poison stacks, wither stacks)
- Hit rate / attacks per second
- Damage per hit (for one-shot potential)
- "Your fire trap contributes 65% of single target DPS"
- "Your RF is 35% of DPS but 90% of clear"
- Damage effectiveness comparison

### Calculation Breakdown ("Show The Math")
Users need to SEE how numbers are derived — not just final results.
This is what makes PoB trusted. We must have the same transparency.
```
Righteous Fire DPS Breakdown:

  Base Burning DPS:              1,245
  × Increased Fire Damage (420%): ×5.20    = 6,474
  × Burning Damage Support (59%): ×1.59    = 10,294
  × Elemental Focus (34% more):  ×1.34    = 13,794
  × Swift Affliction (29% more): ×1.29    = 17,794
  × Efficacy (24% more):         ×1.24    = 22,065
  × Lifetap (20% more):          ×1.20    = 26,478
  × DoT Multiplier (180%):       ×2.80    = 74,138
  × Enemy Fire Res (-23.5%):     ×1.235   = 91,560

  Per-second DPS:                          91,560 (per RF instance)
  × RF coverage (AoE hits all):  ×31 monsters avg = 2,838,360 clear DPS
  
  Single target: 91,560 → but with Fire Trap: +682,000 = 773,560 total
  
  [Click any line to see what items/passives contribute to it]
```
- Every line clickable → shows which items/passives/gems give that modifier
- Change detection: "If you remove Burning Damage Support, DPS drops to X"
- Color-coded: green = this line adds most value, red = least value
- Toggle: show with/without specific buffs (flasks on/off, charges on/off)

### Gem Optimization
- Support gem DPS comparison (swap X for Y = +Z% DPS via exact calc)
- Gem level breakpoints ("21/20 RF = +18% more DPS than 20/20")
- Quality breakpoints ("23% quality > 20% for this gem, via Hillock")
- Awakened gem upgrade path with exact DPS gain and market cost
- Vaal gem suggestions (which skills benefit from Vaal version)
- Empower/Enhance/Enlighten level impact (exact calc per level)
- Alt quality gem comparison (Divergent vs Anomalous vs Phantasmal)
- Gem swap suggestions for different content (mapping vs bossing)
- "Best 6th link for your RF: Efficacy > Conc Effect > Inc AoE"
- "Swapping to Conc Effect for bosses: +22% DPS, -30% AoE"
- 21/23 corruption value per gem (is it worth the risk?)

### Transfigured Gem Variants
- Compare all transfigured versions of your main skill
- Show DPS + mechanic differences for each variant
- "RF of Arcane Devotion: -14% DPS BUT +30% spell damage + life as ES"
- Highlight which variant is best for your specific build archetype
- Factor in support gem compatibility changes between variants
- Auto-detect if a transfigured version unlocks new build possibilities

### Toggle-able Skills & Buffs
- Enable/disable individual auras to see reservation + stat impact
- Toggle flasks on/off for "realistic DPS" vs "tooltip DPS"
- Toggle charges (frenzy/power/endurance) on/off per scenario
- Toggle conditional buffs (Onslaught, Unholy Might, Tailwind, etc.)
- "Map DPS" preset: all buffs on, charges on, flasks on
- "Boss DPS" preset: conditional buffs off, on-kill effects off
- Each toggle shows exact DPS/defense change in real-time
- Auto-detect which buffs your build can realistically maintain

### Automatic Socketed Gem Modifier Application
- Detect "+X to level of socketed gems" on items
- Auto-apply these to gems in that socket group
- Show which items boost which gems: "Helmet +2 fire gems → RF 21→23"
- "If you socket RF in this helmet, it gains +2 levels = +18% DPS"
- Warn if moving gems between items loses socketed bonuses

### Skill Link Suggestions
- Detect suboptimal support gems (using exact DPS calc for every possible swap)
- Suggest better combinations
- Calculate link priority (which socket group benefits most from upgrade)
- Trigger setup detection and optimization (CWDT level vs gem level matching)
- CWDT threshold calculation ("CWDT level 1 triggers at 528 damage taken")
- Aura efficiency analysis (enlighten savings, reservation percentages)
- Aura stacking detection and optimization
- Banner / War Cry / Mark suggestions

### Flask Optimization
- Detect missing critical flasks
- Suggest flask suffixes per build (bleed immune, freeze immune, curse immune)
- Unique flask recommendations with DPS/defense impact
- Flask uptime calculation (charges gained vs used, with Pathfinder/Raider)
- Flask effect duration
- Forbidden Taste / Coruscating Elixir danger detection
- "You have no anti-bleed flask — high risk in maps"
- "Dying Sun would give +2 RF radius and +fire res"
- "Bottled Faith: +15% DPS but costs 25div — worth it?"
- Mageblood / Headhunter belt interaction

### Critical Strike Analysis
- Effective crit chance (with accuracy, diamond flask, power charges)
- Crit multiplier stacking efficiency
- Diamond flask impact calculation
- Power charge value per charge
- Assassin's Mark impact
- Bottled Faith crit ground effect
- "You're at 68% effective crit — investing more has diminishing returns"
- "Switching to Increased Critical Damage gives +9% DPS"
- "Precision accuracy value: +3.2% effective DPS from hit rate"
- Crit vs non-crit build comparison ("would going crit be better?")

---

## MANA MANAGEMENT

### Mana Reservation Calculator
- Total mana reserved (flat + percentage)
- Unreserved mana remaining
- "You have 102 unreserved mana — enough for Shield Charge"
- "Adding Herald of Ash: not possible, only 102 mana free"
- "Enlighten 4 would free up 87 more mana"
- Reservation efficiency optimizer
- Aura fitting: "you can fit one more 25% aura if you get -mana reserve helmet"

### Mana Sustain
- Mana cost per skill use
- Mana regen vs cost analysis
- Mana leech rate
- -mana cost craft suggestions
- Eldritch Battery detection (ES → mana)
- Blood Magic detection (life → mana)
- Lifetap support mana bypass detection
- "Your main skill costs 42 mana, you regen 35/s — you'll run out during sustained DPS"
- "Craft -8 mana cost on ring to fix"

---

## ATTRIBUTE REQUIREMENTS

### Attribute Checker
- Str/Dex/Int requirements for all equipped gear
- Str/Dex/Int requirements for all active gems
- Current attribute totals vs requirements
- "You can't equip this helmet — need 15 more Strength"
- "Removing that +30 Dex passive drops you below boot requirement"
- "Your Flame Dash needs 111 Int — you have 108, it will fail at level 20"
- Attribute stacking build detection (Whispering Ice, Brutus Lead Sprinkler)

### Attribute Planning
- Show cheapest way to meet attribute requirements
- "Add +30 Dex node (1 passive point) vs craft Dex on ring (free)"
- Amulet implicit for attributes
- Passive tree attribute pathing

---

## PASSIVE TREE ANALYSIS

### Node Efficiency Scoring
- Score = total stats gained / points spent to reach
- Flag low-efficiency travel nodes
- Find dead nodes (giving stats your build doesn't use)
- "Node 'Coordination' gives +10 dex — you don't need dex, respec this"
- Cluster jewel vs tree node comparison (which is more efficient)
- Notable vs small node value analysis

### Path Optimization
- Find shorter paths between keystones
- Detect unnecessary travel nodes
- "Respec these 3 nodes, take this path instead: same destination, save 2 points"
- Suggest where to spend saved points (with exact DPS/life impact)
- Show top 5 most impactful unallocated nodes within reach
- Thread of Hope / Intuitive Leap range analysis
- Impossible Escape interaction detection

### Keystone Analysis
- Impact simulation for each keystone (exact calc via PoB engine)
- "Taking Elemental Overload: -0% DPS (you have no crit anyway), good choice"
- "Removing Resolute Technique: +22% DPS if you add accuracy"
- "Chaos Inoculation: removes 6453 life, but with your ES..."
- Ascendancy node comparison with exact numbers
- "Pious Path > Sanctuary for your build (+340 ES regen)"
- Forbidden Flame/Flesh suggestions (steal ascendancy notables)

### Mastery Suggestions
- Check if all available masteries are taken
- Rank masteries by impact for your build (exact calc)
- "Fire Mastery: +20% burning damage > +1 fire exposure"
- Flag mastery conflicts (can only pick one per cluster)
- Show all available masteries you haven't taken

### Jewel Socket Analysis
- Value of each jewel socket (worth pathing to? exact calc per socket)
- Best jewel stats for your build ranked
- Timeless jewel impact estimation (Elegant Hubris, Lethal Pride, Glorious Vanity, Militant Faith, Brutal Restraint)
- Cluster jewel notable rankings with DPS impact
- "This jewel socket costs 3 points to reach — needs a jewel worth 3+ passive points"
- Forbidden Flame/Flesh suggestions based on other ascendancies
- Watcher's Eye mod finder (see dedicated section)

### Next N Points Planner
- "Your next 5 points should go to:" (with exact calc per option)
  1. Life wheel near Marauder (2 points, +340 life, +0% DPS)
  2. Fire DoT mastery (1 point, +0 life, +8% DPS)
  3. Jewel socket (2 points, flexible, depends on jewel)
- Level-by-level passive guide from current level to 100
- Respec plan with point-by-point instructions
- "Full respec costs 24 regret orbs — here's the optimal new tree"
- Multiple respec PLANS (Tank vs DPS vs Balance) — user picks

### Full Passive Tree Visualization

Interactive passive tree viewer styled like PoE's in-game passive tree:

#### Visual Design
- Full tree rendered as SVG/Canvas with all ~1,300+ nodes
- Nodes positioned using the official tree layout data (from PoB/RePoE)
- Allocated nodes: bright gold/yellow with glow
- Unallocated nodes: dim grey
- Class start position highlighted
- Ascendancy nodes shown in a sub-panel
- Zoom + pan (mouse wheel + drag)
- Minimap in corner showing full tree with viewport rectangle

#### Interactive Features
- Hover node → tooltip showing stats it gives
- Click node → show DPS/life impact if allocated
- "What if" mode: click nodes to simulate allocation without applying
- Color-code nodes by value: green = high value, red = low value for YOUR build
- Show recommended path highlighted (gold dotted line)
- Show inefficient nodes highlighted (red border)
- Compare to top poe.ninja builds (overlay their tree as ghost nodes)

#### Integration with Calculator
- Every node click → full recalc → exact stat diff shown
- "Top 10 unallocated nodes for DPS" highlighted on tree
- "Top 10 unallocated nodes for survivability" highlighted on tree
- Keystone analysis: hover keystone → show full impact before taking it
- Cluster jewel sockets: show what cluster setup would be optimal

### Anoint Planner
- Rank all anoints by build impact (exact calc)
- Oil cost calculation per anoint
- "Best anoint: Whispers of Doom (2× Golden + Silver) = run extra curse"
- Compare anoint value vs pathing to same node on tree
- "Anointing Breath of Flames saves you 4 passive points"
- Blight ring anoint suggestions

### Cluster Jewel Builder
- Recommend cluster jewel base type and item level for your build
- Rank all notables by DPS/defense value (exact calc)
- Large → Medium → Small jewel planning
- Crafting guide for target notables (fossil/alt spam/harvest)
- Expected cost to craft vs buy
- "Best Large cluster: 12% Fire Damage, notables: Prismatic Heart + Widespread Destruction"
- "Best Medium cluster: Fire DoT Multi, notables: Blowback + Burning Bright"
- Megalomaniac notable combination search

---

## ITEM ANALYSIS

### In-Game Item Import (Ctrl+C → Paste)
The #1 most-used PoB feature. Player copies item from PoE game → pastes → instant analysis.
```
Player hovers item in PoE, presses Ctrl+C, then pastes into Path of AI:

  ┌─────────────────────────────────────────────┐
  │  📋 Paste Item (Ctrl+V)                     │
  │                                             │
  │  Parsed: Opal Ring (ilvl 84, Rare)          │
  │                                             │
  │  +92 to maximum Life          T1 ★          │
  │  +18% to Fire DoT Multi      T2 ★          │
  │  +38% to Fire Resistance     T2             │
  │  +28% to Cold Resistance     T3             │
  │  +12% to Lightning Res       T4             │
  │  (crafted) 5% inc max Life   benchcraft     │
  │                                             │
  │  Score: 87/100                              │
  │                                             │
  │  ═══ vs Your Current Ring 2 (42/100) ═══    │
  │  DPS:  +15.3% (2.84M → 3.28M)              │
  │  Life: +350 (+92 vs +45)                    │
  │  Resists: OK (still capped)                 │
  │                                             │
  │  VERDICT: ★ SIGNIFICANT UPGRADE             │
  │  Market value: ~5 divine                    │
  │                                             │
  │  [Equip in PoB] [Search Similar] [Dismiss]  │
  └─────────────────────────────────────────────┘
```
- Parse PoE clipboard format (same format PoB uses)
- Auto-detect which slot the item goes in
- Show mod tiers with color-coding
- Compare vs currently equipped item (exact DPS/life diff)
- Show market value from poe.ninja
- One-click equip in PoB (write to XML)
- Global hotkey: Ctrl+Shift+V to paste from anywhere while app runs in background

### Searchable Unique Item Database
- Search all ~1,200+ unique items by name, base type, or mod keywords
- Filter by: slot, level requirement, price range, build relevance
- Each unique shows: all mod rolls (min-max), DPS impact for YOUR build, current market price
- "Search: fire damage shield" → Rise of Phoenix, Saffell's Frame, Aegis Aurora
- Compare uniques: "Rise of Phoenix vs Aegis Aurora for your build"
- Modifier roll selector: "Your Aegis Aurora has +18 max ES (range: 10-20)"
- Show divination cards that award each unique + drop locations
- Highlight league-specific and legacy variants

### Item Crafting Simulator (Live Editor)
Interactive item editor — add/remove mods like PoB's item creator.
```
  ┌──────────────────────────────────────────┐
  │  CREATE ITEM — Opal Ring (ilvl 84)       │
  │                                          │
  │  Implicit: 25% inc Elemental Damage      │
  │                                          │
  │  Prefix 1: [+92 max Life        ▼] T1   │
  │  Prefix 2: [+18% Fire DoT Multi ▼] T2   │
  │  Prefix 3: [empty — click to add ▼]     │
  │                                          │
  │  Suffix 1: [+38% Fire Res       ▼] T2   │
  │  Suffix 2: [+28% Cold Res       ▼] T3   │
  │  Suffix 3: [crafted: 5% max life▼]      │
  │                                          │
  │  Score: 87/100                           │
  │  DPS vs current: +15.3%                  │
  │  Market price: ~5-8 divine               │
  │                                          │
  │  [Equip] [Search Trade] [Save Template]  │
  └──────────────────────────────────────────┘
```
- Dropdown for each affix slot → browse all possible mods for this base+ilvl
- Roll value slider (min to max of tier)
- Auto-calc DPS impact as you change mods
- Save item templates ("dream ring", "budget helmet")
- "What would a PERFECT item look like?" auto-fill with T1 everything

### Mod Tier Detection
- Identify tier of every mod (T1 life = +90-99, T2 = +80-89, etc.)
- Color-code by tier quality (gold T1, green T2, blue T3, white T4, gray T5+)
- "Your helmet has T3 life — a T1 roll would give +21 more life"
- Flag bricked items (bad mod combinations, wrong influence)
- Hybrid mod detection (armour/life, ES/life, etc.)
- Crafted mod detection (benchcraft vs dropped)
- Fractured mod detection
- Synthesized implicit detection

### Open Affix Detection
- Count prefixes and suffixes accurately
- Detect crafted mods (count as prefix or suffix)
- "Your boots have an open suffix — craft movement speed!"
- "Your ring has open prefix — craft life or flat fire damage"
- Suggest best benchcraft for each open slot (ranked by impact with exact calc)
- Detect if item is full (no crafting possible)
- Multimod availability check
- "This item has 2 open suffixes — you could multimod for 2 crafted mods"

### Crafting Planner
- Step-by-step crafting plan for each slot
- Benchcraft recommendations per item with exact stat impact
- Harvest craft possibilities and probabilities
- Essence slam targets and best essence per slot
- Fossil combinations and expected outcomes
- Eldritch implicit suggestions (Searing Exarch / Eater of Worlds)
- "Your helmet can have Eldritch fire exposure implicit — +12% DPS"
- Veiled mod suggestions and where to farm them
- Recombinator strategies for combining good items
- Fracture target analysis ("fracture the T1 life, then spam for resists")
- Expected cost to craft vs buy comparison
- "Craft this yourself: ~5div avg (30 attempts), or buy finished: 12div"
- Crafting step simulator with probability calculator
- Rog crafting optimization suggestions

### Smart Crafting Advisor ("The Forge")

Analyzes the player's actual currency inventory and suggests the best craft
they can do RIGHT NOW — with step-by-step instructions and success probability.

#### Currency-Aware Craft Suggestions
```
Your Currency:
  340 Chaos · 87 Alchemy · 142 Scouring · 680 Fusing · 23 Vaal
  45 Regal · 18 Blessed · 5 Annulment · 8 Exalted
  Essences: 12× Wrath, 8× Anger, 5× Rage, 3× Misery
  Fossils: 6× Pristine, 4× Scorched, 3× Metallic, 2× Frigid

═══════════════════════════════════════════════════
  THE FORGE — Crafts You Can Do Right Now
═══════════════════════════════════════════════════

  #1: BEST VALUE — Ring 2 Replacement via Essence
  ┌──────────────────────────────────────────────┐
  │  Target: Opal Ring with Life + Fire DoT Multi│
  │  Method: Essence of Anger spam               │
  │  You have: 8× Essence of Anger (enough!)     │
  │                                               │
  │  Step 1: Buy ilvl 84+ Opal Ring base (1c)    │
  │  Step 2: Essence of Anger → guaranteed        │
  │          fire damage mod                      │
  │  Step 3: Look for open suffix → craft        │
  │          fire DoT multi or life benchcraft    │
  │                                               │
  │  Success rate: ~30% to get usable ring        │
  │  Expected attempts: 3-4 essences              │
  │  Expected cost: 4 Anger + 1c base = ~2 div   │
  │  vs Buy finished: 5-8 div on trade            │
  │                                               │
  │  DPS impact: +12-18%                          │
  │  VERDICT: CRAFT IT — saves 3-6 divine         │
  └──────────────────────────────────────────────┘

  #2: FREE — Benchcraft on Boots
  ┌──────────────────────────────────────────────┐
  │  Your Boots have 1 open prefix               │
  │  Benchcraft: +70 to maximum Life             │
  │  Cost: 4 Orbs of Alteration (you have 200+)  │
  │  Impact: +70 life instantly                   │
  │  VERDICT: DO IT NOW — literally free          │
  └──────────────────────────────────────────────┘

  #3: MEDIUM VALUE — Helmet Fossil Craft
  ┌──────────────────────────────────────────────┐
  │  Target: Helmet with Life + Fire Res + nearby │
  │  Method: Pristine + Scorched fossil combo     │
  │  You have: 6× Pristine, 4× Scorched          │
  │                                               │
  │  Step 1: Buy Royal Burgonet ilvl 84+ (1c)    │
  │  Step 2: Pristine + Scorched in 2-socket      │
  │          resonator                            │
  │  Step 3: Look for T1-T2 life + fire mods     │
  │                                               │
  │  Success rate: ~15% per attempt               │
  │  Expected attempts: 4-6 fossils used          │
  │  Expected cost: ~3 div in fossils             │
  │  vs Buy finished: 5-8 div                     │
  │  VERDICT: CRAFT IT — good use of your fossils │
  └──────────────────────────────────────────────┘
```

#### Craft vs Buy Decision Engine
For every slot that needs upgrading:
```
Ring 2 Upgrade Options:
  ┌────────────────────────────────────────────────────┐
  │                          CRAFT         BUY         │
  │  Expected cost:          ~2 div        5-8 div     │
  │  Time investment:        10 min        instant     │
  │  Risk:                   may brick     guaranteed  │
  │  Best case:              GG ring       good ring   │
  │  Worst case:             waste 4 ess   just works  │
  │                                                    │
  │  YOUR SITUATION:                                   │
  │  Budget: 47 div (plenty)                           │
  │  Essences: 8× Anger (enough for 2-3 attempts)     │
  │  Recommendation: TRY CRAFT FIRST                   │
  │  If fail after 4 attempts → buy from trade         │
  └────────────────────────────────────────────────────┘
```

#### Crafting Method Comparison
Show all possible methods ranked by cost-efficiency:
```
Methods to craft +1 Fire Gem Amulet:

  #1 Essence of Rage — 1 in 3 chance, ~1.5 div avg
      ✓ You have 5× Rage (enough!)
  
  #2 Harvest "reforge fire" — 1 in 8 chance, ~3 div avg
      ⚠ Need to find Harvest fire craft
  
  #3 Alt + Regal spam — 1 in 340 chance, ~4 div avg
      ✓ You have 200+ alts
  
  #4 Fossil (Scorched) — 1 in 12 chance, ~2 div avg
      ✓ You have 4× Scorched

  Recommended: Essence of Rage (cheapest, you have it)
```

#### Real-Time Craft Tracking
When player starts crafting:
- Track attempts, currency spent, results
- "Attempt 3: +88 Life (T2), +35 Fire Res (T2) — GOOD! Regal?"
- Running total: "Spent so far: 3 Essence of Anger (1.5 div)"
- Compare to expected: "On track (expected 3-4 attempts)"

#### SSF Mode Crafting
In SSF, no trade option → crafting is the ONLY way to upgrade:
- Prioritize crafts using currency you actually have
- Suggest vendor recipes to generate needed currency
- "Sell 3× Two-Stone Ring to vendor → prismatic catalyst"
- Div card farming targets for crafting currency

### Corruption Suggestions
- High-value corruption outcomes per item slot
- Risk assessment (what you lose vs gain, probability)
- "+2 to duration gems on your body armour = +25% DPS (but 75% chance to brick)"
- "+2 to AoE gems = +18% DPS"
- Implicit corruption tier list per slot
- Double corrupt temple value assessment
- "Your Aegis Aurora double corrupt: +2 aura gems + max res = GG"

### Enchantment Suggestions
- Lab enchant priority per build (ranked by DPS impact via exact calc)
- Helmet enchant rankings for your main skill
- Boot enchant suggestions (life regen, movement speed, pen, leech)
- Glove enchant suggestions (of reflection, of spite, commandment)
- "RF helmet enchant: +40% RF damage (+8% DPS) > +RF AoE for your build"
- Lab farming strategy if enchant is valuable enough

### Socket/Link Analysis
- Check if links match skill requirements
- Chrome calculator (off-color probability using Vorici calc)
- "Your chest needs 5R1B — use Vorici bench 3R, cost avg: 120 chromes"
- 6-link priority assessment
- Jeweller/fusing expected cost to 6-link
- Corrupted socket crafting (Vorici bench in Research)
- White socket value assessment
- "6-link your chest first (est: 1200 fusings) before upgrading other slots"

### Item Influence Detection
- Identify influence types (Shaper, Elder, Crusader, Redeemer, Hunter, Warlord)
- Highlight valuable influenced mods available
- Suggest influence crafting strategies
- "Your helmet is Shaper influenced — you can roll Conc Effect support"
- Awakener Orb combination planner
- "Combine Hunter amulet (+1 fire gems) + Crusader amulet (+1 all gems) = GG"
- Maven Orb elevated mod targeting

### Watcher's Eye Finder
- Based on your active auras, rank all Watcher's Eye mods
- "You run Determination + Purity of Fire + Vitality"
- Rank single mods, double combos, and triple combos
- "Best double: Fire DoT Multi (Purity) + Phys Reduction (Determination)"
- Price lookup for specific combinations
- "Triple mod with your auras: 50-200div depending on rolls"
- Prioritize defensive vs offensive mods per build needs

### Minion/Totem/Trap Specific
- Spectre database with recommendations per build
- Minion gear scoring (different stat weights than self-cast)
- Minion survivability check
- Totem placement count optimization
- Trap throw speed breakpoints
- Mine detonation sequence optimization
- Brand attachment and activation frequency
- "Your spectres die to Shaper Slam — need minion life support"
- AG (Animate Guardian) gear suggestions and death risk

---

## INVENTORY INTELLIGENCE

### Stash Tab Scanner (requires PoE OAuth)
- Index all items across all stash tabs
- Find upgrades you already own and aren't using
- "You have gloves in Tab 'Gear' that give +200 life over current pair"
- Cross-character item awareness
- Detect items that fit other characters better
- Quad tab dump tab analysis

### Currency Tracker
- Count all currency types across all stash tabs
- Convert everything to divine/chaos equivalent using live rates
- Track currency changes over time (graph)
- "Total liquid currency: 47.5 divine orbs"
- Budget allocation suggestions based on build needs
- Currency breakdown pie chart
- "You have 340 chaos orbs — convert to 4 divines at current rate"

### Sellable Item Detector
- Find valuable items you're not using
- Price estimation for stash items via poe.ninja/trade
- "You have a Watcher's Eye worth ~15div sitting in your dump tab"
- Identify items worth listing on trade
- Generate trade listing with suggested price
- "Total sellable value in stash: ~32 divines"

### Div Card Tracker
- Track partial div card sets across all stash tabs
- Calculate completion value vs sell-now value
- "You have 4/6 The Doctor cards — 2 more = Headhunter"
- "Sell 4 cards now for 120div, or gamble for HH (180div)"
- Div card drop location reminders

### Fragment/Splinter Counter
- Track all map fragments across stash tabs
- Splinter to emblem/breachstone progress
- "You have 87/100 Simulacrum splinters — 13 more to run"
- "You have 4/4 Shaper fragments — ready to run Shaper"
- Scarab inventory with market value

---

## MARKET INTELLIGENCE

### Live Price Checking
- Price every equipped item via poe.ninja/trade API
- "Your total build cost: 45 divine orbs"
- Per-slot price breakdown
- Historical price tracking (graph over league)
- "Your build was worth 80div week 1 — now 45div (prices dropped)"

### Upgrade Shopping
- Find upgrades within budget (using exact DPS calc to verify each item)
- Sort by cost-efficiency (DPS gained per divine spent)
- Direct trade search links (open in browser)
- Price comparison across similar items
- "Best boot upgrade for 5div: +180 life, +30% ms, tri-res = +12% survivability"
- Filter by online sellers only
- Bulk deal detection ("this seller has 3 pieces you need")

### Budget Upgrade Path
```
Budget: 50 divine orbs
Your build total value: 45 divine orbs

Priority 1: Ring 2 — 3div
  → Current DPS: 2,841,057 → 3,077,797 (+8.3%)
  → Current Life: 6,453 → 6,551 (+98)
  → Specific items found: [trade links]

Priority 2: Amulet — 12div
  → DPS: 3,077,797 → 3,539,366 (+15%)
  → Life: 6,551 → 6,591 (+40)
  → Specific items found: [trade links]

Priority 3: Helmet enchant — 0div (run Uber Lab)
  → DPS: 3,539,366 → 3,822,515 (+8%)
  → Free!

Total after upgrades:
  DPS: 2,841,057 → 3,822,515 (+34.5%)
  Life: 6,453 → 6,591 (+138)
  Cost: 15 divine orbs
  Remaining budget: 35div — save for Aegis Aurora
```

### Price Trend Alerts
- Track price history for items you're watching
- "Aegis Aurora dropped 20% this week — good time to buy"
- "Divine orbs rising — sell chaos, buy divines"
- League economy phase detection (day 1-3 chaos rich, week 2+ divine economy)
- "Week 3 of league — prices stabilizing, good time to invest"

### Snipe Alerts
- Set up custom item filters for real-time monitoring
- "Item matching your search posted at 30% below market!"
- Background trade API polling
- Whisper template auto-generation
- Sound/notification on match
- "Ring with +80 life, +fire DoT multi posted for 2div (market avg: 6div)"

### Trade Scam Detection
- Item verification before accepting trade
- Detect common scam patterns:
  - 6-link → 5-link swap
  - Currency swap (exalt → chaos visual similarity)
  - Quality swap (20% → 0%)
  - Socket color swap
- "Warning: verify item sockets before accepting — common scam target"
- Item comparison overlay (listing vs offered)

---

## BUILD PROGRESSION SYSTEM

### Act-by-Act Guide
```
Act 1-3 (Level 1-30):
  ✅ Pick up BBG links for your setup
  ✅ Get Quicksilver from Medicine Chest
  ❌ Missing: buy Goldrim from trade (1 chaos) [SSF: farm from Hillock]
  → Next: Swap to RF at level 28 with +fire res gear
  → Vendor recipe: Sapphire Ring + white boots = cold res boots

Act 4-6 (Level 30-50):
  ✅ Lab ready check (resist cap, life threshold)
  ❌ Need 4-link before Kitava
  → Vendor recipe: sell RGB linked = chromatic
  → First lab: take Augury of Penitence ascendancy

Act 7-10 (Level 50-70):
  ✅ Uber lab viable check
  ❌ Chaos res needed for Act 9+ (Al-Hezmin influence)
  → Priority: cap resists after Kitava -30% penalty
  → Buy: Goldrim → rare helmet with life + res
```

### Mapping Checklist
```
White Maps Ready? (T1-T5)
  ✅ Resist capped
  ✅ 4000+ life
  ❌ Need anti-freeze flask
  ❌ No movement skill detected
  → Atlas: take Essence nodes first

Yellow Maps Ready? (T6-T10)
  ✅ 5000+ life
  ❌ Need chaos res > 0
  ❌ Missing curse immunity
  → Atlas: add Harvest + Expedition nodes

Red Maps Ready? (T11-T16)
  ❌ Need 5500+ life
  ❌ Need ailment immunity plan
  ❌ DPS below 1M threshold for T14+
  → Priority: 6-link, gem levels, cluster jewels
```

### Boss Readiness
Per-boss gear/stat requirements with exact damage simulation:
```
Shaper:      ✅ READY (survive slam with 1240 remaining)
Elder:       ✅ READY (DPS sufficient for phases)
Uber Elder:  ⚠️ RISKY (no freeze immunity for Elder phases)
Sirus:       ✅ READY (survive Die Beam for 2.1s)
Maven:       ❌ NOT READY (chaos res too low for memory game)
The Feared:  ❌ NOT READY (DPS too low, fights overlap)
Uber Shaper: ❌ NOT READY (one-shot by Uber Slam)
Uber Sirus:  ❌ NOT READY (Die Beam is lethal)
```
- Gear swap suggestions per boss (swap in anti-freeze ring for Elder, etc.)
- Practice fight recommendations ("try normal Shaper first")

### League Mechanic Optimization
- Detect current league mechanics
- Suggest build tweaks for specific league content
- "For Ritual: your AoE is low, consider Inc AoE gem swap"
- "For Delirium: need more move speed and DPS for high rewards"
- "For Blight: your single target is fine, but need AoE — consider anoint"
- "For Expedition: need logbook-specific defenses"

### Current League Support: Mirage (3.28)
Must support all mechanics from the current league at launch:

- **Djinn encounter analysis**: is your build strong enough to clear Astral Realm maps?
- **Wish selection advisor**: which of the 3 Djinn wishes is best for your build?
  - Sand Djinn (Dex support coin) vs Fire Djinn (Str) vs Water Djinn (Int)
  - Based on which support gem effect benefits your main skill most
- **Djinn Coin usage advisor**: which support gem effect to imbue on your level 20 gem?
  - Simulate each possible support imbue → rank by DPS impact
  - "Imbue RF with Burning Damage effect: +12% DPS permanently"
  - "Imbue RF with Elemental Focus effect: +9% DPS but lose ignite"
- **Exceptional Support Gem tracking**: which of the 40+ new supports work for your build?
  - Holy skill gems: Blessed Call, Excommunicate, Exemplar, Hallow
  - Rank all Exceptional Support Gems by DPS/defense impact for your build
- **Reliquarian Scion ascendancy**: new ascendancy support in build detector
- **Atlas rework**: updated atlas passive tree data for 3.28
- **T17 removal**: adjust boss farming strategies (T17 maps gone, bosses remain as Uber Pinnacles)

### New Gem System Support (3.28+)
- **Imbued gems**: gems with permanently imbued support effects (from Djinn Coins)
  - Parser must handle imbued gem data from PoB XML
  - Calculator must include imbued support effect in DPS chain
  - UI shows imbued effect as a special modifier on the gem
- **Exceptional Support Gems**: new support gem category
  - Add to gem database, include in "best support gem" calculations
  - Show DPS impact comparison: regular support vs exceptional support

### Specific Content Readiness
```
Simulacrum:     ❌ Need 3M+ DPS for wave 25+ (you have 2.8M)
Delve:          ⚠️ Need darkness res, light radius, phys mitigation
Heist:          ✅ Ready (no specific requirements)
Blight:         ⚠️ Low AoE — towers will carry, but slower
Sanctum:        ❌ Need evasion/dodge, your build gets hit too often
Lab:            ✅ Ready (high regen, tanky, trap immune via regen)
5-Way Legion:   ⚠️ Need more clear speed for efficient farming
Breach:         ✅ Ready (AoE and DPS sufficient)
Expedition:     ✅ Ready (can handle remnant mods, check immune mods)
```

---

## MAP MOD DANGER SYSTEM

### Dangerous Map Mod Detection
```
Your build CANNOT run:
  ❌ No Regen (you rely on regen for RF — instant death)
  ❌ Elemental Reflect (you deal fire damage — one-shot yourself)

Your build is DANGEROUS on:
  ⚠️ -max res (drops your max fire res, RF degen increases by X%)
  ⚠️ Less Recovery (reduces your regen below RF degen threshold)
  ⚠️ Elemental Weakness (-24% res, check overcap)
  ⚠️ No Leech (if you depend on leech for secondary skills)
  ⚠️ % increased monster AoE (harder to dodge boss mechanics)
  ⚠️ Monsters have % crit chance (random one-shots)

Your build is SAFE on:
  ✅ Physical Reflect (you deal no physical damage)
  ✅ No Mana Regen (you use Lifetap)
  ✅ Hexproof (you don't rely on curses)
  ✅ Reduced Flask Charges (minor inconvenience)
  ✅ Extra monster projectiles (you're melee range)

Suggested: ALWAYS roll over No Regen + Ele Reflect
Chaos sextant re-roll cost: ~2 chaos average
```

### Map Rolling Assistant
- Suggested map mod combinations for your build
- "Roll for: Pack Size + Beyond + Extra Magic/Rare — skip Reflect + No Regen"
- Quantity/Rarity vs danger balance
- Atlas favorite map suggestions based on layout + build clear style
- Scarab pairing recommendations per map strategy

---

## ATLAS / ENDGAME STRATEGY

### Atlas Passive Tree Suggestions
- Based on build archetype, suggest atlas passive allocation
- "Your build is good for Essence farming — take these atlas nodes"
- "RF Inquisitor clears well — optimize for map sustain + Essence + Harvest"
- Content-specific atlas trees (Bossing tree, Mapping tree, Currency tree)

### Favorite Map Suggestions
- Recommend maps based on your build's clear pattern
- "RF prefers open layouts — favorite: Strand, Dunes, Tropical Island"
- Boss difficulty per map vs your build
- Map sustain analysis

### Maven Invitation Planning
- Track which bosses you've witnessed
- Invitation readiness check
- "You need 3 more witnessed bosses for The Formed"
- Difficulty assessment per invitation

### Eldritch Altar Strategy
- Suggest which altar mods to take during mapping
- "Take fire damage altars (you're fire immune), skip cold damage altars"
- Risk/reward analysis per altar option
- "Eldritch currency altar: always take (free crafting materials)"

### Scarab Suggestions
- Recommend scarabs based on atlas strategy and build
- Scarab tier/cost vs reward analysis
- "Polished Essence Scarab: +3 essences per map, costs 2c, avg return 8c"
- Scarab stacking combinations

---

## SYNDICATE / BETRAYAL BOARD

### Optimal Member Placement
- Suggest best syndicate member placement per slot
- "Aisling → Research (veiled chaos orbs for crafting)"
- "Hillock → Transportation (30% weapon quality)"
- "It That Fled → Research (breachstone upgrades)"
- "Vorici → Research (white sockets on corrupted items)"
- Track current board state if provided

### Safehouse Optimization
- Priority safehouse runs based on rewards
- Intelligence gathering strategy
- Rivalry/trust relationship optimization
- Catarina readiness check

---

## BUILD COMPARISON ENGINE

### vs Top Players (poe.ninja)
- Fetch top builds matching your ascendancy + main skill
- Compare gear slot by slot (with item scores)
- Compare passive tree (% overlap, missing key nodes)
- Compare gem setups (different support gems used)
- "Top RF Inquisitors average 4.2M DPS — you're at 2.8M"
- "90% use Aegis Aurora — consider switching from Rise of Phoenix (+35% survival)"
- "Your tree matches 73% of top builds — missing: Sovereignty wheel"
- "Most popular support gem: Awakened Burning Damage (you use regular)"
- Percentile ranking ("Your DPS: top 35% of RF Inquisitors")

### vs Saved Builds
- Compare any two of your local builds side-by-side
- Full stat comparison table
- "Build A: more DPS (+500k), Build B: more tanky (+1200 life)"
- Migration plan between builds (what to change, cost)
- "Converting Build A → Build B costs 15div in regrets + gear changes"

### Historical Self-Comparison
- Track your build's progression over time (automatic snapshots)
- "This week: +800 life, +500k DPS, spent 23div"
- Graph DPS/life/resist over time
- Achievement milestones ("Hit 1M DPS!", "Capped chaos res!")
- Currency invested tracking
- "Total invested in this character: 87 divines over 3 weeks"

---

## MULTI-CHARACTER MANAGEMENT

### Character Dashboard
- All characters at a glance with build scores
- Per-character DPS, life, resist summary
- Shared currency pool across all stash tabs
- "Your Ranger has 3 unused exalts in inventory"
- Quick-switch between character analysis

### Cross-Character Item Optimizer
- "This amulet on your Marauder would be +15% DPS on your Witch"
- Suggest item swaps between characters (with exact impact calc)
- Leveling gear hand-me-down tracking
- "Your Marauder outgrew these boots — they're perfect for your new Ranger"
- Shared unique item pool management

### Next Character Suggestions
- "Based on your stash, cheapest next build: Cold DoT Occultist (12div)"
- "You already own 60% of the gear for a Spark Inquisitor"
- League starter suggestions based on owned gear and currency
- "With your 50div budget, top 3 new builds to try: ..."
- Complementary build suggestions (mapper + bosser combo)

### Party Play & Support Build Analysis
- Analyze party composition: "Your party needs a curser"
- Aura stacking compatibility check between 2+ builds
  - "Your Determination overlaps with Party Member B's — one should swap"
  - "Adding your Anger aura gives Party Member A +22% DPS"
- DPS contribution per party member
- Support build optimizer: which auras/curses benefit the party most
- "Your RF + their Cold DoT = great elemental coverage"
- Aurastacker analysis: diminishing returns detection
- Party EHP: "Member C dies to Shaper slam — they need more life"
- Party curse analysis: "You can apply 3 curses — optimal: Flammability, Elemental Weakness, Temp Chains"

### Build Share Codes
- Generate compact share code from any build (like PoB pastebin but shorter)
- Share via: copy code, generate URL, QR code for mobile
- Import code → auto-create local build
- Version in code: "This build was shared on patch 3.24"
- Share options: full build, tree only, items only, gems only
- "Share your Ring 2 setup" → sends just that item + socket group

---

## AI-POWERED FEATURES (Optional, uses API)

### Multi-Provider Support
```
Settings → AI Provider:
  [Select Provider ▼]
  → OpenAI (ChatGPT) — API key or OAuth
  → Anthropic (Claude) — API key
  → Google (Gemini) — Google OAuth
  → Groq — API key (fast + free tier)
  → Ollama — local, no key needed, fully free
  → OpenRouter — OAuth, access to 100+ models

  [API Key: ••••••••••]
  [Model: claude-sonnet-4 ▼]
  [Test Connection ✓]
```

### Natural Language Build Advice
- "Why am I dying in T16 maps?" → AI analyzes defenses + content difficulty
- "What should I upgrade next with 20 divines?" → AI + exact calc + market data
- "Explain why this passive tree path is better" → AI explains mechanics
- Context-aware — AI sees your full build data, stats, items, tree
- Conversation history within session
- "Compare my build to this poe.ninja build" → AI explains differences

### Build Concept Generator
- "I want a fast mapper that uses fire skills"
- AI generates PoB-importable build concept with tree + gems + gear suggestions
- Iterate with feedback ("make it more tanky", "budget version")
- "Generate a league starter under 5div that can do all content"

### Patch Impact Analysis
- Feed patch notes to AI + exact calc engine
- "Your RF got +5% base damage — DPS increases by 142k (+5%)"
- "Determination reservation increased — you can't fit Vitality anymore"
- "New unique released: synergizes with your build, +22% DPS"
- Automatic build adjustment suggestions with one-click apply
- "After patch: respec 3 nodes, swap 1 gem — here's the optimized version"

### Community Build Translator
- Paste a build guide URL (forum, YouTube, Reddit)
- AI extracts gear/tree/gems/enchant requirements
- Compare against your current build
- "To follow this guide, you need to change: X, Y, Z"
- Shopping list with prices: "Total conversion cost: 30div"
- One-click apply changes to PoB

---

## AUTOMATION & QUALITY OF LIFE

### Loot Filter Integration
- Generate/update loot filter based on your build needs
- "Highlight items with +fire res and +life (your weak stats)"
- FilterBlade API integration for NeverSink filter customization
- "Your loot filter is highlighting cold damage wands — you don't need those anymore"
- Auto-update filter as your gear improves
- "You capped resists — removing resist highlighting, adding damage mods"

### PoE Trade Companion
- Auto-generate trade searches for every upgrade suggestion
- Bulk trade optimization (buy multiple items efficiently)
- "Buy these 5 items together from same seller = save on portal/trade time"
- Whisper template auto-copy
- Trade history log (what you bought, for how much, when)
- Price negotiation suggestions based on listing age

### Build Export/Share
- Export build summary as shareable image (infographic style)
- Generate complete build guide from your build (formatted for forum)
- Reddit/forum formatted build post with gear/tree/gem sections
- Short URL generation for sharing builds
- QR code for mobile viewing
- Discord embed format for sharing in servers

### Hotkey Support
- Global hotkey to show/hide overlay (works while PoE is focused)
- Hotkey to refresh build analysis
- Hotkey to check item price (hover item in PoE → price popup)
- Hotkey to toggle overlay sections
- Customizable key bindings
- Conflict detection with PoE keybinds

### Overlay Mode
- Transparent overlay on top of PoE (always on top)
- Show suggestions while playing
- Quick item price check (item under cursor)
- Mini resistance/stat bar
- DPS meter
- Map mod danger warnings
- Currency counter
- Boss readiness indicator
- Customizable position and opacity

---

## DATA & ANALYTICS

### Build Database
- Store all builds locally with full version history (git-like)
- Tag and categorize builds (league starter, bosser, mapper, etc.)
- Search across all builds ("show me all builds that use Determination")
- Star/favorite builds
- Build templates (save as template for new characters)
- "Show me my best bosser build from last league"

### Economy Dashboard
- Live economy overview (currency rates, popular items, trending builds)
- Currency exchange rates with historical graphs
- Popular item price trends
- League phase indicators ("Day 3: chaos still valuable, divine prices rising")
- "Best currency farming strategy right now: Essence farming (~8div/hr)"
- Mirror tier item tracking

### Personal Statistics
- Total currency earned/spent tracking per league
- Build completion percentage (how close to "finished")
- Achievement system ("First 6-link!", "10M DPS!", "All bosses killed!")
- Time played estimation per character
- Upgrade history timeline
- "This league: 340 divines earned, 280 spent, 3 characters built"
- Most used skill gems across all characters

---

## COMMUNITY FEATURES

### Build Sharing
- Share builds within the app with other users
- Build ratings and reviews from community
- Community tier lists for builds per content type
- "Top rated RF builds this league: ..."
- Import others' builds into your PoB with one click

### TFT / Trading Community Integration
- Link to TFT Discord for crafting services
- Trusted crafter finder for bench crafts you don't have
- Service request templates (e.g., "need Aisling slam on my helmet")
- Vouch system integration

### Build Guide Integration
- Follow a guide step-by-step within the app
- Progress tracking against guide milestones
- "Step 5/12: Get 5-link body armour ✅"
- Deviation detector ("you went off-guide here — intentional?")

### In-App Bug Report / Feedback ("Whisper to the Void")
Built-in feedback system that posts directly to GitHub Issues — no need for users
to have a GitHub account or leave the app.

```
[Report a Bug]  →  Opens feedback panel inside app:

┌─────────────────────────────────────────────────┐
│  ☠ Whisper to the Void — Report an Issue        │
│                                                 │
│  Type: [Bug ▼]  [Feature Request]  [Feedback]   │
│                                                 │
│  Title: [What went wrong?                    ]  │
│                                                 │
│  Description:                                   │
│  ┌──────────────────────────────────────────┐   │
│  │ Describe what happened...                │   │
│  │                                          │   │
│  └──────────────────────────────────────────┘   │
│                                                 │
│  Attachments:                                   │
│  [+ Screenshot]  [+ Video]  [+ PoB Build]       │
│  [+ App Logs]    [+ Build Analysis]             │
│                                                 │
│  ☑ Auto-attach: current build summary (anon)    │
│  ☑ Auto-attach: app version + system info       │
│  ☐ Include my build file (may contain account)  │
│                                                 │
│  [Invoke — Send to the Void]                    │
└─────────────────────────────────────────────────┘
```

#### Supported Attachments
- **Screenshots** — paste from clipboard or drag & drop (PNG/JPG)
- **Video** — screen recording (MP4, WebM, max 10MB)
- **Text** — error logs, PoB paste codes, build XML
- **Images** — annotated screenshots with markup tools
- **Build snapshot** — auto-generated anonymized build summary
- **App logs** — last 100 lines of app log (auto-attached)

#### How It Works (GitHub Issues Backend)
```
User submits feedback
    ↓
App formats as Markdown
  → Title: "[Bug] User's title"
  → Body: description + system info + build summary
  → Labels: auto-tagged (bug/feature/feedback)
    ↓
Post to GitHub Issues via GitHub API
  → Uses app's GitHub token (not user's)
  → Images/videos uploaded as assets
  → User gets link to track their issue
    ↓
Maintainers triage on GitHub
  → Community can upvote/comment
  → Status updates visible in app
```

#### Privacy Controls
- Build data anonymized by default (no account name, no character name)
- User chooses what to attach
- Clear preview of exactly what will be sent before submitting
- "Your whisper has been heard" confirmation with issue link

#### In-App Issue Tracker
- View your submitted issues + status
- See latest known issues (fetched from GitHub)
- "Known Issue: poe.ninja price delay — fix in v0.2.1"
- Upvote existing issues from within the app

---

## LEAGUE START PLANNER

### Pre-League Planning Mode
- Plan builds before league start
- "These items will be cheap day 1: Goldrim (1alch), Tabula (10c)"
- Leveling item checklist per act
- Target unique drop list with farming locations
- Passive tree progression (what order to take nodes while leveling)
- Gem acquisition plan (what quest gives which gem, or buy from vendor)

### Atlas Rush Strategy
- Optimal atlas progression for your build
- Which maps to prioritize
- When to start doing league mechanics
- "Rush to T16 by day 2 — here's the map completion order"
- Watchstone/Voidstone priority

### Day 1-3 Economy Guide
- What to sell, what to keep
- Currency making strategies for league start
- "Sell these div cards immediately — price drops 50% by day 3"
- "Hoard these essences — price doubles by week 2"

---

## TECHNICAL FEATURES

### Performance
- XML parsing < 50ms
- Fast DPS estimation < 50ms
- Exact DPS calculation (PoB engine) < 500ms
- Suggestion engine < 200ms
- File watch latency < 100ms
- Cache poe.ninja data (refresh every 5 min)
- Cache trade API data (refresh on demand, respect rate limits)
- Offline mode (all cached data works without internet)
- Lazy loading for heavy calculations (don't block UI)

### Auto-Update System
- Check GitHub Releases on startup
- Compare version against latest release
- Show changelog before updating
- Download update in background
- One-click update + restart
- Rollback to previous version if update fails
- Signed releases for security
- Beta channel for testing new features

### Settings
- AI provider selection (Claude/GPT/Gemini/Ollama/OpenRouter) with API key/OAuth
- Auto-apply suggestions toggle
- Notification preferences (sound, system tray, overlay)
- Theme (dark/light/PoE themed/custom)
- Language support (English, Korean, Chinese, Russian, etc.)
- Backup retention policy (days/count)
- Trade API rate limit configuration
- Overlay position/size/opacity
- Font size scaling
- Color blind mode for tier colors
- Keyboard shortcut customization
- Data export/import (move settings between PCs)
- First-launch wizard (detect PoB, set up API keys, choose features)

### Security
- API keys stored encrypted locally (OS keychain: Windows Credential Manager)
- OAuth tokens in OS keychain
- No data sent externally without explicit consent
- Privacy dashboard ("what data leaves your PC")
- Open source — fully auditable on GitHub
- No telemetry without opt-in
- Signed executables

### Accessibility
- Color blind mode (different color schemes for tier/score indicators)
- Screen reader support for all UI elements
- Keyboard-only navigation (tab through all interactive elements)
- Font size scaling (small/medium/large/extra large)
- High contrast mode
- Reduced motion mode (disable animations)
- Tooltip delay customization

### Mobile Companion
- View your build on phone while at PC via QR code sync
- Get price alerts on phone
- Check trade offers while AFK
- Push notifications for snipe alerts
- Build summary widget

### Streaming Features
- OBS browser source overlay widget
- Show build stats on stream
- Viewer chat bot integration ("!build" shows current gear score)
- Viewer build comparison ("!compare" lets viewers compare their build)
- Clean overlay theme for stream aesthetic

---

## SINGLE FILE DISTRIBUTION

### Build System
- Tauri (Rust + Web UI)
- Single .exe file (~10-15MB)
- MSI and NSIS installer options
- Portable mode (run from USB, no install needed)
- Bundle LuaJIT runtime for exact DPS calc

### Release Pipeline
```
Developer:
  git tag v1.0.0 → git push --tags
      ↓
GitHub Actions (automatic):
  → Build .exe for Windows
  → Sign executable
  → Create GitHub Release
  → Upload .exe + update manifest (latest.json)
      ↓
Users:
  → App checks GitHub Release for updates
  → Downloads + installs automatically
  → Rollback available if issues
```

### Offline-First Architecture
- All core analysis works without internet
- Mod tier database bundled locally
- Passive tree data per version bundled
- poe.ninja data cached aggressively
- Trade features degrade gracefully when offline
- First-launch downloads game data (~20MB), then works offline

---

## ADDITIONAL FEATURE IDEAS

### Currency Farming Strategy Planner
Based on build capabilities + atlas tree + current economy:
```
Your build: RF Inquisitor (tanky, medium clear, 2.84M DPS)
Current league: Week 3 (stable economy)

Best farming strategies ranked:
  #1 Essence farming — 10 div/hr (your AoE + tankyness ideal)
      Atlas: Essence nodes, Stream of Consciousness
      Maps: Strand T16 (linear, fast clear)
      Scarabs: Polished Essence + Rusted Ambush

  #2 Expedition farming — 8 div/hr (Gwennen gambling + Tujen haggling)
      Atlas: Expedition nodes
      Maps: Dunes T16 (open layout)
      
  #3 Boss farming — 6 div/hr (Shaper/Elder sets, sell fragments)
      Need: 3M+ DPS for efficiency → you're close

Currency goal: 50 divine for Aegis Aurora
At #1 rate: ~5 hours of farming
```

### Build Health Monitor (Persistent)
Running score that tracks your build over time:
- "Your build score went from 68 → 74 this week (+6)"
- "DPS increased 500K from gem levels"
- "You haven't upgraded Ring 2 in 14 days — it's your biggest bottleneck"
- Weekly summary: what changed, what should change next
- Achievement badges: "First 1M DPS!", "All resists capped!", "Uber viable!"

### Damage Log Analyzer
If player records gameplay (via OBS or replay tool):
- Parse damage taken events
- "You died to: Sirus Die Beam (4200 cold damage over 0.5s)"
- "Top 3 deaths this session: Die Beam (3), Maven Ball (2), Disconnect (1)"
- Suggest defenses for most common death causes
- "Add cold res flask suffix to survive Die Beam"

### League Challenge Tracker
- Parse league challenges from PoE profile
- Show progress: "28/40 challenges completed"
- Suggest easiest remaining challenges for your build
- "Challenge: Kill Shaper — you're READY (DPS sufficient)"
- "Challenge: Complete all Uber bosses — BLOCKED (need DPS upgrade)"
- Optimal challenge ordering for reward thresholds (12/24/36)

### Social / Party Play Features
- Party composition analyzer: "Your party needs a curser"
- Aura stacking compatibility check between builds
- DPS contribution per party member
- "Your RF + their Cold DoT = great elemental coverage"
- Support build optimization (which auras benefit party most)

### Sound Alerts
- Custom alert sounds for:
  - Price alert triggered (item dropped below threshold)
  - Build file changed (PoB modified)
  - Seer response ready
  - Dangerous map mod detected
  - Boss fight simulation ready
- Configurable: which events trigger sounds, volume, custom sound files
- PoE-themed default sounds (currency drop, level up, boss spawn)

### Theorycraft Lab
Sandbox mode for testing build ideas without modifying your actual build:
- "What if I took this keystone?"
- "What if I switched to Aegis Aurora?"
- "What if I reallocated 10 passive points?"
- Side-by-side comparison: current vs theoretical
- Save theorycrafts as "Dark Path" evolution options
- Import someone else's build and compare
- "Import top #1 RF Inquisitor from poe.ninja → compare to yours"

### PoE 2 Dual-Game Support

PoE 2 has fundamental mechanic differences. We must support BOTH games.

#### Key PoE 2 Differences Our Engine Must Handle
- **Gem system**: gems socket INTO OTHER GEMS (not into items)
  - Support gems link to skill gems directly
  - No 6-link items — instead, skill gems have their own socket count
  - Our gem UI needs a tree/hierarchy view, not a linear link view
- **Passive tree**: different layout, different keystones, different ascendancies
  - Separate tree data file per game version
  - Tree viewer must load correct version
- **Crafting**: different system (PoE 2 uses different currency/methods)
  - Separate mod weight tables, crafting probability engine per game
- **Items**: different base types, different mod pools
  - Separate item database per game version
- **Combat**: different dodge/block/armour formulas potentially
  - Formula versioning per game (already in our engine design)

#### Implementation
```
game-data/
  poe1/                   # Path of Exile 1 data
    mods/ gems/ tree/ items/ crafting/ bosses/
  poe2/                   # Path of Exile 2 data
    mods/ gems/ tree/ items/ crafting/ bosses/

Settings → Game Version:
  ◉ Path of Exile 1 (default)
  ○ Path of Exile 2

Calculator loads correct game-data/ subfolder based on selection.
UI adapts: gem viewer changes from linear links → gem tree.
```

#### PoE 2-Specific Features
- **Gem leveling planner**: which gems to level, gemcutting priority
- **Dodge mechanics** (PoE 2 has dodge instead of spell block)
- **Spirit resource** (PoE 2 uses Spirit for persistent skills, not mana reservation)
- **Weapon swap builds** (PoE 2 has dual weapon sets with different skills per set)
- **Cross-game build comparison**: "Your PoE 1 RF maps to PoE 2 Infernal Flame"

#### Timeline
- Phase 1: PoE 1 only (MVP)
- Phase 2: Abstract game-data loading per version
- Phase 3: PoE 2 data files when available
- Phase 4: PoE 2-specific UI (gem tree viewer)

---

## MAP RUN STATISTICS (inspired by Mapwatch)

### Client.txt Log Parser
- Parse PoE's Client.txt log file for map entry/exit events
- Track time per map run (enter zone → leave zone)
- Track deaths per map (resurrection events)
- Track boss kill timestamps
- "Average Strand T16: 2:15 clear, 0.3 deaths/run"

### Session Statistics
```
Current Session (2h 30m):
  Maps run: 42
  Average clear: 2:20
  Deaths: 7 (0.17/map)
  XP gained: 18% of level 95
  Currency dropped: ~12 divine equivalent

  Best map:  Strand T16 — 1:48 (personal best!)
  Worst map: Crimson Temple T16 — 4:12 (layout issue)

  Map tier breakdown:
    T16: 38 maps (90%)
    T15: 4 maps (10%)
```

### Historical Trends
- Graph: maps/hour, deaths/hour, XP/hour over time
- Compare sessions: "You're mapping 15% faster this week"
- Identify bottlenecks: "Crimson Temple takes 80% longer than Strand — avoid it"
- Currency/hour estimation based on map strategy

### Integration with Build Advisor
- "After upgrading Ring 2: maps/hour went from 18 → 22 (+22%)"
- Link map performance to gear changes
- Suggest map favorites based on clear speed data

---

## VENDOR RECIPE ADVISOR

### Recipe Detection
Based on stash contents, suggest profitable vendor recipes:
```
Available Recipes:
  ✅ Chaos Recipe — 2 full sets ready (rings in Dump tab)
     Turn in for: 2 chaos orbs
  ✅ Chromatic — 87 RGB-linked items detected
     Turn in for: 87 chromatic orbs
  ⚠ Regal Recipe — 1 set ready, missing ilvl 75+ gloves
     Need: any ilvl 75+ gloves

Vendor Recipe Value: ~4 chaos if turned in now
```

### Smart Recipe Suggestions for SSF
- Vendor recipe alternatives when trade is unavailable
- "+1 gem level wand recipe: sell blue wand + alt quality gem + Orb of Augmentation"
- "Resistance flask recipe: sell white flask + alt + resist ring"
- Track which recipes player has discovered

---

## CHAOS/REGAL RECIPE ENHANCER

### Auto-Detect Recipe Items in Stash
- Scan all stash tabs for unidentified rare items
- Highlight items that complete a Chaos Recipe set (ilvl 60-74)
- Highlight items that complete a Regal Recipe set (ilvl 75+)
- Show which slots are missing: "Need: belt, gloves for Chaos set #3"

### Recipe Overlay
```
Chaos Recipe Status:
  Full sets ready: 2
  Partial sets: 3

  Set #3 missing:  [Belt] [Gloves]
  Set #4 missing:  [Boots] [Ring 2]
  Set #5 missing:  [Helmet] [Body] [Belt] [Ring 1] [Ring 2]

  Quick action: [Turn in 2 sets → 2 chaos] [Invoke]
```

### Auto-Price Check
- Before vendoring: check if any recipe item is actually valuable
- "Warning: this amulet has T1 life + fire DoT multi — worth 3 divine, don't vendor it!"
- Only vendor truly worthless items

---

## MASS CRAFTING SIMULATOR (inspired by Craft of Exile)

### Simulate 1000+ Crafts
Run statistical simulation using real mod weights:
```
Simulate: Chaos spam on Opal Ring ilvl 84
Target: +80 life AND +fire DoT multi
Simulations: 10,000

Results:
  Hit target: 312 / 10,000 (3.1%)
  Average attempts to hit: 32 chaos
  Average cost: 32 chaos ≈ 0.4 divine

  Best result found:
    +95 Life (T1), +18% DoT Multi (T1), +42% Fire Res (T1)
    → Score: 94/100 — this would be GG

  Distribution:
    T1 life + any DoT multi: 8.2% (122 attempts avg)
    Any life + T1 DoT multi: 5.4% (185 attempts avg)
    T1 life + T1 DoT multi:  0.4% (2500 attempts avg)
```

### Method Comparison
Run same target with different crafting methods side by side:
```
Target: +80 life, +fire DoT multi on Opal Ring

Method          | Success/try | Avg cost | Best case | Worst case
Chaos spam      | 3.1%        | 0.4 div  | 0.01 div  | 3 div
Essence Anger   | 12.8%       | 1.2 div  | 0.15 div  | 5 div
Pristine fossil | 6.2%        | 1.8 div  | 0.3 div   | 8 div
Alt + Regal     | 0.8%        | 2.1 div  | 0.05 div  | 15 div

Recommendation: Essence of Anger (highest success rate)
```

---

## WEALTH HISTORY TRACKER (inspired by Exilence Next)

### Total Net Worth Over Time
- Calculate total wealth: equipped gear + stash currency + stash items
- Track daily snapshots
- Graph: net worth over league timeline
```
League Week 1: 5 divine
League Week 2: 23 divine
League Week 3: 47 divine (current)

Growth rate: +12 divine/week
Projection: ~95 divine by week 7
```

### Investment Tracking
- Track currency spent per upgrade
- "You've invested 35 divine in this character"
- ROI analysis: "Aegis Aurora (18d) gave biggest survivability boost per divine"
- "Total spent vs total earned: +12 divine net profit"

---

## ZONE LAYOUT OVERLAY (inspired by PoE-Leveling-Guide)

### Act Zone Layouts
- Show zone layout diagrams during leveling
- "Act 2 Chamber of Sins: go RIGHT at entrance"
- Waypoint location indicators
- Quest objective markers
- Optimal pathing arrows

### Map Layout Ratings
```
Your Favorited Maps (best for your build):
  ★★★ Strand    — Linear, fast, no backtracking
  ★★★ Dunes     — Open, good for RF AoE
  ★★☆ Beach     — Linear but short
  ★☆☆ Crimson Temple — Tight corridors, bad for RF
```

---

## ENHANCED POB PARSER (Missing Fields)

### P0 — Must Parse Now
These PoB XML fields are currently ignored but essential:

- **`<Calcs>` section** — free pre-calculated DPS/life/resists from PoB's own engine
- **`{crafted}` mod flag** — know which mods are benchcrafted (replaceable for free)
- **`<MasteryEffects>`** — which mastery effect chosen at each mastery node
- **Jewel socket assignments** — `<Socket>` mapping jewel items to tree nodes
- **Cluster jewel structure** — enchant type, notable assignments, `{variant:X}` tags

### P1 — Important
- **Notes section** — user build notes (useful context for AI)
- **Spectre list** — `<Spectre>` entries for minion builds
- **Timeless jewel seeds** — seed + keystone transformation
- **Anointment data** — `Allocates X` enchantment on amulet/rings
- **Eldritch implicit source** — Searing Exarch vs Eater of Worlds tags
- **`{fractured}` mod flag** — fractured mods can't be changed
- **Influence type tags** — `{shaper}`, `{elder}`, `{crusader}`, etc.
- **Gem alternate quality** — qualityId: Anomalous/Divergent/Phantasmal

---

## POEDB DATA INTEGRATION

### Mod Weight Tables
- Import exact spawn weights for every mod per base type + ilvl + influence
- Powers the Crafting Probability Calculator and The Forge
- Source: poedb.tw data extraction

### Fossil Weight Multipliers
- Import fossil tag multiplier tables (e.g., Pristine: life ×10, ES ×0)
- Powers fossil craft simulation and suggestions

### Boss Attack Database
- Import boss attack damage values, types, speeds, phase data
- Powers Combat Simulator ("The Arena") with accurate fight modeling
- Source: poewiki.net boss pages

### Monster Scaling Data
- HP/damage/resistance per area level
- Powers accurate map monster kill time calculations

---

## POEWIKI DATA INTEGRATION

### Game Mechanics Formulas
- Complete armour, evasion, block, DoT, ailment, conversion formulas
- Validates our Calculator engine against known correct math

### Divination Card Drop Locations
- Which maps/zones drop which cards
- Powers Div Card Target Farming in Stash tab

### Vendor Recipe Database
- Complete recipe list with inputs/outputs
- Powers Vendor Recipe Advisor

### Damage Conversion Chains
- Exact conversion order (phys → lightning → cold → fire → chaos)
- Validates build analysis for conversion builds

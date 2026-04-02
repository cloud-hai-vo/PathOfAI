# PoB Advisor — Complete Feature Specification

## CORE SYSTEMS

### 1. Two-Way PoB Sync Engine
- Auto-detect PoB install path (`%AppData%/Path of Building/`)
- File watcher with debounce (wait 500ms after last change)
- XML parser for all PoB sections
- XML writer with atomic writes (write temp → rename)
- Auto-backup before every write
- Undo/redo history (last 50 changes)
- Preview diff before applying changes
- File lock detection (don't write while PoB is saving)
- Support multiple PoB installs (community fork vs original)
- Watch multiple build folders including subfolders

### 2. Build Analysis Engine
- Parse all PlayerStat values
- Calculate effective HP (life × mitigation layers)
- Track DPS breakdown by damage type
- Detect build archetype automatically (attack/spell/dot/minion/totem)
- Score build on 0-100 scale across categories

---

## DEFENSIVE ANALYSIS

### Resistance Checker
- Elemental resist cap detection (75% base, higher with max res)
- Overcap calculation (how much buffer for -res curses)
- Chaos resist warning tiers: <0 danger, 0-50 okay, 50-75 good
- Curse penalty awareness (-24% ele res in maps)
- Exposure/penetration vulnerability detection
- "You need X more cold res to survive Elemental Weakness maps"

### Effective HP Calculator
- Raw life pool
- Energy shield (hybrid or CI detection)
- Armour-based physical reduction (with diminishing returns formula)
- Evasion chance + entropy calculation
- Block/spell block contribution
- Guard skill uptime estimation (Molten Shell, Steelskin)
- Fortify detection
- "Your effective physical HP vs a 5000 hit = X"
- "Your effective elemental HP vs Shaper slam = X"

### Ailment Immunity Checklist
```
✅ Freeze immune (source: Brine King + boot craft)
❌ Shock — VULNERABLE (no source detected)
✅ Ignite immune (source: ascendancy)
❌ Bleed — VULNERABLE (need corrupted blood jewel)
❌ Poison — not immune (low priority for your build)
✅ Corrupted Blood — jewel detected
❌ Stun — no stun immunity (dangerous for RF)
❌ Curse immune — no source detected
```

### Damage Taken Simulation
- Simulate specific boss hits against your defenses
- "Shaper Slam: you survive with 1240 life remaining"
- "Sirus Die Beam: you die in 0.8 seconds"
- "Maven Memory Game failure: lethal"
- Compare pre/post upgrade survivability
- Factor in flasks, guard skills, ascendancy

### Recovery Analysis
- Life regen per second (critical for RF)
- Life regen vs RF degen balance
- Leech rate and cap
- Life gain on hit sources
- Flask recovery
- "Your net regen is +450/s — safe margin for RF"
- "Adding Vitality would push regen to +620/s"

---

## OFFENSIVE ANALYSIS

### DPS Breakdown
- Total DPS by damage type
- DPS per gem link analysis
- DPS with/without flasks
- DPS with/without buffs (onslaught, frenzy charges, etc.)
- Boss DPS vs map clear DPS
- Damage over time tracking (ignite, bleed, poison stacks)
- "Your fire trap contributes 65% of single target"

### Gem Optimization
- Support gem DPS comparison (swap X for Y = +Z% DPS)
- Gem level breakpoints ("21/20 RF = +18% more DPS than 20/20")
- Quality breakpoints ("23% quality > 20% for this gem")
- Awakened gem upgrade path ("Awakened Burning = +12% DPS, costs 8div")
- Vaal gem suggestions
- Empower/Enhance/Enlighten level impact
- Alt quality gem comparison
- "Best 6th link for your RF: Efficacy > Conc Effect > Inc AoE"

### Skill Link Suggestions
- Detect suboptimal support gems
- Suggest better combinations
- Calculate link priority (which socket group benefits most from upgrade)
- Trigger setup detection (CWDT, CWC, CoC)
- Aura efficiency (enlighten savings, reservation)

### Flask Optimization
- Detect missing critical flasks
- Suggest flask suffixes (bleed immune, freeze immune)
- Unique flask recommendations
- Flask uptime calculation
- "You have no anti-bleed flask — high risk in maps"
- "Dying Sun would give +2 RF radius and +fire res"

### Critical Strike Analysis (for crit builds)
- Effective crit chance
- Crit multiplier stacking efficiency
- Diamond flask impact
- Power charge value
- "You're at 68% effective crit — investing more has diminishing returns"
- "Switching to Increased Critical Damage gives +9% DPS"

---

## PASSIVE TREE ANALYSIS

### Node Efficiency Scoring
- Score = total stats gained / points spent to reach
- Flag low-efficiency travel nodes
- Find dead nodes (giving stats your build doesn't use)
- "Node 'Coordination' gives +10 dex — you don't need dex, respec this"
- Cluster jewel vs tree node comparison

### Path Optimization
- Find shorter paths between keystones
- Detect unnecessary travel
- "Respec these 3 nodes, take this path instead: same destination, save 2 points"
- Suggest where to spend saved points
- Show top 5 most impactful unallocated nodes within reach

### Keystone Analysis
- Impact simulation for each keystone
- "Taking Elemental Overload: -0% DPS (you have no crit anyway), good choice"
- "Removing Resolute Technique: +22% DPS if you add accuracy"
- Ascendancy node comparison
- "Pious Path > Sanctuary for your build (+340 ES regen)"

### Mastery Suggestions
- Check if all available masteries are taken
- Rank masteries by impact for your build
- "Fire Mastery: +20% burning damage > +1 fire exposure"
- Flag mastery conflicts

### Jewel Socket Analysis
- Value of each jewel socket (worth pathing to?)
- Best jewel stats for your build
- Timeless jewel impact estimation
- Cluster jewel notable rankings
- "This jewel socket costs 3 points to reach — needs a jewel worth 3+ passive points"
- Forbidden Flame/Flesh suggestions

### Next N Points Planner
- "Your next 5 points should go to:"
  1. Life wheel near Marauder (2 points, +340 life)
  2. Fire DoT mastery (1 point, +20% damage)
  3. Jewel socket (2 points, flexible)
- Level-by-level passive guide
- Respec plan with point-by-point instructions

---

## ITEM ANALYSIS

### Mod Tier Detection
- Identify tier of every mod (T1 life = +90-99, T2 = +80-89, etc.)
- Color-code by tier quality
- "Your helmet has T3 life — a T1 roll would give +21 more life"
- Flag bricked items (bad mod combinations)

### Open Affix Detection
- Count prefixes and suffixes
- "Your boots have an open suffix — craft movement speed!"
- "Your ring has open prefix — craft life or flat fire damage"
- Suggest best benchcraft for each open slot
- Detect if item is full (no crafting possible)

### Crafting Suggestions
- Benchcraft recommendations per item
- Harvest craft possibilities
- Essence slam targets
- Eldritch implicit suggestions (Searing/Tangled)
- "Your helmet can have Eldritch fire exposure implicit — huge DPS gain"
- Veiled mod suggestions

### Corruption Suggestions
- High-value corruption outcomes per item
- Risk assessment (what you lose vs gain)
- "+2 to duration gems on your body armour = +25% DPS (but 75% chance to brick)"
- Implicit corruption tier list per slot

### Enchantment Suggestions
- Lab enchant priority per build
- Helmet enchant rankings for your main skill
- Boot enchant suggestions
- Glove enchant suggestions
- "RF helmet enchant: +40% RF damage > +RF AoE for your build"

### Socket/Link Analysis
- Check if links match skill requirements
- Chrome calculator (off-color probability)
- "Your chest needs 5R1B — use Vorici bench method"
- 6-link priority assessment

### Item Influence Detection
- Identify influence types
- Highlight valuable influenced mods
- Suggest influence crafting strategies
- "Your helmet is Shaper influenced — you can roll Conc Effect support"

---

## INVENTORY INTELLIGENCE

### Stash Tab Scanner (requires PoE OAuth)
- Index all items across stash tabs
- Find upgrades you already own
- "You have gloves in Tab 'Gear' that give +200 life over current pair"
- Cross-character item awareness
- Detect items that fit other characters better

### Currency Tracker
- Count all currency types
- Convert to divine/chaos equivalent
- Track currency changes over time
- "Total liquid currency: 47.5 divine orbs"
- Budget allocation suggestions

### Sellable Item Detector
- Find valuable items you're not using
- Price estimation for stash items
- "You have a Watcher's Eye worth ~15div sitting in your dump tab"
- Identify items worth listing on trade

### Div Card Tracker
- Track partial div card sets
- Calculate completion value
- "You have 4/6 The Doctor cards — 2 more = Headhunter"

### Fragment/Splinter Counter
- Track map fragments
- Splinter to emblem progress
- "You have 87/100 Simulacrum splinters — 13 more to run"

---

## MARKET INTELLIGENCE

### Live Price Checking
- Price every equipped item via poe.ninja/trade API
- "Your total build cost: 45 divine orbs"
- Per-slot price breakdown
- Historical price tracking

### Upgrade Shopping
- Find upgrades within budget
- Sort by cost-efficiency (DPS gained per divine spent)
- Direct trade links
- Price comparison across similar items
- "Best boot upgrade for 5div: +180 life, +30% ms, tri-res"

### Budget Upgrade Path
```
Budget: 50 divine orbs

Priority 1: Helmet — 8div
  → Gains: +12% DPS (fire exposure enchant)
  → Specific item link: [trade URL]

Priority 2: Amulet — 12div  
  → Gains: +18% DPS (+1 gem level, DoT multi)
  → Specific item link: [trade URL]

Priority 3: Boots — 6div
  → Gains: +300 life, ailment immunity
  → Specific item link: [trade URL]

Remaining: 24div — save for 6-link upgrade
```

### Price Trend Alerts
- Track price history for items you're watching
- "Aegis Aurora dropped 20% this week — good time to buy"
- "Divine orbs rising — sell chaos, buy divines"
- League economy phase detection

### Snipe Alerts
- Set up item filters
- "Item matching your search posted at 30% below market!"
- Real-time trade monitoring
- Whisper template generation

---

## BUILD PROGRESSION SYSTEM

### Act-by-Act Guide
```
Act 1-3 (Level 1-30):
  ✅ Pick up BBG links for your setup
  ✅ Get Quicksilver from Medicine Chest
  ❌ Missing: buy Goldrim from trade (1 chaos)
  → Next: Swap to RF at level 28 with +fire res gear

Act 4-6 (Level 30-50):
  ✅ Lab ready check (resist cap, life threshold)
  ❌ Need 4-link before Kitava
  → Vendor recipe: sell RGB linked = chromatic
  
Act 7-10 (Level 50-70):
  ✅ Uber lab viable check
  ❌ Chaos res needed for Act 9+
  → Priority: cap resists after Kitava penalty
```

### Mapping Checklist
```
White Maps Ready?
  ✅ Resist capped
  ✅ 4000+ life
  ❌ Need anti-freeze flask
  ❌ No movement skill detected

Yellow Maps Ready?
  ✅ 5000+ life
  ❌ Need chaos res > 0
  ❌ Missing curse immunity
  
Red Maps Ready?
  ❌ Need 5500+ life
  ❌ Need ailment immunity plan
  ❌ DPS below threshold for T14+
```

### Boss Readiness
- Per-boss gear/stat requirements
- "Shaper: READY (DPS sufficient, can survive slam)"
- "Maven: NOT READY (need more chaos res for memory game)"
- "Uber Elder: RISKY (no freeze immunity)"
- Gear swap suggestions per boss

### League Mechanic Optimization
- Detect current league mechanics
- Suggest build tweaks for league content
- "For Ritual: your AoE is low, consider Inc AoE swap"
- "For Delirium: need more move speed and DPS"

---

## BUILD COMPARISON ENGINE

### vs Top Players (poe.ninja)
- Fetch top builds matching your ascendancy + main skill
- Compare gear slot by slot
- Compare passive tree
- "Top RF Inquisitors average 4.2M DPS — you're at 2.8M"
- "90% use Aegis Aurora — consider switching from Rise of Phoenix"
- "Your tree matches 73% of top builds"

### vs Saved Builds
- Compare any two of your local builds
- Side-by-side stat comparison
- "Build A: more DPS, Build B: more tanky"
- Migration plan between builds

### Historical Self-Comparison
- Track your build's progression over time
- "This week: +800 life, +500k DPS, spent 23div"
- Graph DPS/life/resist over time
- Achievement milestones

---

## MULTI-CHARACTER MANAGEMENT

### Character Dashboard
- All characters at a glance
- Per-character build score
- Shared currency pool
- "Your Ranger has 3 unused exalts in inventory"

### Cross-Character Item Optimizer
- "This amulet on your Marauder would be +15% DPS on your Witch"
- Suggest item swaps between characters
- Leveling gear hand-me-down tracking

### Next Character Suggestions
- "Based on your stash, cheapest next build: Cold DoT Occultist (12div)"
- "You already own 60% of the gear for a Spark Inquisitor"
- League starter suggestions based on owned gear

---

## AI-POWERED FEATURES (Optional, uses API)

### Natural Language Build Advice
- "Why am I dying in T16 maps?"
- "What should I upgrade next with 20 divines?"
- "Explain why this passive tree path is better"
- Context-aware — AI sees your full build data

### Build Concept Generator
- "I want a fast mapper that uses fire skills"
- AI generates PoB-importable build concept
- Iterate with feedback

### Patch Impact Analysis
- Feed patch notes to AI
- "Your RF got +5% base damage — DPS increases to X"
- "Determination nerfed — here's how to compensate"
- Automatic build adjustment suggestions

### Community Build Translator
- Paste a build guide URL
- AI extracts gear/tree/gems
- Compare against your current build
- "To follow this guide, you need to change: X, Y, Z (cost: ~30div)"

---

## AUTOMATION & QUALITY OF LIFE

### Loot Filter Integration
- Generate/update loot filter based on your build needs
- "Highlight items with +fire res and +life (your weak stats)"
- FilterBlade API integration
- "Your loot filter is highlighting items you no longer need"

### PoE Trade Companion
- Auto-generate trade searches for upgrades
- Bulk trade optimization
- "Buy these 5 items together from same seller = save on trades"
- Price negotiation suggestions

### Build Export/Share
- Export build summary as image (shareable)
- Generate build guide from your build
- Reddit/forum formatted build post
- Short URL for sharing

### Hotkey Support
- Global hotkey to show/hide overlay
- Hotkey to scan current PoB build
- Hotkey to check prices
- Customizable key bindings

### Overlay Mode
- Transparent overlay on top of PoE
- Show suggestions while playing
- Quick item price check
- Mini resistance/stat bar

---

## DATA & ANALYTICS

### Build Database
- Store all builds locally with version history
- Tag and categorize builds
- Search across all builds
- "Show me all my builds that use Determination"

### Economy Dashboard
- Live economy overview
- Currency exchange rates
- Popular item trends
- League phase indicators

### Personal Statistics
- Total playtime estimation
- Currency earned/spent tracking
- Build completion percentage
- Achievement system ("First 6-link!", "Resist capped for 10 builds")

---

## TECHNICAL FEATURES

### Performance
- XML parsing < 50ms
- Suggestion engine < 200ms
- File watch latency < 100ms
- Cache poe.ninja data (refresh every 5 min)
- Offline mode (cached data works without internet)

### Settings
- AI provider selection (Claude/GPT/Gemini/Ollama/OpenRouter)
- Auto-apply suggestions toggle
- Notification preferences
- Theme (dark/light/PoE themed)
- Language support
- Backup retention policy

### Security
- API keys stored encrypted locally
- OAuth tokens in OS keychain
- No data sent externally without consent
- Open source — auditable

---

## COMBAT SIMULATOR ("The Arena")

### Map Monster Kill Time
- Simulate kill time vs map monsters at each tier
- Factor in monster HP scaling, ele res, damage mods
- Show hits-to-kill and seconds-to-kill
- Compare current build vs upgraded build
- "T16 rare: 1.6s / 3 hits → with upgrade: 1.4s / 2 hits"

### Boss Fight Simulation
- Full boss fight timeline with phases
- HP per phase, immunity windows, DPS check windows
- Death estimation per fight (average deaths)
- "Shaper: ~3:20 total, 3 deaths avg → after upgrades: ~2:30, 1 death"
- Uber boss viability check with minimum gear requirements
- Specific mechanic survival (slam, beam, memory game)

### Map Clear Speed
- Estimate clear time per map based on build clear pattern
- Factor in movement speed, AoE, DPS
- Currency per hour estimation
- Compare efficiency: "Strand T16: 2:30 clear, ~8 div/hr → with upgrade: 1:50, ~11 div/hr"

### Upgrade Impact on Combat
- Every upgrade suggestion includes fight-time impact
- "Ring 2 upgrade: Shaper fight -25s, T16 clear -15s"
- Compare investment cost vs time saved
- Show currency/hour improvement per divine spent

---

## DATA & PERSISTENCE

### Local Build Database (SQLite)
- Store all builds with full version history
- Schema: builds, snapshots, items, analysis_cache, price_history
- Git-like snapshots on every PoB change
- Tag/categorize builds (league starter, bosser, mapper)
- Search across all builds
- Export/import for backup

### Error Handling
- Structured error codes for all failure modes
- Graceful degradation (never crash, always show partial results)
- User-facing error messages with suggested fixes
- Error logging for bug reports

### Data Migration
- Version-tagged data formats
- Automatic migration on app update
- Backward-compatible reads (old data always loads)
- "Your data has been migrated to v2 format" notification

---

## IMPLEMENTATION PRIORITY

### Phase 1 — MVP (Month 1-2)
- Tauri project scaffold (Rust backend + TypeScript frontend)
- PoB XML read/write with backup (port pob-parser to Rust)
- Basic item scoring with mod tier detection
- Resistance checker
- Open affix detection
- Simple upgrade suggestions
- Combat simulator (basic monster/boss kill time)

### Phase 2 — Smart Analysis (Month 2-3)
- Full defensive analysis (EHP, ailment immunity, damage simulation)
- DPS breakdown with multiplier chain
- Passive tree efficiency scoring
- Gem optimization (level/quality/corruption/swap suggestions)
- Flask analysis and warnings

### Phase 3 — Market (Month 3-4)
- poe.ninja price integration
- Budget upgrade paths with cost-efficiency ranking
- Item comparison with DPS impact
- Buy timing advisor (league phase detection)

### Phase 4 — Intelligence (Month 4-5)
- Build progression system (act-by-act, mapping checklist, boss readiness)
- Build comparison vs top players (poe.ninja builds)
- Multi-character management
- Auto-update system for PoE league/patch data

### Phase 5 — AI + Polish (Month 5-6)
- Seer Engine (local AI) — train and integrate ItemNet, BuildNet, QueryNet
- Cloud AI provider integration (Claude/GPT/Gemini/Ollama)
- Natural language build advice
- Overlay mode

### Phase 6 — Advanced (Month 6+)
- Trade companion and snipe alerts
- Loot filter integration
- Patch impact analysis
- Community features (build sharing, ratings)
- macOS build and distribution

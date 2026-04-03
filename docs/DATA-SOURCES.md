# Path of AI — Data Sources & Missing Integrations

## Overview

This document covers what data we get from each source, what we're missing,
and new feature ideas discovered from reviewing PoB, poedb, and poewiki.

---

## 0. RePoE — Primary Game Data Source

**Repository:** [repoe-fork/repoe](https://github.com/repoe-fork/repoe) (more active than original)

ALL game data JSONs we need come from RePoE. Run the extraction on every PoE patch.

### Complete File List (what we MUST download)

| File | What It Contains | Priority |
|---|---|---|
| `mods.json` | ALL mod IDs, stat values, spawn weights, item tags | **P0** |
| `stats.json` | Stat ID definitions (local vs global, aliased) | **P0** |
| `stat_translations.json` | Stat ID → human-readable text (what appears on items) | **P0** |
| `base_items.json` | All base types: inventory size, item class, tags, requirements | **P0** |
| `gems.json` | All skill gems + support gems with stats per level | **P0** |
| `essences.json` | Essence → guaranteed mods per item class per tier | **P0** |
| `fossils.json` | Fossil tag multipliers + aux effects | **P0** |
| `crafting_bench_options.json` | All benchcrafts: cost, item restrictions, stats | **P0** |
| `default_monster_stats.json` | Monster base stats per level | **P0** |
| `characters.json` | Player base stats per class | **P0** |
| `mod_types.json` | Mod type info + fossil-relevant tags | **P0** |
| `tags.json` | All item tags (used in mods + base_items) | **P0** |
| `item_classes.json` | Item class definitions + influence types | **P1** |
| `cluster_jewels.json` | Cluster jewel generation rules | **P1** |
| `cluster_jewel_notables.json` | All cluster notable passives | **P1** |
| `active_skill_types.json` | Skill type categories for gem compatibility | **P1** |
| `gem_tags.json` | Gem tag ID → name translations | **P1** |
| `quest_rewards.json` | Which quest gives which gem reward per class | **P1** |
| `passive_tree.json` | Node positions + connections (for tree viewer) | **P0** |
| `cost_types.json` | Resource cost definitions | **P2** |
| `npc_master.json` | Master signature mods | **P2** |
| `uniques.json` | Unique item names + art files | **P2** |
| `flavour.json` | Flavor text (for UI only) | **P2** |

### How We Use Each File

```
Calculator needs:
  stats.json        → know which stats are local vs global
  mods.json          → mod tier boundaries for scoring
  gems.json          → gem scaling per level for DPS calc
  characters.json    → base stats to start calculation from
  default_monster_stats.json → enemy HP/res for combat sim

Crafting Advisor needs:
  mods.json          → spawn weights for probability calc
  fossils.json       → tag multipliers for fossil craft sim
  essences.json      → guaranteed mods per essence
  crafting_bench_options.json → available benchcrafts
  mod_types.json     → which tags each mod has (for fossil interaction)

Item Analysis needs:
  base_items.json    → validate base type + requirements
  stat_translations.json → display mod text correctly
  tags.json          → item tag validation
  item_classes.json  → categorize items correctly
```

---

## 1. POE OFFICIAL API (PRIMARY — OAuth Character Import)

**This is the DEFAULT way to get build data.** No PoB required.

### OAuth Setup
```
Authorization URL: https://www.pathofexile.com/oauth/authorize
Token URL:         https://www.pathofexile.com/oauth/token
Scopes needed:     account:profile account:stashes account:characters
Redirect URI:      http://localhost:{PORT}/callback (local server)
```

### API Endpoints We Use

> **NOTE:** Exact endpoint URLs must be verified against the official
> [PoE Developer API Reference](https://www.pathofexile.com/developer/docs/reference)
> before implementation. The endpoints below are based on community documentation
> and may need updating. Register for API access via oauth@grindinggear.com.

| Endpoint (verify) | Category | What It Returns |
|---|---|---|
| `GET /api/account/profile` | Account Profile | Account name, badges |
| `GET /api/account/characters` | Account Characters | All characters (name, class, level, league) |
| `GET /api/character/{name}` | Character Detail | Equipped items, gems, sockets |
| `GET /api/character/{name}/passives` | Passive Tree | Allocated nodes, jewels, masteries |
| `GET /api/stash/{league}` | Stash Tabs (PoE1) | Tab list (names, types, colors) |
| `GET /api/stash/{league}/{tabId}` | Stash Items | Items in a specific tab |

**Required headers:**
```
User-Agent: OAuth pathofai/0.1.0 (contact: dev@pathofai.com)
Authorization: Bearer {access_token}
```

**Rate limits (dynamic):**
- Check `X-Rate-Limit-*` response headers
- Typical: 45 requests per 60 seconds
- Implement exponential backoff on 429 responses

### API Response → BuildData Conversion

```
PoE API character response:
  → items[] (each item has: name, base, mods, sockets, links)
  → Convert each mod string to structured ModInfo (tier detection)
  → Map socket groups to gem links
  → Parse passive tree hashes → node IDs → allocated nodes
  → Detect: class, ascendancy, level, bandits, pantheon
  → Build: same BuildData struct as PoB parser produces
  
Result: BuildData that works identically with our Calculator
  regardless of whether it came from OAuth or PoB XML.
```

### Differences: OAuth Data vs PoB Data

| Aspect | PoE OAuth (live) | PoB XML (file) |
|---|---|---|
| **Items** | Exact current gear | May be outdated |
| **Gems** | Exact current levels | May be planned (not yet leveled) |
| **Tree** | Exact allocated nodes | May include planned nodes |
| **Config** | NOT available (no boss selection, no flask config) | Available (boss type, charges, flasks) |
| **Calcs** | NOT available (must use our calculator) | Available (PoB pre-computed DPS/life) |
| **Stash** | Available (live currency, items) | NOT available |
| **Theorycraft** | NOT possible (live data only) | Yes (plan builds before equipping) |

**Implication for our Calculator:**
- OAuth builds: we MUST calculate everything ourselves (no PoB Calcs section)
- PoB builds: we CAN use PoB's pre-computed Calcs as a shortcut/verification
- Our Rust calculator must handle BOTH cases identically

### Rate Limits
```
PoE API rate limits:
  - 45 requests per 60 seconds (per IP)
  - Account-specific endpoints: 30 per 60 seconds
  - Stash endpoints: 20 per 60 seconds
  
Our strategy:
  - Fetch character data on first load → cache for 5 minutes
  - Fetch stash data on demand → cache for 5 minutes
  - Show "refreshing..." when re-fetching
  - Never auto-refresh faster than rate limits
  - Circuit breaker: if 3 consecutive 429s → wait 120 seconds
```

---

## 2. PATH OF BUILDING (PoB XML — OPTIONAL Alternative)

### What We Parse (pob-parser.js)
- Build: class, level, ascendancy, bandit, pantheon, stats
- Items: rarity, name, base, quality, sockets, implicits, explicits
- ItemSets: slot-to-item mapping
- Skills: gem links with gemId, level, quality, enabled
- Tree: active spec, node IDs, class/ascend IDs
- Config: combat settings, boss selection

### P0 — Must Parse (Currently Ignored)

**Calcs Section**
PoB pre-calculates DPS, life, ES, resists, etc. We currently discard this.
This is FREE data that eliminates need for our fast estimator.
```xml
<Calcs>
  <PlayerStat stat="Life" value="6453"/>
  <PlayerStat stat="FireDotDPS" value="2841057"/>
  ...
</Calcs>
```

**Crafted Mod Flag**
Mods tagged `{crafted}` are benchcrafted — can be freely replaced.
Essential for upgrade suggestions ("you can replace this crafted mod").
```
{crafted}+70 to maximum Life
```

**Mastery Effects**
`<MasteryEffects>` inside Spec lists which mastery was chosen at each node.
Mastery choices significantly affect DPS/defense.

**Jewel Socket Assignments**
`<Socket>` elements map jewel items to tree socket nodeIds.
Without this, we can't analyze jewel effectiveness.

**Cluster Jewel Structure**
Cluster jewels have `{variant:X}` tags, notable assignments, enchant type
(Small/Medium/Large, # passives). Parser treats them as generic items.

### P1 — Important

- Notes section (user build notes — useful AI context)
- Spectre list (critical for minion builds)
- Timeless jewel seeds + keystone transforms
- Anointment data (e.g., `Allocates Whispers of Doom`)
- Eldritch implicit source tags (Searing Exarch / Eater of Worlds)
- Fractured mod flag (`{fractured}`)
- Influence type tags (`{shaper}`, `{elder}`, etc.)
- Gem alternate quality (qualityId: Anomalous/Divergent/Phantasmal)

### P2 — Nice to Have
- TreeView section (zoom, search state)
- Veiled mod detection (`{crafted}{veiled}`)
- Item variant field (unique items with variants)
- Mod range rolls (`{range:X}` — roll position 0.0-1.0)

---

## 3. POEDB.TW

### P0 — Must Integrate

**Mod Weighting Tables**
Exact spawn weights for every prefix/suffix per base type + ilvl + influence.
Without this, our Crafting Planner can't calculate accurate costs.
Used for: "Expected cost to craft: ~5 divine (30 attempts avg)"

**Fossil Weight Multipliers**
Exact multiplier each fossil applies to each mod tag.
Required for: "Use Pristine + Scorched fossil combo for +life +fire DoT"

### P1 — Important

- Essence guaranteed mod tiers per item slot
- Harvest craft outcome pools (add fire, remove cold, reforge, etc.)
- Eldritch implicit tier tables (all tiers per slot for Exarch/Eater)
- Monster database (HP, damage, resistances per area level)
- Boss attack damage values (makes combat simulator accurate)
- Atlas passive tree data (node list + stats)
- Veiled mod pool (which Syndicate member → which veiled mod)
- Item base type implicit values (validate implicits against base data)

### P2 — Nice to Have
- League mechanic reward data (drop rates, scaling)
- Recombinator rules and probabilities
- Sanctum boon/affliction data

---

## 4. POEWIKI.NET

### P1 — Important

**Boss Attack Pattern Database**
Full boss moveset: attacks, phases, cooldowns, telegraphs, safe DPS windows.
Would transform combat sim from "can you survive X hit" to "here's the fight flow."

**Skill Gem Tags + Scaling**
Complete tag list per gem + damage effectiveness values.
Needed for "which support gems work with this skill" without relying on PoB.

**Divination Card Drop Locations**
Which maps/zones drop which cards.
Our Div Card Tracker says "drop location reminders" but has no data.

**Vendor Recipe Database**
Full recipe list with inputs/outputs.
Our Act-by-Act Guide mentions recipes but we don't have comprehensive data.

**Damage Conversion Chains**
Exact conversion order (phys → lightning → cold → fire → chaos).
Mentioned in features but needs the actual conversion graph.

**Aura Reservation Formulas**
Exact calculations including reduced mana reservation efficiency.
Improves our Mana Reservation Calculator.

**Game Mechanics Formulas (Complete)**
Armour, evasion entropy, block calculation, DoT mechanics, ailment thresholds.
Our fast estimator approximates these — exact formulas would improve accuracy.

### P2 — Nice to Have
- Ailment threshold formulas (freeze/shock/ignite per monster life)
- Status effect scaling (shock effect, chill, sap/brittle/scorch)
- Curse mechanics (effect scaling, limits, hex vs mark)
- Atlas mechanics (voidstone effects, map tier upgrades)
- Lab enchantment pools per slot

---

## 4. NEW FEATURE IDEAS (from data source review)

### Crafting Probability Calculator
Using poedb mod weights, show exact probability + expected cost:
```
Target: +1 Fire Gem Level on Amulet
Method: Alt spam
Probability per attempt: 1 in 340
Expected cost: ~2 divine (340 alts)
Better method: Essence of Rage (guaranteed, ~0.5 div each)
```

### Boss Fight Playbook
Using poewiki boss attack data, generate per-boss strategy:
```
Shaper Phase 1:
  - Slam (8000 phys): dodge → you survive if hit
  - Beam (12000 cold over 4s): DODGE — lethal in 0.8s
  - Ball Lightning: tank with regen
  - Safe DPS windows: after slam recovery (2s), during teleport (1.5s)
  Estimated DPS uptime: 60% → effective DPS: 1.7M
```

### Crafting Step Simulator
Interactive "try crafting" that uses real mod weights:
```
Step 1: Alteration spam on Opal Ring (ilvl 84)
  [Roll] → +32% Fire Resistance (T3)
  [Roll] → +22% Cold Resistance (T4)  
  [Roll] → +15% Fire DoT Multi (T2) ← HIT! Regal it.
Step 2: Regal Orb
  [Roll] → +62 Life (T4)
  Options: Benchcraft suffix + done, or Annul gamble
```

### Vendor Recipe Advisor
Based on your stash contents, suggest profitable vendor recipes:
```
You have: Sapphire Ring + Iron Ring + any boots with blue socket
Recipe: Sapphire Ring → useful for leveling cold builds
Sell value: 1-2 chaos (trade) vs vendor (chromatic)
```

### Map Strategy Advisor
Combine atlas tree + build analysis + poedb monster data:
```
Your build: RF Inquisitor (fire DoT, tanky, medium clear)
Best atlas strategy: Essence + Harvest + Strongbox
Favorite maps: Strand (linear), Dunes (open), Tropical Island (layout)
Avoid: Crimson Temple (tight corridors reduce RF AoE efficiency)
Scarab combo: Polished Essence + Rusted Ambush + Rusted Divination
Expected return: ~10 div/hr
```

### Div Card Target Farming
From poewiki drop locations + your build's map preferences:
```
Cards you're close to completing:
  The Nurse (7/8) → drops in Tower map
  The Doctor (4/6) → drops in Spider Forest, Burial Chambers

Recommended: Favorite Tower map for The Nurse (1 card away!)
Expected runs: ~30 Tower maps for 1 Nurse drop
```

---

## 5. PRIORITY IMPLEMENTATION ORDER

### Sprint 1: Parse More PoB Data
1. Parse `<Calcs>` section → free stats
2. Detect `{crafted}` mod flag
3. Parse `<MasteryEffects>`
4. Parse jewel socket assignments
5. Parse cluster jewel structure

### Sprint 2: Integrate poedb Mod Data
6. Download mod weight tables → game-data/mods/
7. Implement crafting probability calculator
8. Add fossil/essence craft simulation

### Sprint 3: Boss + Map Intelligence
9. Integrate boss attack data from poewiki
10. Build boss fight playbook generator
11. Add map strategy advisor
12. Add div card target farming

### Sprint 4: Vendor + Atlas
13. Vendor recipe database
14. Atlas passive tree data + suggestions
15. Div card drop location tracking

---

## 5. MISSING DATA SOURCES (Identified in review)

### Map Layout Ratings
**Problem:** Our "Favorite Map Suggestions" feature needs layout quality data
(linear, open, tight, backtrack-heavy) but NO public data source exists.

**Solution options:**
- **Option A:** Community-maintained JSON (we create + maintain, community contributes)
- **Option B:** Scrape from PoE community tier lists (Reddit, Maxroll)
- **Option C:** Hardcode ratings for top 50 maps (good enough for MVP)

**Recommended:** Option C for MVP → Option A post-launch.

```json
// game-data/maps/layout-ratings.json (manually curated)
{
  "Strand": { "layout": "linear", "rating": 5, "backtrack": false, "boss_difficulty": 2 },
  "Dunes": { "layout": "open", "rating": 4, "backtrack": false, "boss_difficulty": 3 },
  "Crimson Temple": { "layout": "tight", "rating": 2, "backtrack": true, "boss_difficulty": 4 },
  // ~50 maps for MVP
}
```

### Anointment Oil Costs
**Problem:** Our "Anoint Planner" needs to show oil combinations + costs per notable.

**Source:** PoB has this data internally (NotableAnointments table in Lua).
Also available on poewiki: `poewiki.net/wiki/List_of_anointments`

**Data structure:**
```json
// game-data/crafting/anointments.json
{
  "Whispers of Doom": {
    "oils": ["Golden", "Golden", "Silver"],
    "oil_cost_chaos": 850,   // from poe.ninja oil prices
    "notable_stats": ["+1 to maximum number of Curses"]
  },
  "Breath of Flames": {
    "oils": ["Amber", "Crimson", "Black"],
    "oil_cost_chaos": 120,
    "notable_stats": ["+20% Fire Damage over Time Multiplier"]
  }
}
```

### Atlas Passive Tree Data
**Problem:** Our "Atlas Strategy Advisor" needs atlas node data.

**Source:** PoB has atlas tree data. Also extractable from PoE's game files via PyPoE.

**Note:** Atlas tree changes every league. Must be updated per league launch.

### Passive Tree Position Data (for Tree Viewer)
**Problem:** Our "Full Passive Tree Visualization" needs node X/Y positions.

**Source:** PoB's tree data includes positions. Also available from:
- GGG's official passive tree JSON: `pathofexile.com/passive-skill-tree` (returns JSON)
- RePoE may not include positions — need to verify

**Action:** Download from GGG's official endpoint, bundle as `game-data/tree/passive-tree-positions.json`

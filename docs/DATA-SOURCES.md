# Path of AI — Data Sources & Missing Integrations

## Overview

This document covers what data we get from each source, what we're missing,
and new feature ideas discovered from reviewing PoB, poedb, and poewiki.

---

## 1. PATH OF BUILDING (PoB XML)

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

## 2. POEDB.TW

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

## 3. POEWIKI.NET

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

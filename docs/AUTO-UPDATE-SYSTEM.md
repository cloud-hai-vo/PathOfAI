# Path of AI — Auto-Update & League Data System

## Problem

Path of Exile releases a new league every ~3 months. Each league brings:
- New/changed skill gems
- Passive tree rework
- New unique items
- Mod tier changes
- New base types
- Balance changes (DPS formulas, defense mechanics)
- New league mechanics

**Our tool must stay current without requiring code changes.**

---

## 1. ARCHITECTURE: DATA-DRIVEN, NOT CODE-DRIVEN

### Core Principle

All PoE game knowledge lives in **data files**, not in code logic. The code reads
data files at runtime. When PoE updates, we update data files only — no code changes.

```
Code (rarely changes)          Data (updates every league)
─────────────────────          ───────────────────────────
pob-parser.js                  data/mods/mod-tiers.json
build-analyzer.js              data/gems/active-gems.json
build-detector.js              data/gems/support-gems.json
mod-impact-calculator.js       data/tree/passive-tree-3.25.json
market-intelligence.js         data/items/unique-items.json
seer-engine.js                 data/items/base-types.json
                               data/crafting/bench-crafts.json
                               data/patches/balance-changes.json
                               data/meta/version.json
```

### Data File Format

Every data file includes a version header:

```json
{
  "_meta": {
    "poeVersion": "3.25",
    "leagueName": "SettlersSeason2",
    "dataVersion": "2026.04.01",
    "source": "RePoE + poe.ninja",
    "generatedBy": "scripts/extract-game-data.py"
  },
  "data": { ... }
}
```

---

## 2. AUTO-UPDATE PIPELINE

### Flow

```
PoE Patch Released (GGG)
        ↓
GitHub Action triggers (manual or scheduled)
        ↓
scripts/extract-game-data.py runs
  → Pulls from RePoE (game data) 
  → Pulls from poe.ninja (meta builds, prices)
  → Generates updated JSON data files
  → Validates data integrity
        ↓
Creates GitHub Release
  → poe-data-v{version}.zip (~20-50MB)
  → release-manifest.json (what changed, checksums)
        ↓
App checks for updates on startup
  → Compares local data/meta/version.json vs remote manifest
  → Downloads delta update (only changed files)
  → Applies update (swap data directory)
  → No restart needed for data-only updates
```

### Update Check Schedule

```
On app startup:           check immediately
Every 6 hours:            background check while app is running
On user trigger:          "Check for Updates" button
On league launch day:     check every 30 minutes for first 48 hours
After major PoE patch:    force check on next app launch
```

---

## 3. DATA EXTRACTION SCRIPTS

### scripts/extract-game-data.py

```python
"""
Extracts all PoE game data from RePoE and other sources.
Run this script after every PoE patch to generate updated data files.

Usage:
  python scripts/extract-game-data.py --poe-version 3.25 --output data/

Sources:
  - RePoE (GitHub): mods, gems, base types, passive tree
  - poe.ninja API: prices, meta builds, economy data
  - PoE CDN: item images manifest
"""

# Step 1: Download latest RePoE data
# Step 2: Parse and transform into our data format
# Step 3: Fetch poe.ninja for current league data
# Step 4: Generate all JSON data files
# Step 5: Generate version manifest
# Step 6: Validate data integrity (schema checks)
# Step 7: Output to data/ directory
```

### What Gets Extracted

| Data File | Source | Size | Update Frequency |
|-----------|--------|------|-----------------|
| mod-tiers.json | RePoE | ~2MB | Every patch |
| active-gems.json | RePoE | ~500KB | Every patch |
| support-gems.json | RePoE | ~300KB | Every patch |
| unique-items.json | RePoE + poedb | ~1MB | Every patch |
| base-types.json | RePoE | ~400KB | Every patch |
| passive-tree-{ver}.json | RePoE | ~2MB | Every patch |
| bench-crafts.json | RePoE | ~200KB | Every patch |
| gem-interactions.json | Community | ~100KB | As needed |
| balance-changes.json | Patch notes | ~50KB | Every patch |
| meta-builds.json | poe.ninja | ~500KB | Daily (auto) |
| price-baselines.json | poe.ninja | ~1MB | Every 5 min (runtime) |
| image-manifest.json | PoE CDN | ~200KB | Every patch |

---

## 4. VERSIONED DATA LOADING

### Runtime Data Loader

```javascript
/**
 * DataLoader — loads game data from versioned JSON files.
 * Falls back to bundled data if update files not found.
 */
class DataLoader {
  constructor(dataDir) {
    this.dataDir = dataDir;           // %AppData%/PathOfAI/data/
    this.bundledDir = './data/';      // Bundled with app (fallback)
    this.version = null;
  }

  async initialize() {
    // 1. Try loading from updated data directory
    // 2. Fall back to bundled data if not found
    // 3. Store version info for update checking
    this.version = await this.loadJSON('meta/version.json');
  }

  async loadModTiers() {
    return this.loadJSON('mods/mod-tiers.json');
  }

  async loadPassiveTree() {
    const ver = this.version?.poeVersion || '3.25';
    return this.loadJSON(`tree/passive-tree-${ver}.json`);
  }

  async loadJSON(relativePath) {
    // Try updated data first, fall back to bundled
    const updatedPath = path.join(this.dataDir, relativePath);
    const bundledPath = path.join(this.bundledDir, relativePath);

    if (await fileExists(updatedPath)) {
      return JSON.parse(await readFile(updatedPath));
    }
    return JSON.parse(await readFile(bundledPath));
  }
}
```

### How Core Modules Use It

```javascript
// Before: hardcoded game data (BAD — breaks every patch)
const GEM_DATABASE = {
  RighteousFire: { tags: ["fire", "dot"], ... }
};

// After: loaded from versioned data files (GOOD — auto-updates)
class BuildDetector {
  constructor(buildData, dataLoader) {
    this.gemDatabase = dataLoader.loadGemDatabase();
    // ... rest of construction
  }
}
```

---

## 5. UPDATE MANIFEST FORMAT

### release-manifest.json (hosted on GitHub Releases)

```json
{
  "latestVersion": "2026.04.01",
  "poeVersion": "3.25",
  "leagueName": "SettlersSeason2",
  "releaseDate": "2026-04-01T00:00:00Z",
  "changelog": [
    "Updated passive tree for 3.25",
    "Added 12 new unique items",
    "Updated mod tiers for balance changes",
    "New league mechanic data"
  ],
  "files": [
    {
      "path": "mods/mod-tiers.json",
      "checksum": "sha256:abc123...",
      "size": 2048000,
      "changed": true
    },
    {
      "path": "tree/passive-tree-3.25.json",
      "checksum": "sha256:def456...",
      "size": 2100000,
      "changed": true
    }
  ],
  "fullPackageUrl": "https://github.com/.../poe-data-v2026.04.01.zip",
  "fullPackageSize": 20000000,
  "fullPackageChecksum": "sha256:..."
}
```

---

## 6. DELTA UPDATES

### Only Download What Changed

```
User has: data version 2026.03.15 (patch 3.24)
Latest:   data version 2026.04.01 (patch 3.25)

Changed files:
  ✓ passive-tree-3.25.json (new file, 2MB)
  ✓ mod-tiers.json (modified, 2MB)
  ✓ unique-items.json (modified, 1MB)
  ✓ active-gems.json (modified, 500KB)
  ✗ bench-crafts.json (unchanged, skip)
  ✗ base-types.json (unchanged, skip)

Download: 5.5MB instead of 20MB full package
```

### Atomic Apply

```
1. Download changed files to temp directory
2. Verify checksums for all downloaded files
3. If all pass: swap data directory atomically
4. If any fail: keep old data, report error
5. User sees: "Game data updated to patch 3.25!"
```

---

## 7. LEAGUE LAUNCH DAY HANDLING

### Automatic Detection

```javascript
class LeagueDetector {
  // GGG announces league dates weeks in advance
  // We maintain a simple schedule file
  knownLeagueDates = {
    "2026-Q1": "2026-01-10",
    "2026-Q2": "2026-04-11",
    "2026-Q3": "2026-07-18",
    "2026-Q4": "2026-10-17",
  };

  isLeagueLaunchWeek() {
    const now = new Date();
    return Object.values(this.knownLeagueDates).some(date => {
      const launch = new Date(date);
      const diffDays = Math.abs(now - launch) / (1000 * 60 * 60 * 24);
      return diffDays <= 7;
    });
  }

  getUpdateCheckInterval() {
    if (this.isLeagueLaunchWeek()) return 30 * 60 * 1000;  // 30 min
    return 6 * 60 * 60 * 1000;                               // 6 hours
  }
}
```

### League Launch Workflow

```
Day -7:  GGG releases patch notes
         → We prepare data extraction scripts for new content
         → Pre-generate as much data as possible from teasers

Day -1:  PoE patch downloads (torrent available)
         → Run PyPoE extraction on new game files
         → Generate data files
         → Push to GitHub Release as "pre-release"

Day 0:   League launches
         → Finalize data (some things only known at launch)
         → Push stable release
         → App auto-updates within 30 min of launch

Day 1-3: Hotfix patches common
         → Monitor for balance hotfixes
         → Quick data updates as needed
         → poe.ninja meta builds start populating

Week 1:  Meta stabilizes
         → Update meta-builds.json from poe.ninja
         → Seer Engine LoRA update with new league Q&A
```

---

## 8. GRACEFUL DEGRADATION

### What Happens If Update Fails

| Scenario | Behavior |
|----------|----------|
| No internet | Use bundled/cached data, show "offline" badge |
| GitHub down | Use last cached data, retry in 1 hour |
| Corrupt download | Keep old data, retry download |
| New PoE version, no data yet | Use last known data + warn user "data may be outdated" |
| Brand new unique item unknown | Show "Unknown item" with generic analysis |
| Passive tree changed | Old tree still works for existing builds, warn about differences |

### Offline-First

The app bundles a complete data set at build time. This means:
- First launch works immediately, no download needed
- Full offline functionality with bundled data
- Updates improve data, but base data is always available
- Users in regions with poor internet still get full functionality

---

## 9. HANDLING BREAKING GAME CHANGES

### The Problem

GGG makes breaking changes every league. Examples from real patches:
```
3.28 Mirage:
  - Awakened Support Gems REMOVED from drop pool (can't suggest them anymore)
  - Djinn Coins added (new currency type our engine doesn't know)
  - Exceptional Support Gems added (40+ new gems to rank)
  - T17 maps removed (boss farming strategy changes)
  - Reliquarian Scion ascendancy added (new build archetype)
  - Atlas passive tree completely reworked

3.25 (hypothetical):
  - Determination nerfed: base armour reduced 30%
  - New keystone: "Eternal Youth" (ES recharge applies to life)
  - Molten Shell reworked: now scales with evasion, not armour
  - Divine Orb replaced by "Chaos Shard" as primary currency
```

**If our app still suggests Awakened gems after 3.28, it's WRONG and users lose trust.**

### Solution: Data Versioning + Breaking Change Detection

Every game data file has a version tag:
```json
{
  "_meta": {
    "poeVersion": "3.28",
    "dataVersion": "2026.03.06",
    "breakingChanges": [
      {
        "type": "removed",
        "category": "gems",
        "ids": ["SupportAwakened*"],
        "description": "Awakened Support Gems removed from game"
      },
      {
        "type": "added",
        "category": "gems",
        "ids": ["SupportExceptional*"],
        "description": "Exceptional Support Gems added"
      },
      {
        "type": "removed",
        "category": "maps",
        "ids": ["MapT17*"],
        "description": "T17 maps removed"
      },
      {
        "type": "changed",
        "category": "formulas",
        "ids": ["determination_base_armour"],
        "description": "Determination base armour reduced 30%"
      },
      {
        "type": "added",
        "category": "currency",
        "ids": ["DjinnCoin*"],
        "description": "Djinn Coins — new league currency"
      }
    ]
  },
  "data": { ... }
}
```

### How the App Handles Each Change Type

```
TYPE: "removed" (item/gem/mechanic no longer exists)
  ──────────────────────────────────────────────────
  ACTION:
    1. Remove from suggestion pool immediately
    2. If user's build uses removed item/gem:
       → Show warning: "⚠ Awakened Burning Damage was removed in 3.28.
         The Seer suggests: Exceptional Burning Damage (new) as replacement."
    3. If user's PoB file references removed gem:
       → Parse still works (backwards compatible)
       → Calculator flags it: "This gem no longer exists in 3.28"
       → Prophecy suggests replacement
    4. Market prices for removed items → mark as "Legacy (Standard only)"

  EXAMPLE:
    User loads build with Awakened Burning Damage Support
    → Calculator still calcs DPS with it (the gem data is in old version)
    → Warning banner: "⚠ This gem was removed in 3.28"
    → Prophecy: "Replace with Exceptional Burning Damage: -2% DPS but available"
    → If user is on Standard league: no warning (Awakened gems still exist there)


TYPE: "added" (new item/gem/mechanic)
  ──────────────────────────────────────────────────
  ACTION:
    1. Add to gem/item database
    2. Calculator includes new gems in "best support" ranking
    3. Prophecy suggests new items if they improve build
    4. The Forge includes new currency types

  EXAMPLE:
    Exceptional Support Gems added in 3.28
    → gem_database gets 40+ new entries
    → Calculator: "Exceptional Burning Damage: +37% more burning → ranks #2 for RF"
    → Prophecy: "New gem available: Exceptional Burning Damage (8 div, +12% DPS)"
    → Djinn Coins added → craft_advisor can suggest imbuing gems


TYPE: "changed" (formula/value modification)
  ──────────────────────────────────────────────────
  ACTION:
    1. Update formula constants in game data JSON (NOT in code)
    2. Re-run calculator → all DPS/defense numbers update automatically
    3. If major change: show notification to user
    4. If formula structure changed (not just values): requires code update

  EXAMPLE:
    Determination base armour reduced 30%
    → game-data/gems/determination.json: base_armour: 3000 → 2100
    → Calculator: DPS unchanged, but armour drops → EHP warning
    → Defense panel: "⚠ Armour dropped from 28,450 to 22,450 after patch"
    → Prophecy: "Consider Grace or Defiance Banner to compensate"


TYPE: "reworked" (mechanic fundamentally changed)
  ──────────────────────────────────────────────────
  ACTION:
    1. Data JSON update handles simple reworks (value changes)
    2. If rework changes HOW a formula works: requires APP UPDATE (not just data)
    3. App checks: "data requires app version >= 0.3.0"
    4. If app version too old: show "Update Path of AI to support 3.28 changes"

  EXAMPLE:
    Molten Shell reworked: now scales with evasion instead of armour
    → This changes the FORMULA, not just values
    → Data file: { "requires_app_version": "0.3.0", "breaking_formula": "guard_molten_shell" }
    → Old app version: "⚠ Molten Shell was reworked in 3.28. Update Path of AI for accurate calc."
    → New app version: formula updated in code, data values updated in JSON
```

### League-Specific Data Handling

```
When new league launches:

DAY -7 (patch notes released):
  → We read patch notes manually
  → Create breaking_changes.json with all removals/additions/changes
  → Pre-generate data files where possible (gem values, tree layout from datamine)

DAY -1 (patch downloadable):
  → Run RePoE extraction on new game files
  → Generate complete data JSONs
  → Verify against breaking_changes.json (did we catch everything?)
  → Push as "pre-release" to GitHub

DAY 0 (league launch):
  → Finalize any last-minute hotfixes
  → Push stable release
  → App auto-downloads within 30 minutes

DAY 1-7 (hotfix period):
  → GGG often hotfixes balance numbers
  → Monitor patch notes → update data JSONs
  → Push micro-updates (just changed files)

WEEK 2+ (stable):
  → Meta settles → update archetype stat weights from poe.ninja
  → Price data stabilizes → buy advisor becomes accurate
  → Craft probabilities verified against community data
```

### League Detection

```rust
pub fn detect_league_context(build: &BuildData, data: &GameData) -> LeagueContext {
    LeagueContext {
        current_league: data.meta.league_name.clone(),     // "Mirage"
        poe_version: data.meta.poe_version.clone(),        // "3.28"
        data_version: data.meta.data_version.clone(),      // "2026.03.06"
        is_standard: build.league == "Standard",
        is_hardcore: build.league.contains("Hardcore"),
        is_ssf: build.league.contains("SSF"),
        
        // Standard league keeps ALL items (including removed ones)
        // League-specific items may not exist in Standard yet
        allow_legacy_items: build.league == "Standard",
        allow_league_items: !build.league.contains("Standard"),
    }
}

// When suggesting items:
if !league_context.allow_legacy_items && gem.removed_in <= current_version {
    skip_suggestion("Gem removed in patch X");
    suggest_replacement(gem);
}
```

---

## 10. DATABASE MIGRATION DURING AUTO-UPDATE

### The Problem

When we update game data, the SQLite database may need schema changes:
- New tables (e.g., `djinn_coins` tracking in Mirage league)
- New columns (e.g., `imbued_support` field on gems table)
- Changed indexes (new query patterns for new features)
- Data cleanup (remove cached prices for items that no longer exist)

### Migration Strategy

```
PRINCIPLE: Data updates and schema updates are SEPARATE.

DATA UPDATE (every patch, automatic):
  → Download new JSON files from GitHub Releases
  → Swap game-data/ directory atomically
  → NO database schema change needed
  → Calculator loads new data → all numbers update

SCHEMA UPDATE (with app updates, less frequent):
  → App version 0.1.0 → 0.2.0 includes new tables/columns
  → On app startup: check schema_version table
  → If schema < expected: run migration scripts
  → Migrations are ADDITIVE (never delete data)

CACHE INVALIDATION (every patch):
  → price_cache: delete ALL rows (prices change every league)
  → price_history: keep (historical data is valuable)
  → build snapshots: keep (user's data)
  → alerts: keep but validate (is alerted item still in game?)
```

### Migration Script System

```sql
-- Stored in app binary as embedded SQL
-- Each migration has a version number and description
-- Run in order, skip already-applied migrations

-- Check current version
SELECT MAX(version) FROM schema_version;
-- Result: 1 (current)

-- If version < 2, run migration v2:
-- Migration v2: Add Mirage league support (3.28)
BEGIN TRANSACTION;

-- Add imbued support tracking to gem analysis cache
ALTER TABLE builds ADD COLUMN imbued_gems TEXT;  -- JSON array of imbued gem data

-- Add djinn coin tracking
CREATE TABLE IF NOT EXISTS league_currency (
    currency_name TEXT PRIMARY KEY,
    count         INTEGER DEFAULT 0,
    league        TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

-- Invalidate stale caches from previous league
DELETE FROM price_cache WHERE league != 'Mirage';

-- Validate alerts: remove alerts for items that no longer exist
-- (app code handles this, not SQL — needs game data lookup)

-- Record migration
INSERT INTO schema_version VALUES (2, datetime('now'), 'Mirage league support');

COMMIT;
```

### Migration Rules

```
RULE 1: NEVER delete user data
  → Don't drop tables with build history
  → Don't delete user settings
  → Don't remove undo snapshots

RULE 2: Migrations are ADDITIVE
  → Only ADD columns, tables, indexes
  → Never REMOVE columns (old data stays, just unused)
  → Exception: cache tables CAN be cleared (they rebuild)

RULE 3: Migrations are IDEMPOTENT
  → Running same migration twice is safe (IF NOT EXISTS)
  → Always check schema_version before running

RULE 4: Migrations run BEFORE app starts
  → User sees: "Updating database for patch 3.28..."
  → Takes < 2 seconds for typical migrations
  → If migration fails: rollback transaction, use old schema, show warning

RULE 5: Data migration happens in BACKGROUND
  → Schema change: synchronous (before app loads)
  → Data validation: asynchronous (after app loads)
  → Example: "Checking 15 price alerts for removed items... 2 removed."
```

### Cache Invalidation Per Patch

```
EVERY LEAGUE START:
  price_cache        → DELETE ALL (prices reset every league)
  price_history      → KEEP (but start new league's history)
  build analysis     → RE-CALCULATE (game data changed → results change)
  stash data         → RE-FETCH (stash resets per league in temp leagues)
  
EVERY PATCH (mid-league):
  price_cache        → DELETE ALL (prices shift after balance changes)
  build analysis     → RE-CALCULATE only if affected by patch changes
  alerts             → VALIDATE (check if alerted item still exists)
  
EXAMPLE — 3.28 Mirage launch:
  1. Download new game data (gems, mods, tree, uniques)
  2. Run schema migration v2 (add imbued_gems, league_currency)
  3. Clear ALL price caches
  4. Re-analyze current build:
     → "⚠ Your Awakened Burning Damage no longer exists in Mirage"
     → "Suggestion: Exceptional Burning Damage (new, similar effect)"
  5. Validate alerts:
     → Alert for "Awakened Burning Damage < 5 div" → remove (item gone)
     → Alert for "Aegis Aurora < 15 div" → keep (still exists)
  6. Update UI: show "Updated to patch 3.28 — Mirage league"
```

### App Version vs Data Version Compatibility

```
COMPATIBILITY MATRIX:

App Version  | Min Data Version | Max Data Version | Notes
0.1.0        | 3.24             | 3.28             | Basic calc, no imbued gems
0.2.0        | 3.28             | 3.29+            | Imbued gems, Exceptional supports
0.3.0        | 3.29             | 3.30+            | Formula reworks from 3.29

DATA FILE:
  {
    "_meta": {
      "requires_app_version": "0.2.0",  // minimum app version for this data
      "poeVersion": "3.28"
    }
  }

ON UPDATE:
  IF data.requires_app_version > current_app_version:
    → Show: "This game data requires Path of AI v0.2.0. You have v0.1.0."
    → Show: [Update App] button
    → DO NOT apply data update (would break calculator)
    → Keep using old data until app is updated
```

---

## 11. CROSS-PLATFORM: macOS SUPPORT

### Tauri for Cross-Platform

Tauri is ideal for Path of AI because:

| Factor | Tauri | Electron | Native |
|--------|-------|----------|--------|
| Bundle size | ~10-15MB | ~150MB+ | varies |
| RAM usage | ~30-50MB | ~200MB+ | ~30MB |
| Startup time | <1s | 2-5s | <1s |
| macOS support | native, M1/M2/M3 optimized | yes | requires separate codebase |
| Windows support | native | yes | requires separate codebase |
| Linux support | native | yes | requires separate codebase |
| GPU access (for AI) | via Rust bindings | limited | native |
| Vibe coding with Claude | web UI = easy, Rust backend = Claude handles well | JS everywhere = easy | harder |
| Auto-update built-in | yes (tauri-plugin-updater) | yes (electron-updater) | manual |
| Security | Rust memory safety, no Node.js attack surface | Node.js + Chromium surface | varies |

### macOS-Specific Considerations

```
PoB on macOS:
  - PoB Community Fork has macOS builds (via Wine or native port)
  - PoB data stored at: ~/Library/Application Support/Path of Building/
  - Or: ~/Library/Application Support/Wine/.../Path of Building/

Path of AI on macOS:
  - Tauri builds .dmg natively
  - Universal binary (Intel + Apple Silicon)
  - Code signing + notarization via CI/CD
  - Data directory: ~/Library/Application Support/PathOfAI/
  - Auto-update: Tauri's built-in sparkle-based updater on macOS
```

### Build Matrix

```yaml
# GitHub Actions CI/CD
strategy:
  matrix:
    include:
      - os: windows-latest
        target: x86_64-pc-windows-msvc
        output: PathOfAI-Setup.exe

      - os: macos-latest
        target: aarch64-apple-darwin    # Apple Silicon
        output: PathOfAI-arm64.dmg

      - os: macos-latest
        target: x86_64-apple-darwin     # Intel Mac
        output: PathOfAI-x64.dmg

      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
        output: PathOfAI.AppImage
```

### Why Tauri Over Rust-Only

Pure Rust desktop UI options (egui, iced, dioxus) would be lighter but:
- PoE-themed UI with custom fonts, gradients, SVG rings → much easier in HTML/CSS
- Rapid UI iteration / vibe coding with Claude → HTML/CSS is fastest
- Game overlay mode → webview overlays are well-supported
- The performance bottleneck is AI inference and XML parsing, not UI rendering
- Tauri's Rust backend handles all the heavy lifting (file I/O, XML, AI model, etc.)
- Web UI handles all the visual presentation

**Best of both worlds:** Rust performance where it matters + HTML/CSS flexibility for UI.

---

## 12. LOCAL ENGINE vs AI MODEL

### Three-Tier Intelligence Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Path of AI                            │
│                                                         │
│  TIER 1: Rule Engine (instant, 100% accurate)           │
│  ┌───────────────────────────────────────────────────┐  │
│  │ - Resist cap checking                             │  │
│  │ - Open affix detection                            │  │
│  │ - Mod tier identification                         │  │
│  │ - Gem level/quality status                        │  │
│  │ - Build checklist evaluation                      │  │
│  │ - Map mod danger detection                        │  │
│  │ - Socket/link validation                          │  │
│  │ - Attribute requirement checking                  │  │
│  │ - Mana reservation calculation                    │  │
│  │                                                   │  │
│  │ HOW: Pure code logic + game data JSON files       │  │
│  │ SPEED: < 10ms | ACCURACY: 100% | COST: $0        │  │
│  └───────────────────────────────────────────────────┘  │
│                                                         │
│  TIER 2: Calculator + Knowledge Base (fast, exact)      │
│  ┌───────────────────────────────────────────────────┐  │
│  │ - Item scoring (weighted formula, not ML)          │  │
│  │ - Build archetype detection (rule engine)          │  │
│  │ - Passive tree optimization (calc each node)       │  │
│  │ - DPS impact (our Rust calc engine)                │  │
│  │ - Upgrade suggestions (calc + rank + validate)     │  │
│  │ - Crafting probability (mod weight tables)         │  │
│  │ - Query routing (regex intent classifier)          │  │
│  │ - Template response generation                     │  │
│  │                                                   │  │
│  │ HOW: Rust calculator + structured JSON data        │  │
│  │ SPEED: < 100ms | ACCURACY: 99%+ | COST: $0       │  │
│  └───────────────────────────────────────────────────┘  │
│                                                         │
│  TIER 3: Cloud AI (deep reasoning, creative advice)     │
│  ┌───────────────────────────────────────────────────┐  │
│  │ - Complex "why" questions                         │  │
│  │ - Creative build concept generation               │  │
│  │ - Patch impact analysis (reading patch notes)     │  │
│  │ - Build guide translation                         │  │
│  │ - Edge case mechanics questions                   │  │
│  │ - When Seer confidence < 70%                      │  │
│  │                                                   │  │
│  │ HOW: Claude / GPT / Gemini / Grok via API         │  │
│  │ SPEED: 1-5s | ACCURACY: varies | COST: API fees  │  │
│  └───────────────────────────────────────────────────┘  │
│                                                         │
│  ROUTING: Most queries → Tier 1 (free, instant)         │
│           Complex queries → Tier 2 (free, fast)         │
│           Creative/deep → Tier 3 (paid, slower)         │
│           ~80% of queries never leave Tier 1             │
└─────────────────────────────────────────────────────────┘
```

### Why Tier 1 (Rule Engine) Handles Most Work

The majority of useful PoB Advisor features are **deterministic calculations**:

```
"Is my fire resist capped?"       → compare number to 75     (Tier 1)
"What mods are open?"             → count prefixes/suffixes   (Tier 1)
"What tier is this life roll?"    → lookup in mod-tiers.json  (Tier 1)
"Can I run this map mod?"         → check against build data  (Tier 1)
"What should I upgrade?"          → score all items, sort     (Tier 1 + 2)
"Why am I dying to Shaper?"       → simulate hit vs defenses  (Tier 1 + 2)
"Design me a league starter"      → creative, open-ended      (Tier 3)
```

**~80% of user value comes from Tier 1 alone.** The Seer Engine (Tier 2) adds
intelligent ranking and natural language. Cloud AI (Tier 3) is only for creative
or deeply complex queries.

This means the app is fully useful offline, for free, with zero AI costs.

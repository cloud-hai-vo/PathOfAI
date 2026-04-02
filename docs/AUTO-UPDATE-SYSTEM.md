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

## 9. CROSS-PLATFORM: macOS SUPPORT

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

## 10. LOCAL ENGINE vs AI MODEL

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
│  TIER 2: Seer Engine — Local AI (fast, domain expert)   │
│  ┌───────────────────────────────────────────────────┐  │
│  │ - Item scoring & ranking (ItemNet)                │  │
│  │ - Build archetype classification (BuildNet)       │  │
│  │ - Passive tree optimization (TreeNet)             │  │
│  │ - Upgrade priority suggestions                    │  │
│  │ - DPS impact estimation                           │  │
│  │ - User question understanding (QueryNet)          │  │
│  │ - Knowledge base search (EmbedNet + RAG)          │  │
│  │ - Natural language response generation            │  │
│  │                                                   │  │
│  │ HOW: 5 small neural networks + RAG + templates    │  │
│  │ SPEED: < 100ms | ACCURACY: 90%+ | COST: $0       │  │
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

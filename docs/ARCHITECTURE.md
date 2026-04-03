# Path of AI — Architecture & Design Patterns

## Overview

Path of AI is a **Tauri desktop app**:
- **Backend:** Rust (performance-critical logic, file I/O, AI inference, system access)
- **Frontend:** Vanilla TypeScript + HTML/CSS (UI rendering, user interaction)
- **Communication:** Tauri IPC commands (frontend calls Rust functions)

Related docs:
- [ENGINE-DESIGN.md](ENGINE-DESIGN.md) — Calculator formulas + three-engine architecture
- [FLOWS.md](FLOWS.md) — All 14 user interaction flows
- [IPC-SPEC.md](IPC-SPEC.md) — Complete Tauri command + event contract
- [DATABASE.md](DATABASE.md) — SQLite schema
- [CONFIG-SCHEMA.md](CONFIG-SCHEMA.md) — settings.json + tauri.conf.json

---

## SYSTEM ARCHITECTURE DIAGRAM

```
┌─────────────────────────────────────────────────────────────────────┐
│                        PATH OF AI (Tauri 2)                        │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  FRONTEND (Vanilla TypeScript + HTML/CSS)                    │   │
│  │                                                              │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │   │
│  │  │ Character │ │   Stat   │ │  Right   │ │   HUD Bar    │   │   │
│  │  │   Viz    │ │ Sidebar  │ │  Panel   │ │ (Gem Buttons)│   │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────┘   │   │
│  │                                                              │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │  Store (state management) ←→ invoke() / listen()     │   │   │
│  │  └──────────────────────────┬───────────────────────────┘   │   │
│  └─────────────────────────────┼───────────────────────────────┘   │
│                                │ Tauri IPC                          │
│  ┌─────────────────────────────┼───────────────────────────────┐   │
│  │  BACKEND (Rust)             │                                │   │
│  │                             ▼                                │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │  COMMANDS (thin adapters — parse → delegate → respond)│   │   │
│  │  │  analyze_build | ask_seer | get_prices | apply_upgrade│   │   │
│  │  └──────────┬──────────────┬─────────────┬──────────────┘   │   │
│  │             │              │             │                   │   │
│  │  ┌──────────▼──────┐  ┌───▼───────┐  ┌──▼───────────┐      │   │
│  │  │  CALCULATOR     │  │   SEER    │  │   MARKET     │      │   │
│  │  │  (Our Rust Calc)│  │  (Router) │  │ (poe.ninja)  │      │   │
│  │  │                 │  │           │  │              │      │   │
│  │  │ offense_calc.rs │  │ intent    │  │ price_cache  │      │   │
│  │  │ defense_calc.rs │  │ classify  │  │ buy_advisor  │      │   │
│  │  │ formulas.rs     │  │ → calc    │  │ upgrade_find │      │   │
│  │  │ what_if.rs      │  │ → KB      │  └──────────────┘      │   │
│  │  │ validator.rs    │  │ → cloud   │                         │   │
│  │  └────────┬────────┘  └─────┬─────┘                         │   │
│  │           │                 │                                │   │
│  │  ┌────────▼─────────────────▼──────────────────────────┐    │   │
│  │  │  CORE (pure domain logic — NO external dependencies) │    │   │
│  │  │  pob_parser | build_analyzer | build_detector       │    │   │
│  │  │  combat_sim | gem_optimizer  | map_mod_analyzer      │    │   │
│  │  └────────┬────────────────────────────────────────────┘    │   │
│  │           │                                                  │   │
│  │  ┌────────▼────────────────────────────────────────────┐    │   │
│  │  │  DATA (game data loading — RePoE JSONs)             │    │   │
│  │  │  mod_database | gem_database | tree_database        │    │   │
│  │  │  unique_database | loader | updater                 │    │   │
│  │  └─────────────────────────────────────────────────────┘    │   │
│  │                                                              │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │  SERVICES (background tasks)                          │   │   │
│  │  │  file_watcher | price_poller | update_checker         │   │   │
│  │  └──────────────────────────────────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │
│  │  SQLite DB  │  │  PoB Files  │  │ PathOfAI_Data│                │
│  │  (cache,    │  │  (XML read/ │  │ (settings,   │                │
│  │   history)  │  │   write)    │  │  game-data,  │                │
│  └─────────────┘  └─────────────┘  │  backups)    │                │
│                                     └─────────────┘                │
├─────────────────────────────────────────────────────────────────────┤
│  EXTERNAL SERVICES (network, optional)                              │
│  ┌───────────┐  ┌──────────────┐  ┌──────────────┐                │
│  │ poe.ninja │  │ Claude API   │  │ GitHub       │                │
│  │ (prices)  │  │ (cloud AI)   │  │ (updates)    │                │
│  └───────────┘  └──────────────┘  └──────────────┘                │
└─────────────────────────────────────────────────────────────────────┘
```

### Data Flow: Build Analysis Pipeline

```
PoB XML file
    ↓ (file_watcher detects change, or user loads)
pob_parser::parse()
    ↓ BuildData { class, items, gems, tree, config }
build_detector::detect()
    ↓ archetype: FireDot, main_skill: RF
calculator::calculate()
    ↓ offense: { dps, breakdown }, defense: { life, resists, ehp }
build_analyzer::analyze()
    ↓ scores, issues, suggestions (multi-path)
validator::validate_all()
    ↓ every suggestion verified (DPS exact, resists checked)
market::check_prices()  (background)
    ↓ prices attached to suggestions
→ AnalysisResult returned to frontend via IPC
    ↓
UI updates all panels simultaneously
```

---

## 1. PROJECT STRUCTURE

```
path-of-ai/
│
├── src-tauri/                        # Rust backend (Tauri)
│   ├── Cargo.toml
│   ├── tauri.conf.json               # Tauri config (window, permissions, updater)
│   ├── src/
│   │   ├── main.rs                   # Tauri entry point
│   │   ├── lib.rs                    # Module registration
│   │   │
│   │   ├── commands/                 # Tauri IPC command handlers
│   │   │   ├── mod.rs
│   │   │   ├── build_commands.rs     # #[tauri::command] for build analysis
│   │   │   ├── market_commands.rs    # #[tauri::command] for market data
│   │   │   ├── file_commands.rs      # #[tauri::command] for PoB file ops
│   │   │   ├── seer_commands.rs      # #[tauri::command] for AI queries
│   │   │   └── feedback_commands.rs  # #[tauri::command] for bug reports → GitHub Issues
│   │   │
│   │   ├── core/                     # Business logic (pure Rust, no Tauri deps)
│   │   │   ├── mod.rs
│   │   │   ├── pob_parser.rs         # PoB XML → BuildData struct
│   │   │   ├── pob_writer.rs         # BuildData → PoB XML (atomic writes)
│   │   │   ├── build_analyzer.rs     # Scoring, issues, suggestions
│   │   │   ├── build_detector.rs     # Archetype / skill / playstyle detection
│   │   │   ├── mod_impact.rs         # DPS/stat impact calculation
│   │   │   ├── defense_sim.rs        # EHP, damage taken simulation
│   │   │   ├── combat_sim.rs         # "The Arena" — combat simulation engine
│   │   │   ├── boss_ai.rs           # Boss attack patterns, phases, telegraphs
│   │   │   ├── map_gen.rs           # Map monster pack generation per tier
│   │   │
│   │   ├── renderer/                # Native GPU renderer (wgpu — character + combat)
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs            # wgpu setup, surface config, render loop
│   │   │   ├── character.rs         # Character sprite/skeleton + equipment overlays
│   │   │   ├── monsters.rs          # Monster sprites + HP bars + death effects
│   │   │   ├── particles.rs         # GPU particle system (fire, ice, lightning)
│   │   │   ├── combat_scene.rs      # Combat simulation scene (map or boss arena)
│   │   │   └── camera.rs            # Isometric camera + viewport
│   │   │   ├── gem_optimizer.rs      # Gem swap / level / quality analysis
│   │   │   ├── map_mod_analyzer.rs   # "Curse Map" — dangerous map mod detection
│   │   │   └── item_image.rs         # Item → PoE CDN image URL resolver
│   │   │
│   │   ├── data/                     # Game data loading & versioning
│   │   │   ├── mod.rs
│   │   │   ├── loader.rs             # Load versioned JSON (poe1/ or poe2/ subfolder)
│   │   │   ├── mod_database.rs       # Mod tier lookups + spawn weights
│   │   │   ├── gem_database.rs       # Gem info + transfigured variants
│   │   │   ├── tree_database.rs      # Passive tree (PoE1 + PoE2 layouts)
│   │   │   ├── unique_database.rs    # Searchable unique item database (~1200 items)
│   │   │   └── updater.rs            # Auto-update data from GitHub Releases
│   │   │
│   │   ├── poe_api/                  # PoE Official API (PRIMARY build import)
│   │   │   ├── mod.rs
│   │   │   ├── oauth.rs              # OAuth flow (authorize, token exchange, refresh)
│   │   │   ├── characters.rs         # Fetch character list + equipped items
│   │   │   ├── passive_tree.rs       # Fetch allocated passives + jewels
│   │   │   ├── stash_tabs.rs         # Fetch stash contents (tabs, items, currency)
│   │   │   └── api_to_build.rs       # Convert PoE API response → BuildData struct
│   │   │
│   │   ├── market/                   # Market intelligence
│   │   │   ├── mod.rs
│   │   │   ├── poe_ninja.rs          # poe.ninja API client
│   │   │   ├── price_cache.rs        # Local price cache with TTL
│   │   │   ├── buy_advisor.rs        # Buy timing / trend analysis
│   │   │   └── upgrade_finder.rs     # Find upgrades within budget
│   │   │
│   │   ├── calculator/               # OUR calculation engine (Rust — primary)
│   │   │   ├── mod.rs
│   │   │   ├── mod_aggregator.rs     # Aggregate modifiers from tree + items + gems
│   │   │   ├── offense_calc.rs       # DPS: base × increased × more × dot × speed
│   │   │   ├── defense_calc.rs       # Life, ES, armour, evasion, block, resists, EHP
│   │   │   ├── formulas.rs           # PoE game formulas (armour, evasion, crit, etc.)
│   │   │   ├── what_if.rs            # "What if I change X?" → recalc → diff
│   │   │   ├── fast_estimate.rs      # Quick estimation path (<10ms)
│   │   │   └── validator.rs          # Validate suggestions (no resist uncap, etc.)
│   │   │
│   │   ├── pob_verify/              # PoB Lua calc engine (OPTIONAL verification)
│   │   │   ├── mod.rs
│   │   │   ├── lua_bridge.rs         # LuaJIT FFI — loads PoB calc modules
│   │   │   └── comparator.rs         # Compare our results vs PoB, report diffs
│   │   │
│   │   ├── seer/                     # The Seer — query routing + response generation
│   │   │   ├── mod.rs
│   │   │   ├── query_router.rs       # Intent classifier (rule-based, not ML)
│   │   │   ├── response_gen.rs       # Template response generator
│   │   │   ├── craft_advisor.rs      # Crafting probability from mod weights
│   │   │   ├── item_import.rs        # Parse PoE clipboard format (Ctrl+C items)
│   │   │   ├── item_editor.rs        # Item crafting simulator (add/remove mods)
│   │   │   ├── build_share.rs        # Generate/parse build share codes
│   │   │   ├── party_analyzer.rs     # Party composition + aura overlap analysis
│   │   │   └── cloud_api.rs          # Optional: Claude/GPT for creative queries
│   │   │
│   │   ├── analytics/                # Session stats, wealth tracking, recipes
│   │   │   ├── mod.rs
│   │   │   ├── map_stats.rs          # Map run statistics (Client.txt parser)
│   │   │   ├── wealth_tracker.rs     # Net worth history over time
│   │   │   ├── recipe_detector.rs    # Chaos/regal recipe detection in stash
│   │   │   └── session.rs            # Per-session currency/XP/death tracking
│   │   │
│   │   ├── services/                 # Long-running background services
│   │   │   ├── mod.rs
│   │   │   ├── file_watcher.rs       # Watch PoB files for changes
│   │   │   ├── price_poller.rs       # Background price updates
│   │   │   └── update_checker.rs     # Check for app/data updates
│   │   │
│   │   └── models/                   # Shared data types
│   │       ├── mod.rs
│   │       ├── build.rs              # BuildData, Stats, etc.
│   │       ├── item.rs               # Item, Mod, ModTier, etc.
│   │       ├── gem.rs                # Gem, SkillGroup, etc.
│   │       ├── tree.rs               # PassiveNode, TreeSpec, etc.
│   │       └── analysis.rs           # AnalysisResult, Issue, Suggestion, etc.
│   │
│   └── game-data/                    # Bundled game data JSON files
│       ├── mods/
│       ├── gems/
│       ├── items/
│       ├── tree/
│       ├── crafting/
│       └── meta/version.json
│
├── src/                              # Frontend (Vanilla TypeScript — NO framework)
│   ├── index.html                    # Main HTML entry (game HUD layout)
│   ├── main.ts                       # Frontend entry, Tauri IPC setup
│   ├── styles/
│   │   ├── theme.css                 # PoE-exact color variables + fonts
│   │   ├── hud.css                   # Game HUD layout (3-column + bottom bar)
│   │   ├── components.css            # Reusable component styles
│   │   └── animations.css            # Aura rings, power-up effects, blood drip
│   ├── components/                   # UI components (vanilla TS, NO React/Vue/Svelte)
│   │   ├── character-viz.ts          # Character body + equipment slots + aura rings
│   │   ├── equip-grid.ts             # Equipment grid (3x4 PoE layout)
│   │   ├── stat-sidebar.ts           # Left sidebar stats + resist orbs
│   │   ├── right-panel.ts            # Context-sensitive right panel
│   │   ├── hud-bar.ts                # Bottom HUD with gem buttons + life/mana orbs
│   │   ├── item-tooltip.ts           # PoE-style item tooltip
│   │   ├── score-ring.ts             # Skill gem-style score rings
│   │   ├── resist-orb.ts             # Element resist orb with glass effect
│   │   ├── issue-card.ts             # Harbinger warning card
│   │   ├── suggestion-card.ts        # Prophecy suggestion card
│   │   ├── seer-chat.ts              # Grimoire chat panel
│   │   ├── stash-grid.ts             # Stash inventory grid (12x12)
│   │   ├── passive-tree.ts           # Passive tree mini-view
│   │   ├── craft-forge.ts            # The Forge crafting advisor
│   │   ├── item-import.ts            # Ctrl+C paste item panel
│   │   ├── item-editor.ts            # Item crafting simulator (live mod editor)
│   │   ├── unique-search.ts          # Searchable unique item database
│   │   ├── calc-breakdown.ts         # DPS calculation breakdown ("Show The Math")
│   │   ├── party-panel.ts            # Party composition analyzer
│   │   ├── build-share.ts            # Build share code generator/importer
│   │   ├── combat-renderer.ts       # Arena: animated combat simulation canvas
│   │   ├── boss-renderer.ts         # Boss sprite + attack animations + phases
│   │   ├── monster-renderer.ts      # Monster packs + HP bars + death effects
│   │   ├── damage-numbers.ts        # Floating damage number particles
│   │   └── upgrade-preview.ts       # Side-by-side before/after simulation
│   ├── services/
│   │   ├── api.ts                    # Typed wrappers around Tauri invoke()
│   │   ├── store.ts                  # Frontend state management
│   │   └── events.ts                 # Tauri event listeners
│   └── types/
│       └── build.ts                  # TypeScript types matching Rust models
│
├── scripts/                          # Build & data tooling
│   ├── extract-game-data.py          # RePoE → JSON data extraction
│   └── generate-training-data.py     # AI training data generation
│
├── docs/                             # Documentation
├── test-data/                        # Sample PoB XML files
└── README.md
```

---

## 2. DESIGN PATTERNS

### 2.1 Command Pattern (Tauri IPC)

Frontend calls Rust backend via **Tauri commands**. Each command is a thin handler
that delegates to core logic.

```rust
// src-tauri/src/commands/build_commands.rs

use crate::core::build_analyzer::BuildAnalyzer;
use crate::core::pob_parser::PobParser;
use crate::models::analysis::AnalysisResult;

/// Tauri command: parse a PoB XML file and return full analysis.
/// Frontend calls: await invoke('analyze_build', { filePath })
#[tauri::command]
pub async fn analyze_build(file_path: String) -> Result<AnalysisResult, String> {
    let xml = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let build_data = PobParser::parse(&xml)
        .map_err(|e| format!("Failed to parse PoB XML: {}", e))?;

    let analyzer = BuildAnalyzer::new(&build_data);
    let result = analyzer.analyze();

    Ok(result)
}

/// Tauri command: get upgrade suggestions for a specific slot
#[tauri::command]
pub async fn get_upgrade_suggestions(
    file_path: String,
    slot: String,
    budget: f64,
) -> Result<Vec<Suggestion>, String> {
    // ...
}
```

```typescript
// src/services/api.ts

import { invoke } from '@tauri-apps/api/core';
import type { AnalysisResult, Suggestion } from '../types/build';

export async function analyzeBuild(filePath: string): Promise<AnalysisResult> {
    return invoke('analyze_build', { filePath });
}

export async function getUpgradeSuggestions(
    filePath: string, slot: string, budget: number
): Promise<Suggestion[]> {
    return invoke('get_upgrade_suggestions', { filePath, slot, budget });
}
```

**Rules:**
- Commands are **thin wrappers** — they parse input, call core logic, return result
- All business logic lives in `core/`, never in `commands/`
- Commands return `Result<T, String>` — errors serialized as strings to frontend
- Commands are `async` to avoid blocking the main thread

---

### 2.2 Repository Pattern (Game Data Access)

Game data (mods, gems, tree, items) is accessed through repository structs
that abstract the data source (bundled JSON, updated JSON, or API).

```rust
// src-tauri/src/data/mod_database.rs

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ModTier {
    pub tier: u8,
    pub min_value: f64,
    pub max_value: f64,
    pub label: String,       // "T1", "T2", etc.
    pub ilvl_required: u8,
}

pub struct ModDatabase {
    tiers: HashMap<String, Vec<ModTier>>,  // mod_id → tiers sorted by value
}

impl ModDatabase {
    /// Load from versioned JSON file
    pub fn load(data_dir: &Path) -> Result<Self, DataError> {
        let path = data_dir.join("mods/mod-tiers.json");
        let json = std::fs::read_to_string(path)?;
        let tiers: HashMap<String, Vec<ModTier>> = serde_json::from_str(&json)?;
        Ok(Self { tiers })
    }

    /// Get the tier for a specific mod value
    pub fn get_tier(&self, mod_id: &str, value: f64) -> Option<&ModTier> {
        self.tiers.get(mod_id)?
            .iter()
            .find(|t| value >= t.min_value && value <= t.max_value)
    }

    /// Get max possible value for a mod
    pub fn max_value(&self, mod_id: &str) -> Option<f64> {
        self.tiers.get(mod_id)?
            .first()
            .map(|t| t.max_value)
    }
}
```

**Rules:**
- One database struct per data domain (`ModDatabase`, `GemDatabase`, `TreeDatabase`)
- Load once at startup, keep in memory (game data is small, ~10MB total)
- Immutable after loading — thread-safe by default
- Reload on data update (swap atomically)

---

### 2.3 Strategy Pattern (Build-Type-Specific Logic)

Different build archetypes need different analysis strategies. Use traits.

```rust
// src-tauri/src/core/build_analyzer.rs

/// Trait for build-type-specific analysis strategies
pub trait AnalysisStrategy {
    fn stat_weights(&self) -> StatWeights;
    fn critical_defenses(&self) -> Vec<DefenseCheck>;
    fn priority_stats(&self) -> Vec<&str>;
    fn dangerous_map_mods(&self) -> Vec<&str>;
}

/// RF / Fire DoT builds
pub struct FireDotStrategy;
impl AnalysisStrategy for FireDotStrategy {
    fn stat_weights(&self) -> StatWeights {
        StatWeights {
            life: 1.2,
            fire_dot_multi: 15.0,
            fire_res: 0.3,
            life_regen: 5.0,  // critical for RF
            ..Default::default()
        }
    }

    fn critical_defenses(&self) -> Vec<DefenseCheck> {
        vec![
            DefenseCheck::LifeRegenAboveDegen,  // RF-specific: regen must exceed degen
            DefenseCheck::MaxFireRes,             // max fire res reduces RF self-damage
            DefenseCheck::ResistCapped,
        ]
    }

    fn dangerous_map_mods(&self) -> Vec<&str> {
        vec!["no_regen", "less_recovery", "minus_max_res", "ele_reflect"]
    }
}

/// Attack builds
pub struct AttackStrategy;
impl AnalysisStrategy for AttackStrategy {
    fn stat_weights(&self) -> StatWeights {
        StatWeights {
            flat_phys: 10.0,
            attack_speed: 8.0,
            crit_chance: 6.0,
            crit_multi: 5.0,
            accuracy: 3.0,
            ..Default::default()
        }
    }
    // ...
}

/// The analyzer picks the right strategy based on detected build type
pub struct BuildAnalyzer<'a> {
    build: &'a BuildData,
    strategy: Box<dyn AnalysisStrategy>,
    mod_db: &'a ModDatabase,
}

impl<'a> BuildAnalyzer<'a> {
    pub fn new(build: &'a BuildData, mod_db: &'a ModDatabase) -> Self {
        let archetype = BuildDetector::detect(build);
        let strategy: Box<dyn AnalysisStrategy> = match archetype {
            Archetype::FireDot => Box::new(FireDotStrategy),
            Archetype::ColdDot => Box::new(ColdDotStrategy),
            Archetype::Attack  => Box::new(AttackStrategy),
            Archetype::Spell   => Box::new(SpellStrategy),
            Archetype::Minion  => Box::new(MinionStrategy),
            _                  => Box::new(DefaultStrategy),
        };
        Self { build, strategy, mod_db }
    }
}
```

---

### 2.4 Observer Pattern (File Watcher → Event System)

PoB file changes trigger a chain of re-analysis. Use Tauri's event system.

```rust
// src-tauri/src/services/file_watcher.rs

use notify::{Watcher, RecursiveMode, Event};
use tauri::{AppHandle, Emitter};
use std::time::Duration;

pub struct PobFileWatcher {
    watcher: notify::RecommendedWatcher,
    debounce_ms: u64,
}

impl PobFileWatcher {
    pub fn start(app: AppHandle, pob_path: &Path) -> Result<Self, WatchError> {
        let app_handle = app.clone();

        let mut watcher = notify::recommended_watcher(move |event: Result<Event, _>| {
            if let Ok(event) = event {
                if event.kind.is_modify() {
                    // Emit event to frontend — UI re-fetches analysis
                    let _ = app_handle.emit("pob-file-changed", event.paths.clone());
                }
            }
        })?;

        watcher.watch(pob_path, RecursiveMode::Recursive)?;

        Ok(Self { watcher, debounce_ms: 500 })
    }
}
```

```typescript
// src/services/events.ts

import { listen } from '@tauri-apps/api/event';
import { analyzeBuild } from './api';
import { store } from './store';

// Listen for PoB file changes from Rust backend
export function setupEventListeners() {
    listen<string[]>('pob-file-changed', async (event) => {
        const filePath = event.payload[0];
        console.log('PoB file changed:', filePath);

        // Re-analyze build
        const result = await analyzeBuild(filePath);
        store.setBuildAnalysis(result);
    });

    listen('data-update-available', (event) => {
        store.setUpdateAvailable(true);
    });
}
```

---

### 2.5 Builder Pattern (Complex Structs)

For structs with many optional fields (common in PoE data):

```rust
// src-tauri/src/models/item.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub slot: String,
    pub name: String,
    pub base: String,
    pub rarity: Rarity,
    pub mods: Vec<Mod>,
    pub implicits: Vec<Mod>,
    pub influence: Option<Influence>,
    pub corrupted: bool,
    pub mirrored: bool,
    pub fractured_mods: Vec<String>,
    pub ilvl: u8,
}

/// Builder for constructing Items in tests and parsers
pub struct ItemBuilder {
    item: Item,
}

impl ItemBuilder {
    pub fn new(slot: &str, base: &str) -> Self {
        Self {
            item: Item {
                id: 0,
                slot: slot.to_string(),
                name: String::new(),
                base: base.to_string(),
                rarity: Rarity::Normal,
                mods: vec![],
                implicits: vec![],
                influence: None,
                corrupted: false,
                mirrored: false,
                fractured_mods: vec![],
                ilvl: 1,
            },
        }
    }

    pub fn name(mut self, name: &str) -> Self { self.item.name = name.to_string(); self }
    pub fn rarity(mut self, r: Rarity) -> Self { self.item.rarity = r; self }
    pub fn ilvl(mut self, ilvl: u8) -> Self { self.item.ilvl = ilvl; self }
    pub fn add_mod(mut self, m: Mod) -> Self { self.item.mods.push(m); self }
    pub fn corrupted(mut self) -> Self { self.item.corrupted = true; self }
    pub fn influence(mut self, i: Influence) -> Self { self.item.influence = Some(i); self }
    pub fn build(self) -> Item { self.item }
}

// Usage:
// let ring = ItemBuilder::new("Ring 1", "Opal Ring")
//     .name("Torment Circle")
//     .rarity(Rarity::Rare)
//     .ilvl(84)
//     .add_mod(life_mod)
//     .add_mod(fire_res_mod)
//     .build();
```

---

### 2.6 State Management Pattern (Frontend)

Simple, predictable state management without heavy frameworks:

```typescript
// src/services/store.ts

import type { AnalysisResult } from '../types/build';

type Listener = () => void;

class Store {
    private state = {
        buildAnalysis: null as AnalysisResult | null,
        activeTab: 'overview' as string,
        currentFile: null as string | null,
        synced: false,
        updateAvailable: false,
        loading: false,
    };

    private listeners: Set<Listener> = new Set();

    getState() { return this.state; }

    // Typed setters — each triggers re-render
    setBuildAnalysis(result: AnalysisResult) {
        this.state = { ...this.state, buildAnalysis: result, synced: true, loading: false };
        this.notify();
    }

    setActiveTab(tab: string) {
        this.state = { ...this.state, activeTab: tab };
        this.notify();
    }

    setLoading(loading: boolean) {
        this.state = { ...this.state, loading };
        this.notify();
    }

    setUpdateAvailable(available: boolean) {
        this.state = { ...this.state, updateAvailable: available };
        this.notify();
    }

    // Subscribe to state changes
    subscribe(listener: Listener): () => void {
        this.listeners.add(listener);
        return () => this.listeners.delete(listener);
    }

    private notify() {
        this.listeners.forEach(fn => fn());
    }
}

export const store = new Store();
```

---

### 2.7 Facade Pattern (The Seer — Three-Engine Architecture)

The Seer is NOT a general-purpose AI model. It's a **purpose-built PoE analysis system
with natural language output**. There are two levels of description — both are correct:

**Routing level (user-visible):** Three engines share query load.

| Engine | % of queries | What it handles | Latency |
|--------|-------------|-----------------|---------|
| Calculator | ~85% | Math queries — DPS, EHP, resist, item compare | <500ms |
| Seer (local) | ~12% | PoE knowledge, build advice, tree, crafting | <100ms |
| Cloud AI | ~3% | Complex theory, new patch content, creative | 1-5s |

**Implementation level (the Seer engine internals):** The local "Seer" engine is
implemented as **5 specialized neural networks + a rule-based ResponseGen**. See
[ALGORITHMS.md — Algorithm 42](ALGORITHMS.md#42-seer-network-architecture) for full
network architecture (ItemNet/BuildNet/TreeNet/QueryNet/EmbedNet + ResponseGen).

These two descriptions do not conflict: "3 engines" is the routing strategy;
"5 networks" is the implementation of the middle engine.

See [ENGINE-DESIGN.md](ENGINE-DESIGN.md) for formula details.

```rust
// src-tauri/src/seer/mod.rs

/// The Seer — routes queries to the right engine.
/// No neural networks. No custom models. Just math + data + templates.
pub struct Seer {
    calculator: PobCalculator,     // PoB Lua calc engine (LuaJIT)
    knowledge: KnowledgeBase,      // Structured game data (JSON)
    response_gen: ResponseGenerator, // Template-based NL output
    cloud_api: Option<CloudApi>,   // Optional Claude/GPT for creative queries
}

impl Seer {
    pub fn new(data_dir: &Path) -> Result<Self, SeerError> {
        Ok(Self {
            calculator: PobCalculator::new(data_dir)?,
            knowledge: KnowledgeBase::load(data_dir)?,
            response_gen: ResponseGenerator::new(),
            cloud_api: None, // enabled when user provides API key
        })
    }

    /// Main entry point: ask a question, get a verified answer.
    pub fn ask(&self, question: &str, build: &BuildData) -> SeerResponse {
        let intent = self.classify_intent(question); // rule-based, NOT ML

        match intent {
            // ENGINE 1: Calculator — 85% of queries (100% accurate)
            Intent::DpsCheck | Intent::ItemCompare | Intent::GemSwap
            | Intent::UpgradeRank | Intent::ResistCheck | Intent::EhpCalc => {
                let result = self.calculator.calculate(build, &intent);
                self.response_gen.format_calculation(result)
            }

            // ENGINE 2: Knowledge Base — 12% of queries (99% accurate)
            Intent::CraftAdvice | Intent::BossMechanic | Intent::ModExplain
            | Intent::MapMod | Intent::GemInteraction => {
                let data = self.knowledge.lookup(&intent, build);
                self.response_gen.format_knowledge(data)
            }

            // ENGINE 3: Cloud AI — 3% of queries (optional)
            Intent::BuildDesign | Intent::WhyQuestion | Intent::OpenEnded => {
                if let Some(api) = &self.cloud_api {
                    let context = self.knowledge.build_context(build);
                    api.query_with_context(question, &context)
                } else {
                    SeerResponse::needs_cloud("The Seer cannot divine this alone. Connect a cloud provider in Settings.")
                }
            }
        }
    }

    /// Score an item — pure math, no AI needed.
    pub fn score_item(&self, item: &Item, build: &BuildData) -> ItemScore {
        let weights = self.knowledge.stat_weights(build.archetype);
        item.mods.iter()
            .map(|m| m.value * weights.get(m.stat_type))
            .sum::<f64>()
            / self.knowledge.expected_score(item.slot, build.level)
    }

    /// "What if I swap this item?" — PoB Lua calc, 100% accurate.
    pub fn what_if(&self, build: &BuildData, change: &Change) -> ValidatedDiff {
        let old = self.calculator.run(build);
        let new_build = build.apply(change);
        let new = self.calculator.run(&new_build);
        ValidatedDiff::compare(old, new) // exact DPS/life/resist diff
    }

    /// Intent classification — 50 regex rules, no ML needed.
    fn classify_intent(&self, query: &str) -> Intent {
        let q = query.to_lowercase();
        if q.contains("dps") || q.contains("damage") { return Intent::DpsCheck; }
        if q.contains("upgrade") || q.contains("replace") { return Intent::UpgradeRank; }
        if q.contains("craft") || q.contains("fossil") || q.contains("essence") { return Intent::CraftAdvice; }
        if q.contains("resist") || q.contains("cap") { return Intent::ResistCheck; }
        if q.contains("boss") || q.contains("shaper") || q.contains("maven") { return Intent::BossMechanic; }
        if q.contains("gem") || q.contains("support") || q.contains("link") { return Intent::GemSwap; }
        if q.contains("why") || q.contains("explain") { return Intent::WhyQuestion; }
        if q.contains("design") || q.contains("build me") || q.contains("create") { return Intent::BuildDesign; }
        if q.contains("map") || q.contains("mod") { return Intent::MapMod; }
        Intent::OpenEnded
    }
}
```

---

### 2.8 Result/Error Pattern (Rust)

Use typed errors with `thiserror`, return `Result` everywhere:

```rust
// src-tauri/src/core/pob_parser.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to read XML: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid XML structure: {0}")]
    XmlError(#[from] quick_xml::Error),

    #[error("Missing required element: {0}")]
    MissingElement(String),

    #[error("Invalid stat value for '{stat}': {value}")]
    InvalidStat { stat: String, value: String },
}

pub struct PobParser;

impl PobParser {
    pub fn parse(xml: &str) -> Result<BuildData, ParseError> {
        let doc = quick_xml::Reader::from_str(xml);
        // ... parsing logic
        // Returns Err(ParseError::MissingElement(...)) if something is wrong
        // Returns Ok(BuildData { ... }) on success
    }
}
```

---

### 2.9 Circuit Breaker Pattern (External APIs)

External APIs (poe.ninja, Cloud AI) can fail. Use circuit breaker to degrade gracefully.

```rust
pub struct CircuitBreaker {
    failures: AtomicU32,
    last_failure: Mutex<Instant>,
    threshold: u32,        // open circuit after N failures
    cooldown: Duration,    // try again after this duration
}

impl CircuitBreaker {
    pub fn is_open(&self) -> bool {
        self.failures.load(Ordering::Relaxed) >= self.threshold
    }

    pub async fn call<F, T, E>(&self, f: F) -> Result<T, E>
    where F: Future<Output = Result<T, E>> {
        if self.is_open() {
            // Check if cooldown elapsed → try one request (half-open)
            if self.last_failure.lock().unwrap().elapsed() < self.cooldown {
                return Err(/* circuit open error */);
            }
        }
        match f.await {
            Ok(v) => { self.failures.store(0, Ordering::Relaxed); Ok(v) }
            Err(e) => { self.failures.fetch_add(1, Ordering::Relaxed); Err(e) }
        }
    }
}
```

Fallback behavior per service:
- **poe.ninja down** → serve stale cached prices (show "prices from X hours ago")
- **Cloud AI down** → "The Seer cannot reach the void. Rephrase for local engine."
- **GitHub update server down** → keep current data, retry in 1 hour

### 2.10 Cache Pattern

```rust
pub struct TtlCache<V> {
    data: HashMap<String, (V, Instant)>,
    ttl: Duration,
}

impl<V: Clone> TtlCache<V> {
    pub fn get(&self, key: &str) -> Option<&V> {
        self.data.get(key).and_then(|(v, ts)| {
            if ts.elapsed() < self.ttl { Some(v) } else { None }
        })
    }
    pub fn get_stale(&self, key: &str) -> Option<&V> {
        // Return even if expired — for circuit breaker fallback
        self.data.get(key).map(|(v, _)| v)
    }
}
```

Cache TTLs:
- Price data: 5 min (poe.ninja refreshes every 5 min)
- Build analysis: until build file changes
- Seer responses: 10 min per (question_hash + build_hash)
- Game data: until app restart or manual refresh

### 2.11 Portable Storage Pattern

Data stored next to the executable, NOT in AppData. User can change in Settings.

```rust
pub struct StorageConfig {
    pub data_dir: PathBuf,  // default: ./PathOfAI_Data/
}

impl StorageConfig {
    pub fn default() -> Self {
        // Get directory where exe is running
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        Self { data_dir: exe_dir.join("PathOfAI_Data") }
    }

    pub fn from_settings(settings_path: &Path) -> Self {
        // User can override via settings.json
        // If user chose custom folder, use that
        // Otherwise default to exe directory
    }
}
```

Directory structure (all portable, copy folder = full backup):
```
PathOfAI.exe
PathOfAI_Data/
  settings.json              # user preferences, API keys (encrypted)
  game-data/                 # bundled + updated game data JSONs
    mods/ gems/ tree/ items/ crafting/ bosses/
  cache/
    prices/                  # poe.ninja price cache
    images/                  # item art cache from PoE CDN
    builds/                  # analyzed build cache
  backups/                   # PoB XML backups before writes
  history/                   # build snapshots for undo/redo
  logs/                      # app logs for bug reports
```

Benefits:
- Put on USB → works on any PC
- No registry, no AppData, no roaming profile issues
- Easy backup (zip the folder)
- Easy uninstall (delete the folder)
- User can choose custom folder in Settings → Paths

### 2.12 Undo/Redo Pattern

```rust
pub struct UndoStack {
    history: Vec<BuildSnapshot>,
    position: usize,
    max_history: usize,  // default 50
}

impl UndoStack {
    pub fn push(&mut self, snapshot: BuildSnapshot) {
        // Truncate any redo history
        self.history.truncate(self.position + 1);
        self.history.push(snapshot);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        self.position = self.history.len() - 1;
    }

    pub fn undo(&mut self) -> Option<&BuildSnapshot> {
        if self.position > 0 { self.position -= 1; }
        self.history.get(self.position)
    }

    pub fn redo(&mut self) -> Option<&BuildSnapshot> {
        if self.position < self.history.len() - 1 { self.position += 1; }
        self.history.get(self.position)
    }
}
```

---

## 3. WHY VANILLA TYPESCRIPT (No React/Vue/Svelte)

### The Decision

The frontend uses **vanilla TypeScript** — no UI framework.

### Why

| Factor | Vanilla TS | React/Vue/Svelte |
|--------|-----------|-----------------|
| Bundle size | ~0KB framework overhead | 40-130KB gzipped |
| Startup time | Instant (no hydration) | 100-300ms framework init |
| Learning curve | Just HTML + TS | Framework-specific patterns |
| Game-like UI | Direct DOM control = easy animations | Virtual DOM = fight for control |
| Tauri integration | Simple invoke() + event listeners | Need framework-specific bindings |
| Long-term maintenance | No framework version upgrades | React 18→19→20 migration pain |
| Custom rendering | Canvas/SVG for character viz = native | Canvas in React = awkward |
| AI-assisted development | Claude writes vanilla TS perfectly | Framework-specific patterns vary |

### Our Component Pattern (No Framework Needed)

```typescript
// src/components/stat-sidebar.ts

export class StatSidebar {
  private el: HTMLElement;

  constructor(container: HTMLElement) {
    this.el = container;
  }

  render(stats: BuildStats) {
    this.el.innerHTML = `
      <div class="stat-row">
        <span class="stat-icon">❤</span>
        <span class="stat-value" style="color:var(--life);">${stats.life.toLocaleString()}</span>
      </div>
      <!-- ... -->
    `;
  }

  update(stats: BuildStats) {
    // Only update changed values (manual diff, very fast)
    const lifeEl = this.el.querySelector('.stat-value');
    if (lifeEl) lifeEl.textContent = stats.life.toLocaleString();
  }
}
```

### Why Not Web Components?

Web Components (Custom Elements + Shadow DOM) are an option, but for a Tauri app
where we control the entire page, they add complexity without benefit. Plain classes
that own a DOM subtree are simpler and faster.

### State Management (No Redux/Zustand Needed)

```typescript
// src/services/store.ts — ~30 lines, replaces any state library

type Listener = () => void;

class Store {
  private state: AppState = { /* initial state */ };
  private listeners = new Set<Listener>();

  get() { return this.state; }

  set(patch: Partial<AppState>) {
    Object.assign(this.state, patch);
    this.listeners.forEach(fn => fn());
  }

  subscribe(fn: Listener) {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }
}

export const store = new Store();
```

---

## 3.x pob_verify Module — PoB Lua Verification Engine

### Purpose

`pob_verify/` provides **optional cross-verification** of our Rust DPS/defence
calculations against Path of Building's own Lua calc engine. This is NOT the primary
calc path — it is a correctness check only.

```
Our Rust calc (primary, always on)
    │
    ├── result ──→ comparator.rs ──→ check for significant diff (>2%)
    │                                     │
    │             pob_verify/ (optional) ◄┘
    │             lua_bridge.rs loads PoB Lua modules via mlua
    │             runs: CalcSetup.PerformCalcs()
    │             extracts: TotalDPS, life, resists, etc.
    │
    └── if diff > threshold → emit "pob-discrepancy" event → user sees warning
```

### When pob_verify Runs

| Trigger | Behavior |
|---------|----------|
| Settings: "Enable PoB verification" ON | Runs after every full calc |
| On-demand: user clicks "Cross-check with PoB" | Single run |
| Settings: OFF or mlua feature not compiled | `pob_verify = None` — silently skipped |

### What Is Compared

```rust
pub struct CalcComparison {
    pub our_dps:      f64,
    pub pob_dps:      f64,
    pub dps_diff_pct: f64,    // abs((ours - pob) / pob * 100)

    pub our_life:     f64,
    pub pob_life:     f64,
    pub life_diff_pct: f64,

    pub our_fire_res: f64,
    pub pob_fire_res: f64,
    // ... cold, lightning, chaos resist

    pub significant_diff: bool,   // any field > 2% off
    pub issues: Vec<String>,      // human-readable list of discrepancies
}
```

### Why 2% Threshold

Floating-point rounding and minor formula interpretation differences (e.g., rounding
order in a product chain) routinely produce sub-2% variation. 2% is large enough to
be a genuine formula bug, small enough to not trigger on normal float noise.

### Dependency

Requires `mlua` crate with `lua54` feature. If not available (e.g., on ARM or minimal
builds), `pob_verify` is compiled out via `#[cfg(feature = "pob_verify")]`. The app
works fully without it — PoB verification is a quality tool, not a hard dependency.

---

## 4. KEY DESIGN PRINCIPLES

### 4.1 Hexagonal Architecture (Ports & Adapters)

The app follows hexagonal architecture: domain logic in the center, external
concerns (Tauri, file system, APIs) as adapters behind trait boundaries.

```
                    ┌──────────────────────────────────┐
                    │      DOMAIN (Pure Rust)           │
                    │                                   │
 ┌──────────┐      │  ┌────────────────────────────┐   │      ┌──────────┐
 │  Tauri   │      │  │  calculator/               │   │      │ poe.ninja│
 │ Commands │◄────►│  │    offense_calc.rs          │   │◄────►│  API     │
 │(adapter) │      │  │    defense_calc.rs          │   │      │(adapter) │
 └──────────┘      │  │    formulas.rs              │   │      └──────────┘
                    │  └────────────────────────────┘   │
 ┌──────────┐      │  ┌────────────────────────────┐   │      ┌──────────┐
 │   File   │      │  │  core/                     │   │      │  Claude  │
 │  System  │◄────►│  │    build_analyzer.rs        │   │◄────►│   API    │
 │(adapter) │      │  │    build_detector.rs        │   │      │(adapter) │
 └──────────┘      │  └────────────────────────────┘   │      └──────────┘
                    │  ┌────────────────────────────┐   │
 ┌──────────┐      │  │  seer/                     │   │
 │ SQLite   │◄────►│  │    query_router.rs          │   │
 │(adapter) │      │  │    response_gen.rs          │   │
 └──────────┘      │  └────────────────────────────┘   │
                    └──────────────────────────────────┘

PORTS (traits):              ADAPTERS (implementations):
  trait PriceProvider          PoeNinjaAdapter (real API)
  trait FileStorage            TauriFileAdapter (Tauri fs)
  trait BuildRepository        SqliteBuildRepo (SQLite)
  trait CloudAiProvider        ClaudeAdapter (Claude API)
```

**Key rule:** Domain code NEVER imports `tauri`, `reqwest`, `rusqlite`, or any
external crate. It only uses its own traits. Adapters implement those traits.

```rust
// PORT — defined in domain, no external deps
pub trait PriceProvider: Send + Sync {
    async fn get_price(&self, item_name: &str) -> Result<f64, PriceError>;
    async fn get_prices_batch(&self, items: &[&str]) -> Result<Vec<(String, f64)>, PriceError>;
}

// ADAPTER — implements port using real API
pub struct PoeNinjaAdapter {
    client: reqwest::Client,
    cache: TtlCache<f64>,
    circuit_breaker: CircuitBreaker,
}

impl PriceProvider for PoeNinjaAdapter {
    async fn get_price(&self, item_name: &str) -> Result<f64, PriceError> {
        // Check cache first
        if let Some(cached) = self.cache.get(item_name) { return Ok(*cached); }
        // Check circuit breaker
        if self.circuit_breaker.is_open() { return self.cache.get_stale(item_name).ok_or(PriceError::Offline); }
        // Fetch from API
        let price = self.client.get(&format!("https://poe.ninja/api/...")).send().await?;
        self.cache.set(item_name, price, Duration::from_secs(300));
        Ok(price)
    }
}

// MOCK — for testing, no network calls
pub struct MockPriceProvider {
    prices: HashMap<String, f64>,
}
impl PriceProvider for MockPriceProvider {
    async fn get_price(&self, item_name: &str) -> Result<f64, PriceError> {
        self.prices.get(item_name).copied().ok_or(PriceError::NotFound)
    }
}
```

**Why this matters:**
- `calculator/` and `core/` compile and test WITHOUT Tauri
- Can reuse domain logic in a CLI tool, web API, or WASM
- Swap poe.ninja for a different price source by writing a new adapter
- Test everything with mocks — no network, no file system, no database

### 4.2 Dependency Graph (No Circular Dependencies)

```
commands/       → depends on tauri + domain ports  (thin adapter layer)
core/           → depends on models + ports ONLY   (pure business logic)
calculator/     → depends on models + data ports   (zero external deps)
models/         → depends on serde ONLY            (data types)
data/           → depends on models + ports         (implements data ports)
market/         → depends on models + ports         (implements PriceProvider)
seer/           → depends on calculator + core     (orchestrates domain)
pob_verify/     → depends on mlua (optional)       (optional verification adapter)
renderer/       → depends on wgpu + winit + glam  (native GPU, center panel only)
services/       → depends on tauri + domain ports  (background task adapters)
```

Domain code (`core/`, `calculator/`, `seer/`, `models/`) can be:
- Unit tested without Tauri, without network, without file system
- Reused in a CLI tool (`path-of-ai-cli`)
- Compiled to WASM for web version
- Tested in CI without any OS-specific setup

### 4.3 Tauri 2 Security Boundaries

```
RULE 1: Never handle secrets in frontend
  API keys → encrypted in OS keychain (via tauri-plugin-stronghold)
  Never pass API keys via invoke() params
  
RULE 2: Validate all input from frontend
  Every #[tauri::command] parameter → validate before domain logic
  Parse to domain types immediately (ProjectName::new(&raw))
  
RULE 3: Never expose internal errors to frontend
  Domain errors → user-friendly messages via From<DomainError> for InvokeError
  Log internal details server-side, return generic message to UI

RULE 4: Minimize IPC surface
  Don't expose one command per function — group by feature
  analyze_build, ask_seer, apply_upgrade, get_prices (not 50 tiny commands)
```

### 4.4 Error Handling Strategy

Two-layer error system: domain errors (detailed) → transport errors (user-friendly).

```rust
// Domain errors — detailed, for logging + debugging
#[derive(Debug, Error)]
pub enum CalcError {
    #[error("Mod '{0}' not found in database")]
    ModNotFound(String),
    #[error("Division by zero in {context}")]
    DivisionByZero { context: String },
    #[error("PoB XML parse error: {0}")]
    ParseError(#[from] quick_xml::Error),
}

// Transport errors — user-friendly, for frontend
impl From<CalcError> for tauri::InvokeError {
    fn from(e: CalcError) -> Self {
        match e {
            CalcError::ModNotFound(m) => {
                tracing::warn!("Unknown mod: {}", m);
                tauri::InvokeError::from("Some item mods could not be analyzed. Results may be approximate.")
            },
            CalcError::DivisionByZero { context } => {
                tracing::error!("DivByZero in {}", context);
                tauri::InvokeError::from("Calculation error. Please report this build for investigation.")
            },
            CalcError::ParseError(e) => {
                tracing::error!("Parse: {:?}", e);
                tauri::InvokeError::from("Could not read PoB file. Is it a valid Path of Building export?")
            },
        }
    }
}
```

### 4.5 Serialization Boundary

All data crossing the Rust ↔ TypeScript boundary must be `Serialize + Deserialize`:

```rust
// Rust side — all models derive Serialize
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub overall_score: u8,
    pub defenses: DefenseAnalysis,
    pub offense: OffenseAnalysis,
    pub issues: Vec<Issue>,
    pub suggestions: Vec<Suggestion>,
}
```

```typescript
// TypeScript side — matching types
interface AnalysisResult {
    overall_score: number;
    defenses: DefenseAnalysis;
    offense: OffenseAnalysis;
    issues: Issue[];
    suggestions: Suggestion[];
}
```

**Rule:** Rust uses `snake_case`, TypeScript uses `camelCase`. Serde's `#[serde(rename_all = "camelCase")]` handles the conversion automatically:

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefenseAnalysis {
    pub total_life: u32,          // → totalLife in JSON/TypeScript
    pub energy_shield: u32,       // → energyShield
    pub fire_resist: i32,         // → fireResist
    pub chaos_resist_tier: String, // → chaosResistTier
}
```

### 3.3 App State Management (Rust Side)

Use Tauri's managed state for shared data:

```rust
// src-tauri/src/main.rs

use std::sync::Mutex;

/// Shared app state — accessible from all commands
pub struct AppState {
    pub current_build: Mutex<Option<BuildData>>,
    pub mod_db: ModDatabase,
    pub gem_db: GemDatabase,
    pub tree_db: TreeDatabase,
    pub calculator: PathCalcEngine,       // OUR Rust calc engine (primary, always on)
    pub pob_verify: Option<PobLuaEngine>, // PoB Lua engine (optional verification)
    pub seer: Seer,                       // Query router + response generator
    pub price_cache: Mutex<PriceCache>,
}

fn main() {
    let mod_db = ModDatabase::load(&data_dir).expect("Failed to load mod database");
    let gem_db = GemDatabase::load(&data_dir).expect("Failed to load gem database");
    let tree_db = TreeDatabase::load(&data_dir).expect("Failed to load tree database");

    let state = AppState {
        current_build: Mutex::new(None),
        mod_db,
        gem_db,
        tree_db,
        calculator: PathCalcEngine::new(&data_dir).expect("Failed to load calc engine"),
        pob_verify: PobLuaEngine::try_load(&data_dir).ok(), // optional, graceful if missing
        seer: Seer::new(&data_dir).expect("Failed to init The Seer"),
        price_cache: Mutex::new(PriceCache::new()),
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::build_commands::analyze_build,
            commands::build_commands::get_upgrade_suggestions,
            commands::market_commands::get_prices,
            commands::seer_commands::ask_seer,
            commands::file_commands::watch_pob_directory,
        ])
        .setup(|app| {
            // Start background services
            services::file_watcher::start(app.handle().clone())?;
            services::update_checker::start(app.handle().clone())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Failed to launch Path of AI");
}
```

Commands access state via `tauri::State`:

```rust
#[tauri::command]
pub async fn analyze_build(
    file_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    let xml = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let build_data = PobParser::parse(&xml).map_err(|e| e.to_string())?;

    let analyzer = BuildAnalyzer::new(&build_data, &state.mod_db);
    let result = analyzer.analyze();

    // Cache current build for other commands
    *state.current_build.lock().unwrap() = Some(build_data);

    Ok(result)
}
```

### 3.4 Immutable Game Data, Mutable App State

```
IMMUTABLE (loaded once, thread-safe):
  ModDatabase, GemDatabase, TreeDatabase
  → No Mutex needed, shared freely across threads
  → Reloaded atomically on data update (swap entire struct)

MUTABLE (changes during runtime, needs Mutex):
  current_build    → changes when user loads/switches builds
  price_cache      → updated by background poller
  seer             → lazy-loaded, then persistent
```

---

## 5. COMMUNICATION PATTERNS

### 4.1 Frontend → Backend (Commands)

```
User action → invoke('command_name', { args }) → Rust handler → Result<T>
```

```typescript
// User clicks "Analyze" button
const result = await analyzeBuild('/path/to/build.xml');
store.setBuildAnalysis(result);
```

### 4.2 Backend → Frontend (Events)

```
Rust detects change → app.emit('event-name', payload) → Frontend listener
```

```rust
// File watcher detects PoB change
app.emit("pob-file-changed", &file_path)?;

// Price update completed
app.emit("prices-updated", &price_summary)?;

// Data update available
app.emit("data-update-available", &update_info)?;
```

### 4.3 Long-Running Operations (Channels)

For operations that take time (AI inference, market search):

```rust
#[tauri::command]
pub async fn ask_seer(
    question: String,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Emit "thinking" event immediately
    app.emit("seer-thinking", &question).ok();

    // Run AI inference (may take 100ms-2s)
    let build = state.current_build.lock().unwrap().clone();
    let seer = state.seer.lock().unwrap();

    if let (Some(build), Some(seer)) = (build, seer.as_ref()) {
        let response = seer.ask(&question, &build);
        // Emit result when done
        app.emit("seer-response", &response).ok();
    }

    Ok(())
}
```

---

## 6. TESTING STRATEGY

### Unit Tests (Rust core/)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phys_reduction_formula() {
        // PoE formula: Armour / (Armour + 5 * Damage) * 100
        assert_eq!(calculate_phys_reduction(25000, 5000), 50.0);
        assert_eq!(calculate_phys_reduction(0, 5000), 0.0);
    }

    #[test]
    fn test_chaos_res_tier() {
        assert_eq!(chaos_res_tier(75).tier, "capped");
        assert_eq!(chaos_res_tier(-31).tier, "negative");
    }

    #[test]
    fn test_parse_sample_build() {
        let xml = include_str!("../../../test-data/SampleRFInquisitor.xml");
        let build = PobParser::parse(xml).unwrap();
        assert_eq!(build.class_name, "Templar");
        assert_eq!(build.ascend_class_name, "Inquisitor");
        assert!(build.stats.life > 5000);
    }
}
```

### Integration Tests (Tauri commands)
```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_full_analysis_pipeline() {
        let data_dir = PathBuf::from("./game-data");
        let mod_db = ModDatabase::load(&data_dir).unwrap();
        let xml = std::fs::read_to_string("./test-data/SampleRFInquisitor.xml").unwrap();
        let build = PobParser::parse(&xml).unwrap();
        let analyzer = BuildAnalyzer::new(&build, &mod_db);
        let result = analyzer.analyze();

        assert!(result.overall_score > 0);
        assert!(!result.issues.is_empty());
        assert!(!result.suggestions.is_empty());
    }
}
```

---

## 7. CRATE DEPENDENCIES

```toml
# src-tauri/Cargo.toml

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
quick-xml = "0.36"                  # Fast XML parsing (PoB files)
notify = "7"                        # File system watcher
reqwest = { version = "0.12", features = ["json"] }  # HTTP client (poe.ninja)
tokio = { version = "1", features = ["full"] }        # Async runtime
thiserror = "2"                     # Error types
mlua = { version = "0.10", features = ["luajit", "vendored"], optional = true }  # Optional: PoB Lua verification
rusqlite = { version = "0.32", features = ["bundled"] }          # SQLite (cache, build history)

[features]
default = []
pob-verify = ["mlua"]  # Enable PoB Lua verification engine
chrono = "0.4"                      # Date/time handling
log = "0.4"                         # Logging
env_logger = "0.11"                 # Log output
sha2 = "0.10"                       # Checksum verification for updates
regex = "1"                          # Intent classification (Seer query router)
uuid = { version = "1", features = ["v4"] }  # Build share codes
base64 = "0.22"                      # Item clipboard format parsing
wgpu = "24"                          # Native GPU rendering (Vulkan/Metal/DX12)
winit = "0.30"                       # Window handle for wgpu surface
image = "0.25"                       # Sprite/texture loading
glam = "0.29"                        # Math (vectors, matrices for rendering)
```

---

## 8. WHY THESE PATTERNS

| Pattern | Where Used | Why |
|---------|-----------|-----|
| **Command** | Tauri IPC | Clean frontend-backend contract, each command is independently testable |
| **Repository** | Game data access | Abstracts data source, supports auto-update without changing consumers |
| **Strategy** | Build analysis | Different build types need different scoring — open/closed principle |
| **Observer** | File watcher + events | Decouples file detection from UI update, reactive data flow |
| **Builder** | Complex structs (Item, Build) | Many optional fields in PoE data, readable test construction |
| **Facade** | Seer Engine | Routes queries to Calculator/KB/Cloud behind one `ask()` API |
| **State** | Tauri managed state | Thread-safe shared data between commands and services |

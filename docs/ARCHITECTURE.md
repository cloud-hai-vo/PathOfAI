# Path of AI — Architecture & Design Patterns

## Overview

Path of AI is a **Tauri desktop app**:
- **Backend:** Rust (performance-critical logic, file I/O, AI inference, system access)
- **Frontend:** HTML/CSS/TypeScript (UI rendering, user interaction)
- **Communication:** Tauri IPC commands (frontend calls Rust functions)

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
│   │   │   ├── combat_sim.rs         # "The Arena" — boss/monster fight simulation
│   │   │   ├── gem_optimizer.rs      # Gem swap / level / quality analysis
│   │   │   ├── map_mod_analyzer.rs   # "Curse Map" — dangerous map mod detection
│   │   │   └── item_image.rs         # Item → PoE CDN image URL resolver
│   │   │
│   │   ├── data/                     # Game data loading & versioning
│   │   │   ├── mod.rs
│   │   │   ├── loader.rs             # Load versioned JSON game data
│   │   │   ├── mod_database.rs       # Mod tier lookups
│   │   │   ├── gem_database.rs       # Gem info lookups
│   │   │   ├── tree_database.rs      # Passive tree data
│   │   │   └── updater.rs            # Auto-update data from GitHub Releases
│   │   │
│   │   ├── market/                   # Market intelligence
│   │   │   ├── mod.rs
│   │   │   ├── poe_ninja.rs          # poe.ninja API client
│   │   │   ├── price_cache.rs        # Local price cache with TTL
│   │   │   ├── buy_advisor.rs        # Buy timing / trend analysis
│   │   │   └── upgrade_finder.rs     # Find upgrades within budget
│   │   │
│   │   ├── seer/                     # Local AI engine
│   │   │   ├── mod.rs
│   │   │   ├── item_net.rs           # Item scoring neural network
│   │   │   ├── build_net.rs          # Build classification network
│   │   │   ├── tree_net.rs           # Passive tree optimization
│   │   │   ├── query_net.rs          # NLU / query understanding
│   │   │   ├── embed_net.rs          # RAG embedding + vector search
│   │   │   ├── response_gen.rs       # Template-based response generator
│   │   │   └── onnx_runtime.rs       # ONNX model loading & inference
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
├── src/                              # Frontend (TypeScript + HTML/CSS)
│   ├── index.html                    # Main HTML entry
│   ├── main.ts                       # Frontend entry, Tauri IPC setup
│   ├── styles/
│   │   ├── theme.css                 # PoE theme variables, colors, fonts
│   │   ├── components.css            # Reusable component styles
│   │   └── layout.css                # Grid, panels, tabs
│   ├── components/                   # UI components (vanilla TS or framework)
│   │   ├── overview-panel.ts
│   │   ├── item-list.ts
│   │   ├── resist-bar.ts
│   │   ├── score-ring.ts
│   │   ├── issue-card.ts
│   │   ├── suggestion-card.ts
│   │   ├── gem-table.ts
│   │   ├── checklist.ts
│   │   └── seer-chat.ts
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

### 2.7 Facade Pattern (Seer Engine)

The Seer Engine is complex internally (5 neural networks + RAG). Expose a simple facade.

```rust
// src-tauri/src/seer/mod.rs

/// The Seer — single entry point for all AI queries.
/// Hides the complexity of 5 neural networks + RAG + templates.
pub struct Seer {
    item_net: ItemNet,
    build_net: BuildNet,
    tree_net: TreeNet,
    query_net: QueryNet,
    embed_net: EmbedNet,
    response_gen: ResponseGenerator,
    knowledge_base: KnowledgeBase,
}

impl Seer {
    /// Load all models and knowledge base
    pub fn load(model_dir: &Path, kb_dir: &Path) -> Result<Self, SeerError> {
        Ok(Self {
            item_net: ItemNet::load(model_dir.join("item_net.onnx"))?,
            build_net: BuildNet::load(model_dir.join("build_net.onnx"))?,
            tree_net: TreeNet::load(model_dir.join("tree_net.onnx"))?,
            query_net: QueryNet::load(model_dir.join("query_net.gguf"))?,
            embed_net: EmbedNet::load(model_dir.join("minilm.onnx"))?,
            response_gen: ResponseGenerator::new(),
            knowledge_base: KnowledgeBase::load(kb_dir)?,
        })
    }

    /// Simple API: ask a question, get an answer
    pub fn ask(&self, question: &str, build: &BuildData) -> SeerResponse {
        // 1. Understand the question
        let intent = self.query_net.classify(question);
        let entities = self.query_net.extract_entities(question);

        // 2. Retrieve relevant knowledge
        let context = self.embed_net.search(&self.knowledge_base, question, 5);

        // 3. Route to appropriate network
        let data = match intent {
            Intent::ItemScore    => self.item_net.score_items(build, &entities),
            Intent::BuildIssue   => self.build_net.detect_issues(build),
            Intent::TreeAdvice   => self.tree_net.suggest_changes(build),
            Intent::General      => self.response_gen.from_context(&context, build),
        };

        // 4. Generate natural language response
        self.response_gen.generate(intent, data, &context)
    }

    /// Score a specific item (direct API, no NLU needed)
    pub fn score_item(&self, item: &Item, build: &BuildData) -> ItemScore {
        self.item_net.score(item, build)
    }

    /// Get upgrade priority for all slots
    pub fn upgrade_priorities(&self, build: &BuildData) -> Vec<UpgradePriority> {
        self.item_net.rank_slots(build)
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

## 3. KEY DESIGN PRINCIPLES

### 3.1 Clean Boundary: Rust Core Has No Tauri Dependency

```
commands/       → depends on tauri + core    (thin glue layer)
core/           → depends on models ONLY     (pure business logic)
models/         → depends on serde ONLY      (data types)
data/           → depends on models + serde  (data loading)
services/       → depends on tauri + core    (background tasks)
seer/           → depends on models + onnx   (AI inference)
```

This means `core/` can be:
- Unit tested without Tauri
- Reused in a CLI tool
- Compiled to WASM for web if needed

### 3.2 Serialization Boundary

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
    pub seer: Mutex<Option<Seer>>,
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
        seer: Mutex::new(None),  // lazy-loaded on first AI query
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

## 4. COMMUNICATION PATTERNS

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

## 5. TESTING STRATEGY

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

## 6. CRATE DEPENDENCIES

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
ort = "2"                           # ONNX Runtime (neural network inference)
rusqlite = { version = "0.32", features = ["bundled"] }  # SQLite (vector search, cache)
chrono = "0.4"                      # Date/time handling
log = "0.4"                         # Logging
env_logger = "0.11"                 # Log output
sha2 = "0.10"                       # Checksum verification for updates
```

---

## 7. WHY THESE PATTERNS

| Pattern | Where Used | Why |
|---------|-----------|-----|
| **Command** | Tauri IPC | Clean frontend-backend contract, each command is independently testable |
| **Repository** | Game data access | Abstracts data source, supports auto-update without changing consumers |
| **Strategy** | Build analysis | Different build types need different scoring — open/closed principle |
| **Observer** | File watcher + events | Decouples file detection from UI update, reactive data flow |
| **Builder** | Complex structs (Item, Build) | Many optional fields in PoE data, readable test construction |
| **Facade** | Seer Engine | Hides 5 neural networks behind one simple `ask()` API |
| **State** | Tauri managed state | Thread-safe shared data between commands and services |

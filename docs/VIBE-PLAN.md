# Path of AI — Vibe Coding Plan

How to build this app session by session with Claude Code as your co-pilot.
Each session is ~1-3 hours. Start a new Claude Code session for each one.

**Before every session:** open Claude Code in `d:\Work\g\PathOfAI\` — it will automatically read `CLAUDE.md`.

---

## SESSION 0 — Scaffold the Tauri 2 Project

**Goal:** Working Tauri window showing the existing UI prototype.

### Commands to run first
```bash
cd d:\Work\g\PathOfAI

# Install Tauri CLI (if not already)
npm install -g @tauri-apps/cli

# Scaffold — when prompted:
#   App name:    path-of-ai
#   Identifier:  com.pathofai.app
#   Frontend:    Vanilla TypeScript
#   Package manager: npm
npm create tauri@latest -- --template vanilla-ts --identifier com.pathofai.app path-of-ai-tauri
```

### Prompt for Claude
```
I just scaffolded a Tauri 2 project. I have an existing UI prototype at ui/path-of-ai-game.html
and assets at ui/assets/. I want to:

1. Copy the CSS from ui/styles/ into src/styles/
2. Set up tauri.conf.json so the window opens at 1400x900, titled "Path of AI", with devPath pointing to src/
3. Create a minimal src/index.html that loads the CSS and shows a loading screen with our logo (ui/assets/icon.svg)
4. Add a single Tauri command `get_version` that returns the app version string from Cargo.toml
5. Wire it: frontend calls invoke('get_version') and shows it in the loading screen

Read CLAUDE.md and docs/CODE-PATTERNS.md first.
```

### Done when
- `npm run tauri dev` opens a window with the loading screen
- Version string appears from Rust backend

---

## SESSION 1 — Core Data Models + PoB Parser

**Goal:** Parse `test-data/SampleRFInquisitor.xml` into Rust structs. This is the foundation everything else builds on.

### Prompt for Claude
```
I need to implement the PoB XML parser in Rust. 

Read these first:
- docs/ALGORITHMS.md (Algorithm 2 — PoB Parser)
- prototypes/pob-parser.js (JS reference implementation to port)
- test-data/SampleRFInquisitor.xml (test file)
- docs/CODE-PATTERNS.md (project structure)

Then:
1. Create src-tauri/src/models/build.rs — define BuildData, Item, Gem, PassiveTree, BuildConfig structs (see Algorithm 2 for exact fields)
2. Create src-tauri/src/core/pob_parser.rs — implement parse_file(path: &str) -> Result<BuildData, PobError>
3. Port the logic from prototypes/pob-parser.js 1:1 to Rust using quick-xml crate
4. Create src-tauri/src/commands/build_commands.rs — thin handler for analyze_build IPC command
5. Register the command in lib.rs

Write tests that:
- Parse SampleRFInquisitor.xml and assert item count >= 10
- Assert class_name == "Templar"
- Assert at least one gem in the main skill
```

### Done when
- `cargo test` passes
- `invoke('analyze_build', { filePath: '...' })` returns a BuildData JSON

---

## SESSION 2 — Defense Calculator

**Goal:** Calculate life, ES, resists, armour, evasion from a parsed build.

### Prompt for Claude
```
I need to implement the defense calculator in Rust.

Read first:
- docs/ALGORITHMS.md (Algorithm 5 — Resistance Aggregation, Algorithm 14 — Effective HP Calculator)
- docs/ENGINE-DESIGN.md (defense formulas section)
- prototypes/build-analyzer.js (JS reference)

Then:
1. Create src-tauri/src/calculator/defense_calc.rs
   - calculate_life(build: &BuildData) -> u32
   - calculate_es(build: &BuildData) -> u32
   - calculate_resistances(build: &BuildData) -> ResistanceProfile
   - calculate_armour_reduction(armour: u32, hit: u32) -> f32   (PoE formula: A / (A + 10*D))
   - calculate_effective_hp(build: &BuildData) -> EffectiveHP
2. Add DefenseStats to models/analysis.rs
3. Wire into the analyze_build command — include DefenseStats in AnalysisResult

Write unit tests:
- RF Inquisitor life should be ~5000-7000 (parse SampleRFInquisitor.xml)
- Armour reduction formula: armour=10000, hit=1000 → ~50% reduction
- Resistances: all three ele res capped at 75%, chaos at whatever the build has
```

### Done when
- `cargo test` passes for defense_calc
- `invoke('analyze_build')` response includes life, ES, resists, effective HP

---

## SESSION 3 — DPS Calculator

**Goal:** Calculate total DPS for the main skill.

### Prompt for Claude
```
I need to implement the DPS calculator in Rust, focused on DoT/RF builds first.

Read first:
- docs/ALGORITHMS.md (Algorithm 3 — DPS Calculation Engine)
- docs/ENGINE-DESIGN.md (offense formulas)
- prototypes/build-analyzer.js (JS reference — DPS section)

Then:
1. Create src-tauri/src/calculator/offense_calc.rs
   - calculate_dps(build: &BuildData) -> DpsBreakdown
   - calculate_dot_dps(build: &BuildData) -> f64   (for RF/poison/bleed)
   - apply_multiplier_chain(base: f64, increased: f64, more_mults: &[f64]) -> f64
   - calculate_hit_dps(build: &BuildData) -> f64
2. DpsBreakdown struct: total, per_source (Vec<DpsSource>), multiplier_chain
3. Wire into AnalysisResult

Write tests:
- RF DPS formula: verify multiplier chain math
- Test with SampleRFInquisitor.xml — result should be > 1M DPS
- Test that "more" multipliers stack multiplicatively
```

### Done when
- DPS appears in `invoke('analyze_build')` response with a per-source breakdown

---

## SESSION 4 — Build Analyzer (Issues + Item Scoring)

**Goal:** Detect build problems and score each equipment slot 0-100.

### Prompt for Claude
```
I need to implement the build analyzer — issue detection and item scoring.

Read first:
- docs/ALGORITHMS.md (Algorithm 6 — Item Scorer, Algorithm 7 — Issue Detector)
- docs/FEATURES.md (CORE SYSTEMS section — Resistance Checker, Ailment Immunity Checklist)
- prototypes/build-analyzer.js (JS reference)
- prototypes/build-detector.js (JS reference — archetype detection)

Then:
1. Create src-tauri/src/core/build_analyzer.rs
   - detect_issues(build: &BuildData, defenses: &DefenseStats) -> Vec<Issue>
   - Issue: { severity: Severity, title: String, detail: String, fix: String }
   - Issues to detect: uncapped resists, low life (<4000), missing ailment immunity (freeze/shock/bleed/corrupted blood), no movement skill, low DPS
2. Create src-tauri/src/core/build_detector.rs
   - detect_archetype(build: &BuildData) -> Archetype  (rule-based, see prototype)
3. Create src-tauri/src/core/item_scorer.rs
   - score_item(item: &Item, archetype: Archetype, data: &GameData) -> u8  (0-100)
4. Add issues + scores to AnalysisResult

Tests:
- SampleRFInquisitor: detect if chaos res is uncapped (it likely is)
- Verify archetype detected as FireDoT or similar
- Verify main skill items score higher than off-meta items
```

### Done when
- `invoke('analyze_build')` returns issues array and per-item scores

---

## SESSION 5 — UI Data Wiring (Connect Real Data to Panels)

**Goal:** The actual Tauri frontend shows real data from the Rust backend.

### Prompt for Claude
```
I need to wire the Tauri frontend to show real build data in the UI panels.

Read first:
- docs/FLOWS.md (Flow 2 — Load Build, Flow 3 — Click Equipment Slot)
- docs/IPC-SPEC.md (commands and events)
- ui/path-of-ai-game.html (the UI prototype — port this to TypeScript)
- docs/CODE-PATTERNS.md (TypeScript conventions)

Then:
1. Create src/types/index.ts — TypeScript interfaces matching Rust models (AnalysisResult, Issue, Item, DefenseStats, DpsBreakdown)
2. Create src/services/bridge.ts — typed invoke() wrappers for all commands
3. Create src/services/store.ts — simple observable store (no framework)
   - currentBuild: BuildData | null
   - analysis: AnalysisResult | null
   - subscribe(listener) / notify()
4. Port the HUD layout from ui/path-of-ai-game.html to src/index.html + src/main.ts
   - Header with build name, level, class
   - Left stat sidebar (life, ES, DPS, resists)
   - Center character visualization
   - Right panel system (Prophecy panel by default)
5. Wire "Import PoB File" button → invoke('analyze_build') → update store → re-render all panels
6. Wire Defenses panel to show real DefenseStats from store
7. Wire DPS panel to show real DpsBreakdown from store

Keep all CSS variables from ui/styles/ — do not redesign the theme.
```

### Done when
- Open app, click "Import PoB", select SampleRFInquisitor.xml
- Life / resists / DPS appear with real numbers
- Issues list shows in Prophecy panel

---

## SESSION 6 — Seer Query Router

**Goal:** "Ask The Seer" works for common questions using the rule-based router.

### Prompt for Claude
```
I need to implement the Seer query router — the AI that answers build questions.

Read first:
- docs/ALGORITHMS.md (Algorithm 1 — Seer Query Router)
- docs/FLOWS.md (Flow 4 — Ask The Seer a Question)
- docs/FEATURES.md (THE SEER section)
- prototypes/seer-engine.js (JS reference)

Then:
1. Create src-tauri/src/seer/intent_classifier.rs
   - classify(question: &str) -> Intent  (50 regex rules — see Algorithm 1)
   - Intent enum: DpsQuery, ResistQuery, UpgradeQuery, CraftQuery, BossQuery, GemQuery, Fallback
2. Create src-tauri/src/seer/response_generator.rs
   - generate(intent: Intent, build: &BuildData, analysis: &AnalysisResult) -> String
   - Template responses for each intent type
3. Create src-tauri/src/seer/router.rs
   - route(question: &str, build: &BuildData, analysis: &AnalysisResult) -> SeerResponse
   - For Fallback: return "I need a connected AI for that question. [Connect Claude →]"
4. Wire to ask_seer Tauri command
5. Wire Grimoire panel in UI to show Seer conversation

Tests:
- "what is my dps" → DpsQuery → response contains actual DPS number
- "why am I dying" → ResistQuery or DefenseQuery → response mentions resistances
- "what should I upgrade first" → UpgradeQuery → response mentions top issue
- "tell me a story" → Fallback
```

### Done when
- Type "what is my dps?" in Grimoire panel → Seer answers with real number
- Type "what's my weakest defense?" → Seer identifies the issue

---

## SESSION 7 — Market Integration (poe.ninja)

**Goal:** Real item prices from poe.ninja appear on suggestions.

### Prompt for Claude
```
I need to implement the poe.ninja market integration.

Read first:
- docs/ALGORITHMS.md (Algorithm 21 — Price Cache, Algorithm 8 — Upgrade Suggestion Ranker)
- docs/DATA-SOURCES.md (poe.ninja API section)
- prototypes/market-intelligence.js (JS reference)

Then:
1. Create src-tauri/src/market/poe_ninja_client.rs
   - fetch_item_price(item_name: &str, league: &str) -> Result<f64, MarketError>
   - Uses reqwest async client
   - Endpoints: /api/data/itemoverview, /api/data/currencyoverview
2. Create src-tauri/src/market/price_cache.rs
   - 5-minute TTL cache (HashMap + timestamp)
   - Stale fallback: return last known price if API down
   - Circuit breaker: stop calling after 3 failures
3. Create src-tauri/src/market/upgrade_finder.rs
   - find_upgrades(slot: &str, build: &BuildData, budget: f64) -> Vec<TradeResult>
   - Rank by DPS-per-divine cost efficiency
4. Wire get_prices and search_upgrades Tauri commands
5. Update Prophecy panel: each suggestion shows price estimate

Tests (use mock HTTP):
- Cache returns stale data when API down
- Circuit breaker opens after 3 failures
- Prices refresh after TTL expires
```

### Done when
- Prophecy panel shows "Watcher's Eye: ~8.5 div" next to each suggestion

---

## SESSION 8 — PoE OAuth (Live Character Data)

**Goal:** "Connect PoE Account" works — fetches live character from GGG servers.

### Prompt for Claude
```
I need to implement PoE OAuth 2.0 PKCE flow for live character import.

Read first:
- docs/ALGORITHMS.md (Algorithm 37 — OAuth PKCE, Algorithm 45 — Character Fetch Pipeline)
- docs/FLOWS.md (Flow 8 — Connect PoE OAuth)
- docs/IPC-SPEC.md (OAuth commands)

Then:
1. Create src-tauri/src/core/oauth.rs
   - generate_pkce_pair() -> (code_verifier: String, code_challenge: String)
   - SHA256 hash → base64url encode (no padding)
   - generate_state() -> String  (16 random bytes → hex)
2. Create local redirect server (tokio tiny HTTP server on port 29473)
   - Captures ?code=... and ?state=... from PoE callback
3. Create src-tauri/src/core/characters.rs
   - fetch_characters(token: &str) -> Vec<CharacterSummary>
   - fetch_character_items(token: &str, name: &str) -> Vec<Item>
   - fetch_character_passives(token: &str, name: &str) -> PassiveTree
4. Open browser to PoE OAuth URL, wait for callback, exchange code for token
5. Save encrypted token to PathOfAI_Data/auth.json (AES-256)
6. Wire load_character Tauri command
7. Add "Connect Account" button to settings panel

Tests (mock GGG API):
- PKCE: code_challenge = base64url(sha256(code_verifier))
- State CSRF: reject callback if state mismatch
- Token storage: encrypted at rest
```

### Done when
- Click "Connect Account" → browser opens → login to PoE → app shows your characters
- Select character → app analyzes it like a PoB import

---

## SESSION 9 — SQLite + History

**Goal:** Build history, undo/redo, and wealth snapshots persisted to SQLite.

### Prompt for Claude
```
I need to implement SQLite persistence for build history and wealth tracking.

Read first:
- docs/DATABASE.md (full schema — all 13 tables)
- docs/ALGORITHMS.md (Algorithm 33 — Snapshot, Algorithm 53 — Map Run Accumulator)

Then:
1. Add rusqlite to Cargo.toml with bundled feature
2. Create src-tauri/src/db/mod.rs — connection pool + migration runner
3. Create src-tauri/src/db/schema.rs — run CREATE TABLE IF NOT EXISTS for all 13 tables from DATABASE.md
4. Create src-tauri/src/db/builds.rs — save_build(), load_build(), list_builds()
5. Create src-tauri/src/db/snapshots.rs — snapshot_before_change(), get_undo_stack(), restore_snapshot()
6. Wire undo_last_change and redo_change Tauri commands
7. Create src-tauri/src/db/wealth.rs — record_wealth_snapshot() triggered from price poller

Tests:
- Save build → load build → same data
- Snapshot: save 3 snapshots, undo twice, verify correct state
- Keep only last 50 snapshots per build
```

### Done when
- Undo/redo works (Ctrl+Z / Ctrl+Y)
- Build history list shows in settings

---

## SESSION 10 — The Forge (Crafting Advisor)

**Goal:** "The Forge" panel shows craft suggestions with probability estimates.

### Prompt for Claude
```
I need to implement the crafting advisor ("The Forge").

Read first:
- docs/ALGORITHMS.md (Algorithm 24 — Crafting Advisor, Algorithm 47 — Craft Suggestion Ranker)
- docs/FEATURES.md (THE FORGE section)
- docs/FLOWS.md (Flow 7 — Open The Forge)

Then:
1. Create src-tauri/src/core/craft_advisor.rs
   - get_craft_suggestions(build: &BuildData, currency: &CurrencyInventory) -> Vec<CraftSuggestion>
   - CraftSuggestion: { method, target_mod, probability, expected_cost, dps_gain }
   - compare_craft_vs_buy(slot: &str, build: &BuildData) -> CraftVsBuyResult
   - geometric_99th_percentile(p: f64) -> u32  (= ceil(log(0.01)/log(1-p)) — Algorithm 47)
2. Create src-tauri/src/data/mod_database.rs — load mod weights from poedb JSON
3. Wire get_craft_suggestions Tauri command
4. Build The Forge panel UI with craft method cards (chaos/essence/fossil/benchcraft)
   Each card shows: probability, expected currency cost, DPS gain if successful

Tests:
- geometric_99th_percentile: p=0.5 → 7 attempts for 99% success
- Craft suggestion: benchcraft has p=1.0 (deterministic) → always first if applicable
- compare_craft_vs_buy: if craft expected cost > buy price → recommend buy
```

### Done when
- Open The Forge panel → see craft options with real probabilities and costs
- "Craft vs Buy" shows which is better for each slot

---

## After Session 10

You now have a fully functional MVP. Continue with:

```
Session 11: Passive Tree Viewer (Algorithm 23, 48)
Session 12: Stash Tab Integration (Algorithm 38, with OAuth)
Session 13: Map Run Tracker + Wealth Accumulator (Algorithm 53)
Session 14: Price Alerts (Algorithm 50)
Session 15: Build Comparator + poe.ninja top builds (Algorithm 49)
Session 16: Combat Simulator wgpu renderer (Algorithm 20)
Session 17: Auto-update system (docs/AUTO-UPDATE-SYSTEM.md)
Session 18: Testing + polish (docs/TESTING.md)
```

---

## Tips for Vibe Coding

**Start each session with:**
```
Read CLAUDE.md first, then [specific docs listed in session prompt].
```

**When Claude writes wrong logic:**
```
Read docs/ALGORITHMS.md Algorithm [N] again — the implementation should match that spec exactly.
```

**When the UI looks wrong:**
```
Reference ui/path-of-ai-game.html for the correct visual design. Match the existing CSS variables and layout.
```

**When tests fail:**
```
Read test-data/SampleRFInquisitor.xml and trace through the algorithm manually to find where the output diverges.
```

**Checkpoint after each session:**
```bash
git add -A && git commit -m "Session N: [what you built]"
```

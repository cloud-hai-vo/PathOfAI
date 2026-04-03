# Path of AI — Implementation Plan

## MVP Target: 20 weeks (5 months)

```
CRITICAL PATH:
  Rust Calculator → Test Data → Game Data Loader → Suggestion Engine → UI
  (everything else depends on the calculator being correct)
```

---

## Sprint 1-2: Rust Calculator Core (Weeks 1-4)

**Goal:** Calculate life, ES, resists, armour, basic DPS for any PoB build.

```
Week 1: Parser + Defense Basics
  - Port pob-parser.js to Rust (pob_parser.rs)
  - Parse: build stats, items, skills, tree, config
  - Parse: <Calcs> section (free pre-computed stats)
  - Parse: {crafted} mod flag, influence tags
  - Implement: life, ES, mana calculation
  - Implement: resistance aggregation (flat + %, overcap)
  - Test: parse SampleRFInquisitor.xml → verify life = 6453

Week 2: Defense + Basic Offense
  - Implement: armour formula (phys reduction vs N damage)
  - Implement: evasion, block, spell block
  - Implement: max resistance tracking
  - Implement: base damage → %increased → more multiplier chain
  - Implement: DoT DPS (burning, poison, bleed)
  - Test: 10 builds from poe.ninja → DPS within ±5% of PoB

Week 3: Advanced Offense
  - Implement: crit chance + crit multi
  - Implement: hit chance / accuracy
  - Implement: damage conversion chains
  - Implement: attack/cast speed
  - Implement: penetration + exposure
  - Test: 20 builds → DPS within ±2% of PoB

Week 4: Edge Cases
  - Implement: conditional modifiers (while_moving, on_full_life, etc.)
  - Implement: guard skill uptime (Molten Shell, Steelskin)
  - Implement: life regen vs degen balance (RF)
  - Implement: leech rate + cap
  - Test: 50 builds (all archetypes) → all stats within ±2%
```

**Deliverable:** `calculator/` module that produces same numbers as PoB for 50 test builds.

---

## Sprint 3: Test Infrastructure + Game Data (Weeks 5-6)

**Goal:** Robust test suite + bundled game data.

```
Week 5: Test Data + Unit Tests
  - Create 8 test PoB XML files (RF, ColdDoT, Attack, Minion, CoC, SSF, Empty, Malformed)
  - Write 50 unit tests for formulas (phys reduction, crit, conversion, etc.)
  - Write 20 parser tests (all item slots, all gem types, tree, config)
  - Set up GitHub Actions CI: cargo test on every push

Week 6: Game Data Loader
  - Run RePoE extraction → generate mod-tiers.json, gems.json, tree.json
  - Implement data/loader.rs — load versioned JSON at startup
  - Implement data/mod_database.rs — get_tier(), max_value(), weight()
  - Implement data/gem_database.rs — compatible_supports(), scaling()
  - Test: loader returns correct data for known items/gems
```

**Deliverable:** CI pipeline + 70 tests passing + bundled game data.

---

## Sprint 4: Build Analysis + Suggestions (Weeks 7-10)

**Goal:** Analyze a build, detect issues, suggest upgrades with verified DPS numbers.

```
Week 7: Build Analyzer
  - Port build-analyzer.js to Rust
  - Score items (0-100) using archetype stat weights
  - Detect issues (uncapped resists, low life, missing ailment immunity)
  - Detect archetype (rule engine, not ML)
  - Test: analyzer finds all known issues in test builds

Week 8: Suggestion Engine
  - Generate upgrade candidates per slot
  - Calculate exact DPS/life diff using our Rust calculator
  - Validate: no resist uncap, no stat requirement break
  - Rank by cost-efficiency (DPS per divine)
  - Test: suggestions for RF Inquisitor match expected priorities

Week 9: Market Integration
  - Implement poe.ninja API client (price fetching)
  - Implement price cache with 5-min TTL + stale fallback
  - Implement circuit breaker (handle API downtime)
  - Implement craft-vs-buy comparison using mod weights
  - Test: prices match poe.ninja within 1 refresh cycle

Week 10: Crafting Advisor ("The Forge")
  - Implement crafting probability calculator (mod weights from poedb)
  - Implement craft method comparison (chaos vs essence vs fossil)
  - Implement currency-aware suggestions ("you have 8 Essence of Anger")
  - Test: probabilities within ±5% of Craft of Exile
```

**Deliverable:** Full analysis pipeline: parse → detect → score → suggest → validate → rank.

---

## Sprint 5: The Seer Query Engine (Weeks 11-12)

**Goal:** User asks question → gets accurate, build-specific answer.

```
Week 11: Query Router + Response Templates
  - Implement intent classifier (50 regex rules)
  - Route: DPS/resist/upgrade queries → Calculator
  - Route: crafting/boss/gem queries → Knowledge Base
  - Route: creative queries → Cloud API (or "not available")
  - Build template response generator
  - Test: 100 sample queries correctly classified and answered

Week 12: Cloud AI Integration
  - Implement context injection (build data → prompt)
  - Wire Claude API (Anthropic SDK in Rust)
  - Wire OpenAI API (optional)
  - Per-provider context format (XML for Claude, JSON for GPT)
  - Privacy: anonymize build data, never send account info
  - Test: cloud responses are build-specific and accurate
```

**Deliverable:** Seer answers 100 test questions correctly.

---

## Sprint 6-7: Frontend (Weeks 13-16)

**Goal:** Game-like HUD UI in vanilla TypeScript.

```
Week 13: Core Layout + Tauri Commands
  - Set up Tauri 2 project scaffold
  - Implement 5 Tauri commands: analyze_build, ask_seer, get_prices, apply_upgrade, watch_pob
  - Build HUD layout (header, left sidebar, center, right panel, bottom bar)
  - Wire invoke() calls from frontend to backend

Week 14: Character Viz + Equipment
  - SVG character body with equipment slots
  - Aura rings + buff bar
  - Click slot → item tooltip in right panel
  - Score rings + stat cards

Week 15: All Panels
  - Prophecy (suggestions with market data)
  - Grimoire (Seer chat with quick questions)
  - The Forge (crafting advisor)
  - Defenses, DPS, Gems, Flasks panels
  - Combat (boss readiness, map clear)
  - Stash (inventory grid, currency, div cards)

Week 16: Polish + Animations
  - Power-up effect on upgrade apply
  - Blood drip, aura glow, ember particles
  - Passive tree mini-view
  - Dark Path (build evolution)
  - Blood Pact (checklist)
  - Settings (AI provider, PoE OAuth, PoB path)
```

**Deliverable:** Fully interactive UI with all panels working.

---

## Sprint 8: File Watcher + Auto-Update (Weeks 17-18)

**Goal:** Real-time sync with PoB + data stays current.

```
Week 17: File Watcher + Undo/Redo
  - Watch PoB build directory for changes
  - Debounce 500ms → re-analyze → push to frontend
  - Atomic file writes (temp → rename)
  - Undo/redo stack (50 snapshots)
  - Backup before every write

Week 18: Auto-Update System
  - Check GitHub Releases for data updates on startup
  - Download delta files (only changed JSONs)
  - Verify checksums → atomic swap
  - League launch detection (check every 30 min on launch week)
```

**Deliverable:** Live sync with PoB, auto-updating game data.

---

## Sprint 9-10: E2E Testing + Release (Weeks 19-20)

```
Week 19: E2E Tests + Performance
  - Set up Playwright for full workflow tests
  - 8 E2E scenarios (load build, upgrade, craft, ask Seer, etc.)
  - Performance benchmarks: XML parse <50ms, calc <100ms, suggestion <200ms
  - Cross-platform build: Windows + macOS + Linux

Week 20: Release Prep
  - Code signing (Windows + macOS)
  - GitHub Actions release pipeline
  - CHANGELOG.md
  - README.md with screenshots
  - v0.1.0 tag → GitHub Release → auto-built executables
```

**Deliverable:** v0.1.0 release on GitHub with downloadable .exe/.dmg/.AppImage.

---

## Post-MVP Phase 1 (Month 6-7): Key Missing Features

```
Week 21-22: In-Game Item Import + Calc Breakdown
  - Parse PoE clipboard format (Ctrl+C → paste → instant analysis)
  - Global hotkey (Ctrl+Shift+V) for background paste
  - Show mod tiers, DPS impact vs current, market value
  - Calculation breakdown panel ("Show The Math")
    → Every DPS step shown with source (which items/passives)
    → Click any line to see what contributes

Week 23-24: Item Editor + Unique Database
  - Searchable unique item database (~1,200 items)
  - Filter by slot, price, build relevance
  - Item crafting simulator (add/remove prefixes/suffixes)
  - Modifier roll selector for uniques (min-max range)
  - Save item templates ("dream ring", "budget helmet")

Week 25-26: Build Sharing + Interactive Tree
  - Build share codes (compact encode/decode)
  - Interactive passive tree editing (click to allocate/deallocate)
  - Shift+click path planning
  - "What if" tree mode (simulate without applying)
```

## Post-MVP Phase 2 (Month 8-9): Community + Advanced

```
Week 27-28: Party Play + Stash Intelligence
  - Party composition analyzer (aura overlap, curse sharing)
  - PoE OAuth for stash tab access
  - Live stash grid view
  - Wealth history tracker

Week 29-30: Transfigured Gems + Toggle System
  - Transfigured gem variant comparison
  - Toggle-able auras/buffs/flasks with real-time DPS update
  - Socketed gem modifier auto-application
  - Map run statistics (Client.txt parser)

Week 31-32: Polish + PoE 2 Prep
  - PoB Lua verification engine (optional feature)
  - Overlay mode (transparent over PoE)
  - PoE 2 data file structure (dual-game support)
  - Community build sharing + rating system
```

## Post-MVP Phase 3 (Month 10+): PoE 2 + Platform

- PoE 2 passive tree + gem system support
- PoE 2 specific UI (gem tree viewer instead of linear links)
- Mobile companion app
- Streaming/OBS integration
- Build guide integration (follow step-by-step in app)

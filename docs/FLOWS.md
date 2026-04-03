# Path of AI — User Flows & Interaction Sequences

Every user action documented step-by-step. Each flow shows:
what the user does, what the system does, what the UI shows, and what happens on error.

## IMPORTANT: Data Import Strategy

**PoE OAuth is the DEFAULT** way to get build data. PoB file import is OPTIONAL.

```
PRIMARY (default):   PoE OAuth → fetch character data directly from GGG servers
OPTIONAL:            PoB file import → parse XML from Path of Building

Why OAuth-first:
  - Not every player uses Path of Building
  - OAuth gets LIVE character data (items, skills, passives, level)
  - OAuth also gets stash tabs, currency, characters
  - No need to install PoB or export XML files
  - Works for PoE 2 players who may never use PoB

Why PoB is still supported:
  - PoB has THEORYCRAFT data (planned builds, not yet equipped)
  - PoB has CONFIG settings (boss type, flask uptime, charge assumptions)
  - PoB users expect to import their builds
  - PoB has the Calcs section (pre-computed stats)
  - Some players prefer offline-only (no OAuth)
```

---

## 1. First Launch Wizard

```
User double-clicks PathOfAI.exe for first time
  ↓
APP: Check if PathOfAI_Data/settings.json exists
  → NO (first launch) → show wizard
  → YES → skip wizard, load normally
  ↓
WIZARD STEP 1: Welcome
  UI shows: "Welcome to Path of AI, Exile."
  UI shows: app logo + version
  UI shows: "Connect your PoE account to get started — or import from Path of Building."
  Button: [Begin Setup]
  ↓
WIZARD STEP 2: Choose Import Method
  UI shows:
  ┌────────────────────────────────────────────────────────┐
  │  How would you like to import your build?              │
  │                                                        │
  │  ◉ Connect PoE Account (recommended)                   │
  │    Live character data directly from GGG servers.       │
  │    Also unlocks: stash tabs, currency, characters.      │
  │    Requires: pathofexile.com login (OAuth).             │
  │                                                        │
  │  ○ Import from Path of Building                        │
  │    Load builds from PoB XML files.                     │
  │    Best for: theorycrafting, planned builds.            │
  │    Requires: PoB installed on this PC.                 │
  │                                                        │
  │  ○ Both (PoE Account + PoB)                            │
  │    Use live character data AND PoB theorycrafts.        │
  │    Best experience — recommended for PoB users.        │
  │                                                        │
  │  ○ Skip — I'll set this up later                       │
  │                                                        │
  │  [Next]                                                │
  └────────────────────────────────────────────────────────┘
  ↓
IF "Connect PoE Account" selected:
  → WIZARD STEP 2A: OAuth Flow
  APP: Open browser to pathofexile.com/oauth/authorize
  USER: Logs in + authorizes "Path of AI"
  APP: Receives OAuth callback → store token in OS keychain
  APP: Fetch account info:
    → Characters: list all characters with class/level/league
    → Stash tabs: count and names
  UI shows: "✅ Connected! Found 3 characters in Mirage league."
  UI shows: Character selector dropdown
  USER: Selects a character (e.g., "ExileRF — Inquisitor Lv.95")
  ↓
  APP: Fetch character data from PoE API:
    → Equipped items (all slots, all mods, sockets)
    → Passive tree (allocated nodes, jewels, masteries)
    → Skills (socketed gems, links, levels)
    → Level, class, ascendancy
  APP: Convert to BuildData struct (same format as PoB parse)
  → Continue to Step 3
  ↓
IF "Import from PoB" selected:
  → WIZARD STEP 2B: Detect PoB
  APP: Scan for PoB install:
    1. %AppData%/Path of Building Community/
    2. %AppData%/Path of Building/
    3. Custom path (user browse)
  UI shows: "Found: PoB Community Fork — 12 build files detected"
  If NOT found: show manual path selector + option to paste PoB code
  USER: Selects a build file
  APP: Parse PoB XML → BuildData
  → Continue to Step 3
  ↓
IF "Both" selected:
  → Do Step 2A (OAuth) first, then Step 2B (PoB detection)
  → User gets both live characters AND PoB theorycrafts in build list
  ↓
WIZARD STEP 3: AI Provider (optional)
  UI shows: "The Seer works locally — no API key needed."
  UI shows: "Optional: connect cloud AI for creative questions."
  Options:
    ◉ The Seer only (recommended, free)
    ○ Add Claude API key
    ○ Skip for now
  Button: [Next]
  ↓
WIZARD STEP 4: Ready
  UI shows: "Path of AI is ready, Exile."
  UI shows: list of characters/builds available
  
  IF OAuth connected:
    Shows: "ExileRF — Inquisitor Lv.95 (live)" ← from PoE account
    Shows: "ExileCold — Occultist Lv.88 (live)" ← from PoE account
  IF PoB connected:
    Shows: "RF Inquisitor (PoB)" ← from PoB file
    Shows: "Cold DOT Theorycraftv2 (PoB)" ← from PoB file
  IF Both:
    Shows all of the above, tagged (live) or (PoB)
  
  Button: [Open first build]
  ↓
APP: Save settings.json with import method + OAuth token + PoB path
APP: Load selected build → normal app flow (Flow #2)
```

---

## 2. Load Build (OAuth Character OR PoB File)

```
User selects a build from the build list
  → Could be: live character (OAuth) or PoB file (XML)
  ↓
UI: Show loading state (spinner in center panel)
  ↓
IF OAuth character:
  BACKEND: invoke('load_character', { characterName })
  RUST: poe_api::fetch_character(token, characterName)
    → GET /character-window/get-items
    → GET /character-window/get-passive-skills
    → Convert API response → BuildData struct
  ↓
IF PoB file:
  BACKEND: invoke('analyze_build', { filePath })
  RUST: pob_parser::parse(xml)
  → OK → BuildData struct (class, level, items, gems, tree, config)
  → ERR → return ParseError to frontend
  ↓
RUST: build_detector::detect(&build_data)
  → archetype: FireDot
  → main_skill: RighteousFire
  → playstyle: mapper_tank
  ↓
RUST: calculator::calculate(&build_data)
  → offense: { total_dps: 2841057, breakdown: [...] }
  → defense: { life: 6453, es: 1820, resists: {...}, ehp: {...} }
  ↓
RUST: build_analyzer::analyze(&build_data, &calc_result)
  → scores: { overall: 74, defense: 82, offense: 66 }
  → issues: [{ severity: warning, issue: "Chaos res 15%", ... }]
  → suggestions: [{ priority: high, slot: "Ring 2", dps_change: +15.3%, ... }]
  ↓
RUST: market::check_prices(&suggestions)  (background, non-blocking)
  → prices: [{ name: "Woe Circle", price: 3.0, ... }]
  ↓
RUST: Store in AppState.current_build
RUST: Return AnalysisResult to frontend
  ↓
FRONTEND: store.setBuildAnalysis(result)
  ↓
UI UPDATE (all at once):
  - Header: show class name + level
  - Left sidebar: life, ES, armour, DPS, resists
  - Center: character viz with equipment + aura rings + buffs
  - Center: score rings (74/82/66)
  - Center: harbinger warnings (top 3)
  - Center: passive tree mini-view
  - Right panel: Prophecy (default) with suggestions
  - Bottom HUD: life/mana orbs + all gem buttons active

ERROR HANDLING:
  ParseError → UI: "Could not read PoB file. Is it valid?"
  FileNotFound → UI: "File not found. Was it moved?"
  PermissionDenied → UI: "Cannot read file. Check permissions."
```

---

## 3. Click Equipment Slot

```
User clicks a character equipment slot (e.g., Ring 2)
  ↓
FRONTEND: showItemDetail('Ring 2')
  ↓
FRONTEND: Find item in BUILD.items where slot === 'Ring 2'
  ↓
FRONTEND: Highlight slot on character body (gold border + glow)
  ↓
RIGHT PANEL: Show PoE-style item tooltip:
  ┌─────────────────────────────────┐
  │  💍 Doom Whorl                  │
  │  Opal Ring                      │
  │  ───────────────────            │
  │  25% inc Ele Dmg (implicit)     │
  │  +45 to maximum Life     T5 ⬤  │
  │  +22% to Fire Res        T4    │
  │  +15% to Lightning Res   T4    │
  │  5% inc maximum Life     T3    │
  │  ───────────────────            │
  │  Open: 1 prefix, 1 suffix      │
  │  ⚠ Very low life roll (45)     │
  │  ⚠ Missing fire DoT multi      │
  │  ───────────────────            │
  │  Score: 42/100                  │
  │  Market: 1.5 divine             │
  │                                 │
  │  [Seek Upgrade →]              │
  └─────────────────────────────────┘

User clicks [Seek Upgrade →]:
  → Right panel switches to Prophecy filtered for this slot
  → Shows multi-path options (craft/trade/benchcraft/divine)
```

---

## 4. Ask The Seer a Question

```
User clicks Grimoire gem button in HUD
  ↓
RIGHT PANEL: Show Grimoire panel
  - Active provider badge (The Seer / Claude / etc.)
  - Quick question buttons
  - Chat history
  - Input field + [Invoke] button
  ↓
User types: "Why am I dying in T16 maps?"
User clicks [Invoke] or presses Enter
  ↓
FRONTEND: invoke('ask_seer', { question, buildId })
  ↓
RUST: seer::classify_intent(question)
  → regex match "dying" + "map" → Intent::DefenseAnalysis
  ↓
RUST: ROUTE → Engine 1 (Calculator) + Engine 2 (Knowledge Base)
  Calculator: check all defense layers
    → chaos_res: 15% (LOW)
    → overcap: cold +1 (LOW)
    → shock_immune: false (VULNERABLE)
    → ehp_vs_t16_rare: 8400 (marginal)
  Knowledge: T16 map monster damage ranges
    → typical hit: 4000-6000 physical + elemental
    → with -max res mod: your max fire res drops → RF degen increases
  ↓
RUST: response_gen::format_defense_analysis(calc_result, kb_data)
  → template: "Your defenses have {count} weaknesses, Exile. {list}."
  → confidence: 95% (Calculator + KB answered fully)
  ↓
FRONTEND: Display response in Grimoire chat area:
  "🔮 The Seer speaks:
   Your defenses have 3 weaknesses, Exile.
   
   1. Chaos Resistance at 15% — Al-Hezmin and chaos-damage rares
      feast upon this frailty. Remedy: +chaos res on Ring 2 or amulet.
   
   2. Shock vulnerable — no anti-shock source detected.
      A single shock increases ALL damage taken by 50%.
      Remedy: Flask suffix 'of Grounding' or Tempest Shield.
   
   3. Low resist overcap — Elemental Weakness curse strips
      your cold res to 52%. Remedy: +24% overcap on all elements.
   
   Your effective HP vs T16 rare hit: 8,400 (marginal).
   With fixes above: ~14,200 (comfortable)."

IF confidence < 70% AND cloud AI configured:
  UI: "The Seer is uncertain. Consult a greater power?"
  [Use Claude] [Use local answer]
  
IF cloud AI selected:
  RUST: cloud_api::query(question, build_context)
  → inject full build context template (from ENGINE-DESIGN.md §4)
  → send to Claude/GPT API
  → display response in chat
```

---

## 5. Apply Upgrade Suggestion

```
User sees Prophecy suggestion: "Replace Ring 2 — worst item (42/100)"
User clicks [Invoke] button on suggestion
  ↓
FRONTEND: invoke('apply_upgrade', { suggestionId, buildId })
  ↓
RUST: VALIDATE
  1. Is current build still the same? (check file mtime)
  2. Does suggestion still apply? (re-run quick calc)
  3. Will this break any resists? (calc new resists)
  4. Will this break stat requirements? (check str/dex/int)
  ↓
  → If validation fails:
    return { error: "Build changed since suggestion. Re-analyze first." }
  → If resist would uncap:
    return { warning: "This change uncaps cold resist (76→72%). Proceed?" }
  ↓
RUST: BACKUP
  1. Copy current PoB XML to PathOfAI_Data/backups/{timestamp}.xml
  2. Push snapshot to undo stack
  ↓
RUST: MODIFY
  1. Clone BuildData
  2. Apply item change (swap Ring 2 with suggested item)
  3. Write modified XML to temp file
  4. Rename temp → original (atomic write)
  ↓
RUST: RE-ANALYZE
  1. Re-parse modified XML
  2. Re-run calculator
  3. Compare old vs new stats
  ↓
RUST: Return UpgradeResult { old_dps, new_dps, old_life, new_life, ... }
  ↓
FRONTEND: Show power-up animation on character
FRONTEND: Update all panels with new stats
FRONTEND: Show diff: "DPS: 2.84M → 3.27M (+15.3%) ✓"
FRONTEND: Show undo button: "Undo (revert to backup)"

USER CLICKS UNDO:
  FRONTEND: invoke('undo_last_change')
  RUST: Pop from undo stack → restore backup XML → re-analyze
```

---

## 6. Paste Item from PoE (Ctrl+C)

```
User is playing PoE, hovers over an item, presses Ctrl+C
  (PoE copies item data to clipboard in its text format)
  ↓
User switches to Path of AI (or app running in background)
User presses Ctrl+Shift+V (global hotkey)
  ↓
APP: Read clipboard text
APP: Detect if it's a PoE item format:
  → Starts with "Rarity: " → YES, parse it
  → Otherwise → ignore (not a PoE item)
  ↓
RUST: item_import::parse_clipboard(text)
  Parse: rarity, name, base type, ilvl, implicits, explicits, sockets
  Detect: mod tiers, open affixes, influence, corrupted status
  ↓
RUST: calculator::score_item(parsed_item, current_build)
  → score: 87/100
  → Compare to currently equipped item in same slot
  → DPS diff: +15.3%, Life diff: +350
  ↓
UI: Show floating modal over app (or tray notification if minimized):
  ┌─────────────────────────────────────────┐
  │  📋 Item Pasted — Opal Ring (ilvl 84)   │
  │                                         │
  │  +92 to maximum Life          T1 ★      │
  │  +18% to Fire DoT Multi      T2 ★      │
  │  +38% to Fire Resistance     T2         │
  │  +28% to Cold Resistance     T3         │
  │                                         │
  │  Score: 87/100                          │
  │                                         │
  │  ═══ vs Your Ring 2 (42/100) ═══       │
  │  DPS:  +15.3% (2.84M → 3.27M)         │
  │  Life: +350 (+92 vs +45)               │
  │  Resists: OK (still capped)            │
  │                                         │
  │  ★ SIGNIFICANT UPGRADE                 │
  │  Market value: ~5 divine               │
  │                                         │
  │  [Equip in PoB] [Dismiss]              │
  └─────────────────────────────────────────┘

User clicks [Equip in PoB]:
  → Follow Flow #5 (Apply upgrade) with this item
```

---

## 7. Open The Forge (Crafting Advisor)

```
User clicks Forge gem button (🔨) in HUD
  ↓
RIGHT PANEL: Show The Forge panel
  ↓
RUST: invoke('get_craft_suggestions', { buildId })
  ↓
RUST: craft_advisor::suggest(build, currency_inventory)
  1. For each slot that needs upgrading:
     a. What base type is ideal?
     b. What mods do we want? (from archetype stat weights)
     c. What crafting methods does player have currency for?
     d. Calculate probability per method
     e. Calculate expected cost per method
     f. Compare to market buy price
  2. Rank by value (DPS gained per divine spent)
  ↓
UI: Show ranked craft suggestions with steps:
  #1 BEST VALUE — Ring 2 via Essence of Anger
    Steps: buy base → essence spam → look for open suffix → benchcraft
    Success rate: 30% per try
    You have: 8 essences (enough)
    Cost: ~2 div vs Buy: 5-8 div
    [Start Crafting →]

User clicks [Start Crafting →]:
  UI: Show step-by-step crafting wizard:
    Step 1: "Buy ilvl 84+ Opal Ring base (1 chaos)"
      [I have the base ✓]
    Step 2: "Use Essence of Anger"
      [Roll!] → simulated result shown
      "Result: +78 life (T2), +fire damage, +22% cold res"
      "Open suffix? YES → Benchcraft fire DoT multi!"
      [Benchcraft →] or [Try again →]
    Running total: "Attempts: 2, Spent: 2 Essence of Anger (~1 div)"
```

---

## 8. Connect PoE OAuth

```
User opens Settings (⚙ gem button)
User scrolls to "Path of Exile Account" section
User clicks [Connect Path of Exile Account]
  ↓
RUST: Open system browser to:
  https://www.pathofexile.com/oauth/authorize?
    client_id=path-of-ai&
    redirect_uri=http://localhost:PORT/callback&
    scope=account:stashes+account:characters&
    response_type=code
  ↓
USER: Logs into pathofexile.com in browser
USER: Clicks "Authorize" for Path of AI
  ↓
BROWSER: Redirects to http://localhost:PORT/callback?code=AUTH_CODE
  ↓
RUST: Local server receives callback
RUST: Exchange auth code for access token
RUST: Store token encrypted in OS keychain (tauri-plugin-stronghold)
RUST: Fetch account info (character names, stash tabs)
  ↓
UI: Update Settings panel:
  "✅ Connected — ExilePlayer#1234"
  "3 characters • 24 stash tabs • Softcore Trade"
  [Disconnect]
  ↓
APP: Now stash tab features work:
  - Stash panel shows live grid
  - "Upgrades in your stash" populates
  - Currency tracking active
  - Div card progress tracks

ERROR HANDLING:
  User cancels OAuth → UI: "Connection cancelled. Try again?"
  Token expired → UI: "Session expired. Reconnect?"
  API rate limited → UI: "PoE API busy. Retrying in 60s..."
```

---

## 9. Connect Cloud AI (Claude)

```
User opens Settings (⚙ gem button)
User scrolls to "AI Provider" section
User clicks "Claude (Anthropic)" radio button
  ↓
UI: Show API key input field
  [API Key: _________ ] [Test Connection]
  [Model: claude-sonnet-4 ▼]
  ↓
User enters API key: sk-ant-...
User clicks [Test Connection]
  ↓
RUST: invoke('test_cloud_ai', { provider: 'claude', apiKey, model })
  → Send minimal test request to Claude API
  → Verify: valid key, model accessible, response received
  ↓
  → SUCCESS:
    RUST: Encrypt API key → store in OS keychain
    RUST: Save provider choice to settings.json
    UI: "✅ Connection successful! Claude Sonnet 4 responding."
    UI: Grimoire panel now shows "Powered by Claude (Anthropic)"
    
  → FAILURE:
    UI: "❌ Invalid API key" or "Model not available" or "Rate limited"
    UI: Keep showing input field for retry

AFTER CONNECTED:
  Grimoire queries that need creative reasoning (3%) → sent to Claude
  All other queries (97%) → still handled by local Calculator + KB
  
  UI shows which engine answered:
    "🔮 The Seer (local)" — for calc/KB queries
    "🔮 Claude (cloud)" — for creative queries
```

---

## 10. Auto-Update Triggers

```
APP STARTUP:
  ↓
RUST: services::update_checker::check()
  → Fetch https://github.com/path-of-ai/releases/latest/manifest.json
  → Compare local data version vs remote version
  ↓
  → SAME VERSION: do nothing
  → NEW VERSION AVAILABLE:
    ↓
    UI: Show subtle notification in header:
      "🔄 Game data update available (patch 3.28.1)"
      [Update Now] [Later]
    ↓
    User clicks [Update Now]:
      ↓
      RUST: Download delta files (only changed JSONs)
      UI: Show progress in header: "Downloading... 45%"
      ↓
      RUST: Verify checksums for each file
        → ALL PASS:
          RUST: Atomic swap (rename new data dir over old)
          RUST: Reload game data (mod_db, gem_db, tree_db)
          UI: "✅ Updated to patch 3.28.1"
          UI: Re-analyze current build with new data
        → CHECKSUM FAIL:
          RUST: Keep old data, delete corrupt download
          UI: "❌ Update failed (corrupt download). Will retry later."
    ↓
    User clicks [Later]:
      APP: Check again in 6 hours
      UI: Dismiss notification

LEAGUE LAUNCH WEEK:
  APP: Check every 30 minutes instead of every 6 hours
  UI: "🔥 New league data downloading..."
```

---

## 11. PoB File Changes Externally

```
User modifies build in PoB (adds item, changes gem, respec tree)
User clicks Save in PoB
  ↓
RUST: services::file_watcher detects file change (notify crate)
  → Start 500ms debounce timer
  → If more changes within 500ms: reset timer
  → After 500ms of no changes: proceed
  ↓
RUST: Check file is readable (not locked by PoB)
  → LOCKED: wait 200ms, retry up to 3 times
  → STILL LOCKED: skip this change, wait for next
  ↓
RUST: Re-parse XML → new BuildData
RUST: Re-run calculator → new CalcResult
RUST: Compare old vs new: what changed?
  → items_changed: ["Ring 2"]
  → gems_changed: ["Burning Damage level 20→21"]
  → tree_changed: ["+2 nodes near Marauder"]
  ↓
RUST: Emit event to frontend: 'build-changed'
  payload: { changes: [...], new_analysis: {...} }
  ↓
FRONTEND: Listen for 'build-changed' event
FRONTEND: store.setBuildAnalysis(newAnalysis)
FRONTEND: Update ALL panels
FRONTEND: Show change notification:
  "PoB updated: Ring 2 changed, +2 passive points allocated"
  "DPS: 2.84M → 3.12M (+9.9%)"

ERROR HANDLING:
  Corrupt XML → UI: "PoB file appears corrupt. Waiting for valid save..."
  File deleted → UI: "Build file was deleted. Load a different build?"
```

---

## 12. Open Passive Tree Viewer

```
User clicks passive tree mini-view in center panel
  (or clicks Passive Tree gem button in HUD)
  ↓
RIGHT PANEL: Switch to Passive Tree panel
  ↓
RUST: invoke('get_tree_analysis', { buildId })
  ↓
RUST: Load passive tree position data (game-data/tree/passive-tree-positions.json)
RUST: Load current allocated nodes from build
RUST: For each unallocated node within reach:
  → Simulate allocation → recalc → diff vs current
  → Score: ΔLife, ΔDPS, ΔResists
RUST: Find inefficient allocated nodes:
  → Simulate deallocation → recalc → if loss < threshold → mark inefficient
  ↓
RIGHT PANEL: Show analysis:
  - Points: 121/123 (2 unallocated)
  - Keystones active: list
  - Next best 5 points (ranked by impact)
  - Inefficient nodes to respec
  - Jewel socket status
  - Anointment suggestion

CENTER PANEL: Passive tree mini-view updates:
  - Allocated nodes glow gold
  - Recommended nodes glow green
  - Inefficient nodes glow red
  - Click node → show exact impact in right panel

FULL TREE VIEW (future — post-MVP):
  - Click "Open Full Tree" → modal/overlay with zoomable SVG
  - All ~1300 nodes rendered with positions from GGG data
  - Pan + zoom (mouse drag + wheel)
  - Click node → "What if I take this?" → instant recalc
  - Shift+click → plan path (sequence of nodes)
  - Compare button → overlay top poe.ninja builds
```

---

## 13. Share Build

```
User clicks "Share" button (in header or settings)
  ↓
UI: Show share modal:
  ┌────────────────────────────────────────┐
  │  Share Your Build                       │
  │                                         │
  │  Format:                               │
  │    ◉ Share code (compact text)         │
  │    ○ URL (opens in browser)            │
  │    ○ QR code (for mobile)              │
  │                                         │
  │  Include:                              │
  │    ☑ Full build (tree + items + gems)  │
  │    ☐ Tree only                         │
  │    ☐ Items only                        │
  │    ☐ Gems only                         │
  │                                         │
  │  Code: poa_v1_eJzTSS7ILEpVSMsvyk...   │
  │                                         │
  │  [Copy Code] [Copy URL] [Show QR]      │
  └────────────────────────────────────────┘

SHARE CODE FORMAT:
  prefix: "poa_v1_"
  body: base64(zlib_compress(json({
    version: "1.0",
    patch: "3.28",
    class: "Templar",
    ascendancy: "Inquisitor",
    level: 95,
    tree_nodes: [node_ids...],
    items: [{ slot, name, base, mods... }],
    gems: [{ skill, gems... }],
  })))
  
RECIPIENT IMPORTS:
  Recipient pastes code into Path of AI
  → APP: Decode → decompress → parse JSON → create local build
  → UI: "Imported: RF Inquisitor Lv.95 by ExilePlayer"
```

---

## 14. Switch Game Version (PoE 1 → PoE 2)

```
User opens Settings → Game Version
  ↓
UI: Show version selector:
  ◉ Path of Exile 1 (current)
  ○ Path of Exile 2
  ↓
User selects "Path of Exile 2"
  ↓
UI: Show confirmation:
  "Switching to PoE 2 mode. This will:
   - Load PoE 2 passive tree + gem system
   - Load PoE 2 mod database + base types
   - Change gem viewer from linear links → gem tree
   - Your PoE 1 builds remain saved
   
   [Switch to PoE 2] [Cancel]"
  ↓
RUST: Reload all game data from game-data/poe2/ instead of game-data/poe1/
  → mod_database: reload
  → gem_database: reload
  → tree_database: reload (PoE 2 tree layout)
  ↓
UI: Update all components:
  - Gem viewer: switch from linear socket links → gem hierarchy tree
  - Passive tree: load PoE 2 layout
  - Calculator: use PoE 2 formulas (dodge instead of spell block, Spirit instead of mana reservation)
  - Current build: prompt to load a PoE 2 build file
  ↓
UI: "Now in PoE 2 mode. Load a PoE 2 build to begin."
```

---

## 15. Open Stash Tab (with PoE OAuth)

```
User clicks Stash gem button (📦) in HUD
  (requires PoE OAuth connected — see Flow #8)
  ↓
IF NOT connected:
  RIGHT PANEL: Show "Connect PoE Account" prompt → Flow #8
  ↓
IF connected:
  RUST: invoke('fetch_stash_tabs')
  → Fetch stash tab list from PoE API
  → Cache locally (refresh every 5 min)
  ↓
RIGHT PANEL: Show Stash panel
  - Tab selector (Currency, Gear, Dump, Fragments, Maps)
  - 12×12 grid view with items placed by position/size
  - Currency totals: "47.3 divine equivalent"
  - Free upgrades found in stash
  - Sellable items with market prices
  - Div card progress bars
  - Splinter/fragment tracking
  ↓
User clicks a stash tab:
  RUST: invoke('fetch_stash_items', { tabId })
  → Fetch items in that tab from PoE API
  → Parse each item: mod tiers, score, value
  ↓
  UI: Render 12×12 grid with items
  UI: Highlight upgrades (green dot on items better than equipped)
  UI: Show sellable items with prices
  ↓
User clicks an item in stash grid:
  → Show item tooltip (same as Flow #3)
  → Compare to currently equipped item
  → [Equip] button to swap (writes to PoB XML)

ERROR HANDLING:
  PoE API rate limited → "Rate limited. Retrying in 60s..."
  OAuth token expired → "Session expired. Reconnect in Settings."
  Stash empty → "No items found in this tab."
```

---

## 16. Compare Two Builds

```
User has 2+ builds loaded (or imports from poe.ninja)
User opens Build Comparison:
  → From Settings → "Compare Builds"
  → Or from HUD: long-press any gem button → "Compare"
  ↓
UI: Build selector:
  "Select Build A: [Current: RF Inquisitor ▼]"
  "Select Build B: [poe.ninja Top #1 RF ▼]"
  [Compare]
  ↓
RUST: invoke('compare_builds', { buildIdA, buildIdB })
  → Calculate both builds
  → Diff every stat: DPS, life, resists, items, tree
  ↓
RIGHT PANEL: Show comparison:
  ┌──────────────────────────────────────┐
  │  Build A          │  Build B          │
  │  Your Build       │  Top #1 RF        │
  │  2.84M DPS        │  4.20M DPS        │
  │  6,453 Life       │  7,200 Life        │
  │  Score: 74        │  Score: 91         │
  │────────────────────────────────────────│
  │  Key Differences:                      │
  │  • They use Aegis Aurora (you: Rise)   │
  │  • They have 21/23 all gems (you: 20/20)│
  │  • They have +1 fire gems amulet       │
  │  • Tree overlap: 73%                   │
  │  • Missing nodes: Sovereignty wheel    │
  │────────────────────────────────────────│
  │  Cost to match: ~45 divine             │
  │  [Show upgrade path to match Build B]  │
  └────────────────────────────────────────┘
```

---

## 17. Set Price Alert

```
User is looking at a suggested upgrade (e.g., Aegis Aurora)
User clicks "Set Alert" on the item card
  ↓
UI: Show alert setup modal:
  "Alert when Aegis Aurora drops below [15] divine"
  Notify: [Popup ▼] / Sound / Silent
  [Set Alert]
  ↓
RUST: invoke('set_price_alert', { itemName, threshold, comparison, notifyMethod })
  → Save to alerts table in SQLite
  → Background price poller now checks this item
  ↓
UI: "✅ Alert set. You'll be notified when Aegis Aurora < 15 div."
  ↓
BACKGROUND (every 5 min):
  RUST: services::price_poller checks all active alerts
  → Fetch latest price from poe.ninja
  → Compare to threshold
  ↓
  IF TRIGGERED:
    RUST: emit('price-alert-triggered', { alertId, itemName, currentPrice, threshold })
    FRONTEND: Show notification:
      "🔔 Aegis Aurora dropped to 14.2 divine! (your alert: < 15)"
      [Buy Now →] [Dismiss] [Snooze 1hr]
    IF sound enabled: play PoE currency drop sound
```

---

## 18. Report Bug (Whisper to the Void)

```
User clicks "Report Bug" in Settings
  (or encounters an error and clicks "Report")
  ↓
UI: Show feedback panel:
  ┌─────────────────────────────────────────────┐
  │  ☠ Whisper to the Void — Report an Issue    │
  │                                              │
  │  Type: [Bug ▼]  [Feature]  [Feedback]       │
  │                                              │
  │  Title: [________________________]           │
  │                                              │
  │  Description:                                │
  │  ┌──────────────────────────────────┐        │
  │  │                                  │        │
  │  └──────────────────────────────────┘        │
  │                                              │
  │  Attachments:                                │
  │  ☑ App logs (last 100 lines)                │
  │  ☑ Build summary (anonymized)               │
  │  ☐ Screenshot                                │
  │  ☐ Full build file                           │
  │                                              │
  │  Preview: [See what will be sent]            │
  │                                              │
  │  [Invoke — Send to the Void]                │
  └─────────────────────────────────────────────┘
  ↓
User fills in details, clicks [Invoke]
  ↓
RUST: Format as GitHub Issue markdown
  → Title: "[Bug] User's title"
  → Body: description + system info + build summary + app version
  → Labels: auto-tagged (bug/feature/feedback)
  → POST to GitHub Issues API (using app's token, not user's)
  ↓
UI: "Your whisper has been heard, Exile. Tracking: github.com/path-of-ai/issues/42"
```

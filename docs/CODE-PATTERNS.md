# Path of AI — Code Patterns & Style Guide

## Overview

This document defines the code patterns, architecture conventions, and clean code
standards for the Path of AI project. All contributors and AI coding assistants
should follow these patterns.

**Tech stack:** Tauri 2 (Rust backend + TypeScript/HTML/CSS frontend)

For the full architecture, design patterns, project structure, and Rust code examples, see:
**[ARCHITECTURE.md](ARCHITECTURE.md)**

This document covers **code-level** conventions: naming, functions, data flow, testing, UI theme.

---

## 1. PROJECT STRUCTURE

The project is a Tauri app with two sides:

```
src-tauri/src/              # Rust backend — business logic, file I/O, AI, APIs
  commands/                 # Tauri IPC handlers (thin wrappers)
  core/                     # Pure business logic (no Tauri dependency)
  calculator/               # OUR Rust calc engine (primary — we own this)
  pob_verify/               # PoB Lua verification (optional, feature-gated)
  models/                   # Shared data types (Serialize + Deserialize)
  data/                     # Game data loading (versioned JSON from poedb/RePoE)
  market/                   # poe.ninja client, price cache, buy advisor
  seer/                     # Query router + response generator (no ML models)
  services/                 # Background tasks (file watcher, price poller)

src/                        # Frontend (Vanilla TypeScript — NO framework)
  components/               # UI components (plain TS classes, not React/Vue)
  services/                 # Tauri invoke wrappers, state store, events
  styles/                   # PoE-exact themed CSS (from horadric-helper palette)
  types/                    # TypeScript types matching Rust models

prototypes/                 # JS prototypes (historical — NOT for implementation, see prototypes/README.md)
ui/                         # HTML/JSX prototypes (used during initial development)
```

### Key Principle: Separation of Concerns

```
src-tauri/src/core/     = LOGIC      — pure Rust, no Tauri deps, no side effects
src-tauri/src/commands/ = GLUE       — thin IPC handlers, delegates to core/
src/                    = DISPLAY    — renders data from backend via invoke()
```

- **core/** is pure business logic. Testable without Tauri. Reusable in CLI.
- **commands/** are thin handlers — parse input, call core, return result.
- **Frontend** only renders data it receives. No business logic in the UI.

The original `core/` and `ui/` directories contain **JavaScript prototypes**
created during initial design. These will be ported to Rust and TypeScript
as the Tauri app is built.

---

## 2. MODULE PATTERN

### Hexagonal Architecture (Ports & Adapters)

See [ARCHITECTURE.md §4.1](ARCHITECTURE.md) for the full hexagonal diagram.
Every module follows this structure:

```rust
/// ModuleName — one-line description
///
/// This is a DOMAIN module — no external dependencies.
/// Uses traits (ports) for anything external.
pub struct BuildAnalyzer<P: PriceProvider, D: ModDatabase> {
    prices: P,      // Injected via trait — can be real API or mock
    mods: D,        // Injected via trait — can be JSON loader or mock
}

impl<P: PriceProvider, D: ModDatabase> BuildAnalyzer<P, D> {
    pub fn new(prices: P, mods: D) -> Self {
        Self { prices, mods }
    }

    /// Public method — describe what it does and returns
    pub async fn analyze(&self, build: &BuildData) -> Result<AnalysisResult, AnalyzeError> {
        // Domain logic here — NEVER calls tauri, reqwest, fs directly
        let score = self.score_items(build)?;
        let issues = self.detect_issues(build)?;
        Ok(AnalysisResult { score, issues })
    }

    /// Private helper — not pub
    fn score_items(&self, build: &BuildData) -> Result<u8, AnalyzeError> {
        // Internal logic
    }
}
```

### Rules:

1. **One struct per file** — file name matches struct in snake_case
   - `BuildAnalyzer` → `build_analyzer.rs`
   - `ModImpactCalculator` → `mod_impact.rs`

2. **Dependencies via trait generics** — never import concrete implementations
   ```rust
   // GOOD: dependency injection via traits
   pub struct BuildAnalyzer<P: PriceProvider> {
       prices: P,  // can be real or mock
   }

   // BAD: importing concrete type
   class BuildAnalyzer {
     constructor() {
       this.build = globalBuildState.current; // NO
     }
   }
   ```

3. **Default export** — one class/function per module, exported as default
   ```javascript
   export default BuildAnalyzer;
   ```

4. **No circular dependencies** — if A imports B, B must not import A

---

## 3. NAMING CONVENTIONS

### Files
```
core/build-analyzer.js       # kebab-case for files
core/mod-impact-calculator.js # descriptive, not abbreviated
```

### Classes
```javascript
class BuildAnalyzer {}        // PascalCase
class ModImpactCalculator {}  // descriptive noun phrases
class ItemImageResolver {}
```

### Methods
```javascript
analyzeDefenses()             // camelCase, verb + noun
detectBuildType()             // verb describes the action
calculatePhysReduction()      // calculate/detect/generate/parse/format
scoreItem()                   // short verbs for simple operations
```

### Variables
```javascript
const totalDPS = 2841057;     // camelCase
const fireRes = stats.FireResist; // abbreviations OK if obvious
const ehpPhys = life / (1 - reduction); // domain abbreviations OK (EHP, DPS, DoT)
```

### Constants (game data)
```javascript
const GEM_DATABASE = { ... };     // UPPER_SNAKE_CASE for static lookups
const ARCHETYPE_RULES = { ... };  // These are reference data, not mutable state
const MOCK_BUILD = { ... };       // Test data constants
```

### Boolean variables
```javascript
const isCapped = value >= 75;     // is/has/can prefix
const hasLife = item.mods.some(...);
const canCraft = prefixes < 3;
```

---

## 4. FUNCTION PATTERNS

### Pure Functions Preferred
```javascript
// GOOD: pure function — same input always gives same output
calculatePhysReduction(armour, damage) {
  if (armour <= 0) return 0;
  return Math.min(90, (armour / (armour + 5 * damage)) * 100);
}

// BAD: depends on external state
calculatePhysReduction(damage) {
  return Math.min(90, (this.currentArmour / (this.currentArmour + 5 * damage)) * 100);
}
```

### Return Structured Objects
```javascript
// GOOD: return a descriptive object
chaosResTier(chaosRes) {
  if (chaosRes >= 75) return { tier: "capped", label: "Excellent", color: "green" };
  if (chaosRes >= 50) return { tier: "good", label: "Good", color: "blue" };
  // ...
}

// BAD: return magic strings
chaosResTier(chaosRes) {
  if (chaosRes >= 75) return "green";  // what does "green" mean?
}
```

### Tiered Scoring Pattern
Many PoE values need tiered evaluation. Use a consistent pattern:

```javascript
// Pattern: descending threshold checks, return structured result
dpsTier(dps) {
  if (dps >= 10_000_000) return { tier: "S", label: "God-tier", color: "purple" };
  if (dps >= 5_000_000)  return { tier: "A", label: "Excellent", color: "green" };
  if (dps >= 2_000_000)  return { tier: "B", label: "Good", color: "blue" };
  if (dps >= 1_000_000)  return { tier: "C", label: "Average", color: "yellow" };
  if (dps >= 500_000)    return { tier: "D", label: "Low", color: "orange" };
  return { tier: "F", label: "Needs work", color: "red" };
}
```

### Analysis Pattern
Analyzers follow: collect → process → score → return

```javascript
analyzeDefenses() {
  // 1. Collect raw stats
  const life = stats.Life || 0;
  const armour = stats.Armour || 0;

  // 2. Process / calculate derived values
  const physReduction = this.calculatePhysReduction(armour, 5000);
  const ehpPhys = life / (1 - physReduction / 100);

  // 3. Score
  const score = this.scoreDefenses(life, resists, armour, block);

  // 4. Return structured result
  return { life, armour, physReduction, ehpPhysical: Math.round(ehpPhys), score };
}
```

---

## 5. DATA FLOW PATTERN

### Input → Parse → Analyze → Present

```
PoB XML File
    ↓
pob-parser.js         → { build, items, skills, tree, config }
    ↓
build-detector.js     → { mainSkill, archetype, dpsType, playstyle }
    ↓
build-analyzer.js     → { scores, defenses, offense, issues, suggestions }
    ↓
mod-impact-calculator.js → { per-item DPS/life impact numbers }
    ↓
market-intelligence.js → { prices, upgrade paths, buy timing }
    ↓
UI layer              → renders all of the above
```

### Data Shape Convention

All parsed PoB data follows this shape:
```javascript
{
  build: {
    level: Number,
    className: String,
    ascendClassName: String,
    mainSocketGroup: Number,
    stats: { Life: Number, FireResist: Number, TotalDPS: Number, ... }
  },
  items: [{ id, slot, name, base, rarity, mods: [...], explicits: [...] }],
  itemSets: [{ id, slots: { "Helmet": itemId, ... } }],
  skills: [{ skills: [{ slot, label, gems: [{ gemId, level, quality, enabled }] }] }],
  tree: { nodes: [...], keystones: [...] },
  config: { ... }
}
```

Every downstream module expects this shape. Do not change it without updating all consumers.

---

## 6. ERROR HANDLING

### Defensive Defaults
```javascript
// GOOD: default to safe values, never crash
const life = stats.Life || 0;
const resists = stats.FireResist || 0;
const items = buildData.items || [];

// BAD: assume data exists
const life = stats.Life;  // crashes if stats is undefined
```

### Guard Clauses
```javascript
// GOOD: early return for invalid input
analyzeItems() {
  const activeSet = this.itemSets?.[0];
  if (!activeSet) return [];
  // ... rest of logic
}

// BAD: deep nesting
analyzeItems() {
  if (this.itemSets) {
    if (this.itemSets[0]) {
      // ... deeply nested logic
    }
  }
}
```

### No Try-Catch for Flow Control
```javascript
// GOOD: check before acting
if (item.mods && item.mods.length > 0) {
  // process mods
}

// BAD: use exceptions for expected cases
try {
  item.mods.forEach(...)
} catch (e) {
  // mods was undefined
}
```

---

## 7. GAME DATA PATTERNS

### Static Lookup Tables
Game data that doesn't change during runtime uses constant objects:

```javascript
const GEM_DATABASE = {
  RighteousFire: {
    name: "Righteous Fire",
    tags: ["fire", "spell", "aoe", "dot"],
    damageType: "fire_dot",
    element: "fire",
    mechanic: "self-cast",
  },
  // ...
};
```

### Weight Systems
Build-type-specific stat weights use a factory pattern:

```javascript
getStatWeights() {
  const buildType = this.detectBuildType();
  const WEIGHTS = {
    fire_dot: { life: 1.2, dotMulti: 15, fireRes: 0.3, ... },
    cold_dot: { life: 1.0, coldDotMulti: 15, coldRes: 0.3, ... },
    attack:   { life: 1.0, flatPhys: 10, attackSpeed: 8, ... },
    default:  { life: 1.0, fireRes: 0.5, coldRes: 0.5, ... },
  };
  return WEIGHTS[buildType] || WEIGHTS.default;
}
```

### Mod Detection
Use lowercase string matching with `.includes()` for mod text:

```javascript
// GOOD: resilient to minor text variations
const hasLife = item.mods.some(m => m.raw?.toLowerCase().includes("life"));
const hasMS = item.mods.some(m => m.raw?.toLowerCase().includes("movement speed"));

// BAD: exact match breaks easily
const hasLife = item.mods.some(m => m.raw === "+94 to maximum Life");
```

---

## 8. UI PATTERNS

### Data-Driven Rendering
UI receives data objects and renders them. No business logic in UI:

```javascript
// GOOD: UI just renders what it's given
function renderIssue(issue) {
  return `<div class="issue-card ${issue.severity}">
    <div class="issue-title">${issue.issue}</div>
    <div class="issue-fix">Fix: ${issue.fix}</div>
  </div>`;
}

// BAD: UI calculates business logic
function renderResists(stats) {
  const isCapped = stats.FireResist >= 75; // this belongs in core/
}
```

### Color Convention (PoE-Exact from horadric-helper)

See [DESIGN-DECISIONS.md](DESIGN-DECISIONS.md) for full palette with rationale.

```
Backgrounds (warm brown-black, NOT purple):
  bg-dark:   #0c0b0a     bg-panel:  #171411
  bg-card:   #1e1b16     bg-hover:  #282420
  border:    #3c3630     border-gold: #8c7a30

Text (high contrast):
  text:       #d4cfc4    text-bright: #f0ece4
  text-value: #ffffff    text-muted:  #7f7f7f

Damage types (PoE exact):
  Fire: #960000    Cold: #366492    Lightning: gold    Chaos: #d02090

Status:
  Success: #4ae63a (quest green)    Danger: #d20000 (corrupted)
  Warning: gold (lightning)         Info: #88f (magic blue)

Rarity (PoE exact):
  Normal: #c8c8c8  Magic: #88f  Rare: #ff7  Unique: #af6025

Tier colors:
  T1: #e8d44d (gold)    T2: #4ae63a (quest green)    T3: #366492 (cold blue)
  T4: #7f7f7f (grey)    T5: #d20000 (corrupted red)

Special:
  Corrupted: #d20000    Crafted: #b8daf2 (light blue)
  Fractured: #a29162    Augmented: #88f (blue for modified stats)
```

IMPORTANT: Never use Tailwind colors (#ef4444, #eab308, #a3e635, #3b82f6, etc.).
Chaos is MAGENTA-PINK (#d02090), NOT purple.

### Font Convention
```
Headings/titles: 'Cinzel', serif (PoE-like decorative)
Body text:       'Lora', Georgia, serif (readable)
Numbers/data:    'JetBrains Mono', monospace (aligned, precise)
Labels:          'JetBrains Mono', monospace (uppercase, spaced)
```

---

## 9. PERFORMANCE PATTERNS

### Targets
```
XML parsing:          < 50ms
Fast DPS estimation:  < 50ms
Build analysis:       < 200ms
File watch latency:   < 100ms
UI render:            < 16ms (60fps)
```

### Lazy Computation
```javascript
// GOOD: compute once, cache result
const sortedItems = useMemo(
  () => [...buildData.items].sort((a, b) => a.score - b.score),
  [buildData.items]
);

// In vanilla JS, use getter with cache
get sortedItems() {
  if (!this._sortedItems) {
    this._sortedItems = [...this.items].sort((a, b) => a.score - b.score);
  }
  return this._sortedItems;
}
```

### Avoid Redundant Work
```javascript
// GOOD: detect build type once, reuse
const buildType = this.detectBuildType();
const weights = WEIGHTS[buildType];

// BAD: detect build type every time
for (const item of items) {
  const weights = WEIGHTS[this.detectBuildType()]; // re-detects every iteration
}
```

---

## 10. TESTING PATTERNS

### Test Data
Use `test-data/` directory for sample PoB XML files:
```
test-data/
  SampleRFInquisitor.xml    # RF Inquisitor build
  SampleColdDOT.xml         # Cold DoT Occultist (future)
  SampleAttack.xml          # Attack build (future)
  SampleMinion.xml          # Minion build (future)
```

### Test Structure
```javascript
// test pattern: describe what, given what input, expect what output
describe('BuildAnalyzer', () => {
  describe('chaosResTier', () => {
    it('returns capped for 75+', () => {
      expect(analyzer.chaosResTier(75).tier).toBe('capped');
    });
    it('returns negative for below -30', () => {
      expect(analyzer.chaosResTier(-31).tier).toBe('negative');
    });
  });
});
```

---

## 11. DOCUMENTATION PATTERN

### Code Comments
- **Do not** add comments for self-evident code
- **Do** add comments for PoE-specific formulas and game mechanics
- **Do** add comments for non-obvious business logic

```javascript
// GOOD: explains a PoE-specific formula
// PoE armour formula: reduction = Armour / (Armour + 5 * Damage)
// Source: https://www.poewiki.net/wiki/Armour
calculatePhysReduction(armour, damage) {
  return Math.min(90, (armour / (armour + 5 * damage)) * 100);
}

// BAD: states the obvious
// Returns the fire resistance
getFireRes() {
  return this.stats.FireResist;
}
```

### JSDoc for Public Methods
```javascript
/**
 * Score an item on 0-100 scale based on build relevance.
 * @param {Object} item - Parsed item with mods array
 * @param {string} slot - Equipment slot name
 * @returns {number} Score 0-100
 */
scoreItem(item, slot) { ... }
```

---

## 12. VERSION & COMPATIBILITY

### PoE Version Awareness
The app must handle multiple PoE versions. All game data should be versioned:

```javascript
// Game data is loaded by version, not hardcoded
const modDatabase = loadModDatabase(poeVersion); // "3.24", "3.25", etc.
const passiveTree = loadPassiveTree(poeVersion);
```

### Auto-Update Pattern
See [AUTO-UPDATE-SYSTEM.md](AUTO-UPDATE-SYSTEM.md) for the full auto-update architecture
that keeps game data current across league launches.

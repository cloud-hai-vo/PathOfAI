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
  models/                   # Shared data types (Serialize + Deserialize)
  data/                     # Game data loading (versioned JSON)
  market/                   # poe.ninja client, price cache, buy advisor
  seer/                     # Local AI engine (ONNX models + RAG)
  services/                 # Background tasks (file watcher, price poller)

src/                        # TypeScript frontend — UI rendering
  components/               # UI components
  services/                 # Tauri invoke wrappers, state store, events
  styles/                   # PoE-themed CSS
  types/                    # TypeScript types matching Rust models

core/                       # JS prototypes (used during initial development)
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

### Every core module follows this structure:

```javascript
/**
 * ModuleName — one-line description of what it does
 * 
 * Responsibilities:
 *   - Responsibility 1
 *   - Responsibility 2
 * 
 * Dependencies: list other core modules it imports
 * Side effects: none (or list them if unavoidable)
 */

class ModuleName {
  constructor(dependencies) {
    // Store injected dependencies
    // Initialize internal state
  }

  /** Public method — describe what it does and returns */
  publicMethod(input) {
    // Implementation
    return result;
  }

  // Private helpers prefixed with underscore
  _privateHelper() {
    // Internal logic
  }
}

export default ModuleName;
```

### Rules:

1. **One class per file** — file name matches the class in kebab-case
   - `BuildAnalyzer` → `build-analyzer.js`
   - `ModImpactCalculator` → `mod-impact-calculator.js`

2. **Constructor takes dependencies** — never import globals or singletons
   ```javascript
   // GOOD: dependency injection
   class BuildAnalyzer {
     constructor(buildData) {
       this.build = buildData.build;
       this.items = buildData.items;
     }
   }

   // BAD: importing global state
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

### Color Convention (PoE Theme)
```
Background:    #0d0a07 (dark)
Panel:         #1a1410 (slightly lighter)
Border:        #3d2e1f (brown border)
Text primary:  #c4b5a0 (parchment)
Text heading:  #e8dcc8 (bright parchment)
Text muted:    #6b5a45 (dim)
Text dimmer:   #4a3d2e (very dim)
Accent gold:   #8b6914
Success:       #a3e635 (green)
Warning:       #eab308 (yellow)
Danger:        #ef4444 (red)
Info:          #3b82f6 (blue)
Purple:        #a855f7

Tier colors:
  T1: #ffd700 (gold)
  T2: #22c55e (green)
  T3: #4a9eff (blue)
  T4: #9ca3af (gray)
  T5: #ef4444 (red)

Rarity colors (matching PoE):
  Normal:  #c8c8c8
  Magic:   #8888ff
  Rare:    #ffff77
  Unique:  #af6025
```

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

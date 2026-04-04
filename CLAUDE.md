# Path of AI — Claude Skills File

> This file is automatically read by Claude Code at the start of every session.
> It gives Claude the project context needed to write correct, consistent code.

---

## What This Project Is

**Path of AI** is a Tauri 2 desktop app for Path of Exile build analysis.
- **Backend:** Rust (calculator, parser, seer, market, OAuth)
- **Frontend:** Vanilla TypeScript + HTML/CSS (no React, no Vue, no framework)
- **IPC:** Tauri `invoke()` / `emit()` — frontend calls Rust functions

The UI prototype lives in `ui/path-of-ai-game.html` — use it as the design reference for all UI work.
JS prototypes in `prototypes/` are reference implementations to port to Rust — do not extend them.

---

## Current State

```
EXISTS (design + docs):
  ui/path-of-ai-game.html        ← complete UI prototype (HTML/JS/CSS)
  ui/assets/                     ← logo.svg, icon.svg, favicon.svg
  prototypes/*.js                ← JS reference implementations
  test-data/SampleRFInquisitor.xml ← test PoB build file
  docs/                          ← full documentation (see Docs Map below)

DOES NOT EXIST YET (needs to be built):
  src-tauri/                     ← Rust backend (scaffold with `npm create tauri@latest`)
  src/                           ← TypeScript frontend (replaces ui/ prototype)
```

---

## Docs Map

Always read the relevant doc before writing code in that area.

| Area | Doc |
|------|-----|
| Features (what to build) | `docs/FEATURES.md` |
| Architecture + patterns | `docs/ARCHITECTURE.md` |
| All 54 algorithms | `docs/ALGORITHMS.md` |
| Tauri IPC commands | `docs/IPC-SPEC.md` |
| User flows (step-by-step) | `docs/FLOWS.md` |
| SQLite schema | `docs/DATABASE.md` |
| Code style + conventions | `docs/CODE-PATTERNS.md` |
| Engine design | `docs/ENGINE-DESIGN.md` |
| Config schema | `docs/CONFIG-SCHEMA.md` |
| Data sources | `docs/DATA-SOURCES.md` |
| Build plan (milestones) | `docs/IMPLEMENTATION-PLAN.md` |
| Vibe-coding sessions | `docs/VIBE-PLAN.md` |

---

## Project Structure (target layout — build toward this)

```
src-tauri/src/
  commands/          # Tauri IPC handlers — THIN wrappers only
  core/              # Pure business logic — no Tauri deps, fully testable
    pob_parser.rs    # Port of prototypes/pob-parser.js
    build_analyzer.rs
    build_detector.rs
    combat_sim.rs
    gem_optimizer.rs
    map_mod_analyzer.rs
  calculator/        # Our Rust damage/defense calculator
    offense_calc.rs
    defense_calc.rs
    formulas.rs
    what_if.rs
  models/            # Shared structs (Serialize + Deserialize)
    build.rs         # BuildData, Item, Gem, PassiveTree, etc.
    analysis.rs      # AnalysisResult, Issue, Suggestion, etc.
  data/              # Game data loader (versioned JSON)
  market/            # poe.ninja client, price cache
  seer/              # Query router + response generator
  services/          # Background tasks (file watcher, price poller)
  db/                # SQLite operations (rusqlite)
  lib.rs             # Tauri setup + AppState

src/                 # Vanilla TypeScript frontend
  components/        # UI components (plain TS, no framework)
  services/          # invoke() wrappers, state store
  types/             # TS types matching Rust models
  styles/            # CSS (PoE dark theme — copy from ui/styles/)

docs/
prototypes/          # Historical JS — read-only reference
ui/                  # HTML prototype — design reference only
test-data/           # PoB XML test builds
```

---

## Code Conventions

### Rust
- `commands/` handlers must be **thin**: validate input → call `core/` or `calculator/` → return result. No business logic.
- `core/` must be **pure**: no `tokio`, no Tauri deps, no network. Testable with `cargo test`.
- All IPC return types must implement `Serialize`. Errors return `Result<T, String>`.
- Use `thiserror` for domain errors, `anyhow` for application errors.
- Prefer `&str` over `String` in function params where possible.
- Algorithm reference: when implementing a function, read the corresponding algorithm in `docs/ALGORITHMS.md` first.

```rust
// commands/ pattern (thin handler)
#[tauri::command]
pub async fn analyze_build(
    file_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    let build = core::pob_parser::parse_file(&file_path)
        .map_err(|e| e.to_string())?;
    let result = core::build_analyzer::analyze(&build, &state.game_data);
    Ok(result)
}

// core/ pattern (pure logic)
pub fn analyze(build: &BuildData, data: &GameData) -> AnalysisResult {
    // ... no Tauri, no tokio, no network
}
```

### TypeScript
- **No framework** — plain TS classes and functions.
- All backend calls go through `src/services/bridge.ts` (typed invoke wrappers).
- UI state lives in `src/services/store.ts` — a simple observable store.
- Never put business logic in the frontend — only rendering and event handling.
- Types in `src/types/` must match Rust `models/` structs exactly.

### CSS / UI Theme
- Use CSS variables from `ui/styles/` — do not hardcode colors.
- Key colors: `--life` (#e05050), `--mana` (#4a8fc7), `--es` (#7fbfbf), `--fire` (#cf3a1f), `--gold` (#c4a830), `--chaos` (#d020a0), `--success` (#4ae63a), `--danger` (#e04040).
- Font: `JetBrains Mono` for numbers/stats, `Cinzel` for PoE-flavored headings.
- All UI panels follow the dark PoE aesthetic — see `ui/path-of-ai-game.html`.

---

## Key Algorithms (quick reference)

When implementing these features, read the full algorithm in `docs/ALGORITHMS.md`:

| Feature | Algorithm |
|---------|-----------|
| PoB XML parser | Alg 2 |
| DPS calculation | Alg 3 |
| Resistance aggregation | Alg 5 |
| Item scoring | Alg 6 |
| Issue detection | Alg 7 |
| Upgrade ranking | Alg 9 |
| Seer query routing | Alg 1 |
| poe.ninja price fetch | Alg 21 |
| Combat simulation | Alg 20 |
| Gem optimizer | Alg 22 |
| PoE OAuth PKCE | Alg 37 |
| Stash tab processor | Alg 38 |
| Map mod danger scorer | Alg 39 |
| Portable storage init | Alg 44a |
| PoB file watcher | Alg 44b |
| PoE character fetch | Alg 45 |
| PoB write-back | Alg 46 |
| Craft suggestion ranker | Alg 47 |
| Passive node recommender | Alg 48 |
| Build comparator | Alg 49 |
| Price alert manager | Alg 50 |
| Item image resolver | Alg 51 |
| Buy timing advisor | Alg 52 |
| Map run accumulator | Alg 53 |
| Cloud AI connection | Alg 54 |
| Seer network architecture | Alg 42 |

---

## Important Constraints

- **Portable storage:** all data goes in `PathOfAI_Data/` next to the exe — never AppData. See Algorithm 44a.
- **PoE API rate limit:** 45 requests / 60 seconds. Always use token-bucket limiter. See Algorithm 38.
- **poe.ninja cache:** 5-minute TTL, stale fallback, circuit breaker. Never hammer the API.
- **PoB write-back:** always backup before patching, check file lock, atomic write. See Algorithm 46.
- **OAuth tokens:** store in `PathOfAI_Data/auth.json` (AES-256 encrypted), never OS keychain.
- **No business logic in frontend:** all calculations happen in Rust.
- **All algorithm code in Rust** — not JS. Prototypes are read-only reference.

---

## Tauri IPC Contract

All commands are documented in `docs/IPC-SPEC.md`. Key commands:

```typescript
invoke('analyze_build', { filePath: string })      → AnalysisResult
invoke('load_character', { characterName: string }) → AnalysisResult
invoke('ask_seer', { question: string, buildId: string }) → SeerResponse
invoke('get_prices', { itemNames: string[] })       → PriceResult[]
invoke('apply_upgrade', { suggestionId: string, buildId: string }) → UpgradeResult
invoke('parse_clipboard_item', { clipboardText: string, buildId: string }) → ParsedItemResult
```

Events (backend → frontend):
```typescript
listen('analysis-complete', handler)   // build re-analyzed
listen('price-updated', handler)        // poe.ninja refresh
listen('pob-file-changed', handler)     // external PoB edit detected
listen('price-alert-triggered', handler) // price alert fired
```

---

## Do / Don't

| Do | Don't |
|----|-------|
| Read the relevant doc before writing code | Guess at algorithms — read ALGORITHMS.md |
| Keep `commands/` thin | Put logic in Tauri command handlers |
| Keep `core/` pure | Import `tokio` or Tauri in core/ |
| Match Rust types to TS types exactly | Create separate type systems |
| Use the UI prototype as visual reference | Rewrite the UI from scratch without looking at ui/ |
| Write tests for all calculator functions | Ship calculator code without tests |
| Use the existing CSS variables | Hardcode colors |
| Port prototype JS logic to Rust 1:1 | Re-invent algorithms the prototype already solved |
| Follow TDD: tests first, then implement | Add features without test coverage |
| Every commit must pass `cargo test && npm test` | Commit code with failing tests |

---

## Testing Approach (TDD)

**Rule:** every change must be covered by tests before the commit. Three layers required:

### Layer 1 — Rust Unit Tests (`cargo test`)
- Location: `#[cfg(test)] mod tests { ... }` inside each source file
- Use `tempfile` for DB tests (never use real filesystem)
- Use `#[tokio::test]` for async tests
- Make private helpers `pub(crate)` when direct testing adds value
- Target: every `core/` and `calculator/` function has at least one test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn my_pure_function_returns_expected_value() {
        assert_eq!(my_function(input), expected);
    }

    #[tokio::test]
    async fn my_async_function_handles_error() {
        let result = my_async_fn(bad_input).await;
        assert!(result.is_err());
    }
}
```

### Layer 2 — Frontend Unit Tests (`npm test`)
- Framework: **Vitest** + `jsdom`
- Location: `src/**/__tests__/*.test.ts`
- Mock Tauri `invoke()` via `vi.mock('@tauri-apps/api/core')`
- Test store state transitions and bridge wrappers
- Run: `npm test` (vitest run)

```typescript
import { describe, it, expect, vi } from 'vitest';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
```

### Layer 3 — E2E Tests (`npm run test:e2e`)
- Framework: **Playwright** targeting Vite dev server
- Location: `e2e/*.spec.ts`
- Tests run against mocked Tauri backend (window.__TAURI__ stubbed)
- Smoke tests: app renders, panels switch, loading states work
- Run: `npm run test:e2e`

### Commit Gate
Every commit must satisfy:
```
cargo test          # all Rust unit tests pass
npm test            # all Vitest tests pass
npm run test:e2e    # Playwright smoke tests pass
```

### Test File Naming
- Rust: inline `#[cfg(test)]` module in the same file
- TS unit: `src/services/__tests__/bridge.test.ts`
- E2E: `e2e/smoke.spec.ts`

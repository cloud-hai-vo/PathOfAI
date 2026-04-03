# Prototypes (Historical — Do NOT Use for Implementation)

These JavaScript files are **design-phase prototypes** created during the initial
exploration of Path of AI. They helped us understand the problem domain and were
used to draft the architecture documents.

**They are NOT production code.**

The actual implementation will be:
- **Backend:** Rust (in `src-tauri/src/`) — see [ARCHITECTURE.md](../docs/ARCHITECTURE.md)
- **Frontend:** Vanilla TypeScript (in `src/`) — see [ARCHITECTURE.md §3](../docs/ARCHITECTURE.md)

## Prototype → Production Mapping

| Prototype (JS) | Production (Rust) | Notes |
|---|---|---|
| pob-parser.js | `src-tauri/src/core/pob_parser.rs` | Port XML parsing to quick-xml |
| build-analyzer.js | `src-tauri/src/core/build_analyzer.rs` | Port scoring + issue detection |
| build-detector.js | `src-tauri/src/core/build_detector.rs` | Port archetype detection |
| mod-impact-calculator.js | `src-tauri/src/calculator/offense_calc.rs` | Replace with full calc engine |
| pob-writer.js | `src-tauri/src/core/pob_writer.rs` | Port atomic write + backup |
| market-intelligence.js | `src-tauri/src/market/poe_ninja.rs` | Port API client + cache |
| file-watcher.js | `src-tauri/src/services/file_watcher.rs` | Port with notify crate |
| item-image-resolver.js | `src-tauri/src/data/item_image.rs` | Port CDN URL resolver |
| portable-storage.js | Built into Tauri config | Use PathOfAI_Data/ portable storage |
| seer-engine.js | `src-tauri/src/seer/` | Replace with query router (no neural networks) |

## What to Reference

These prototypes are useful as **domain knowledge reference** — they show:
- What data structures PoB XML contains (pob-parser.js)
- What stat weights matter per archetype (build-analyzer.js)
- What game mechanics exist (build-detector.js)
- How poe.ninja API works (market-intelligence.js)

But the **actual implementation patterns** should follow:
- [ENGINE-DESIGN.md](../docs/ENGINE-DESIGN.md) — calculation formulas + engine architecture
- [ARCHITECTURE.md](../docs/ARCHITECTURE.md) — Rust module structure + hexagonal pattern
- [CODE-PATTERNS.md](../docs/CODE-PATTERNS.md) — Rust coding conventions

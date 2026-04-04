# Path of AI — Workspace Instructions

## TDD is Non-Negotiable (RED → GREEN → REFACTOR)

**Every feature must follow this exact cycle. No exceptions.**

1. **RED** — Write a failing test that describes the desired behavior. Run it. Confirm it fails.
2. **GREEN** — Write the *minimum* code to make the test pass. Nothing more.
3. **REFACTOR** — Clean up code. Tests must still pass after refactoring.

**You must not write implementation code before a test exists for it.**

### Commit Gate — Every commit must pass all three:
```
cargo test          # Rust unit tests
npm test            # Vitest frontend tests
npm run test:e2e    # Playwright smoke tests
```

**Never commit with a failing test. Never commit implementation without a test.**

## Architecture

- **Backend**: Rust (Tauri 2). `commands/` = thin IPC wrappers. `core/` = pure logic, no Tauri deps.
- **Frontend**: Vanilla TypeScript, no framework. All Tauri calls through `src/services/bridge.ts`.
- See `CLAUDE.md` for full context, doc map, and algorithm references.

## Build & Test

```bash
# Rust
cd src-tauri && cargo test

# Frontend unit
npm test

# E2E
npm run test:e2e

# Full build
npm run build
```

## Conventions

- Algorithm implementations: always read `docs/ALGORITHMS.md` for the specific algorithm first.
- New Rust module: add `#[cfg(test)] mod tests { ... }` in the same file, tests written before code.
- New TS function: add test in `src/**/__tests__/*.test.ts` before writing the function body.
- DB changes: update schema in `db/mod.rs` AND add a migration test using `tempfile`.
- All `core/` functions must be pure — no `tokio`, no Tauri, no network calls.

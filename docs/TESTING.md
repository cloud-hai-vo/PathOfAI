# Path of AI — Testing Strategy

## Overview

Testing is organized in 4 layers: unit, integration, E2E, and performance benchmarks.
Total coverage target: **80%+ for core business logic, 60%+ for UI, 100% CI pass on every PR.**

Related docs:
- [ENGINE-DESIGN.md](ENGINE-DESIGN.md) — Calculator engine architecture (what we're testing)
- [ARCHITECTURE.md](ARCHITECTURE.md) — Module structure (where tests go)
- [DATA-SOURCES.md](DATA-SOURCES.md) — Game data that feeds the calculator

---

## 1. TEST INFRASTRUCTURE

### Required Test Data

```
test-data/
  SampleRFInquisitor.xml        ← EXISTS (fire DoT tank)
  SampleColdDOTOccultist.xml    ← NEEDED (cold DoT, CI/ES)
  SampleLightningAttack.xml     ← NEEDED (attack crit, evasion)
  SampleMinionNecro.xml         ← NEEDED (minion, low personal DPS)
  SampleCoCAssassin.xml         ← NEEDED (CoC trigger, fast attack)
  SampleSSFStarter.xml          ← NEEDED (no trade items, low budget)
  MalformedBuild.xml            ← NEEDED (broken XML for error handling)
  EmptyBuild.xml                ← NEEDED (no items, no tree)
  game-data/                    ← test fixtures
    mods/mod-tiers-test.json    ← subset of mod database for tests
    gems/active-gems-test.json  ← subset of gem data
```

### Tools & Dependencies

#### Rust Backend
```toml
# src-tauri/Cargo.toml [dev-dependencies]
tokio = { version = "1", features = ["full"] }    # async test runtime
mockall = "0.12"                                    # mock traits
wiremock = "0.6"                                    # mock HTTP (poe.ninja)
tempfile = "3"                                      # temp dirs for file watcher tests
criterion = "0.5"                                   # benchmarks
```

#### TypeScript Frontend
```json
// package.json devDependencies
{
  "vitest": "^2.0",
  "@testing-library/dom": "^10.0",
  "@testing-library/user-event": "^14.5",
  "msw": "^2.0",
  "@vitest/coverage-v8": "^2.0"
}
```

#### E2E
```json
{
  "@playwright/test": "^1.45"
}
```

---

## 2. UNIT TESTS (Rust — cargo test)

### 2.1 PoE Math Formulas
Every PoE formula must have boundary tests:

```rust
#[cfg(test)]
mod formula_tests {
    use super::*;

    #[test]
    fn phys_reduction_at_zero_armour() {
        assert_eq!(calculate_phys_reduction(0, 5000), 0.0);
    }

    #[test]
    fn phys_reduction_standard_hit() {
        // Armour / (Armour + 5 * Damage) * 100
        // 25000 / (25000 + 25000) = 50%
        let result = calculate_phys_reduction(25000, 5000);
        assert!((result - 50.0).abs() < 0.01);
    }

    #[test]
    fn phys_reduction_capped_at_90() {
        // Even with massive armour, can't exceed 90%
        assert!(calculate_phys_reduction(999999999, 100) <= 90.0);
    }

    #[test]
    fn chaos_res_tier_boundaries() {
        assert_eq!(chaos_res_tier(75).tier, "capped");
        assert_eq!(chaos_res_tier(74).tier, "good");
        assert_eq!(chaos_res_tier(50).tier, "good");
        assert_eq!(chaos_res_tier(49).tier, "okay");
        assert_eq!(chaos_res_tier(0).tier, "okay");
        assert_eq!(chaos_res_tier(-1).tier, "low");
        assert_eq!(chaos_res_tier(-30).tier, "low");
        assert_eq!(chaos_res_tier(-31).tier, "negative");
    }

    #[test]
    fn dps_tier_boundaries() {
        assert_eq!(dps_tier(10_000_000).tier, "S");
        assert_eq!(dps_tier(9_999_999).tier, "A");
        assert_eq!(dps_tier(5_000_000).tier, "A");
        assert_eq!(dps_tier(499_999).tier, "F");
    }

    #[test]
    fn mod_tier_detection_life() {
        assert_eq!(estimate_mod_tier("maximum_life", "flat", 95).tier, 1); // T1: 90-99
        assert_eq!(estimate_mod_tier("maximum_life", "flat", 85).tier, 2); // T2: 80-89
        assert_eq!(estimate_mod_tier("maximum_life", "flat", 45).tier, 5); // T5+
    }

    #[test]
    fn effective_hp_calculation() {
        let life = 6000;
        let phys_reduction = 50.0; // 50% from armour
        let ehp = life as f64 / (1.0 - phys_reduction / 100.0);
        assert_eq!(ehp, 12000.0);
    }

    #[test]
    fn mana_reservation_calculation() {
        let total_mana = 534;
        let reserved = 432; // Determination + Purity + Vitality
        let unreserved = total_mana - reserved;
        assert_eq!(unreserved, 102);
        assert!(unreserved > 0); // must have mana for skills
    }
}
```

### 2.2 PoB Parser Tests
```rust
#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn parse_rf_inquisitor() {
        let xml = include_str!("../../../test-data/SampleRFInquisitor.xml");
        let build = PobParser::parse(xml).unwrap();
        assert_eq!(build.class_name, "Templar");
        assert_eq!(build.ascend_class_name, "Inquisitor");
        assert!(build.stats.life > 5000);
        assert!(build.stats.fire_resist >= 75);
    }

    #[test]
    fn parse_empty_build_gracefully() {
        let xml = include_str!("../../../test-data/EmptyBuild.xml");
        let build = PobParser::parse(xml).unwrap();
        assert_eq!(build.stats.life, 0);
        assert!(build.items.is_empty());
    }

    #[test]
    fn reject_malformed_xml() {
        let result = PobParser::parse("<not valid>");
        assert!(result.is_err());
    }

    #[test]
    fn parse_all_item_slots() {
        let xml = include_str!("../../../test-data/SampleRFInquisitor.xml");
        let build = PobParser::parse(xml).unwrap();
        let slots: Vec<&str> = build.items.iter().map(|i| i.slot.as_str()).collect();
        assert!(slots.contains(&"Helmet"));
        assert!(slots.contains(&"Body Armour"));
        assert!(slots.contains(&"Boots"));
    }

    #[test]
    fn parse_skill_gems_with_levels() {
        let xml = include_str!("../../../test-data/SampleRFInquisitor.xml");
        let build = PobParser::parse(xml).unwrap();
        let rf_gem = build.gems.iter()
            .flat_map(|g| &g.gems)
            .find(|g| g.name == "Righteous Fire");
        assert!(rf_gem.is_some());
        assert!(rf_gem.unwrap().level >= 20);
    }
}
```

### 2.3 Build Detection Tests
```rust
#[test]
fn detect_rf_as_fire_dot() {
    let build = parse_test_build("SampleRFInquisitor.xml");
    let detector = BuildDetector::new(&build);
    assert_eq!(detector.archetype, Archetype::FireDot);
    assert_eq!(detector.main_skill.name, "Righteous Fire");
}

#[test]
fn detect_cold_dot_as_cold_dot() {
    let build = parse_test_build("SampleColdDOTOccultist.xml");
    let detector = BuildDetector::new(&build);
    assert_eq!(detector.archetype, Archetype::ColdDot);
}

#[test]
fn detect_minion_build() {
    let build = parse_test_build("SampleMinionNecro.xml");
    let detector = BuildDetector::new(&build);
    assert_eq!(detector.archetype, Archetype::Minion);
}
```

### 2.4 Market Intelligence Tests
```rust
#[tokio::test]
async fn fetch_prices_from_mock_ninja() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/api/data/itemoverview"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({"lines":[{"name":"Aegis Aurora","chaosValue":1500}]})))
        .mount(&mock_server).await;

    let client = PoeNinjaClient::new(&mock_server.uri());
    let price = client.get_unique_price("Aegis Aurora").await.unwrap();
    assert!((price - 18.0).abs() < 1.0); // 1500 chaos / 85 ratio ≈ 18 divine
}

#[test]
fn price_cache_returns_stale_when_offline() {
    let cache = PriceCache::new();
    cache.set("Aegis Aurora", 18.0, Duration::from_secs(300));
    // Even if expired, return stale with warning
    let result = cache.get("Aegis Aurora");
    assert!(result.is_some());
}
```

---

## 3. INTEGRATION TESTS

### Full Analysis Pipeline
```rust
#[tokio::test]
async fn full_rf_analysis_pipeline() {
    let data_dir = PathBuf::from("./game-data");
    let mod_db = ModDatabase::load(&data_dir).unwrap();
    let xml = std::fs::read_to_string("./test-data/SampleRFInquisitor.xml").unwrap();
    let build = PobParser::parse(&xml).unwrap();
    let analyzer = BuildAnalyzer::new(&build, &mod_db);
    let result = analyzer.analyze();

    // Verify structure
    assert!(result.overall_score > 0 && result.overall_score <= 100);
    assert!(!result.issues.is_empty());
    assert!(!result.suggestions.is_empty());

    // RF-specific checks
    assert!(result.defenses.life_regen > 0); // RF needs regen
    assert!(result.defenses.fire_resist >= 75); // must be capped for RF

    // Issues should flag chaos res (our sample has 15%)
    let chaos_issue = result.issues.iter().find(|i| i.issue.contains("Chaos"));
    assert!(chaos_issue.is_some());
}
```

### Tauri Command Integration
```rust
#[tauri::test]
async fn test_analyze_build_command() {
    let mod_db = ModDatabase::load(&PathBuf::from("./game-data")).unwrap();
    let state = AppState {
        current_build: Mutex::new(None),
        mod_db,
        // ... other fields with test defaults
    };

    let result = analyze_build(
        "test-data/SampleRFInquisitor.xml".to_string(),
        tauri::State(&state)
    ).await;

    assert!(result.is_ok());
    let analysis = result.unwrap();
    assert_eq!(analysis.defenses.fire_resist, 80);
}
```

---

## 4. E2E TESTS (Playwright)

```typescript
// tests/e2e/load-build.spec.ts
import { test, expect } from '@playwright/test';

test('load PoB build and verify Omen tab', async ({ page }) => {
    await page.goto('tauri://localhost');

    // Verify header
    await expect(page.locator('.app-title')).toContainText('PATH');

    // Equipment grid should render
    await expect(page.locator('.char-slot')).toHaveCount(9);

    // Life value should be visible
    await expect(page.locator('[data-stat="life"]')).toContainText('6,453');
});

test('click equipment slot shows item tooltip', async ({ page }) => {
    await page.goto('tauri://localhost');

    await page.click('.char-slot[data-slot="Helmet"]');

    // Right panel should show item details
    await expect(page.locator('#rp-title')).toContainText('Glyph Crest');
    await expect(page.locator('#right-panel-content')).toContainText('+78 to maximum Life');
});

test('HUD gem buttons switch right panel', async ({ page }) => {
    await page.goto('tauri://localhost');

    // Click Grimoire gem
    await page.click('[data-panel="grimoire"]');
    await expect(page.locator('#rp-title')).toContainText('Grimoire');

    // Click Defenses gem
    await page.click('[data-panel="defenses"]');
    await expect(page.locator('#rp-title')).toContainText('Defenses');
});

test('AI provider selection works', async ({ page }) => {
    await page.goto('tauri://localhost');

    await page.click('[data-panel="settings"]');
    await page.click('input[value="claude"]');

    // API key section should appear
    await expect(page.locator('#api-key-section')).toBeVisible();
});
```

---

## 5. PERFORMANCE BENCHMARKS

```rust
// benches/core_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_xml_parsing(c: &mut Criterion) {
    let xml = include_str!("../test-data/SampleRFInquisitor.xml");
    c.bench_function("parse_pob_xml", |b| {
        b.iter(|| PobParser::parse(xml))
    });
    // TARGET: < 50ms
}

fn bench_build_analysis(c: &mut Criterion) {
    let xml = include_str!("../test-data/SampleRFInquisitor.xml");
    let build = PobParser::parse(xml).unwrap();
    let mod_db = ModDatabase::load_test();

    c.bench_function("full_analysis", |b| {
        b.iter(|| BuildAnalyzer::new(&build, &mod_db).analyze())
    });
    // TARGET: < 200ms
}

fn bench_dps_estimation(c: &mut Criterion) {
    let build = parse_test_build("SampleRFInquisitor.xml");
    c.bench_function("dps_fast_estimate", |b| {
        b.iter(|| ModImpactCalculator::estimate_dps_change(&build, &test_mod))
    });
    // TARGET: < 50ms
}

criterion_group!(benches, bench_xml_parsing, bench_build_analysis, bench_dps_estimation);
criterion_main!(benches);
```

Performance targets (from CODE-PATTERNS.md):
| Operation | Target | Benchmark |
|-----------|--------|-----------|
| XML parsing | < 50ms | `bench_xml_parsing` |
| Fast DPS estimation | < 50ms | `bench_dps_estimation` |
| Full build analysis | < 200ms | `bench_build_analysis` |
| File watch latency | < 100ms | `bench_file_watcher` |
| Seer inference | < 100ms | `bench_seer_query` |

---

## 6. CI/CD PIPELINE

```yaml
# .github/workflows/test.yml
name: Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  rust-tests:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [windows-latest, ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run unit tests
        run: cargo test --all
        working-directory: src-tauri
      - name: Run benchmarks (check only)
        run: cargo bench --no-run
        working-directory: src-tauri

  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: npm ci
      - run: npm test -- --coverage
      - name: Upload coverage
        uses: codecov/codecov-action@v4

  e2e-tests:
    runs-on: windows-latest
    needs: [rust-tests, frontend-tests]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: npm ci
      - run: cargo tauri build
        working-directory: src-tauri
      - run: npx playwright test

  build-release:
    runs-on: ${{ matrix.os }}
    needs: [e2e-tests]
    if: startsWith(github.ref, 'refs/tags/v')
    strategy:
      matrix:
        include:
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: npm ci
      - run: cargo tauri build
        working-directory: src-tauri
      - name: Upload Release
        uses: softprops/action-gh-release@v2
        with:
          files: src-tauri/target/release/bundle/**/*
```

---

## 7. TEST COVERAGE TARGETS

| Module | Target | Priority |
|--------|--------|----------|
| PoE math formulas | 95%+ | P0 — correctness critical |
| PoB XML parser | 90%+ | P0 — everything depends on this |
| Build detector | 85%+ | P1 — archetype drives all analysis |
| Build analyzer | 80%+ | P1 — core value proposition |
| Mod impact calculator | 85%+ | P1 — DPS numbers must be accurate |
| Market intelligence | 70%+ | P2 — external API, harder to test |
| Seer engine | 60%+ | P2 — ML model, harder to assert |
| Tauri commands | 80%+ | P1 — IPC boundary |
| Frontend components | 60%+ | P2 — visual, harder to test |
| File watcher | 70%+ | P2 — OS-dependent |
| Auto-updater | 70%+ | P2 — network-dependent |

---

## 8. IMPLEMENTATION PRIORITY

### Sprint 1 (Week 1-2): Foundation
- Add Cargo.toml dev-dependencies
- Create 4 more test PoB XML files
- Write 15 unit tests for math formulas
- Set up GitHub Actions CI with `cargo test`

### Sprint 2 (Week 3-4): Core Analysis
- Write 20 parser tests (all slots, gems, tree, config)
- Write 15 build detection tests (all archetypes)
- Write 10 analyzer integration tests
- Mock poe.ninja with wiremock

### Sprint 3 (Week 5-6): Frontend & Commands
- Set up Vitest + @testing-library
- Write 10 component tests (Omen stats, Arsenal grid, Prophecy cards)
- Write 10 Tauri command tests with mocked state
- Add coverage reporting

### Sprint 4 (Week 7-8): E2E & Performance
- Set up Playwright
- Write 8 E2E workflow tests
- Add criterion benchmarks for performance targets
- Add coverage gates to CI (fail if < target)

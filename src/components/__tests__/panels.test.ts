/**
 * Tests for panel render functions in src/components/panels.ts
 * TDD: these tests are written BEFORE the implementation.
 *
 * Panel renderers return HTML strings — tests verify the structure and content.
 */
import { describe, it, expect } from 'vitest';
import type { AnalysisResult } from '../../types/index.js';
import {
  renderArenaPanel,
  renderGemsPanel,
  renderBloodPactPanel,
  renderDarkPathPanel,
  renderCurseMapPanel,
  renderPassiveTreePanel,
  renderStashPanel,
  renderHarbingerPanel,
} from '../panels.js';

// ── Shared test fixture ───────────────────────────────────────────────────────

function makeAnalysis(overrides: Partial<AnalysisResult> = {}): AnalysisResult {
  return {
    build_id: 'build-1',
    build_name: 'RF Inquisitor',
    class_name: 'Templar',
    ascendancy: 'Inquisitor',
    level: 92,
    archetype: 'fire_dot',
    archetype_label: 'RF Inquisitor',
    overall_score: 74,
    defenses: {
      life: 5200,
      energy_shield: 0,
      mana: 800,
      life_regen_flat: 420,
      life_regen_pct: 8.1,
      resistances: {
        fire: 75, cold: 75, lightning: 75, chaos: -20,
        max_fire: 75, max_cold: 75, max_lightning: 75, max_chaos: 75,
        fire_overcap: 0, cold_overcap: 0, lightning_overcap: 0,
      },
      armour: 15000,
      armour_phys_reduction: 0.60,
      evasion: 0,
      evasion_chance: 0,
      block_chance: 0.30,
      spell_block_chance: 0.30,
      effective_hp: { vs_physical: 37500, vs_elemental: 20800, vs_chaos: 5200 },
      ailment_immunity: {
        freeze: true, freeze_source: 'Purity of Ice',
        shock: false,
        ignite: false,
        bleed: false,
        corrupted_blood: true, corrupted_blood_source: 'Corrupted Blood jewel',
        poison: false,
        stun: false,
        curse_immune: false,
      },
    },
    offense: {
      total_dps: 2_840_000,
      dps_label: '2.84M',
      main_skill: 'Righteous Fire',
      hit_dps: 0,
      dot_dps: 2_840_000,
      crit_chance: 0,
      crit_multiplier: 1,
      attack_speed: 0,
      cast_speed: 0,
      hit_chance: 1,
      sources: [
        { source: 'RF DoT', value: 2_840_000, percent_of_total: 100, color: '#cf3a1f' },
      ],
      multiplier_chain: [],
    },
    issues: [
      { id: 'chaos_res', severity: 'Major', title: 'Chaos Res low', detail: '-20%', fix: 'Add chaos res', slot: undefined },
    ],
    suggestions: [
      { id: 's1', slot: 'Ring', title: 'Better ring', detail: '+40 life', dps_gain: 0, dps_gain_pct: 0, life_gain: 40, estimated_cost_div: 2, efficiency: 20, priority: 1, trade_url: undefined },
    ],
    item_scores: [
      { slot: 'Helmet', item_name: 'Rare Helmet', score: 55, tier: 'Acceptable', top_issue: 'Missing life' },
    ],
    gem_setups: [
      {
        skill: 'Righteous Fire',
        slot: 'Body Armour',
        socket_colors: 'RRRR',
        gems: [
          { name: 'Righteous Fire', level: 21, quality: 20, is_support: false, is_vaal: false, is_awakened: false, is_maxed: true },
          { name: 'Burning Damage', level: 20, quality: 20, is_support: true, is_vaal: false, is_awakened: false, is_maxed: true },
          { name: 'Elemental Focus', level: 20, quality: 20, is_support: true, is_vaal: false, is_awakened: false, is_maxed: true },
          { name: 'Swift Affliction', level: 20, quality: 20, is_support: true, is_vaal: false, is_awakened: false, is_maxed: true },
        ],
        is_main_skill: true,
      },
    ],
    ...overrides,
  };
}

// ── Arena panel ───────────────────────────────────────────────────────────────

describe('renderArenaPanel', () => {
  it('shows placeholder when no analysis', () => {
    const html = renderArenaPanel(null);
    expect(html).toContain('Load a build first');
  });

  it('renders DPS and main skill name', () => {
    const html = renderArenaPanel(makeAnalysis());
    expect(html).toContain('2.84M');
    expect(html).toContain('Righteous Fire');
  });

  it('renders boss readiness rows with TTK', () => {
    const html = renderArenaPanel(makeAnalysis());
    expect(html).toContain('Shaper');
    expect(html).toContain('Elder');
    expect(html).toContain('Sirus');
    expect(html).toContain('Maven');
    expect(html).toContain('TTK');
  });

  it('marks boss as READY if TTK < threshold', () => {
    // 2.84M DPS vs Shaper (10M HP) → TTK ~3.5s → should be READY
    const html = renderArenaPanel(makeAnalysis());
    expect(html).toContain('READY');
  });

  it('marks boss as NOT READY for extremely high HP bosses with low DPS', () => {
    const lowDps = makeAnalysis();
    lowDps.offense.total_dps = 100_000;
    lowDps.offense.dps_label = '100K';
    const html = renderArenaPanel(lowDps);
    expect(html).toContain('NOT READY');
  });
});

// ── Gems panel ────────────────────────────────────────────────────────────────

describe('renderGemsPanel', () => {
  it('shows placeholder when no analysis', () => {
    const html = renderGemsPanel(null);
    expect(html).toContain('Load a build first');
  });

  it('renders the main skill name', () => {
    const html = renderGemsPanel(makeAnalysis());
    expect(html).toContain('Righteous Fire');
  });

  it('renders socket colors', () => {
    const html = renderGemsPanel(makeAnalysis());
    // Socket colors RRRR → should render 4 sockets
    expect(html).toContain('RRRR');
  });

  it('renders each gem with level and quality', () => {
    const html = renderGemsPanel(makeAnalysis());
    expect(html).toContain('Burning Damage');
    expect(html).toContain('20/20');
  });

  it('shows placeholder when gem_setups is empty', () => {
    const a = makeAnalysis({ gem_setups: [] });
    const html = renderGemsPanel(a);
    expect(html).toContain('No gem');
  });
});

// ── Blood Pact panel ──────────────────────────────────────────────────────────

describe('renderBloodPactPanel', () => {
  it('shows placeholder when no analysis', () => {
    const html = renderBloodPactPanel(null);
    expect(html).toContain('Load a build first');
  });

  it('renders sworn goals section', () => {
    const html = renderBloodPactPanel(makeAnalysis());
    expect(html).toContain('Sworn Goals');
  });

  it('renders blood rituals quick actions', () => {
    const html = renderBloodPactPanel(makeAnalysis());
    expect(html).toContain('Invoke');
  });

  it('converts top issues into checklist items', () => {
    const html = renderBloodPactPanel(makeAnalysis());
    expect(html).toContain('Chaos Res low');
  });

  it('converts top suggestions into checklist items', () => {
    const html = renderBloodPactPanel(makeAnalysis());
    expect(html).toContain('Better ring');
  });
});

// ── Dark Path panel ───────────────────────────────────────────────────────────

describe('renderDarkPathPanel', () => {
  it('shows placeholder when no analysis', () => {
    const html = renderDarkPathPanel(null);
    expect(html).toContain('Load a build first');
  });

  it('renders build identity section', () => {
    const html = renderDarkPathPanel(makeAnalysis());
    expect(html).toContain('RF Inquisitor');
  });

  it('renders three evolution path cards', () => {
    const html = renderDarkPathPanel(makeAnalysis());
    expect(html).toContain('Immortal');
    expect(html).toContain('Inferno');
    expect(html).toContain('Exile');
  });

  it('each path card shows cost and trade-off', () => {
    const html = renderDarkPathPanel(makeAnalysis());
    expect(html).toContain('div');        // cost in divines
    expect(html).toContain('DPS');
    expect(html).toContain('Surv');
  });
});

// ── Passive Tree panel ────────────────────────────────────────────────────────

describe('renderPassiveTreePanel', () => {
  it('shows placeholder when no analysis', () => {
    const html = renderPassiveTreePanel(null);
    expect(html).toContain('Load a build first');
  });

  it('shows points spent bar', () => {
    const html = renderPassiveTreePanel(makeAnalysis({ level: 90 }));
    expect(html).toContain('Points Spent');
    expect(html).toContain('/ 123');
  });

  it('shows archetype label', () => {
    const html = renderPassiveTreePanel(makeAnalysis());
    expect(html).toContain('RF Inquisitor');
  });

  it('shows class and ascendancy', () => {
    const html = renderPassiveTreePanel(makeAnalysis());
    expect(html).toContain('Templar');
    expect(html).toContain('Inquisitor');
  });
});

// ── Stash panel ───────────────────────────────────────────────────────────────

describe('renderStashPanel', () => {
  it('shows connect prompt when no items', () => {
    const html = renderStashPanel([]);
    expect(html).toContain('Connect your PoE account');
  });

  it('shows total value in divines', () => {
    const items = [
      { name: "Watcher's Eye", chaos_value: 440, stack_size: 1, tab_name: 'Gear' },
      { name: 'Divine Orb',    chaos_value: 220, stack_size: 5, tab_name: 'Currency' },
    ];
    const html = renderStashPanel(items, 220);
    // 440*1 + 220*5 = 1540 chaos / 220 = 7.0 divine
    expect(html).toContain('7.0d');
    expect(html).toContain('2');  // 2 unique items
  });

  it('shows top items sorted by value', () => {
    const items = [
      { name: 'Chaos Orb',     chaos_value: 1,   stack_size: 100, tab_name: 'Currency' },
      { name: "Watcher's Eye", chaos_value: 440, stack_size: 1,   tab_name: 'Gear' },
    ];
    const html = renderStashPanel(items, 220);
    const watcherPos  = html.indexOf("Watcher's Eye");
    const chaosPos    = html.indexOf('Chaos Orb');
    expect(watcherPos).toBeLessThan(chaosPos); // higher value item first
  });
});

// ── Harbinger panel ───────────────────────────────────────────────────────────

describe('renderHarbingerPanel', () => {
  it('shows placeholder when no analysis', () => {
    const html = renderHarbingerPanel(null);
    expect(html).toContain('Load a build first');
  });

  it('shows no-issue message when issues array is empty', () => {
    const html = renderHarbingerPanel(makeAnalysis({ issues: [] }));
    expect(html).toContain('No issues detected');
  });

  it('shows issue count in header', () => {
    const issues = [
      { id: '1', severity: 'Critical' as const, title: 'Low life', detail: 'Need more life', fix: 'Add life', slot: 'Ring' },
      { id: '2', severity: 'Minor' as const,    title: 'Low res',  detail: 'Resist low',     fix: 'Add res'  },
    ];
    const html = renderHarbingerPanel(makeAnalysis({ issues }));
    expect(html).toContain('(2)');
  });

  it('renders critical issues with red styling', () => {
    const issues = [
      { id: '1', severity: 'Critical' as const, title: 'Chaos res 0%', detail: 'Instant death', fix: 'Get chaos res' },
    ];
    const html = renderHarbingerPanel(makeAnalysis({ issues }));
    expect(html).toContain('Critical');
    expect(html).toContain('Chaos res 0%');
    expect(html).toContain('Instant death');
  });
});

// ── Curse Map panel ───────────────────────────────────────────────────────────

describe('renderCurseMapPanel', () => {
  it('shows placeholder when no analysis', () => {
    const html = renderCurseMapPanel(null);
    expect(html).toContain('Load a build first');
  });

  it('renders Cannot Run section for RF builds (No Regen, Ele Reflect)', () => {
    const html = renderCurseMapPanel(makeAnalysis());
    expect(html).toContain('Cannot Run');
    expect(html).toContain('No Regeneration');
    expect(html).toContain('Elemental Reflect');
  });

  it('renders Dangerous section for relevant mods', () => {
    const html = renderCurseMapPanel(makeAnalysis());
    expect(html).toContain('Dangerous');
    expect(html).toContain('max Resist');
  });

  it('renders Safe section for irrelevant mods', () => {
    const html = renderCurseMapPanel(makeAnalysis());
    expect(html).toContain('Safe');
    expect(html).toContain('Physical Reflect');
  });

  it('reflects build archetype in map mod advice', () => {
    // fire_dot archetype should warn about No Regeneration
    const html = renderCurseMapPanel(makeAnalysis());
    expect(html).toContain('fire');
  });
});

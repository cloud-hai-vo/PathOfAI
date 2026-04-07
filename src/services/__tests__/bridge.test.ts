/**
 * Tests for bridge.ts — typed Tauri invoke() wrappers.
 * Mocks the Tauri API so tests run in Node/jsdom without a Tauri process.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { AnalysisResult, PriceResult, BuildSummary, AlertCondition } from '../../types/index.js';

// Mock the Tauri invoke before importing bridge
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

// Import after mocking
const bridgeModule = await import('../bridge.js');

function makeAnalysisResult(overrides: Partial<AnalysisResult> = {}): AnalysisResult {
  return {
    build_id: 'test-build-1',
    build_name: 'RF Inquisitor',
    class_name: 'Templar',
    ascendancy: 'Inquisitor',
    level: 90,
    archetype: 'fire_dot',
    archetype_label: 'RF Inquisitor',
    overall_score: 72,
    defenses: {
      life: 5000,
      energy_shield: 0,
      mana: 500,
      life_regen_flat: 100,
      life_regen_pct: 2.5,
      resistances: { fire: 75, cold: 75, lightning: 75, chaos: -60, max_fire: 75, max_cold: 75, max_lightning: 75, max_chaos: 75, fire_overcap: 0, cold_overcap: 0, lightning_overcap: 0 },
      armour: 10000,
      armour_phys_reduction: 0.5,
      evasion: 0,
      evasion_chance: 0,
      block_chance: 0,
      spell_block_chance: 0,
      effective_hp: { vs_physical: 7500, vs_elemental: 10000, vs_chaos: 5000 },
      ailment_immunity: { freeze: false, shock: false, ignite: false, bleed: false, corrupted_blood: false, poison: false, stun: false, curse_immune: false },
    },
    offense: {
      total_dps: 1_500_000,
      dps_label: '1.50M',
      main_skill: 'Righteous Fire',
      hit_dps: 0,
      dot_dps: 1_500_000,
      crit_chance: 0.05,
      crit_multiplier: 1.5,
      attack_speed: 0,
      cast_speed: 0,
      hit_chance: 1.0,
      sources: [],
      multiplier_chain: [],
    },
    issues: [],
    suggestions: [],
    item_scores: [],
    gem_setups: [],
    ...overrides,
  };
}

describe('bridge.ts', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  // ── analyzeBuild ────────────────────────────────────────────────────────────

  it('analyzeBuild calls invoke with correct command and args', async () => {
    const expected = makeAnalysisResult();
    mockInvoke.mockResolvedValueOnce(expected);

    const result = await bridgeModule.analyzeBuild('/path/to/build.xml');

    expect(mockInvoke).toHaveBeenCalledWith('analyze_build', { filePath: '/path/to/build.xml' });
    expect(result.build_id).toBe('test-build-1');
  });

  // ── getPrices ───────────────────────────────────────────────────────────────

  it('getPrices passes item names array to backend', async () => {
    const mockPrices: PriceResult[] = [
      {
        item_name: "Kaom's Heart",
        price_div: 4.0,
        price_chaos: 800.0,
        confidence: 'High',
        listings: 150,
        cached: false,
        cache_age_secs: 0,  // added field
      },
    ];
    mockInvoke.mockResolvedValueOnce(mockPrices);

    const result = await bridgeModule.getPrices(["Kaom's Heart"]);

    expect(mockInvoke).toHaveBeenCalledWith('get_prices', { itemNames: ["Kaom's Heart"] });
    expect(result).toHaveLength(1);
    expect(result[0].price_div).toBe(4.0);
  });

  // ── startOAuth ──────────────────────────────────────────────────────────────

  it('startOAuth calls start_oauth command', async () => {
    mockInvoke.mockResolvedValueOnce('OAuth flow started');

    await bridgeModule.startOAuth();

    expect(mockInvoke).toHaveBeenCalledWith('start_oauth');
  });

  // ── getAuthStatus ───────────────────────────────────────────────────────────

  it('getAuthStatus returns boolean from backend', async () => {
    mockInvoke.mockResolvedValueOnce(true);

    const status = await bridgeModule.getAuthStatus();

    expect(mockInvoke).toHaveBeenCalledWith('get_auth_status');
    expect(status).toBe(true);
  });

  // ── listBuilds ──────────────────────────────────────────────────────────────

  it('listBuilds returns array of BuildSummary', async () => {
    const mockBuilds: BuildSummary[] = [
      {
        id: 'build-1',
        name: 'RF Inquisitor',
        class_name: 'Templar',
        ascendancy: 'Inquisitor',
        level: 90,
        last_analyzed: '2026-04-04T00:00:00Z',
      },
    ];
    mockInvoke.mockResolvedValueOnce(mockBuilds);

    const result = await bridgeModule.listBuilds();

    expect(mockInvoke).toHaveBeenCalledWith('list_builds');
    expect(result).toHaveLength(1);
    expect(result[0].name).toBe('RF Inquisitor');
  });

  // ── undoLastChange ──────────────────────────────────────────────────────────

  it('undoLastChange passes buildId to backend', async () => {
    const expected = makeAnalysisResult({ build_id: 'build-abc' });
    mockInvoke.mockResolvedValueOnce(expected);

    const result = await bridgeModule.undoLastChange('build-abc');

    expect(mockInvoke).toHaveBeenCalledWith('undo_last_change', { buildId: 'build-abc' });
    expect(result.build_id).toBe('build-abc');
  });

  // ── getBuyRecommendation ────────────────────────────────────────────────────

  it('getBuyRecommendation serialises history to JSON and passes league phase', async () => {
    const mockRec: import('../bridge.js').BuyRecommendation = {
      action: 'BuyWhenReady',
      reason: 'Watcher\'s Eye is stable at 45.0 div',
      urgency: 'None',
      confidence: 'High',
      current_div: 45.0,
      trend: 'Stable',
      change_7d: 0.5,
      league_phase: 'PeakEconomy',
      sparkline: [44, 44, 45, 45, 45, 45, 45],
    };
    mockInvoke.mockResolvedValueOnce(mockRec);

    const history: import('../bridge.js').PricePoint[] = [
      { price_divine: 44 }, { price_divine: 44 }, { price_divine: 45 },
      { price_divine: 45 }, { price_divine: 45 }, { price_divine: 45 },
      { price_divine: 45 },
    ];
    const result = await bridgeModule.getBuyRecommendation("Watcher's Eye", history, 'PeakEconomy');

    expect(mockInvoke).toHaveBeenCalledWith('get_buy_recommendation', {
      itemKey: "Watcher's Eye",
      historyJson: JSON.stringify(history),
      leaguePhase: 'PeakEconomy',
    });
    expect(result.action).toBe('BuyWhenReady');
    expect(result.confidence).toBe('High');
  });

  // ── estimateItemSwap ───────────────────────────────────────────────────��────

  it('estimateItemSwap serialises mods and passes buildId', async () => {
    const mockEst: import('../bridge.js').EstimateResult = {
      dps_change: 50000,
      life_change: 80,
      is_estimate: true,
    };
    mockInvoke.mockResolvedValueOnce(mockEst);

    const newMods: Array<[import('../bridge.js').StatType, number]> = [['FlatLife', 80], ['FireDotMulti', 20]];
    const curMods: Array<[import('../bridge.js').StatType, number]> = [['FlatLife', 50]];
    const result = await bridgeModule.estimateItemSwap('build-1', newMods, curMods);

    expect(mockInvoke).toHaveBeenCalledWith('estimate_item_swap', {
      buildId:         'build-1',
      newItemJson:     JSON.stringify(newMods),
      currentItemJson: JSON.stringify(curMods),
    });
    expect(result.dps_change).toBe(50000);
    expect(result.is_estimate).toBe(true);
  });

  // ── resolveItemImage ────────────────────────────────────────────────────────

  it('resolveItemImage passes type and name to backend', async () => {
    mockInvoke.mockResolvedValueOnce('https://web.poecdn.com/image/Art/2DItems/Armours/KaomsHeart.png');

    const url = await bridgeModule.resolveItemImage('unique', "Kaom's Heart");

    expect(mockInvoke).toHaveBeenCalledWith('resolve_item_image', {
      itemType: 'unique',
      itemName: "Kaom's Heart",
    });
    expect(url).toContain('poecdn.com');
  });

  // ── compareBuilds ───────────────────────────────────────────────────────────

  it('compareBuilds serialises both snapshots', async () => {
    const mockCmp = { build_a: 'A', build_b: 'B', stat_deltas: [], tree_overlap_pct: 50, shared_gems: [], unique_to_a: [], unique_to_b: [], summary_winner: 'B' };
    mockInvoke.mockResolvedValueOnce(mockCmp);

    const a = { id: 'A', name: 'A', stats: { dps: 1000 }, passives: [1, 2], gems: ['RF'] };
    const b = { id: 'B', name: 'B', stats: { dps: 2000 }, passives: [1, 3], gems: ['RF'] };
    const result = await bridgeModule.compareBuilds(a, b);

    expect(mockInvoke).toHaveBeenCalledWith('compare_builds_cmd', {
      buildAJson: JSON.stringify(a),
      buildBJson: JSON.stringify(b),
    });
    expect(result.summary_winner).toBe('B');
  });

  // ── tallyStashWealth ────────────────────────────────────────────────────────

  it('tallyStashWealth passes items and divine price', async () => {
    const mockWealth = { total_chaos: 400.0, total_divine: 2.0, currency_map: {}, total_items: 2 };
    mockInvoke.mockResolvedValueOnce(mockWealth);

    const items = [
      { id: '1', name: 'Chaos Orb', type_line: 'Chaos Orb', chaos_value: 1.0, stack_size: 200, tab_name: 'Currency' },
      { id: '2', name: 'Exalted Orb', type_line: 'Exalted Orb', chaos_value: 100.0, stack_size: 2, tab_name: 'Currency' },
    ];
    const result = await bridgeModule.tallyStashWealth(items, 200.0);

    expect(mockInvoke).toHaveBeenCalledWith('tally_stash_wealth', {
      itemsJson: JSON.stringify(items),
      divinePriceC: 200.0,
    });
    expect(result.total_divine).toBe(2.0);
  });

  // ── getMapStats ─────────────────────────────────────────────────────────────

  it('getMapStats serialises runs array', async () => {
    const mockStats = { total_runs: 2, total_time_secs: 600, avg_duration: 300, total_loot_chaos: 200, chaos_per_hour: 1200, most_run_map: 'Lookout', by_zone: { Lookout: 2 } };
    mockInvoke.mockResolvedValueOnce(mockStats);

    const runs = [
      { zone_name: 'Lookout', started_at: 0, ended_at: 300, duration_secs: 300, loot_chaos: 100 },
      { zone_name: 'Lookout', started_at: 300, ended_at: 600, duration_secs: 300, loot_chaos: 100 },
    ];
    const result = await bridgeModule.getMapStats(runs);

    expect(mockInvoke).toHaveBeenCalledWith('get_map_stats', { runsJson: JSON.stringify(runs) });
    expect(result.most_run_map).toBe('Lookout');
  });

  // ── checkPriceAlerts ────────────────────────────────────────────────────────

  it('checkPriceAlerts passes alerts and prices as JSON', async () => {
    const mockFired = [{ alert_id: 'a1', item_key: 'Chaos Orb', current_price: 0.9, threshold: 1.0, condition: 'Below' as AlertCondition, message: 'Chaos Orb below 1.0c' }];
    mockInvoke.mockResolvedValueOnce(mockFired);

    const alerts = [{ id: 'a1', item_key: 'Chaos Orb', condition: 'Below' as AlertCondition, threshold: 1.0, active: true, created_at: 0 }];
    const prices = { 'Chaos Orb': 0.9 };
    const result = await bridgeModule.checkPriceAlerts(alerts, prices);

    expect(mockInvoke).toHaveBeenCalledWith('check_price_alerts', {
      alertsJson: JSON.stringify(alerts),
      pricesJson: JSON.stringify(prices),
    });
    expect(result).toHaveLength(1);
    expect(result[0].alert_id).toBe('a1');
  });

  // ── deactivatePriceAlert ────────────────────────────────────────────────────

  it('deactivatePriceAlert passes alerts and alert id', async () => {
    const updatedAlerts = [{ id: 'a1', item_key: 'Chaos Orb', condition: 'Below' as AlertCondition, threshold: 1.0, active: false, created_at: 0 }];
    mockInvoke.mockResolvedValueOnce(updatedAlerts);

    const alerts = [{ id: 'a1', item_key: 'Chaos Orb', condition: 'Below' as AlertCondition, threshold: 1.0, active: true, created_at: 0 }];
    const result = await bridgeModule.deactivatePriceAlert(alerts, 'a1');

    expect(mockInvoke).toHaveBeenCalledWith('deactivate_price_alert', {
      alertsJson: JSON.stringify(alerts),
      alertId: 'a1',
    });
    expect(result[0].active).toBe(false);
  });

  // ── switchCharacter ─────────────────────────────────────────────────────────

  it('switchCharacter passes character name', async () => {
    const expected = makeAnalysisResult({ build_id: 'char-1' });
    mockInvoke.mockResolvedValueOnce(expected);

    const result = await bridgeModule.switchCharacter('MyInquisitor');

    expect(mockInvoke).toHaveBeenCalledWith('switch_character', { characterName: 'MyInquisitor' });
    expect(result.build_id).toBe('char-1');
  });

  // ── analyzeMapMods ──────────────────────────────────────────────────────────

  it('analyzeMapMods serialises mods and analysis', async () => {
    const mockResult = {
      mods: [{ mod_text: 'No Regen', level: 'Critical', reason: 'RF requires regen' }],
      worst: 'Critical',
      verdict: 'Skip',
      fatal_mods: ['No Regen'],
      total_score: 100,
    };
    mockInvoke.mockResolvedValueOnce(mockResult);

    const analysis = makeAnalysisResult();
    const mods = ['Players cannot regenerate Life, Mana or Energy Shield'];
    const result = await bridgeModule.analyzeMapMods(mods, analysis);

    expect(mockInvoke).toHaveBeenCalledWith('analyze_map_mods', {
      mapModsJson: JSON.stringify(mods),
      analysisJson: JSON.stringify(analysis),
    });
    expect(result.verdict).toBe('Skip');
    expect(result.fatal_mods).toHaveLength(1);
  });

  // ── setPriceAlert ───────────────────────────────────────────────────────────

  it('setPriceAlert serialises alert to JSON', async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    const alert = {
      id: 'a1',
      item_name: "Watcher's Eye",
      condition: { Below: 15.0 },
      active: true,
    };
    await bridgeModule.setPriceAlert(alert as any);

    expect(mockInvoke).toHaveBeenCalledWith('set_price_alert', {
      alertJson: JSON.stringify(alert),
    });
  });

  // ── listPriceAlerts ─────────────────────────────────────────────────────────

  it('listPriceAlerts returns array from backend', async () => {
    const mockAlerts = [
      { id: '1', item_name: "Watcher's Eye", condition: { Below: 15.0 }, active: true },
    ];
    mockInvoke.mockResolvedValueOnce(mockAlerts);

    const result = await bridgeModule.listPriceAlerts();

    expect(mockInvoke).toHaveBeenCalledWith('list_price_alerts');
    expect(result).toHaveLength(1);
  });

  // ── removePriceAlert ────────────────────────────────────────────────────────

  it('removePriceAlert passes alert id string', async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await bridgeModule.removePriceAlert('42');

    expect(mockInvoke).toHaveBeenCalledWith('remove_price_alert', { alertId: '42' });
  });

  // ── calculateManaReservation ────────────────────────────────────────────────

  it('calculateManaReservation serialises skills and player stats', async () => {
    const mockResult = {
      skills: [{ name: 'Determination', base_reservation: 35, effective_reservation: 350, is_percentage: true }],
      total_reserved: 350,
      free_mana: 650,
      effective_pool: 1000,
      over_reserved: false,
      reservation_pct_of_pool: 35,
    };
    mockInvoke.mockResolvedValueOnce(mockResult);

    const skills = [{ name: 'Determination', base_reservation: 35, is_percentage: true, tags: ['aura'] }];
    const player = { max_mana: 1000, max_es: 0, reservation_efficiency: 100, increased_mana_reservation: 0, reduced_mana_reservation: 0, main_skill_mana_cost: 10, has_eldritch_battery: false };
    const result = await bridgeModule.calculateManaReservation(skills, player);

    expect(mockInvoke).toHaveBeenCalledWith('calculate_mana_reservation', {
      skillsJson: JSON.stringify(skills),
      playerJson: JSON.stringify(player),
    });
    expect(result.free_mana).toBe(650);
    expect(result.over_reserved).toBe(false);
  });

  // ── generateShareCode ───────────────────────────────────────────────────────

  it('generateShareCode serialises payload and returns code string', async () => {
    mockInvoke.mockResolvedValueOnce('pofai:ABC123');

    const payload = { version: 1, build_id: 'b1', build_name: 'RF', class_name: 'Templar', ascendancy: 'Inquisitor', level: 90, archetype: 'fire_dot', tree_nodes: [1, 2], gems: ['RF'], total_dps: 2e6, total_life: 5000 };
    const code = await bridgeModule.generateShareCode(payload);

    expect(mockInvoke).toHaveBeenCalledWith('generate_share_code', {
      payloadJson: JSON.stringify(payload),
    });
    expect(code).toBe('pofai:ABC123');
  });

  // ── importShareCode ─────────────────────────────────────────────────────────

  it('importShareCode passes code string and returns payload', async () => {
    const mockPayload = { version: 1, build_id: 'b1', build_name: 'RF', class_name: 'Templar', ascendancy: 'Inquisitor', level: 90, archetype: 'fire_dot', tree_nodes: [], gems: [], total_dps: 0, total_life: 0 };
    mockInvoke.mockResolvedValueOnce(mockPayload);

    const result = await bridgeModule.importShareCode('pofai:ABC123');

    expect(mockInvoke).toHaveBeenCalledWith('import_share_code', { code: 'pofai:ABC123' });
    expect(result.build_name).toBe('RF');
  });

  // ── saveSettings ────────────────────────────────────────────────────────────

  it('saveSettings serialises settings to JSON', async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    const settings = { league: 'Settlers', default_boss: 'Shaper', price_refresh_secs: 300, price_currency: 'divine' as const, sound_enabled: true, pob_watch_dir: '' };

    await bridgeModule.saveSettings(settings);

    expect(mockInvoke).toHaveBeenCalledWith('save_settings', { settingsJson: JSON.stringify(settings) });
  });

  it('loadSettings returns AppSettings from backend', async () => {
    const mockSettings = { league: 'HC', default_boss: 'Maven', price_refresh_secs: 60, price_currency: 'chaos' as const, sound_enabled: false, pob_watch_dir: '/tmp/pob' };
    mockInvoke.mockResolvedValueOnce(mockSettings);

    const result = await bridgeModule.loadSettings();

    expect(mockInvoke).toHaveBeenCalledWith('load_settings');
    expect(result.league).toBe('HC');
    expect(result.sound_enabled).toBe(false);
  });

  // ── simulateBoss ─────────────────────────────────────────────────────────────

  it('simulateBoss passes boss_id and player/defense/offense JSON', async () => {
    const mockResult = { clear_time_ms: 12000, kills: 1, deaths: 0, ticks: 1200 };
    mockInvoke.mockResolvedValueOnce(mockResult);

    const result = await bridgeModule.simulateBoss('{}', '{}', '{}', 'shaper');

    expect(mockInvoke).toHaveBeenCalledWith('simulate_boss', {
      playerJson: '{}', defenseJson: '{}', offenseJson: '{}', bossId: 'shaper',
    });
    expect(result.kills).toBe(1);
  });

  // ── simulateMapClear ──────────────────────────────────────────────────────────

  it('simulateMapClear passes mapTier and optional monsterCount', async () => {
    const mockResult = { clear_time_ms: 45000, kills: 100, deaths: 0, ticks: 4500 };
    mockInvoke.mockResolvedValueOnce(mockResult);

    const result = await bridgeModule.simulateMapClear('{}', '{}', '{}', 16, 100);

    expect(mockInvoke).toHaveBeenCalledWith('simulate_map_clear', {
      playerJson: '{}', defenseJson: '{}', offenseJson: '{}', mapTier: 16, monsterCount: 100,
    });
    expect(result.kills).toBe(100);
  });
});

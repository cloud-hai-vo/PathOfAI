/**
 * Tests for bridge.ts — typed Tauri invoke() wrappers.
 * Mocks the Tauri API so tests run in Node/jsdom without a Tauri process.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { AnalysisResult, PriceResult, BuildSummary } from '../../types/index.js';

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
    archetype: 'RFInquisitor',
    issues: [],
    suggestions: [],
    item_scores: [],
    defense: {
      life: 5000,
      energy_shield: 0,
      armour: 10000,
      evasion: 0,
      effective_hp: 5000,
      fire_res: 75,
      cold_res: 75,
      lightning_res: 75,
      chaos_res: -60,
      block: 0,
    },
    offense: {
      total_dps: 1_500_000,
      hit_dps: 0,
      ailment_dps: 1_500_000,
      skill_name: 'Righteous Fire',
      skill_is_aoe: true,
    },
    score: 72,
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
        cache_age_secs: 0,
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
});

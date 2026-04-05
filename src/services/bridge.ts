/**
 * Typed invoke() wrappers — all backend calls go through here.
 * Never call invoke() directly from components.
 */
import { invoke } from '@tauri-apps/api/core';
import type {
  AnalysisResult, SeerResponse, PriceResult, AppInfo,
  Item, CraftSuggestion, BuildSummary, CraftVsBuyResult,
  SimResult, BuildComparison, BuildSnapshot, WealthSummary, StashItem,
  MapRun, MapStats, PriceAlert, AlertFired, MapDangerResult,
  ReservationSkill, PlayerReservationStats, ReservationResult,
  SharePayload, BuildData,
  StatCheckResult, RecipeCandidate, RecipeAnalysis,
} from '../types/index.js';

// --- Info ---

export const getVersion = (): Promise<string> =>
  invoke('get_version');

export const getAppInfo = (): Promise<AppInfo> =>
  invoke('get_app_info');

// --- Build ---

export const analyzeBuild = (filePath: string): Promise<AnalysisResult> =>
  invoke('analyze_build', { filePath });

export const refreshAnalysis = (buildId: string): Promise<AnalysisResult> =>
  invoke('refresh_analysis', { buildId });

export const undoLastChange = (buildId: string): Promise<AnalysisResult> =>
  invoke('undo_last_change', { buildId });

export const redoChange = (buildId: string): Promise<AnalysisResult> =>
  invoke('redo_change', { buildId });

export const listBuilds = (): Promise<BuildSummary[]> =>
  invoke('list_builds');

// --- Character (OAuth) ---

export const loadCharacter = (characterName: string): Promise<AnalysisResult> =>
  invoke('load_character', { characterName });

export const listCharacters = (): Promise<Array<{name: string; class: string; level: number; league: string}>> =>
  invoke('list_characters');

export const startOAuth = (): Promise<string> =>
  invoke('start_oauth');

export const getAuthStatus = (): Promise<boolean> =>
  invoke('get_auth_status');

// --- Seer ---

export const askSeer = (question: string, buildId: string): Promise<SeerResponse> =>
  invoke('ask_seer', { question, buildId });

export const getCraftSuggestions = (buildId: string, currencyJson: string): Promise<CraftSuggestion[]> =>
  invoke('get_craft_suggestions', { buildId, currencyJson });

export const compareCraftVsBuy = (buildId: string, slot: string, buyPriceDiv: number): Promise<CraftVsBuyResult> =>
  invoke('compare_craft_vs_buy', { buildId, slot, buyPriceDiv });

// --- Market ---

export const getPrices = (itemNames: string[]): Promise<PriceResult[]> =>
  invoke('get_prices', { itemNames });

export const searchUpgrades = (slot: string, budgetDiv: number, buildId: string) =>
  invoke('search_upgrades', { slot, budgetDiv, buildId });

export interface PricePoint {
  price_divine: number;
}

export interface BuyRecommendation {
  action: 'Wait' | 'BuySoon' | 'BuyNow' | 'BuyNowOrWait' | 'BuyWhenReady' | 'Monitor';
  reason: string;
  urgency: 'None' | 'Low' | 'Medium' | 'High';
  confidence: 'Low' | 'Medium' | 'High';
  current_div: number;
  trend: 'DroppingFast' | 'DroppingSlow' | 'Stable' | 'RisingSlow' | 'RisingFast';
  change_7d: number;
  league_phase: string;
  sparkline: number[];
}

export type LeaguePhase = 'LaunchFrenzy' | 'CrashPeriod' | 'Stabilization' | 'PeakEconomy' | 'LateLeague';

export const getBuyRecommendation = (
  itemKey: string,
  history: PricePoint[],
  leaguePhase: LeaguePhase,
): Promise<BuyRecommendation> =>
  invoke('get_buy_recommendation', {
    itemKey,
    historyJson: JSON.stringify(history),
    leaguePhase,
  });

// --- Items ---

export const parseClipboardItem = (clipboardText: string, buildId: string): Promise<{item: Item; score: number}> =>
  invoke('parse_clipboard_item', { clipboardText, buildId });

export const applyUpgrade = (suggestionId: string, buildId: string): Promise<AnalysisResult> =>
  invoke('apply_upgrade', { suggestionId, buildId });

export type StatType =
  | 'FlatLife' | 'PercentLife' | 'FireDotMulti' | 'FlatPhysDamage'
  | 'AttackSpeed' | 'CritChance' | 'CritMultiplier'
  | 'FireRes' | 'ColdRes' | 'LightningRes';

export interface EstimateResult {
  dps_change: number;
  life_change: number;
  is_estimate: boolean;
}

/** Resolve an item image URL (CDN/disk cache — Algorithm 51). */
export const resolveItemImage = (
  itemType: 'unique' | 'base' | 'gem' | 'currency',
  itemName: string,
): Promise<string> =>
  invoke('resolve_item_image', { itemType, itemName });

/** Fast item swap estimate using Algorithm 25 ImpactTable. */
export const estimateItemSwap = (
  buildId: string,
  newItemMods: Array<[StatType, number]>,
  currentItemMods: Array<[StatType, number]>,
): Promise<EstimateResult> =>
  invoke('estimate_item_swap', {
    buildId,
    newItemJson:     JSON.stringify(newItemMods),
    currentItemJson: JSON.stringify(currentItemMods),
  });

// --- Simulation & Analysis ---

export const runSimulation = (
  playerJson: string,
  defenseJson: string,
  offenseJson: string,
  monstersJson: string,
  flasksJson: string,
): Promise<SimResult> =>
  invoke('run_simulation', { playerJson, defenseJson, offenseJson, monstersJson, flasksJson });

export const compareBuilds = (buildA: BuildSnapshot, buildB: BuildSnapshot): Promise<BuildComparison> =>
  invoke('compare_builds_cmd', {
    buildAJson: JSON.stringify(buildA),
    buildBJson: JSON.stringify(buildB),
  });

export const tallyStashWealth = (items: StashItem[], divinePriceC: number): Promise<WealthSummary> =>
  invoke('tally_stash_wealth', {
    itemsJson: JSON.stringify(items),
    divinePriceC,
  });

export const getMapStats = (runs: MapRun[]): Promise<MapStats> =>
  invoke('get_map_stats', { runsJson: JSON.stringify(runs) });

export const checkPriceAlerts = (
  alerts: PriceAlert[],
  prices: Record<string, number>,
): Promise<AlertFired[]> =>
  invoke('check_price_alerts', {
    alertsJson: JSON.stringify(alerts),
    pricesJson: JSON.stringify(prices),
  });

export const deactivatePriceAlert = (alerts: PriceAlert[], alertId: string): Promise<PriceAlert[]> =>
  invoke('deactivate_price_alert', {
    alertsJson: JSON.stringify(alerts),
    alertId,
  });

export const switchCharacter = (characterName: string): Promise<AnalysisResult> =>
  invoke('switch_character', { characterName });

export const analyzeMapMods = (
  mapMods: string[],
  analysis: AnalysisResult,
): Promise<MapDangerResult> =>
  invoke('analyze_map_mods', {
    mapModsJson: JSON.stringify(mapMods),
    analysisJson: JSON.stringify(analysis),
  });

export const setPriceAlert = (alert: PriceAlert): Promise<void> =>
  invoke('set_price_alert', { alertJson: JSON.stringify(alert) });

export const listPriceAlerts = (): Promise<PriceAlert[]> =>
  invoke('list_price_alerts');

export const removePriceAlert = (alertId: string): Promise<void> =>
  invoke('remove_price_alert', { alertId });

export const calculateManaReservation = (
  skills: ReservationSkill[],
  player: PlayerReservationStats,
): Promise<ReservationResult> =>
  invoke('calculate_mana_reservation', {
    skillsJson: JSON.stringify(skills),
    playerJson: JSON.stringify(player),
  });

export const generateShareCode = (payload: SharePayload): Promise<string> =>
  invoke('generate_share_code', { payloadJson: JSON.stringify(payload) });

export const importShareCode = (code: string): Promise<SharePayload> =>
  invoke('import_share_code', { code });

export const checkStatRequirements = (
  build: BuildData,
  candidate?: Item,
): Promise<StatCheckResult> =>
  invoke('check_stat_requirements', {
    buildJson:     JSON.stringify(build),
    candidateJson: candidate ? JSON.stringify(candidate) : null,
  });

export const detectVendorRecipes = (items: RecipeCandidate[]): Promise<RecipeAnalysis> =>
  invoke('detect_vendor_recipes', { itemsJson: JSON.stringify(items) });

// --- Settings / Cloud AI ---

export interface ConnectionTestResult {
  provider: string;
  success: boolean;
  latency_ms: number;
  error?: string;
}

export const testCloudAi = (provider: string, apiKey: string): Promise<ConnectionTestResult> =>
  invoke('test_cloud_ai', { provider, apiKey });

export const saveAiKey = (provider: string, apiKey: string): Promise<void> =>
  invoke('save_ai_key', { provider, apiKey });

export const removeAiKey = (provider: string): Promise<void> =>
  invoke('remove_ai_key', { provider });

export const getConfiguredProviders = (): Promise<string[]> =>
  invoke('get_configured_providers');

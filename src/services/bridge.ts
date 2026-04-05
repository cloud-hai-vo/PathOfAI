/**
 * Typed invoke() wrappers — all backend calls go through here.
 * Never call invoke() directly from components.
 */
import { invoke } from '@tauri-apps/api/core';
import type {
  AnalysisResult, SeerResponse, PriceResult, AppInfo,
  Item, CraftSuggestion, BuildSummary, CraftVsBuyResult,
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

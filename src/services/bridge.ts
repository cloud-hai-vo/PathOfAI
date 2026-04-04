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

// --- Items ---

export const parseClipboardItem = (clipboardText: string, buildId: string): Promise<{item: Item; score: number}> =>
  invoke('parse_clipboard_item', { clipboardText, buildId });

export const applyUpgrade = (suggestionId: string, buildId: string): Promise<AnalysisResult> =>
  invoke('apply_upgrade', { suggestionId, buildId });

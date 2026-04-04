/**
 * Typed invoke() wrappers — all backend calls go through here.
 * Never call invoke() directly from components.
 */
import { invoke } from '@tauri-apps/api/core';
import type {
  AnalysisResult, SeerResponse, PriceResult, AppInfo,
  Item, ItemScore
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

// --- Character (OAuth) ---

export const loadCharacter = (characterName: string): Promise<AnalysisResult> =>
  invoke('load_character', { characterName });

export const listCharacters = (): Promise<Array<{name: string; class: string; level: number; league: string}>> =>
  invoke('list_characters');

// --- Seer ---

export const askSeer = (question: string, buildId: string): Promise<SeerResponse> =>
  invoke('ask_seer', { question, buildId });

// --- Market ---

export const getPrices = (itemNames: string[]): Promise<PriceResult[]> =>
  invoke('get_prices', { itemNames });

// --- Items ---

export const parseClipboardItem = (clipboardText: string, buildId: string): Promise<{item: Item; score: number}> =>
  invoke('parse_clipboard_item', { clipboardText, buildId });

export const applyUpgrade = (suggestionId: string, buildId: string): Promise<AnalysisResult> =>
  invoke('apply_upgrade', { suggestionId, buildId });

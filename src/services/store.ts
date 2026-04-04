/**
 * Application state store — single source of truth for the frontend.
 * Simple observable pattern — no framework needed.
 */
import type { AnalysisResult, AppInfo } from '../types/index.js';

export interface AppState {
  appInfo: AppInfo | null;
  analysis: AnalysisResult | null;
  activePanel: string;
  activeItemSlot: string | null;
  isLoading: boolean;
  loadingMessage: string;
  error: string | null;
}

type Listener = (state: AppState) => void;

class Store {
  private state: AppState = {
    appInfo: null,
    analysis: null,
    activePanel: 'prophecy',
    activeItemSlot: null,
    isLoading: true,
    loadingMessage: 'Consulting the Seer…',
    error: null,
  };

  private listeners: Set<Listener> = new Set();

  get(): AppState {
    return this.state;
  }

  set(patch: Partial<AppState>): void {
    this.state = { ...this.state, ...patch };
    this.notify();
  }

  subscribe(listener: Listener): () => void {
    this.listeners.add(listener);
    listener(this.state); // immediate call with current state
    return () => this.listeners.delete(listener);
  }

  private notify(): void {
    this.listeners.forEach(fn => fn(this.state));
  }
}

export const store = new Store();

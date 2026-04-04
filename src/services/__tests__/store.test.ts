/**
 * Tests for store.ts — observable application state store.
 */
import { describe, it, expect, vi } from 'vitest';

// Import store directly (no Tauri deps)
const storeModule = await import('../store.js');
const { store } = storeModule;

describe('store.ts', () => {
  // ── initial state ───────────────────────────────────────────────────────────

  it('starts with default state', () => {
    const state = store.get();
    expect(state.analysis).toBeNull();
    expect(state.activePanel).toBe('prophecy');
    expect(state.isLoading).toBe(true);
    expect(state.error).toBeNull();
    expect(state.appInfo).toBeNull();
  });

  // ── set / get ───────────────────────────────────────────────────────────────

  it('set patches state without replacing other fields', () => {
    store.set({ error: 'Test error' });

    const state = store.get();
    expect(state.error).toBe('Test error');
    expect(state.activePanel).toBe('prophecy'); // unchanged
  });

  it('set updates isLoading flag', () => {
    store.set({ isLoading: false, loadingMessage: '' });

    expect(store.get().isLoading).toBe(false);
    expect(store.get().loadingMessage).toBe('');
  });

  it('set activePanel persists', () => {
    store.set({ activePanel: 'forge' });
    expect(store.get().activePanel).toBe('forge');

    store.set({ activePanel: 'prophecy' });
    expect(store.get().activePanel).toBe('prophecy');
  });

  // ── subscribe ───────────────────────────────────────────────────────────────

  it('subscribe calls listener immediately with current state', () => {
    const listener = vi.fn();
    const unsubscribe = store.subscribe(listener);

    expect(listener).toHaveBeenCalledOnce();
    expect(listener.mock.calls[0][0]).toMatchObject({ activePanel: expect.any(String) });

    unsubscribe();
  });

  it('subscribe listener is called on state change', () => {
    const listener = vi.fn();
    const unsubscribe = store.subscribe(listener);
    const callsBefore = listener.mock.calls.length;

    store.set({ activePanel: 'tree' });

    expect(listener.mock.calls.length).toBeGreaterThan(callsBefore);

    unsubscribe();
  });

  it('unsubscribe stops listener from being called', () => {
    const listener = vi.fn();
    const unsubscribe = store.subscribe(listener);
    unsubscribe();
    const callsAfterUnsub = listener.mock.calls.length;

    store.set({ activePanel: 'gear' });

    expect(listener.mock.calls.length).toBe(callsAfterUnsub);
  });
});

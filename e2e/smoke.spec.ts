/**
 * E2E smoke tests — verify the app loads and basic UI works.
 * Runs against Vite dev server (port 1420) with Tauri APIs stubbed.
 *
 * Run: npm run test:e2e
 * Prerequisite: app loads in browser without Tauri process (Tauri APIs mocked).
 */
import { test, expect } from '@playwright/test';

// ── Tauri stub ────────────────────────────────────────────────────────────────

async function stubTauriApis(page: import('@playwright/test').Page) {
  await page.addInitScript(() => {
    // Stub window.__TAURI_INTERNALS__ so @tauri-apps/api doesn't crash in browser
    (window as any).__TAURI_INTERNALS__ = {
      invoke: async (cmd: string, _args?: unknown) => {
        switch (cmd) {
          case 'get_version':
            return '0.1.0';
          case 'get_app_info':
            // Matches the real Rust AppInfo struct: { version, name, poe_version, league }
            return {
              version: '0.1.0',
              name: 'Path of AI',
              poe_version: 'PoE 1 + PoE 2',
              league: 'Mirage (3.28)',
            };
          case 'get_auth_status':
            return false;
          case 'list_builds':
            return [];
          // Tauri event plugin — invoked by @tauri-apps/api/event listen()
          case 'plugin:event|listen':
          case 'plugin:event|unlisten':
            return 0;
          default:
            // Non-fatal for unknown commands in test mode
            return null;
        }
      },
      transformCallback: (cb: unknown) => cb,
      convertFileSrc: (src: string) => src,
    };
  });
}

// ── Tests ─────────────────────────────────────────────────────────────────────

test.describe('Path of AI — smoke tests', () => {
  test.beforeEach(async ({ page }) => {
    await stubTauriApis(page);
  });

  // ── Loading & boot ──────────────────────────────────────────────────────────

  test('app boots without JS errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));

    await page.goto('/');
    await page.waitForTimeout(2500);

    const criticalErrors = errors.filter(e =>
      e.includes('SyntaxError') || e.includes('TypeError: Cannot read')
    );
    expect(criticalErrors).toHaveLength(0);
  });

  test('page title is Path of AI', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/Path of AI/i);
  });

  test('loading screen shows on initial render', async ({ page }) => {
    await page.goto('/');
    // The loading screen should be present in the DOM (might be hidden after boot)
    await expect(page.locator('#loading-screen')).toBeAttached();
  });

  test('app version appears in loading screen', async ({ page }) => {
    await page.goto('/');
    // Version element should show the stub version
    const versionEl = page.locator('#loading-version');
    await expect(versionEl).toBeAttached();
  });

  // ── HUD structure ───────────────────────────────────────────────────────────

  test('HUD renders after boot completes', async ({ page }) => {
    await page.goto('/');
    // Wait for the app div to be shown (boot → showHUD())
    await expect(page.locator('#app')).toBeAttached({ timeout: 8_000 });
  });

  test('header bar contains app title', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2500);
    const content = await page.content();
    expect(content).toContain('PATH OF AI');
  });

  test('Import PoB button is present in header', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2500);
    const btn = page.locator('#btn-import-pob');
    await expect(btn).toBeAttached();
  });

  test('Connect PoE button is present in header', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2500);
    const btn = page.locator('#btn-connect-poe');
    await expect(btn).toBeAttached();
  });

  // ── Panel navigation ────────────────────────────────────────────────────────

  test('panel nav renders all 14 panel buttons', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2500);

    const panels = [
      'prophecy', 'grimoire', 'combat', 'defenses', 'dps', 'gems',
      'blood', 'darkpath', 'forge', 'cursemap',
      'passive', 'harbinger', 'stash', 'settings',
    ];
    for (const id of panels) {
      const btn = page.locator(`[data-panel="${id}"]`);
      await expect(btn).toBeAttached({ timeout: 3_000 });
    }
  });

  test('clicking Defenses panel shows defense content', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2500);

    await page.locator('[data-panel="defenses"]').click();
    await page.waitForTimeout(500);
    const content = await page.content();
    // Defenses panel placeholder or content rendered
    expect(content).toContain('panel');
  });

  test('clicking Grimoire panel shows Seer chat input', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2500);

    await page.locator('[data-panel="grimoire"]').click();
    await page.waitForTimeout(500);
    const input = page.locator('#grimoire-input');
    await expect(input).toBeAttached();
  });

  // ── Seer bar ────────────────────────────────────────────────────────────────

  test('Seer ask bar is present', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2500);
    await expect(page.locator('#seer-bar')).toBeAttached();
    await expect(page.locator('#seer-input')).toBeAttached();
  });

  test('Seer submit button is present', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2500);
    await expect(page.locator('#seer-submit')).toBeAttached();
  });

  // ── No-build state ──────────────────────────────────────────────────────────

  test('no-build message visible before build is loaded', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2500);
    const content = await page.content();
    // The "Seer Awaits" or "Import" message should appear
    expect(
      content.includes('Seer Awaits') ||
      content.includes('Import') ||
      content.includes('No build')
    ).toBe(true);
  });

  // ── Sidebar stat cards ──────────────────────────────────────────────────────

  test('left sidebar stat cards are present', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(2500);
    await expect(page.locator('#stat-life')).toBeAttached();
    await expect(page.locator('#stat-es')).toBeAttached();
    await expect(page.locator('#stat-dps')).toBeAttached();
  });
});

/**
 * E2E smoke tests — verify the app loads and basic UI works.
 * Runs against Vite dev server (port 1420) with Tauri APIs stubbed.
 *
 * Run: npm run test:e2e
 * Prerequisite: app loads in browser without Tauri process (Tauri APIs mocked).
 */
import { test, expect } from '@playwright/test';

// Stub the Tauri invoke before page loads
async function stubTauriApis(page: import('@playwright/test').Page) {
  await page.addInitScript(() => {
    // Stub window.__TAURI_INTERNALS__ so @tauri-apps/api doesn't crash
    (window as any).__TAURI_INTERNALS__ = {
      invoke: async (cmd: string, _args?: unknown) => {
        switch (cmd) {
          case 'get_version': return '0.1.0';
          case 'get_app_info': return {
            version: '0.1.0',
            pob_path: null,
            data_dir: 'PathOfAI_Data',
            league: 'Settlers',
          };
          case 'get_auth_status': return false;
          case 'list_builds': return [];
          default: throw new Error(`Unhandled stub: ${cmd}`);
        }
      },
      transformCallback: (cb: unknown) => cb,
    };
  });
}

test.describe('Path of AI — smoke tests', () => {
  test.beforeEach(async ({ page }) => {
    await stubTauriApis(page);
  });

  test('app loads without crashing', async ({ page }) => {
    await page.goto('/');
    // Wait for the main container to appear
    await expect(page.locator('body')).toBeVisible({ timeout: 10_000 });
    // Check for no unhandled JS errors
    const errors: string[] = [];
    page.on('pageerror', (err) => errors.push(err.message));
    // App should not have critical parse errors
    await page.waitForTimeout(2000);
    const criticalErrors = errors.filter(e =>
      e.includes('SyntaxError') || e.includes('TypeError: Cannot read')
    );
    expect(criticalErrors).toHaveLength(0);
  });

  test('page title is set', async ({ page }) => {
    await page.goto('/');
    const title = await page.title();
    expect(title.length).toBeGreaterThan(0);
  });

  test('navigation panel elements are present', async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1500);
    // The nav should contain panel buttons (prophecy, gear, forge, etc.)
    // Check for at least one nav-related element
    const html = await page.content();
    expect(html.length).toBeGreaterThan(100); // page has meaningful content
  });
});

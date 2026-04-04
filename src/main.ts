/**
 * Path of AI — main entry point
 * Initializes the backend connection, loads the HUD, wires all panels.
 */
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { store } from './services/store.js';
import { getAppInfo, analyzeBuild, askSeer } from './services/bridge.js';
import type { AnalysisResult, SeerResponse } from './types/index.js';

// ─── Boot sequence ────────────────────────────────────────────────────────────

async function boot() {
  setLoadingProgress(20, 'Loading app info…');

  try {
    const appInfo = await getAppInfo();
    store.set({ appInfo });

    setLoadingProgress(50, 'Connecting to Path of Exile…');

    // Update version display
    const versionEl = document.getElementById('loading-version');
    if (versionEl) versionEl.textContent = `v${appInfo.version}`;

    setLoadingProgress(80, 'Ready.');

    // Register Tauri event listeners
    await registerBackendEvents();

    setLoadingProgress(100, 'Ready.');

    // Small delay so the loading bar reaches 100% visually
    await delay(300);

    showHUD();
    renderHUD();

  } catch (err) {
    showError(`Failed to connect to backend: ${err}`);
  }
}

function setLoadingProgress(pct: number, message: string) {
  const bar = document.getElementById('loading-bar');
  const subtitle = document.querySelector('.loading-subtitle');
  if (bar) bar.style.width = `${pct}%`;
  if (subtitle) subtitle.textContent = message;
}

function showHUD() {
  const loading = document.getElementById('loading-screen');
  const app = document.getElementById('app');
  if (loading) loading.style.display = 'none';
  if (app) { app.style.display = 'flex'; app.style.flexDirection = 'column'; }
}

function showError(message: string) {
  store.set({ error: message, isLoading: false });
  const subtitle = document.querySelector('.loading-subtitle');
  if (subtitle) {
    subtitle.textContent = `⚠ ${message}`;
    (subtitle as HTMLElement).style.color = 'var(--danger)';
  }
}

async function registerBackendEvents() {
  await listen('analysis-complete', (event) => {
    const result = event.payload as AnalysisResult;
    store.set({ analysis: result, isLoading: false });
    renderAnalysis();
  });

  await listen('pob-file-changed', () => {
    showNotification('PoB file changed — re-analyzing…');
  });

  await listen('price-alert-triggered', (event: any) => {
    showNotification(`🔔 ${event.payload.itemName} hit your price alert!`);
  });
}

// ─── HUD Rendering ────────────────────────────────────────────────────────────

function renderHUD() {
  const app = document.getElementById('app')!;

  app.innerHTML = `
    <!-- Header bar -->
    <div class="hud-header" id="hud-header">
      <div class="header-left">
        <img src="assets/icon.svg" width="28" height="28" alt="Path of AI" />
        <span class="header-title">PATH OF AI</span>
        <span class="header-version" id="header-version"></span>
      </div>
      <div class="header-center" id="header-build-info">
        <span class="header-build-name" id="header-build-name">No build loaded</span>
        <span class="header-build-meta" id="header-build-meta"></span>
      </div>
      <div class="header-right">
        <button class="hud-btn" id="btn-import-pob" title="Import PoB file">📁 Import PoB</button>
        <button class="hud-btn" id="btn-connect-poe" title="Connect PoE Account">⚡ Connect PoE</button>
        <button class="hud-btn" id="btn-settings" title="Settings">⚙</button>
      </div>
    </div>

    <!-- Main HUD area -->
    <div class="hud-main">
      <!-- Left stat sidebar -->
      <div class="sidebar-left" id="sidebar-left">
        <div class="stat-card" id="stat-life">
          <div class="stat-label">❤ Life</div>
          <div class="stat-value life" id="val-life">—</div>
        </div>
        <div class="stat-card" id="stat-es">
          <div class="stat-label">🔵 ES</div>
          <div class="stat-value es" id="val-es">—</div>
        </div>
        <div class="stat-card" id="stat-dps">
          <div class="stat-label">💥 DPS</div>
          <div class="stat-value dps" id="val-dps">—</div>
        </div>
        <div class="stat-card resistances">
          <div class="stat-label">Resistances</div>
          <div class="res-row fire">🔥 <span id="val-res-fire">—</span></div>
          <div class="res-row cold">❄ <span id="val-res-cold">—</span></div>
          <div class="res-row lightning">⚡ <span id="val-res-lightning">—</span></div>
          <div class="res-row chaos">☠ <span id="val-res-chaos">—</span></div>
        </div>
        <div class="stat-card score-card">
          <div class="stat-label">Build Score</div>
          <div class="stat-value score" id="val-score">—</div>
        </div>
      </div>

      <!-- Center: character + passive tree mini -->
      <div class="hud-center" id="hud-center">
        <div class="no-build-message" id="no-build-msg">
          <img src="assets/logo.svg" width="120" height="120" alt="" />
          <div style="font-family:'Cinzel',serif;font-size:18px;color:var(--text-bright);margin:16px 0 8px;">
            The Seer Awaits
          </div>
          <div style="font-size:12px;color:var(--text-muted);margin-bottom:24px;max-width:300px;text-align:center;line-height:1.6;">
            Import a Path of Building file or connect your PoE account to begin your consultation.
          </div>
          <button class="action-btn primary" id="btn-import-center">📁 Import from PoB</button>
          <button class="action-btn" id="btn-connect-center" style="margin-top:8px;">⚡ Connect PoE Account</button>
        </div>
      </div>

      <!-- Right panel -->
      <div class="sidebar-right" id="sidebar-right">
        <div class="panel-nav" id="panel-nav"></div>
        <div class="panel-content" id="panel-content">
          <div class="panel-placeholder">Select a panel</div>
        </div>
      </div>
    </div>

    <!-- Bottom gem/skill bar -->
    <div class="hud-bottom" id="hud-bottom">
      <div class="bottom-skills" id="bottom-skills"></div>
      <div class="bottom-status" id="bottom-status">Ready — import a build to begin</div>
    </div>

    <!-- Seer ask bar -->
    <div class="seer-bar" id="seer-bar">
      <span class="seer-icon">👁</span>
      <input
        type="text"
        id="seer-input"
        class="seer-input"
        placeholder="Ask the Seer… (e.g. 'what should I upgrade first?')"
        autocomplete="off"
      />
      <button class="seer-submit" id="seer-submit">Invoke →</button>
    </div>
  `;

  // Wire up event handlers
  document.getElementById('btn-import-pob')?.addEventListener('click', importFromPoB);
  document.getElementById('btn-import-center')?.addEventListener('click', importFromPoB);
  document.getElementById('btn-connect-poe')?.addEventListener('click', connectPoE);
  document.getElementById('btn-connect-center')?.addEventListener('click', connectPoE);
  document.getElementById('seer-submit')?.addEventListener('click', submitSeerQuestion);
  document.getElementById('seer-input')?.addEventListener('keydown', (e) => {
    if ((e as KeyboardEvent).key === 'Enter') submitSeerQuestion();
  });

  // Update version in header
  const state = store.get();
  if (state.appInfo) {
    const vEl = document.getElementById('header-version');
    if (vEl) vEl.textContent = `v${state.appInfo.version}`;
  }

  // Build panel nav
  renderPanelNav();

  // Subscribe to store changes
  store.subscribe((state) => {
    if (state.analysis) renderAnalysis();
    if (state.isLoading) {
      const status = document.getElementById('bottom-status');
      if (status) status.textContent = state.loadingMessage;
    }
  });
}

function renderPanelNav() {
  const panels = [
    { id: 'prophecy', icon: '🔮', label: 'Prophecy' },
    { id: 'grimoire', icon: '📖', label: 'Grimoire' },
    { id: 'combat',   icon: '⚔',  label: 'The Arena' },
    { id: 'defenses', icon: '🛡',  label: 'Defenses' },
    { id: 'dps',      icon: '💥', label: 'DPS' },
    { id: 'gems',     icon: '💎', label: 'Gems' },
    { id: 'blood',    icon: '☠',  label: 'Blood Pact' },
    { id: 'darkpath', icon: '🗡', label: 'Dark Path' },
    { id: 'forge',    icon: '🔨', label: 'The Forge' },
    { id: 'cursemap', icon: '🗺', label: 'Curse Map' },
  ];

  const nav = document.getElementById('panel-nav');
  if (!nav) return;

  nav.innerHTML = panels.map(p => `
    <button class="panel-nav-btn" data-panel="${p.id}" title="${p.label}">
      <span class="panel-nav-icon">${p.icon}</span>
      <span class="panel-nav-label">${p.label}</span>
    </button>
  `).join('');

  nav.querySelectorAll('.panel-nav-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      const panelId = (btn as HTMLElement).dataset.panel ?? 'prophecy';
      store.set({ activePanel: panelId });
      nav.querySelectorAll('.panel-nav-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      renderActivePanel(panelId);
    });
  });

  // Activate default
  nav.querySelector('[data-panel="prophecy"]')?.classList.add('active');
  renderActivePanel('prophecy');
}

function renderActivePanel(panelId: string) {
  const content = document.getElementById('panel-content');
  if (!content) return;

  const state = store.get();
  const analysis = state.analysis;

  switch (panelId) {
    case 'prophecy':
      content.innerHTML = renderProphecyPanel(analysis);
      break;
    case 'grimoire':
      content.innerHTML = renderGrimoirePanel();
      wireGrimoirePanel();
      break;
    case 'defenses':
      content.innerHTML = renderDefensesPanel(analysis);
      break;
    case 'dps':
      content.innerHTML = renderDPSPanel(analysis);
      break;
    default:
      content.innerHTML = `<div class="panel-placeholder">
        ${panelId} panel — coming in next session
      </div>`;
  }
}

function renderAnalysis() {
  const { analysis } = store.get();
  if (!analysis) return;

  // Update stat sidebar
  setEl('val-life', analysis.defenses.life.toLocaleString());
  setEl('val-es', analysis.defenses.energy_shield > 0
    ? analysis.defenses.energy_shield.toLocaleString() : '—');
  setEl('val-dps', analysis.offense.dps_label);

  const res = analysis.defenses.resistances;
  setResEl('val-res-fire', res.fire, res.max_fire);
  setResEl('val-res-cold', res.cold, res.max_cold);
  setResEl('val-res-lightning', res.lightning, res.max_lightning);
  setResEl('val-res-chaos', res.chaos, res.max_chaos);
  setEl('val-score', String(analysis.overall_score));

  // Update header
  setEl('header-build-name', analysis.build_name);
  setEl('header-build-meta', `${analysis.class_name} · ${analysis.ascendancy} · Lv${analysis.level}`);

  // Hide no-build message
  const noMsg = document.getElementById('no-build-msg');
  if (noMsg) noMsg.style.display = 'none';

  // Update bottom status
  const issues = analysis.issues.filter(i => i.severity === 'Critical' || i.severity === 'Major');
  setEl('bottom-status',
    issues.length > 0
      ? `⚠ ${issues.length} issue${issues.length > 1 ? 's' : ''} detected — check Prophecy panel`
      : `✓ ${analysis.archetype_label} · Score ${analysis.overall_score}/100`
  );

  // Re-render active panel with new data
  renderActivePanel(store.get().activePanel);
}

// ─── Panel renderers ──────────────────────────────────────────────────────────

function renderProphecyPanel(analysis: AnalysisResult | null): string {
  if (!analysis) {
    return `<div class="panel-placeholder">
      <div style="font-size:24px;margin-bottom:12px;">🔮</div>
      Import a build to see upgrade suggestions and issue detection.
    </div>`;
  }

  const issuesHTML = analysis.issues.map(issue => `
    <div class="issue-card ${issue.severity.toLowerCase()}">
      <div class="issue-severity">${severityIcon(issue.severity)}</div>
      <div class="issue-body">
        <div class="issue-title">${issue.title}</div>
        <div class="issue-detail">${issue.detail}</div>
        <div class="issue-fix">Fix: ${issue.fix}</div>
      </div>
    </div>
  `).join('');

  const suggestionsHTML = analysis.suggestions.slice(0, 5).map((s, i) => `
    <div class="suggestion-card">
      <div class="suggestion-num">${i + 1}</div>
      <div class="suggestion-body">
        <div class="suggestion-title">${s.title}</div>
        <div class="suggestion-detail">${s.detail}</div>
        ${s.estimated_cost_div > 0
          ? `<div class="suggestion-cost">~${s.estimated_cost_div.toFixed(1)} div</div>`
          : ''}
      </div>
      ${s.trade_url
        ? `<button class="suggestion-trade-btn" onclick="window.open('${s.trade_url}')">Trade →</button>`
        : ''}
    </div>
  `).join('');

  return `
    <div class="section-label">Issues (${analysis.issues.length})</div>
    ${issuesHTML || '<div class="panel-placeholder">No issues detected ✓</div>'}
    <div class="section-label" style="margin-top:12px;">Upgrade Suggestions</div>
    ${suggestionsHTML || '<div class="panel-placeholder">No suggestions available</div>'}
  `;
}

function renderGrimoirePanel(): string {
  return `
    <div class="section-label">👁 Ask The Seer</div>
    <div class="seer-chat" id="seer-chat">
      <div class="seer-message system">
        The Seer awaits your question, Exile. Ask about your DPS, resistances, upgrades, crafting, or bosses.
      </div>
    </div>
    <div class="seer-panel-input">
      <input type="text" id="grimoire-input" class="seer-input" placeholder="Ask the Seer…" />
      <button id="grimoire-submit" class="seer-submit">Invoke</button>
    </div>
  `;
}

function wireGrimoirePanel() {
  const submit = () => {
    const input = document.getElementById('grimoire-input') as HTMLInputElement;
    if (!input?.value.trim()) return;
    submitSeerQuestionToPanel(input.value.trim());
    input.value = '';
  };

  document.getElementById('grimoire-submit')?.addEventListener('click', submit);
  document.getElementById('grimoire-input')?.addEventListener('keydown', (e) => {
    if ((e as KeyboardEvent).key === 'Enter') submit();
  });
}

function renderDefensesPanel(analysis: AnalysisResult | null): string {
  if (!analysis) return `<div class="panel-placeholder">Load a build first</div>`;

  const d = analysis.defenses;
  const res = d.resistances;

  return `
    <div class="section-label">⛊ Defense Layers</div>
    <div class="defense-list">
      ${defenseRow('❤', 'Life', d.life.toLocaleString(), 'var(--life)')}
      ${d.energy_shield > 0 ? defenseRow('🔵', 'Energy Shield', d.energy_shield.toLocaleString(), 'var(--es)') : ''}
      ${d.armour > 0 ? defenseRow('🛡', 'Armour', `${d.armour.toLocaleString()} (${(d.armour_phys_reduction*100).toFixed(0)}% vs 5k hit)`, 'var(--text-bright)') : ''}
      ${d.evasion > 0 ? defenseRow('🌀', 'Evasion', `${d.evasion.toLocaleString()} (${(d.evasion_chance*100).toFixed(0)}% chance)`, 'var(--cold)') : ''}
      ${d.block_chance > 0 ? defenseRow('🔰', 'Block', `${(d.block_chance*100).toFixed(0)}% attack / ${(d.spell_block_chance*100).toFixed(0)}% spell`, 'var(--warning)') : ''}
      ${d.life_regen_flat > 0 ? defenseRow('💚', 'Life Regen', `${d.life_regen_flat.toFixed(0)}/s`, 'var(--success)') : ''}
    </div>

    <div class="section-label" style="margin-top:10px;">Resistances</div>
    <div class="res-table">
      ${resTableRow('🔥 Fire', res.fire, res.max_fire)}
      ${resTableRow('❄ Cold', res.cold, res.max_cold)}
      ${resTableRow('⚡ Lightning', res.lightning, res.max_lightning)}
      ${resTableRow('☠ Chaos', res.chaos, res.max_chaos)}
    </div>

    <div class="section-label" style="margin-top:10px;">Ailment Immunity</div>
    <div class="ailment-list">
      ${ailmentRow('Freeze', d.ailment_immunity.freeze, d.ailment_immunity.freeze_source)}
      ${ailmentRow('Shock', d.ailment_immunity.shock, d.ailment_immunity.shock_source)}
      ${ailmentRow('Ignite', d.ailment_immunity.ignite, d.ailment_immunity.ignite_source)}
      ${ailmentRow('Bleed', d.ailment_immunity.bleed, d.ailment_immunity.bleed_source)}
      ${ailmentRow('Corrupted Blood', d.ailment_immunity.corrupted_blood, d.ailment_immunity.corrupted_blood_source)}
    </div>
  `;
}

function renderDPSPanel(analysis: AnalysisResult | null): string {
  if (!analysis) return `<div class="panel-placeholder">Load a build first</div>`;

  const o = analysis.offense;
  return `
    <div class="section-label">Total DPS</div>
    <div class="dps-total">${o.dps_label}</div>
    <div style="font-size:10px;color:var(--text-muted);margin-bottom:12px;">${o.main_skill}</div>

    <div class="section-label">Damage Sources</div>
    ${o.sources.map(s => `
      <div style="margin-bottom:8px;">
        <div style="display:flex;justify-content:space-between;font-size:10px;margin-bottom:2px;">
          <span>${s.source}</span>
          <span style="color:${s.color};font-family:'JetBrains Mono',monospace;font-weight:700;">
            ${formatDps(s.value)} <span style="font-size:8px;color:var(--text-dim);">(${s.percent_of_total.toFixed(0)}%)</span>
          </span>
        </div>
        <div style="height:6px;background:#1a0a04;border-radius:2px;overflow:hidden;">
          <div style="height:100%;width:${s.percent_of_total}%;background:${s.color};border-radius:2px;"></div>
        </div>
      </div>
    `).join('')}
  `;
}

// ─── Actions ──────────────────────────────────────────────────────────────────

async function importFromPoB() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Path of Building', extensions: ['xml'] }],
    });

    if (!selected) return;

    const filePath = typeof selected === 'string' ? selected : selected;
    store.set({ isLoading: true, loadingMessage: 'Analyzing build…' });
    showNotification('Analyzing build…');

    const result = await analyzeBuild(filePath as string);
    store.set({ analysis: result, isLoading: false });
    showNotification(`✓ ${result.build_name} analyzed — Score ${result.overall_score}/100`);

  } catch (err) {
    showNotification(`⚠ Import failed: ${err}`);
    store.set({ isLoading: false });
  }
}

async function connectPoE() {
  showNotification('OAuth flow — coming in Session 8!');
}

async function submitSeerQuestion() {
  const input = document.getElementById('seer-input') as HTMLInputElement;
  if (!input?.value.trim()) return;
  const question = input.value.trim();
  input.value = '';

  // Switch to Grimoire panel
  store.set({ activePanel: 'grimoire' });
  document.querySelector('[data-panel="grimoire"]')?.classList.add('active');
  renderActivePanel('grimoire');

  await submitSeerQuestionToPanel(question);
}

async function submitSeerQuestionToPanel(question: string) {
  const { analysis } = store.get();
  if (!analysis) {
    appendSeerMessage('system', 'Load a build first, Exile.');
    return;
  }

  appendSeerMessage('user', question);
  appendSeerMessage('system', '…consulting the Seer…');

  try {
    const response = await askSeer(question, analysis.build_id);
    // Remove "consulting" message
    const chat = document.getElementById('seer-chat');
    const dots = chat?.querySelector('.consulting');
    dots?.remove();

    appendSeerMessage('seer', response.answer);

    if (response.follow_up_questions.length > 0) {
      appendSeerMessage('followup', response.follow_up_questions.join(' · '));
    }
  } catch (err) {
    const chat = document.getElementById('seer-chat');
    const dots = chat?.querySelector('.consulting');
    dots?.remove();
    appendSeerMessage('system', `The Seer is unavailable: ${err}`);
  }
}

function appendSeerMessage(type: 'user' | 'seer' | 'system' | 'followup' | 'consulting', text: string) {
  const chat = document.getElementById('seer-chat');
  if (!chat) return;

  const div = document.createElement('div');
  div.className = `seer-message ${type}`;
  if (type === 'consulting') div.classList.add('consulting');
  div.textContent = text;
  chat.appendChild(div);
  chat.scrollTop = chat.scrollHeight;
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

function setEl(id: string, value: string) {
  const el = document.getElementById(id);
  if (el) el.textContent = value;
}

function setResEl(id: string, val: number, max: number) {
  const el = document.getElementById(id);
  if (!el) return;
  el.textContent = `${val}%`;
  el.style.color = val >= max ? 'var(--success)' : val >= max - 10 ? 'var(--warning)' : 'var(--danger)';
}

function severityIcon(sev: string): string {
  return { Critical: '💀', Major: '⚠️', Minor: '⚡', Info: 'ℹ' }[sev] ?? '•';
}

function defenseRow(icon: string, label: string, value: string, color: string): string {
  return `<div class="defense-row">
    <span>${icon} ${label}</span>
    <span style="color:${color};font-family:'JetBrains Mono',monospace;">${value}</span>
  </div>`;
}

function resTableRow(label: string, val: number, max: number): string {
  const color = val >= max ? 'var(--success)' : val >= max - 15 ? 'var(--warning)' : 'var(--danger)';
  const overcap = val > max ? ` (+${val - max})` : '';
  return `<div class="res-table-row">
    <span>${label}</span>
    <span style="color:${color};font-family:'JetBrains Mono',monospace;">${Math.min(val,max)}%${overcap}</span>
  </div>`;
}

function ailmentRow(name: string, immune: boolean, source?: string): string {
  return `<div class="ailment-row">
    <span style="color:${immune ? 'var(--success)' : 'var(--danger)'}">${immune ? '✓' : '✗'}</span>
    <span style="color:${immune ? 'var(--text-muted)' : 'var(--text)'}">${name}</span>
    ${source ? `<span style="font-size:9px;color:var(--text-dim)">(${source})</span>` : ''}
  </div>`;
}

function formatDps(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toFixed(0);
}

function showNotification(msg: string) {
  const status = document.getElementById('bottom-status');
  if (status) status.textContent = msg;
}

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// ─── Boot ─────────────────────────────────────────────────────────────────────
boot();

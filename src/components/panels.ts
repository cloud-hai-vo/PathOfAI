/**
 * Panel render functions — one per right-sidebar panel.
 * Each function returns an HTML string given an AnalysisResult (or null).
 * Extracted here so they are testable without a DOM.
 */
import type { AnalysisResult, GemSetup, Gem } from '../types/index.js';
export type { AnalysisResult };

// ── Boss table ────────────────────────────────────────────────────────────────

interface BossEntry {
  name: string;
  hp: number;         // in millions
  ele_res: number;    // 0.0–1.0 fractional
  ttk_cap: number;    // seconds — anything <= this is READY
}

const BOSSES: BossEntry[] = [
  { name: 'T16 Rare',   hp:  4,    ele_res: 0.30, ttk_cap: 3  },
  { name: 'Shaper',     hp: 10,    ele_res: 0.40, ttk_cap: 8  },
  { name: 'Elder',      hp:  9,    ele_res: 0.40, ttk_cap: 8  },
  { name: 'Uber Elder', hp: 17,    ele_res: 0.40, ttk_cap: 12 },
  { name: 'Sirus A9',   hp: 12,    ele_res: 0.40, ttk_cap: 10 },
  { name: 'Maven',      hp: 22,    ele_res: 0.50, ttk_cap: 20 },
  { name: 'The Feared', hp: 60,    ele_res: 0.50, ttk_cap: 60 },
];

function bossReadiness(dps: number, boss: BossEntry): { ttk: number; ready: boolean } {
  const effectiveDps = dps * (1 - boss.ele_res);
  const hpHits = boss.hp * 1_000_000;
  const ttk = effectiveDps > 0 ? hpHits / effectiveDps : Infinity;
  return { ttk, ready: ttk <= boss.ttk_cap };
}

function formatTtk(sec: number): string {
  if (!isFinite(sec)) return '∞';
  if (sec < 60) return `${sec.toFixed(1)}s`;
  return `${Math.floor(sec / 60)}m${(sec % 60).toFixed(0)}s`;
}

// ── Arena (Combat Simulator) ─────────────────────────────────────────────────

export function renderArenaPanel(analysis: AnalysisResult | null): string {
  if (!analysis) return `<div class="panel-placeholder">Load a build first</div>`;

  const dps = analysis.offense.total_dps;

  const rows = BOSSES.map(boss => {
    const { ttk, ready } = bossReadiness(dps, boss);
    const label    = ready ? 'READY'    : 'NOT READY';
    const cssClass = ready ? 'ready'    : 'not-ready';
    const color    = ready ? 'var(--success)' : 'var(--danger)';
    return `
      <div class="boss-row">
        <span class="boss-name">${boss.name}</span>
        <span class="boss-ttk" style="font-family:'JetBrains Mono',monospace;font-size:10px;color:var(--text-muted)">
          TTK ${formatTtk(ttk)}
        </span>
        <span class="boss-status ${cssClass}" style="color:${color};font-weight:700;font-size:10px;">${label}</span>
      </div>
    `;
  }).join('');

  return `
    <div class="section-label">⚔ Combat Stats</div>
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:6px;margin-bottom:10px;">
      <div class="stat-mini"><div style="font-size:9px;color:var(--text-muted)">DPS</div>
        <div style="font-family:'JetBrains Mono',monospace;font-size:14px;color:var(--fire)">${analysis.offense.dps_label}</div></div>
      <div class="stat-mini"><div style="font-size:9px;color:var(--text-muted)">Main Skill</div>
        <div style="font-size:10px;color:var(--text-bright)">${analysis.offense.main_skill}</div></div>
    </div>

    <div class="section-label" style="margin-top:8px;">Boss Readiness</div>
    <div class="boss-list">${rows}</div>
  `;
}

// ── Gems ─────────────────────────────────────────────────────────────────────

const SOCKET_COLOR: Record<string, string> = {
  R: '#d20000', G: '#1ea82e', B: '#1e6ec4', W: '#c8c8c8',
};

function socketDots(colors: string): string {
  return colors.split('').map(c =>
    `<span style="display:inline-block;width:9px;height:9px;border-radius:50%;
      background:radial-gradient(circle at 35% 35%, ${SOCKET_COLOR[c] ?? '#888'} 0%, #111 100%);
      margin-right:2px;"></span>`
  ).join('');
}

function gemBadge(gem: Gem): string {
  if (!gem.is_maxed && gem.level >= 20) return '<span class="gem-badge" style="color:var(--warning)">Level up</span>';
  if (gem.is_awakened)                  return '<span class="gem-badge" style="color:var(--gold)">Awakened</span>';
  if (gem.is_vaal)                      return '<span class="gem-badge" style="color:var(--chaos)">Vaal</span>';
  return '';
}

function renderGemSetup(setup: GemSetup): string {
  const gemsHtml = setup.gems.map(g => `
    <div class="gem-row" style="display:flex;align-items:center;gap:6px;padding:3px 0;border-bottom:1px solid #1a0a04;">
      <span style="font-size:9px;font-family:'JetBrains Mono',monospace;color:var(--text-muted);min-width:28px;">${g.level}/${g.quality}</span>
      <span style="font-size:10px;color:${g.is_support ? 'var(--text-muted)' : 'var(--text-bright)'};">${g.name}</span>
      ${gemBadge(g)}
    </div>
  `).join('');

  return `
    <div class="gem-group-card" style="background:#120704;border:1px solid #3c3630;border-radius:4px;padding:8px;margin-bottom:8px;">
      <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:6px;">
        <span style="font-size:10px;font-weight:700;color:var(--gold);">${setup.skill}</span>
        <span style="font-size:9px;color:var(--text-muted);">${setup.slot}</span>
      </div>
      <div style="margin-bottom:6px;">${socketDots(setup.socket_colors)} <span style="font-size:9px;color:var(--text-dim);margin-left:2px;">${setup.socket_colors}</span></div>
      ${gemsHtml}
    </div>
  `;
}

export function renderGemsPanel(analysis: AnalysisResult | null): string {
  if (!analysis) return `<div class="panel-placeholder">Load a build first</div>`;

  const setups = analysis.gem_setups ?? [];
  if (setups.length === 0) {
    return `<div class="panel-placeholder">No gem setups detected in build</div>`;
  }

  return `
    <div class="section-label">💎 Gem Links</div>
    ${setups.map(renderGemSetup).join('')}
  `;
}

// ── Blood Pact ────────────────────────────────────────────────────────────────

export function renderBloodPactPanel(analysis: AnalysisResult | null): string {
  if (!analysis) return `<div class="panel-placeholder">Load a build first</div>`;

  // Build sworn goals from top issues + top suggestions
  const goalItems = [
    ...analysis.issues.slice(0, 3).map(i => ({
      label: i.title,
      detail: i.fix,
      done: false,
    })),
    ...analysis.suggestions.slice(0, 2).map(s => ({
      label: s.title,
      detail: s.detail,
      done: false,
    })),
  ];

  const goalsHtml = goalItems.length > 0
    ? goalItems.map(g => `
        <div class="check-item" style="display:flex;align-items:flex-start;gap:8px;padding:5px 0;border-bottom:1px solid #1a0a04;">
          <div class="check-box" style="width:14px;height:14px;border:1px solid #5a4030;border-radius:2px;flex-shrink:0;margin-top:1px;"></div>
          <div>
            <div style="font-size:10px;color:var(--text-bright);">${g.label}</div>
            <div style="font-size:9px;color:var(--text-muted);margin-top:2px;">${g.detail}</div>
          </div>
        </div>
      `).join('')
    : `<div style="font-size:10px;color:var(--success);padding:8px 0;">✓ All goals met — build looks solid!</div>`;

  const rituals = [
    { icon: '🌳', title: 'Respec passive tree', detail: 'Apply Seer\'s recommended respec path', action: 'ask-seer-respec' },
    { icon: '🔨', title: 'Channel benchcrafts', detail: 'Auto-suggest bench crafts for open affixes', action: 'ask-seer-craft' },
    { icon: '💎', title: 'Optimise gem links', detail: 'Find best support gems for main skill', action: 'ask-seer-gems' },
  ];

  const ritualsHtml = rituals.map(r => `
    <button class="blood-ritual-btn" data-action="${r.action}"
      style="display:flex;align-items:center;gap:8px;width:100%;padding:8px;
             background:#1a0a04;border:1px solid #3c3630;border-radius:4px;
             cursor:pointer;margin-bottom:6px;text-align:left;
             transition:border-color 0.2s;"
      onmouseover="this.style.borderColor='var(--gold)'"
      onmouseout="this.style.borderColor='#3c3630'">
      <span style="font-size:18px;">${r.icon}</span>
      <div style="flex:1;">
        <div style="font-size:10px;color:var(--text-bright);font-weight:700;">${r.title}</div>
        <div style="font-size:9px;color:var(--text-muted);">${r.detail}</div>
      </div>
      <span style="color:var(--gold);font-size:10px;">Invoke →</span>
    </button>
  `).join('');

  return `
    <div class="section-label">☠ Sworn Goals</div>
    <div class="goals-list">${goalsHtml}</div>

    <div class="section-label" style="margin-top:12px;">Blood Rituals</div>
    <div class="rituals-list">${ritualsHtml}</div>
  `;
}

// ── Dark Path ─────────────────────────────────────────────────────────────────

interface DarkPathEntry {
  icon: string;
  name: string;
  goal: string;
  upgrades: string[];
  cost: string;
  dps_delta: string;
  surv_delta: string;
  border: string;
}

// Archetype → evolution paths lookup
function getEvolutionPaths(archetype: string): DarkPathEntry[] {
  if (archetype.startsWith('fire')) {
    return [
      {
        icon: '🛡', name: 'The Immortal',
        goal: 'Max survivability — facetank everything',
        upgrades: ['Aegis Aurora (+ES on block)', 'Melding of the Flesh (+max res 90%)'],
        cost: '~40-60 div', dps_delta: '-5%', surv_delta: '+massive',
        border: 'var(--life)',
      },
      {
        icon: '🔥', name: 'The Inferno',
        goal: 'Max damage — melt bosses faster',
        upgrades: ['Ashes of the Stars (+1 all gems)', 'Awakened supports + Clusters'],
        cost: '~30-50 div', dps_delta: '+45%', surv_delta: 'same',
        border: 'var(--fire)',
      },
      {
        icon: '⚡', name: 'The Exile',
        goal: 'Max speed — zoom currency farming',
        upgrades: ['Mageblood belt (permanent flasks)', '+30% MS boots + Onslaught'],
        cost: '~80-150 div', dps_delta: 'same', surv_delta: '+clear +60%',
        border: 'var(--gold)',
      },
    ];
  }

  if (archetype.startsWith('cold')) {
    return [
      {
        icon: '❄', name: 'The Glacier',
        goal: 'Max freeze — lock down every boss',
        upgrades: ['Awakened Cold Penetration', 'Heatshiver helm + Polaric Devastation'],
        cost: '~20-40 div', dps_delta: '+30%', surv_delta: 'same',
        border: 'var(--cold)',
      },
      {
        icon: '🛡', name: 'The Fortress',
        goal: 'Layered defense — CI + block',
        upgrades: ['Dissolution of the Flesh', 'Progenesis for siphon'],
        cost: '~60-100 div', dps_delta: '-10%', surv_delta: '+massive',
        border: 'var(--es)',
      },
      {
        icon: '⚡', name: 'The Exile',
        goal: 'Speed run currency farming',
        upgrades: ['Mageblood belt', 'Synthesised implicit gloves'],
        cost: '~80-150 div', dps_delta: 'same', surv_delta: '+clear +50%',
        border: 'var(--gold)',
      },
    ];
  }

  // Generic fallback for attack/other archetypes
  return [
    {
      icon: '💀', name: 'Glass Cannon',
      goal: 'Maximum damage — one-shot everything',
      upgrades: ['Mirror-tier weapons', 'Cluster jewels (damage notables)'],
      cost: '~50-200 div', dps_delta: '+100%', surv_delta: '-20%',
      border: 'var(--danger)',
    },
    {
      icon: '⚖', name: 'Balanced',
      goal: 'Optimal damage-to-tankiness ratio',
      upgrades: ['Rare items with life + damage', 'Watchers Eye'],
      cost: '~20-50 div', dps_delta: '+30%', surv_delta: '+20%',
      border: 'var(--success)',
    },
    {
      icon: '🛡', name: 'The Bunker',
      goal: 'Unkillable — maximum layers of defense',
      upgrades: ['Brass Dome / Rare chest', 'Block cap items'],
      cost: '~30-80 div', dps_delta: '-15%', surv_delta: '+massive',
      border: 'var(--mana)',
    },
  ];
}

export function renderDarkPathPanel(analysis: AnalysisResult | null): string {
  if (!analysis) return `<div class="panel-placeholder">Load a build first</div>`;

  const paths = getEvolutionPaths(analysis.archetype);

  const pathsHtml = paths.map(p => `
    <div class="dark-path-card" style="background:#120704;border:1px solid #3c3630;
         border-left:3px solid ${p.border};border-radius:4px;padding:10px;margin-bottom:8px;cursor:pointer;"
      onmouseover="this.style.background='#1a0a04'"
      onmouseout="this.style.background='#120704'">
      <div style="display:flex;align-items:center;gap:6px;margin-bottom:4px;">
        <span style="font-size:16px;">${p.icon}</span>
        <span style="font-size:11px;font-weight:700;color:var(--text-bright);">${p.name}</span>
      </div>
      <div style="font-size:9px;color:var(--text-muted);margin-bottom:6px;">${p.goal}</div>
      <div style="font-size:9px;color:var(--text-dim);margin-bottom:6px;">
        ${p.upgrades.map(u => `• ${u}`).join('<br>')}
      </div>
      <div style="display:flex;gap:10px;font-size:9px;font-family:'JetBrains Mono',monospace;">
        <span>Cost: <b style="color:var(--gold)">${p.cost}</b></span>
        <span>DPS: <b style="color:${p.dps_delta.startsWith('+') ? 'var(--success)' : p.dps_delta.startsWith('-') ? 'var(--danger)' : 'var(--text-muted)'}">${p.dps_delta}</b></span>
        <span>Surv: <b style="color:var(--text-muted)">${p.surv_delta}</b></span>
      </div>
    </div>
  `).join('');

  return `
    <div class="section-label">🗡 Build Identity</div>
    <div style="background:#120704;border:1px solid #3c3630;border-radius:4px;padding:8px;margin-bottom:10px;">
      <div style="font-size:12px;font-weight:700;color:var(--text-bright);margin-bottom:4px;">${analysis.archetype_label}</div>
      <div style="font-size:9px;color:var(--text-muted);">Score ${analysis.overall_score}/100 · Lv${analysis.level} ${analysis.ascendancy}</div>
    </div>

    <div class="section-label" style="margin-top:2px;">Choose Your Dark Path</div>
    ${pathsHtml}
  `;
}

// ── Curse Map ─────────────────────────────────────────────────────────────────

interface MapModEntry {
  mod: string;
  reason: string;
  danger: 'fatal' | 'dangerous' | 'safe';
  archetypes?: string[];   // if set, only show for these archetype prefixes
}

const MAP_MODS: MapModEntry[] = [
  // Fatal for fire DoT (RF)
  { mod: 'No Regeneration',        reason: 'RF degen without regen = instant death',         danger: 'fatal',     archetypes: ['fire'] },
  { mod: 'Elemental Reflect',      reason: 'fire/cold/lightning reflects back — one-shots you', danger: 'fatal',   archetypes: ['fire', 'cold', 'lightning'] },
  // Fatal generically
  { mod: 'No Leech',               reason: 'Critical for leech-based builds',                danger: 'fatal',     archetypes: ['attack'] },
  // Dangerous for fire DoT
  { mod: '-max Resist',            reason: 'Max res reduced — RF degen increases significantly', danger: 'dangerous' },
  { mod: 'Less Recovery Rate',     reason: 'Regen falls — may not sustain RF degen',         danger: 'dangerous', archetypes: ['fire'] },
  { mod: 'Ele Weakness Curse',     reason: '-24% res — requires overcapping',                danger: 'dangerous' },
  { mod: 'Enfeeble',               reason: 'Reduced damage — mapping slows significantly',   danger: 'dangerous' },
  { mod: 'Temporal Chains',        reason: 'Slowed action speed — particularly painful',     danger: 'dangerous' },
  // Generally safe
  { mod: 'Physical Reflect',       reason: 'No physical damage — irrelevant',                danger: 'safe' },
  { mod: 'No Mana Regen',          reason: 'Lifetap or mana leech handles this',             danger: 'safe' },
  { mod: 'Hexproof',               reason: 'No curses in this build — no impact',            danger: 'safe' },
  { mod: 'Reduced Flask Charges',  reason: 'Minor inconvenience only',                       danger: 'safe' },
];

export function renderCurseMapPanel(analysis: AnalysisResult | null): string {
  if (!analysis) return `<div class="panel-placeholder">Load a build first</div>`;

  const arch = analysis.archetype;

  const relevant = MAP_MODS.filter(m =>
    !m.archetypes || m.archetypes.some(a => arch.includes(a))
  );

  const fatal     = relevant.filter(m => m.danger === 'fatal');
  const dangerous = relevant.filter(m => m.danger === 'dangerous');
  const safe      = MAP_MODS.filter(m => m.danger === 'safe');

  const modRow = (m: MapModEntry, color: string, icon: string) => `
    <div style="display:flex;align-items:flex-start;gap:8px;padding:5px 0;border-bottom:1px solid #1a0a04;">
      <span style="font-size:12px;flex-shrink:0;">${icon}</span>
      <div>
        <div style="font-size:10px;font-weight:700;color:${color};">${m.mod}</div>
        <div style="font-size:9px;color:var(--text-muted);">${m.reason}</div>
      </div>
    </div>
  `;

  return `
    <div class="section-label" style="color:var(--danger);">🚫 Cannot Run</div>
    ${fatal.length > 0
      ? fatal.map(m => modRow(m, 'var(--danger)', '💀')).join('')
      : `<div style="font-size:9px;color:var(--text-muted);padding:4px 0;">None — this build survives all mods</div>`
    }

    <div class="section-label" style="margin-top:8px;color:var(--warning);">⚠ Dangerous</div>
    ${dangerous.length > 0
      ? dangerous.map(m => modRow(m, 'var(--warning)', '⚠')).join('')
      : `<div style="font-size:9px;color:var(--text-muted);padding:4px 0;">No particularly dangerous mods</div>`
    }

    <div class="section-label" style="margin-top:8px;color:var(--success);">✓ Safe</div>
    ${safe.map(m => modRow(m, 'var(--success)', '✓')).join('')}
  `;
}

// ── Settings ──────────────────────────────────────────────────────────────────

interface AiProvider {
  id: string;
  label: string;
  placeholder: string;
  url: string;
}

const AI_PROVIDERS: AiProvider[] = [
  { id: 'claude',     label: 'Claude (Anthropic)',  placeholder: 'sk-ant-…',    url: 'https://console.anthropic.com/' },
  { id: 'gpt4',       label: 'GPT-4 (OpenAI)',      placeholder: 'sk-…',        url: 'https://platform.openai.com/api-keys' },
  { id: 'gemini',     label: 'Gemini (Google)',      placeholder: 'AIza…',       url: 'https://aistudio.google.com/app/apikey' },
  { id: 'openrouter', label: 'OpenRouter',           placeholder: 'sk-or-…',    url: 'https://openrouter.ai/keys' },
  { id: 'ollama',     label: 'Ollama (local)',       placeholder: 'no key needed', url: 'https://ollama.ai/' },
];

// ─── Passive Tree Panel ───────────────────────────────────────────────────────

export function renderPassiveTreePanel(analysis: AnalysisResult | null): string {
  if (!analysis) return `<div class="panel-placeholder">Load a build first</div>`;
  const pts = analysis.level > 0 ? Math.min(analysis.level - 1, 123) : 0;
  return `
    <div class="section-label">🌳 Passive Tree — ${analysis.build_name}</div>

    <div style="background:var(--bg-card);border:1px solid var(--border);border-radius:2px;padding:10px;margin-bottom:8px;">
      <div style="display:flex;justify-content:space-between;margin-bottom:6px;">
        <span style="font-size:12px;color:var(--text-bright);font-weight:600;font-family:'Cinzel',serif;">Points Spent</span>
        <span style="font-size:14px;font-weight:800;color:var(--gold);font-family:'JetBrains Mono',monospace;">${pts} / 123</span>
      </div>
      <div style="height:5px;background:var(--bg-dark);border-radius:3px;overflow:hidden;">
        <div style="height:100%;width:${Math.round(pts / 123 * 100)}%;background:linear-gradient(90deg,var(--gold),var(--success));border-radius:3px;"></div>
      </div>
      <div style="font-size:10px;color:var(--text-dim);margin-top:4px;">${123 - pts} unallocated points available</div>
    </div>

    <div style="font-size:10px;color:var(--text-dim);text-transform:uppercase;letter-spacing:1px;font-family:'Cinzel',serif;margin:10px 0 6px;">🔑 Build Archetype</div>
    <div style="background:var(--bg-card);border:1px solid var(--border);border-left:3px solid var(--fire);border-radius:2px;padding:8px 10px;margin-bottom:8px;">
      <div style="font-size:12px;color:var(--text-bright);font-weight:600;">${analysis.archetype_label}</div>
      <div style="font-size:9px;color:var(--text-muted);margin-top:2px;">${analysis.class_name} — ${analysis.ascendancy}</div>
    </div>

    <div style="font-size:10px;color:var(--text-dim);text-transform:uppercase;letter-spacing:1px;font-family:'Cinzel',serif;margin:10px 0 6px;">⬆ Seer Recommendations</div>
    <div style="background:var(--bg-card);border:1px solid var(--border);border-left:3px solid var(--success);border-radius:2px;padding:8px 10px;margin-bottom:4px;">
      <div style="display:flex;justify-content:space-between;font-size:11px;">
        <span style="color:var(--text-bright);">Ask the Seer for tree advice</span>
        <span style="color:var(--success);font-size:10px;">→ Grimoire</span>
      </div>
      <div style="font-size:9px;color:var(--text-muted);">The Seer can recommend the next 3-5 passive points for your archetype.</div>
    </div>

    <div style="margin-top:10px;padding:8px;background:#c4a83008;border:1px solid #c4a83020;border-radius:2px;font-size:10px;color:var(--gold);text-align:center;">
      🌳 Full interactive passive tree viewer — open Path of Building for node details
    </div>
  `;
}

// ─── Stash Panel ──────────────────────────────────────────────────────────────

export interface StashPanelItem {
  name: string;
  chaos_value: number;
  stack_size: number;
  tab_name: string;
}

export function renderStashPanel(items: StashPanelItem[], divinePriceC = 220): string {
  if (items.length === 0) {
    return `
      <div class="section-label">📦 Stash — Inventory Intelligence</div>
      <div style="text-align:center;padding:30px 0;color:var(--text-muted);font-size:11px;">
        <div style="font-size:24px;margin-bottom:8px;">📦</div>
        Connect your PoE account to view stash contents.<br>
        <small style="color:var(--text-dim);">Settings → Connect PoE Account</small>
      </div>
    `;
  }

  const totalChaos = items.reduce((s, i) => s + i.chaos_value * i.stack_size, 0);
  const totalDiv   = divinePriceC > 0 ? totalChaos / divinePriceC : 0;

  const tabNames = [...new Set(items.map(i => i.tab_name))];
  const topItems = [...items]
    .sort((a, b) => b.chaos_value * b.stack_size - a.chaos_value * a.stack_size)
    .slice(0, 10);

  return `
    <div class="section-label">📦 Stash — Inventory Intelligence</div>

    <div style="display:flex;gap:8px;margin-bottom:10px;">
      <div style="flex:1;background:var(--bg-card);border:1px solid var(--border);border-radius:2px;padding:8px;text-align:center;">
        <div style="font-size:18px;font-weight:800;color:var(--gold);font-family:'JetBrains Mono',monospace;">${totalDiv.toFixed(1)}d</div>
        <div style="font-size:9px;color:var(--text-muted);">Total Value</div>
      </div>
      <div style="flex:1;background:var(--bg-card);border:1px solid var(--border);border-radius:2px;padding:8px;text-align:center;">
        <div style="font-size:18px;font-weight:800;color:var(--text-bright);font-family:'JetBrains Mono',monospace;">${items.length}</div>
        <div style="font-size:9px;color:var(--text-muted);">Unique Items</div>
      </div>
      <div style="flex:1;background:var(--bg-card);border:1px solid var(--border);border-radius:2px;padding:8px;text-align:center;">
        <div style="font-size:18px;font-weight:800;color:var(--text-bright);font-family:'JetBrains Mono',monospace;">${tabNames.length}</div>
        <div style="font-size:9px;color:var(--text-muted);">Tabs</div>
      </div>
    </div>

    <div style="font-size:10px;color:var(--text-dim);text-transform:uppercase;letter-spacing:1px;font-family:'Cinzel',serif;margin-bottom:6px;">💰 Top Items by Value</div>
    ${topItems.map(item => {
      const val = item.chaos_value * item.stack_size;
      const divVal = divinePriceC > 0 ? (val / divinePriceC).toFixed(1) : '?';
      return `
        <div style="display:flex;justify-content:space-between;align-items:center;padding:5px 8px;background:var(--bg-card);border:1px solid var(--border);border-radius:2px;margin-bottom:3px;">
          <div>
            <span style="font-size:11px;color:var(--text-bright);">${item.name}</span>
            ${item.stack_size > 1 ? `<span style="font-size:9px;color:var(--text-muted);margin-left:4px;">×${item.stack_size}</span>` : ''}
            <div style="font-size:8px;color:var(--text-dim);">${item.tab_name}</div>
          </div>
          <span style="font-size:11px;font-weight:700;color:var(--gold);font-family:'JetBrains Mono',monospace;">${divVal}d</span>
        </div>
      `;
    }).join('')}
  `;
}

// ─── Harbinger Panel (Issues list) ───────────────────────────────────────────

export function renderHarbingerPanel(analysis: AnalysisResult | null): string {
  if (!analysis) return `<div class="panel-placeholder">Load a build first</div>`;

  const issues = analysis.issues;
  if (issues.length === 0) {
    return `
      <div class="section-label">⚠ Harbinger Warnings (0)</div>
      <div style="text-align:center;padding:20px 0;color:var(--success);font-size:12px;">✓ No issues detected — build looks solid!</div>
    `;
  }

  const bySeverity = (s: string) => issues.filter(i => i.severity === s);
  const sections: Array<[string, string, string]> = [
    ['Critical', 'var(--danger)',  '🔴'],
    ['Major',    'var(--warning)', '🟠'],
    ['Minor',    'var(--gold)',    '🟡'],
    ['Info',     'var(--info)',    'ℹ'],
  ];

  return `
    <div class="section-label">⚠ Harbinger Warnings (${issues.length})</div>
    ${sections.map(([sev, color, icon]) => {
      const grp = bySeverity(sev);
      if (grp.length === 0) return '';
      return `
        <div style="font-size:10px;color:${color};text-transform:uppercase;letter-spacing:1px;font-family:'Cinzel',serif;margin:8px 0 4px;">${icon} ${sev} (${grp.length})</div>
        ${grp.map(issue => `
          <div style="background:var(--bg-card);border:1px solid var(--border);border-left:3px solid ${color};border-radius:2px;padding:8px 10px;margin-bottom:4px;">
            <div style="font-size:11px;color:${color};font-weight:600;margin-bottom:3px;">${issue.title}</div>
            <div style="font-size:9px;color:var(--text-muted);margin-bottom:3px;">${issue.detail}</div>
            <div style="font-size:9px;color:var(--text-dim);">Fix: ${issue.fix}</div>
            ${issue.slot ? `<div style="font-size:8px;color:var(--text-dim);margin-top:2px;">Slot: ${issue.slot}</div>` : ''}
          </div>
        `).join('')}
      `;
    }).join('')}
  `;
}

export function renderSettingsPanel(configuredProviders: string[] = []): string {
  const providerRows = AI_PROVIDERS.map(p => {
    const configured = configuredProviders.includes(p.label);
    return `
      <div class="settings-row" style="border:1px solid #3c3630;border-radius:4px;padding:8px;margin-bottom:6px;background:#120704;">
        <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:6px;">
          <span style="font-size:10px;font-weight:700;color:${configured ? 'var(--success)' : 'var(--text-bright)'};">
            ${configured ? '✓ ' : ''}${p.label}
          </span>
          <a href="${p.url}" target="_blank" style="font-size:8px;color:var(--text-muted);text-decoration:none;">Get key ↗</a>
        </div>
        <div style="display:flex;gap:6px;">
          <input
            type="password"
            id="ai-key-${p.id}"
            class="settings-input"
            placeholder="${p.placeholder}"
            style="flex:1;background:#0a0604;border:1px solid #3c3630;border-radius:3px;
                   color:var(--text);padding:4px 6px;font-size:9px;font-family:'JetBrains Mono',monospace;"
          />
          <button
            class="settings-test-btn"
            data-provider="${p.id}"
            style="padding:4px 8px;font-size:9px;background:#1a0a04;border:1px solid #5a4030;
                   border-radius:3px;color:var(--text-bright);cursor:pointer;"
            title="Test connection">
            Test
          </button>
          <button
            class="settings-save-btn"
            data-provider="${p.id}"
            style="padding:4px 8px;font-size:9px;background:#1a2010;border:1px solid #4a6030;
                   border-radius:3px;color:var(--success);cursor:pointer;"
            title="Save key">
            Save
          </button>
        </div>
        <div id="ai-status-${p.id}" style="font-size:8px;margin-top:4px;min-height:12px;"></div>
      </div>
    `;
  }).join('');

  return `
    <div class="section-label">⚙ Seer Intelligence</div>
    <div style="font-size:9px;color:var(--text-muted);margin-bottom:10px;line-height:1.5;">
      Add an AI provider key to unlock intelligent Seer responses.
      Without a key, the Seer uses built-in PoE knowledge only.
    </div>
    ${providerRows}

    <div class="section-label" style="margin-top:12px;">League</div>
    <div style="display:flex;gap:6px;align-items:center;">
      <input
        type="text"
        id="settings-league"
        value="Mirage (3.28)"
        style="flex:1;background:#0a0604;border:1px solid #3c3630;border-radius:3px;
               color:var(--text);padding:4px 6px;font-size:9px;font-family:'JetBrains Mono',monospace;"
      />
      <button id="settings-save-league"
        style="padding:4px 8px;font-size:9px;background:#1a2010;border:1px solid #4a6030;
               border-radius:3px;color:var(--success);cursor:pointer;">
        Save
      </button>
    </div>

    <div class="section-label" style="margin-top:12px;">PoB Watch Directory</div>
    <div style="font-size:9px;color:var(--text-muted);margin-bottom:6px;">
      Set to your Path of Building folder to enable auto-reload on save.
    </div>
    <div style="display:flex;gap:6px;align-items:center;">
      <input
        type="text"
        id="settings-pob-dir"
        placeholder="C:\\Users\\…\\Path of Building"
        style="flex:1;background:#0a0604;border:1px solid #3c3630;border-radius:3px;
               color:var(--text);padding:4px 6px;font-size:9px;font-family:'JetBrains Mono',monospace;"
      />
      <button id="settings-browse-pob"
        style="padding:4px 8px;font-size:9px;background:#1a0a04;border:1px solid #5a4030;
               border-radius:3px;color:var(--text-bright);cursor:pointer;">
        Browse
      </button>
    </div>
  `;
}

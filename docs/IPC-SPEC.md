# Path of AI — IPC Specification (Tauri Commands + Events)

## Tauri Commands (Frontend → Backend)

Every `invoke()` call from TypeScript to Rust.

### Build Commands
```typescript
// Load character directly from PoE account (OAuth — PRIMARY method)
invoke('load_character', {
  characterName: string
}): Promise<AnalysisResult>

// Load and analyze a PoB build file (OPTIONAL alternative)
invoke('analyze_build', { 
  filePath: string 
}): Promise<AnalysisResult>

// Re-analyze current build (after external change)
invoke('refresh_analysis'): Promise<AnalysisResult>

// Apply an upgrade suggestion to PoB file
invoke('apply_upgrade', { 
  suggestionId: string,
  buildId: string 
}): Promise<UpgradeResult>

// Undo last change
invoke('undo_last_change'): Promise<AnalysisResult>

// Redo (if undone)
invoke('redo_change'): Promise<AnalysisResult>
```

### Seer Commands
```typescript
// Ask The Seer a question
invoke('ask_seer', { 
  question: string,
  buildId: string 
}): Promise<SeerResponse>

// Get craft suggestions based on player currency
invoke('get_craft_suggestions', { 
  buildId: string,
  currency: CurrencyInventory 
}): Promise<CraftSuggestion[]>

// Get passive tree analysis
invoke('get_tree_analysis', { 
  buildId: string 
}): Promise<TreeAnalysis>
```

### Market Commands
```typescript
// Get current prices for equipped items
invoke('get_prices', { 
  itemNames: string[] 
}): Promise<PriceResult[]>

// Search trade for upgrade items
invoke('search_upgrades', { 
  slot: string,
  budget: number,
  buildId: string 
}): Promise<TradeResult[]>
```

### Item Commands
```typescript
// Parse pasted PoE item from clipboard
invoke('parse_clipboard_item', { 
  clipboardText: string,
  buildId: string 
}): Promise<ParsedItemResult>

// Score a custom/edited item against build
invoke('score_item', { 
  item: ItemData,
  buildId: string 
}): Promise<ItemScore>
```

### Settings Commands
```typescript
// Test cloud AI connection
invoke('test_cloud_ai', { 
  provider: string,
  apiKey: string,
  model: string 
}): Promise<ConnectionTestResult>

// Save settings
invoke('save_settings', { 
  settings: AppSettings 
}): Promise<void>

// Start/stop PoB file watching
invoke('watch_pob_directory', { 
  path: string 
}): Promise<void>

// Connect PoE OAuth (opens browser)
invoke('start_poe_oauth'): Promise<void>

// Generate build share code
invoke('generate_share_code', { 
  buildId: string,
  includeItems: boolean,
  includeTree: boolean,
  includeGems: boolean 
}): Promise<string>

// Import build from share code
invoke('import_share_code', { 
  code: string 
}): Promise<AnalysisResult>
```

### Stash Commands (requires PoE OAuth)
```typescript
// Fetch stash tab grid data
invoke('fetch_stash_tabs'): Promise<StashTab[]>

// Fetch items in a specific stash tab
invoke('fetch_stash_items', {
  tabId: string
}): Promise<StashItem[]>

// Find upgrades already in stash
invoke('find_stash_upgrades', {
  buildId: string
}): Promise<StashUpgrade[]>

// Get currency totals across all tabs
invoke('get_currency_totals'): Promise<CurrencyTotal>
```

### Combat Simulation Commands
```typescript
// Simulate boss fight
invoke('simulate_boss', {
  buildId: string,
  bossId: string         // 'shaper', 'maven', 'uber_shaper', etc.
}): Promise<BossSimResult>

// Simulate map clear
invoke('simulate_map_clear', {
  buildId: string,
  mapTier: number,
  mapName?: string
}): Promise<MapClearResult>

// Analyze map mods (Curse Map)
invoke('analyze_map_mods', {
  buildId: string,
  mods: string[]
}): Promise<MapModAnalysis>
```

### Price Alert Commands
```typescript
// Set a price alert
invoke('set_price_alert', {
  itemName: string,
  threshold: number,
  comparison: 'below' | 'above' | 'change_percent',
  notifyMethod: 'popup' | 'sound' | 'silent'
}): Promise<string>  // returns alertId

// List active alerts
invoke('list_price_alerts'): Promise<PriceAlert[]>

// Remove alert
invoke('remove_price_alert', {
  alertId: string
}): Promise<void>
```

### Build Comparison Commands
```typescript
// Compare two builds side-by-side
invoke('compare_builds', {
  buildIdA: string,
  buildIdB: string
}): Promise<BuildComparison>

// Compare to poe.ninja top builds
invoke('compare_to_top', {
  buildId: string,
  limit: number           // how many top builds to compare
}): Promise<TopBuildComparison>
```

### Multi-Character Commands
```typescript
// List all PoE characters (requires OAuth)
invoke('list_characters'): Promise<PoeCharacter[]>

// Switch active character
invoke('switch_character', {
  characterName: string
}): Promise<AnalysisResult>
```

---

## Events (Backend → Frontend) — Complete List

```typescript
// === Build Events ===
listen('build-changed', (e: { filePath: string, changes: string[], newAnalysis: AnalysisResult }) => void)
listen('build-score-changed', (e: { buildId: string, oldScore: number, newScore: number }) => void)
listen('upgrade-applied', (e: { slot: string, oldItem: string, newItem: string, dpsDiff: number }) => void)

// === Market Events ===
listen('prices-updated', (e: { updatedItems: string[], timestamp: number }) => void)
listen('price-alert-triggered', (e: { alertId: string, itemName: string, currentPrice: number, threshold: number }) => void)

// === Seer Events ===
listen('seer-thinking', (e: { question: string }) => void)
listen('seer-response', (e: { answer: string, engine: string, confidence: number }) => void)

// === System Events ===
listen('data-update-available', (e: { version: string, changelog: string[], sizeBytes: number }) => void)
listen('update-progress', (e: { percent: number, currentFile: string }) => void)
listen('update-complete', (e: { version: string }) => void)

// === OAuth Events ===
listen('poe-oauth-complete', (e: { accountName: string, characters: number, stashTabs: number }) => void)
listen('poe-oauth-error', (e: { error: string }) => void)

// === Stash Events ===
listen('stash-loaded', (e: { tabCount: number, totalValue: number }) => void)
```

---

## TypeScript Types — Complete

```typescript
// === Core Analysis Types ===
interface AnalysisResult {
  buildId: string;
  className: string;
  ascendancy: string;
  level: number;
  archetype: string;
  mainSkill: string;
  overallScore: number;
  offense: OffenseResult;
  defense: DefenseResult;
  items: ItemAnalysis[];
  issues: Issue[];
  suggestions: Suggestion[];
  checklist: ChecklistItem[];
  progression: ProgressionPhase;
  flasks: FlaskAnalysis[];
  gems: GemGroupAnalysis[];
  tree: TreeSummary;
}

interface OffenseResult {
  totalDps: number;
  dpsBreakdown: DpsSource[];
  multiplierChain: Multiplier[];
  gemAnalysis: GemAnalysis[];
  dpsWithFlasks: number;
  dpsWithoutFlasks: number;
  bossDps: number;       // effective DPS vs boss (with 40% res, curse penalty)
  clearDps: number;      // effective DPS for mapping
}
interface DpsSource { source: string; value: number; percent: number; type: string; }
interface Multiplier { label: string; value: string; type: 'base'|'inc'|'more'|'multi'|'pen'; }
interface GemAnalysis { name: string; level: number; quality: number; maxed: boolean; slot: string; }

interface DefenseResult {
  life: number;
  energyShield: number;
  armour: number;
  evasion: number;
  block: number;
  spellBlock: number;
  spellSuppression: number;
  resists: { fire: number; cold: number; lightning: number; chaos: number };
  overcap: { fire: number; cold: number; lightning: number };
  maxResists: { fire: number; cold: number; lightning: number };
  lifeRegen: number;
  netRegen: number;    // after RF degen etc.
  leechRate: number;
  ailmentImmunity: Record<string, { immune: boolean; source: string|null }>;
  ehp: { physical: number; elemental: number; chaos: number };
  guardSkill: { name: string; shieldAmount: number; uptime: number } | null;
  fortify: boolean;
}

// === Item Types ===
interface ItemAnalysis {
  id: number;
  slot: string;
  name: string;
  base: string;
  rarity: 'NORMAL'|'MAGIC'|'RARE'|'UNIQUE';
  score: number;
  value: number;         // divine orbs
  sockets: string|null;  // "RRRBGR"
  mods: ModInfo[];
  openAffixes: { prefixes: number; suffixes: number };
  weaknesses: { severity: string; issue: string }[];
  influence: string|null;
  corrupted: boolean;
  enchant: string|null;
}
interface ModInfo { text: string; tier: string; color: string; crafted: boolean; fractured: boolean; }

interface ParsedItemResult {
  item: ItemAnalysis;
  comparisonSlot: string;
  currentItem: ItemAnalysis;
  dpsChange: number;
  lifeChange: number;
  resistChanges: { fire: number; cold: number; lightning: number; chaos: number };
  isUpgrade: boolean;
  marketValue: number;
}

interface ItemScore { score: number; dpsImpact: number; lifeImpact: number; }

// === Market Types ===
interface PriceResult { itemName: string; priceDivine: number; priceChaos: number; trend: 'rising'|'stable'|'falling'; }
interface TradeResult { name: string; price: number; dpsGain: number; lifeGain: number; link: string; }
interface MarketItem { name: string; price: string; dpsGain: string; lifeGain: string; }

// === Stash Types ===
interface StashTab { id: string; name: string; type: string; color: string; itemCount: number; }
interface StashItem { name: string; x: number; y: number; w: number; h: number; rarity: string; value: number; }
interface StashUpgrade { itemName: string; slot: string; benefit: string; tabName: string; }
interface CurrencyTotal { divine: number; chaos: number; exalted: number; totalDivine: number; breakdown: Record<string, number>; }

// === Combat Types ===
interface BossSimResult {
  bossName: string;
  bossHp: number;
  effectiveDps: number;
  fightTime: string;
  avgDeaths: number;
  survivesSlam: boolean;
  survivesDieBeam: boolean;
  readiness: 'ready'|'risky'|'not_ready'|'lethal';
  upgradeImpact: { afterUpgrade: string; fightTime: string; deaths: number };
}
interface MapClearResult {
  mapName: string;
  tier: number;
  clearTime: string;
  killsPerSecond: number;
  currencyPerHour: number;
  moveSpeed: number;
}
interface MapModAnalysis {
  canRun: string[];       // safe mods
  dangerous: string[];    // dangerous but survivable
  cannotRun: string[];    // lethal mods (no regen, reflect)
  overallRisk: 'safe'|'moderate'|'dangerous'|'lethal';
}

// === Seer Types ===
interface SeerResponse {
  answer: string;
  engine: 'calculator'|'knowledge'|'cloud';
  confidence: number;
  provider: string;
}
interface CraftSuggestion {
  rank: number;
  method: string;        // 'essence', 'fossil', 'chaos', 'benchcraft'
  slot: string;
  target: string;
  successRate: number;
  expectedCost: number;
  buyPrice: number;
  steps: string[];
  verdict: 'craft'|'buy';
  playerHasCurrency: boolean;
}
interface TreeAnalysis {
  pointsSpent: number;
  pointsTotal: number;
  keystones: string[];
  nextBestPoints: { node: string; impact: string; points: number }[];
  inefficientNodes: { node: string; reason: string }[];
  jewelSockets: { name: string; status: 'equipped'|'empty'; costToReach: number }[];
  bestAnoint: { notable: string; oils: string[]; oilCost: number; impact: string };
}

// === Alerts ===
interface PriceAlert { id: string; itemName: string; threshold: number; comparison: string; active: boolean; lastTriggered: string|null; }

// === Build Comparison ===
interface BuildComparison {
  buildA: { name: string; dps: number; life: number; score: number };
  buildB: { name: string; dps: number; life: number; score: number };
  dpsDiff: number;
  lifeDiff: number;
  treeDiffPercent: number;
  gearDiffs: { slot: string; itemA: string; itemB: string; scoreDiff: number }[];
}
interface TopBuildComparison {
  yourDps: number;
  avgTopDps: number;
  percentile: number;
  gearGaps: { slot: string; yourScore: number; avgScore: number }[];
  treeOverlap: number;
  missingNodes: string[];
  popularGems: { gem: string; usagePercent: number; youUse: boolean }[];
}

// === Misc ===
interface ChecklistItem { label: string; done: boolean; }
interface ProgressionPhase { phase: string; level: number; nextGoals: string[]; }
interface FlaskAnalysis { name: string; suffix: string; mods: string[]; warnings: string[]; }
interface GemGroupAnalysis { skill: string; color: string; sockets: string; gems: GemAnalysis[]; }
interface TreeSummary { pointsSpent: number; pointsTotal: number; keystones: string[]; }
interface ConnectionTestResult { success: boolean; message: string; model: string; }
interface UpgradeResult { applied: boolean; oldDps: number; newDps: number; oldLife: number; newLife: number; changes: string[]; }
interface PoeCharacter { name: string; class: string; level: number; league: string; }
interface AppSettings { [key: string]: any; } // matches CONFIG-SCHEMA.md structure
```

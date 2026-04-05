// TypeScript types — must match Rust models exactly (src-tauri/src/models/)

export interface BuildData {
  id: string;
  name: string;
  class_name: string;
  ascendancy: string;
  level: number;
  items: Item[];
  gems: GemSetup[];
  passive_tree: PassiveTree;
  config: BuildConfig;
}

export interface Item {
  id: number;
  name: string;
  base_type: string;
  slot: string;
  rarity: 'Normal' | 'Magic' | 'Rare' | 'Unique';
  level_requirement: number;
  item_level: number;
  quality: number;
  sockets: string;
  mods: ItemMod[];
  is_corrupted: boolean;
  score?: number;
}

export interface ItemMod {
  id: string;
  text: string;
  value1: number;
  value2?: number;
  mod_type: 'Prefix' | 'Suffix' | 'Implicit' | 'Enchant' | 'Corrupted';
  is_crafted: boolean;
}

export interface GemSetup {
  skill: string;
  slot: string;
  socket_colors: string;
  gems: Gem[];
  is_main_skill: boolean;
}

export interface Gem {
  name: string;
  level: number;
  quality: number;
  is_support: boolean;
  is_vaal: boolean;
  is_awakened: boolean;
  is_maxed: boolean;
}

export interface PassiveTree {
  allocated_nodes: number[];
  jewels: TreeJewel[];
  masteries: MasterySelection[];
}

export interface TreeJewel {
  socket_id: number;
  item: Item;
}

export interface MasterySelection {
  node_id: number;
  effect_id: number;
  effect_text: string;
}

export interface BuildConfig {
  boss_name: string;
  map_tier: number;
  is_uberlab: boolean;
  flask_uptime: number;
}

// --- Analysis types ---

export interface AnalysisResult {
  build_id: string;
  build_name: string;
  class_name: string;
  ascendancy: string;
  level: number;
  archetype: string;
  archetype_label: string;
  overall_score: number;
  defenses: DefenseStats;
  offense: OffenseStats;
  issues: Issue[];
  suggestions: Suggestion[];
  item_scores: ItemScore[];
  gem_setups: GemSetup[];
}

export interface DefenseStats {
  life: number;
  energy_shield: number;
  mana: number;
  life_regen_flat: number;
  life_regen_pct: number;
  resistances: ResistanceProfile;
  armour: number;
  armour_phys_reduction: number;
  evasion: number;
  evasion_chance: number;
  block_chance: number;
  spell_block_chance: number;
  effective_hp: EffectiveHP;
  ailment_immunity: AilmentImmunity;
}

export interface ResistanceProfile {
  fire: number;
  cold: number;
  lightning: number;
  chaos: number;
  max_fire: number;
  max_cold: number;
  max_lightning: number;
  max_chaos: number;
  fire_overcap: number;
  cold_overcap: number;
  lightning_overcap: number;
}

export interface EffectiveHP {
  vs_physical: number;
  vs_elemental: number;
  vs_chaos: number;
}

export interface AilmentImmunity {
  freeze: boolean;
  freeze_source?: string;
  shock: boolean;
  shock_source?: string;
  ignite: boolean;
  ignite_source?: string;
  bleed: boolean;
  bleed_source?: string;
  corrupted_blood: boolean;
  corrupted_blood_source?: string;
  poison: boolean;
  stun: boolean;
  curse_immune: boolean;
}

export interface OffenseStats {
  total_dps: number;
  dps_label: string;
  main_skill: string;
  hit_dps: number;
  dot_dps: number;
  crit_chance: number;
  crit_multiplier: number;
  attack_speed: number;
  cast_speed: number;
  hit_chance: number;
  sources: DpsSource[];
  multiplier_chain: MultiplierStep[];
}

export interface DpsSource {
  source: string;
  value: number;
  percent_of_total: number;
  color: string;
}

export interface MultiplierStep {
  label: string;
  multiplier: number;
  step_type: 'Base' | 'Increased' | 'More' | 'Penetration';
}

export interface Issue {
  id: string;
  severity: 'Critical' | 'Major' | 'Minor' | 'Info';
  title: string;
  detail: string;
  fix: string;
  slot?: string;
}

export interface Suggestion {
  id: string;
  slot: string;
  title: string;
  detail: string;
  dps_gain: number;
  dps_gain_pct: number;
  life_gain: number;
  estimated_cost_div: number;
  efficiency: number;
  priority: number;
  trade_url?: string;
}

export interface ItemScore {
  slot: string;
  item_name: string;
  score: number;
  tier: 'BiS' | 'Excellent' | 'Good' | 'Acceptable' | 'Upgrade' | 'Replace';
  top_issue?: string;
}

export interface SeerResponse {
  answer: string;
  engine: 'Calculator' | 'Knowledge' | 'Cloud' | 'Fallback';
  confidence: number;
  follow_up_questions: string[];
  related_suggestions: string[];
}

export interface PriceResult {
  item_name: string;
  price_div: number;
  price_chaos: number;
  confidence: 'High' | 'Medium' | 'Low' | 'Guess';
  listings: number;
  cached: boolean;
  cache_age_secs: number;
}

export interface AppInfo {
  version: string;
  name: string;
  poe_version: string;
  league: string;
}

export interface CraftSuggestion {
  method: 'BenchCraft' | 'Essence' | 'Chaos' | 'Fossil' | 'Harvest' | 'Recombinator';
  target_mod: string;
  probability: number;
  attempts_99pct: number;
  expected_cost_chaos: number;
  dps_gain: number;
  verdict: 'BestOption' | 'SafeOption' | 'HighRisk' | 'NotWorthIt';
}

export interface CraftVsBuyResult {
  slot: string;
  current_item: string;
  craft_cost_div: number;
  buy_cost_div: number;
  verdict: 'BestOption' | 'SafeOption' | 'HighRisk' | 'NotWorthIt';
  recommendation: string;
}

export interface BuildSummary {
  id: string;
  name: string;
  class_name: string;
  ascendancy: string;
  level: number;
  last_analyzed: string;
}

// --- Map Mod Analysis ---

export type DangerLevel = 'Safe' | 'Minor' | 'Moderate' | 'Major' | 'Critical';

export interface ModDanger {
  mod_text: string;
  level: DangerLevel;
  reason: string;
}

export interface MapDangerResult {
  mods: ModDanger[];
  worst: DangerLevel;
  verdict: string;        // "Run" | "Run carefully" | "Reroll" | "Skip"
  fatal_mods: string[];
  total_score: number;    // 0-100
}

// --- Combat Simulation ---

export interface SimResult {
  clear_time_ms: number;
  kills: number;
  deaths: number;
  ticks: number;
}

// --- Build Comparator ---

export type DeltaDir = 'Better' | 'Worse' | 'Same';

export interface StatDelta {
  key: string;
  value_a: number;
  value_b: number;
  delta: number;
  delta_pct: number;
  direction: DeltaDir;
  higher_is_better: boolean;
}

export interface BuildComparison {
  build_a: string;
  build_b: string;
  stat_deltas: StatDelta[];
  tree_overlap_pct: number;
  shared_gems: string[];
  unique_to_a: string[];
  unique_to_b: string[];
  summary_winner?: string;
}

export interface BuildSnapshot {
  id: string;
  name: string;
  stats: Record<string, number>;
  passives: number[];
  gems: string[];
}

// --- Stash ---

export interface StashItem {
  id: string;
  name: string;
  type_line: string;
  chaos_value: number;
  stack_size: number;
  tab_name: string;
}

export interface WealthSummary {
  total_chaos: number;
  total_divine: number;
  currency_map: Record<string, number>;
  total_items: number;
}

// --- Map Tracker ---

export interface MapRun {
  zone_name: string;
  started_at: number;
  ended_at: number;
  duration_secs: number;
  loot_chaos: number;
}

export interface MapStats {
  total_runs: number;
  total_time_secs: number;
  avg_duration: number;
  total_loot_chaos: number;
  chaos_per_hour: number;
  most_run_map: string;
  by_zone: Record<string, number>;
}

// --- Price Alerts ---

export type AlertCondition = 'Below' | 'Above' | 'ChangePercent';

export interface PriceAlert {
  id: string;
  item_key: string;
  condition: AlertCondition;
  threshold: number;
  active: boolean;
  created_at: number;
}

export interface AlertFired {
  alert_id: string;
  item_key: string;
  current_price: number;
  threshold: number;
  condition: AlertCondition;
  message: string;
}

// --- Trade ---

export interface TradeResult {
  item_name: string;
  base_type: string;
  slot: string;
  price_div: number;
  dps_gain: number;
  life_gain: number;
  score: number;
  trade_url: string;
}

// --- Characters ---

export interface CharacterSummary {
  name: string;
  class: string;
  level: number;
  league: string;
}

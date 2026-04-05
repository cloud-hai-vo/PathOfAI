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

/// passive_tree.rs — Tests written FIRST (TDD RED phase).
/// Run `cargo test` now → all tests fail → then implement to make them pass.
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::models::build::PassiveTree;

// ─── Types (stubs — just enough to compile for RED phase) ─────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRecommendation {
    pub node_id:     u32,
    pub node_name:   String,
    pub stats:       Vec<String>,
    pub value_score: f64,
    pub path_cost:   u32,
    pub efficiency:  f64,
    pub path:        Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveNode {
    pub id:          u32,
    pub name:        String,
    pub stats:       Vec<PassiveStat>,
    pub neighbors:   Vec<u32>,
    pub is_keystone: bool,
    pub is_notable:  bool,
    pub is_travel:   bool,
    pub class_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PassiveStat {
    pub text:      String,
    pub stat_type: StatType,
    pub value:     f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub enum StatType {
    #[default] Life,
    EnergyShield, Armour, Evasion, Resistances,
    FireDamage, ColdDamage, LightningDamage, ChaosDamage,
    SpellDamage, AttackSpeed, CastSpeed, CritChance, CritMultiplier,
    DotMultiplier, MovementSpeed, Strength, Dexterity, Intelligence, Other,
}

// ─── Function stubs — all unimplemented!() → RED ──────────────────────────────

pub fn archetype_weight(archetype: &str, stat: &StatType) -> f64 {
    let arch = archetype.to_lowercase();
    match stat {
        StatType::Life => {
            if arch.contains("life") { 2.5 }
            else if arch.contains("es") || arch.contains("occultist") { 0.5 }
            else { 1.5 }
        }
        StatType::EnergyShield => {
            if arch.contains("es") || arch.contains("occultist") { 2.5 }
            else if arch.contains("life") { 0.5 }
            else { 1.0 }
        }
        StatType::DotMultiplier => {
            if arch.contains("rf") || arch.contains("dot") || arch.contains("inquisitor") { 3.0 }
            else { 0.5 }
        }
        StatType::FireDamage => {
            if arch.contains("rf") || arch.contains("fire") { 2.5 }
            else { 0.5 }
        }
        StatType::Armour => {
            if arch.contains("armor") || arch.contains("armour") || arch.contains("tank") { 2.0 }
            else { 0.8 }
        }
        StatType::Evasion => {
            if arch.contains("eva") || arch.contains("ranger") { 2.0 }
            else { 0.5 }
        }
        StatType::AttackSpeed => {
            if arch.contains("attack") || arch.contains("slayer") { 2.5 }
            else if arch.contains("rf") { 0.2 }
            else { 1.0 }
        }
        StatType::CastSpeed => {
            if arch.contains("spell") || arch.contains("cast") { 2.0 }
            else if arch.contains("rf") { 0.5 }
            else { 1.0 }
        }
        StatType::CritChance | StatType::CritMultiplier => {
            if arch.contains("crit") { 2.5 }
            else if arch.contains("rf") || arch.contains("dot") { 0.3 }
            else { 1.0 }
        }
        StatType::Resistances => 1.5,
        StatType::MovementSpeed => 1.0,
        StatType::Strength | StatType::Intelligence | StatType::Dexterity => 0.5,
        _ => 0.5,
    }
}

pub(crate) fn node_value(node: &PassiveNode, archetype: &str) -> f64 {
    node.stats.iter().map(|s| s.value * archetype_weight(archetype, &s.stat_type)).sum()
}

pub(crate) fn bfs_reachable(
    allocated: &HashSet<u32>,
    nodes:     &HashMap<u32, PassiveNode>,
) -> HashMap<u32, (usize, Vec<u32>)> {
    if allocated.is_empty() {
        return HashMap::new();
    }
    let mut visited: HashMap<u32, (usize, Vec<u32>)> = HashMap::new();
    let mut queue: std::collections::VecDeque<(u32, usize, Vec<u32>)> = std::collections::VecDeque::new();

    for &id in allocated {
        visited.insert(id, (0, vec![id]));
        queue.push_back((id, 0, vec![id]));
    }

    while let Some((current, dist, path)) = queue.pop_front() {
        if let Some(node) = nodes.get(&current) {
            for &neighbor in &node.neighbors {
                if !visited.contains_key(&neighbor) {
                    let new_dist = dist + 1;
                    let mut new_path = path.clone();
                    new_path.push(neighbor);
                    visited.insert(neighbor, (new_dist, new_path.clone()));
                    queue.push_back((neighbor, new_dist, new_path));
                }
            }
        }
    }
    visited
}

pub fn recommend_next_points(
    tree:      &PassiveTree,
    nodes:     &HashMap<u32, PassiveNode>,
    archetype: &str,
    top_n:     usize,
) -> Vec<NodeRecommendation> {
    let allocated: HashSet<u32> = tree.allocated_nodes.iter().cloned().collect();
    let reachable = bfs_reachable(&allocated, nodes);

    let mut candidates: Vec<NodeRecommendation> = reachable.iter()
        .filter(|(id, _)| !allocated.contains(id))
        .filter_map(|(&id, (dist, path))| {
            let node = nodes.get(&id)?;
            if node.is_travel { return None; }
            let value = node_value(node, archetype);
            if value <= 0.0 { return None; }
            let cost = *dist as u32;
            if cost == 0 { return None; }
            let efficiency = value / cost as f64;
            Some(NodeRecommendation {
                node_id:     id,
                node_name:   node.name.clone(),
                stats:       node.stats.iter().map(|s| s.text.clone()).collect(),
                value_score: value,
                path_cost:   cost,
                efficiency,
                path:        path.clone(),
            })
        })
        .collect();

    candidates.sort_by(|a, b| b.efficiency.partial_cmp(&a.efficiency).unwrap_or(std::cmp::Ordering::Equal));
    candidates.truncate(top_n);
    candidates
}

pub fn tree_overlap_pct(a: &[u32], b: &[u32]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let set_a: HashSet<u32> = a.iter().cloned().collect();
    let set_b: HashSet<u32> = b.iter().cloned().collect();
    let shared = set_a.intersection(&set_b).count();
    let union  = set_a.union(&set_b).count();
    if union == 0 { return 0.0; }
    (shared as f64 / union as f64) * 100.0
}

// ─── Tests (written BEFORE implementation) ────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ───────────────────────────────────────────────────────────────

    fn node(id: u32, name: &str, neighbors: Vec<u32>, stats: Vec<PassiveStat>, is_travel: bool) -> PassiveNode {
        PassiveNode { id, name: name.to_string(), stats, neighbors,
            is_keystone: false, is_notable: !is_travel, is_travel, class_start: false }
    }
    fn life_stat(v: f64) -> PassiveStat {
        PassiveStat { text: format!("+{v} to maximum Life"), stat_type: StatType::Life, value: v }
    }
    fn dot_stat(v: f64) -> PassiveStat {
        PassiveStat { text: format!("{v}% DoT Multi"), stat_type: StatType::DotMultiplier, value: v }
    }

    /// Graph: 1(start) — 2(travel) — 4(life notable)
    ///                 \ 3(dot notable)
    fn small_tree() -> HashMap<u32, PassiveNode> {
        let mut m = HashMap::new();
        m.insert(1, node(1, "Start",   vec![2, 3], vec![], false));
        m.insert(2, node(2, "Travel",  vec![1, 4], vec![PassiveStat { text: "+10 Str".into(), stat_type: StatType::Strength, value: 10.0 }], true));
        m.insert(3, node(3, "DoT",     vec![1],    vec![dot_stat(20.0)], false));
        m.insert(4, node(4, "Life",    vec![2],    vec![life_stat(50.0)], false));
        m
    }

    // ── bfs_reachable ─────────────────────────────────────────────────────────

    #[test]
    fn bfs_finds_all_nodes_from_start() {
        let nodes = small_tree();
        let allocated: HashSet<u32> = [1].into();
        let r = bfs_reachable(&allocated, &nodes);
        assert!(r.contains_key(&2));
        assert!(r.contains_key(&3));
        assert!(r.contains_key(&4));
    }

    #[test]
    fn bfs_distance_is_correct() {
        let nodes = small_tree();
        let allocated: HashSet<u32> = [1].into();
        let r = bfs_reachable(&allocated, &nodes);
        assert_eq!(r[&3].0, 1, "node 3 is 1 hop");
        assert_eq!(r[&2].0, 1, "node 2 is 1 hop");
        assert_eq!(r[&4].0, 2, "node 4 is 2 hops via travel node 2");
    }

    #[test]
    fn bfs_allocated_nodes_have_distance_zero() {
        let nodes = small_tree();
        let allocated: HashSet<u32> = [1, 3].into();
        let r = bfs_reachable(&allocated, &nodes);
        assert_eq!(r[&1].0, 0);
        assert_eq!(r[&3].0, 0);
    }

    #[test]
    fn bfs_empty_allocation_returns_empty() {
        let nodes = small_tree();
        let allocated: HashSet<u32> = HashSet::new();
        let r = bfs_reachable(&allocated, &nodes);
        assert!(r.is_empty(), "no seed nodes → no reachable nodes");
    }

    // ── archetype_weight ──────────────────────────────────────────────────────

    #[test]
    fn dot_multiplier_weight_highest_for_fire_dot() {
        let dot_w  = archetype_weight("RFInquisitor", &StatType::DotMultiplier);
        let life_w = archetype_weight("RFInquisitor", &StatType::Life);
        let atk_w  = archetype_weight("RFInquisitor", &StatType::AttackSpeed);
        assert!(dot_w > life_w, "DoT multi should outweigh life for fire DoT");
        assert!(dot_w > atk_w,  "DoT multi should outweigh attack speed for fire DoT");
    }

    #[test]
    fn life_weight_higher_for_life_build_than_es_build() {
        let life_w_life = archetype_weight("LifeBuild", &StatType::Life);
        let life_w_es   = archetype_weight("ESOccultist", &StatType::Life);
        assert!(life_w_life > life_w_es, "Life weight should be lower for ES builds");
    }

    #[test]
    fn weights_are_non_negative() {
        let archetypes = ["RFInquisitor", "AttackSlayer", "ESOccultist"];
        let stats = [StatType::Life, StatType::DotMultiplier, StatType::CritChance,
                     StatType::Armour, StatType::MovementSpeed];
        for arch in &archetypes {
            for stat in &stats {
                assert!(archetype_weight(arch, stat) >= 0.0,
                    "{arch} / {stat:?} weight must be non-negative");
            }
        }
    }

    // ── node_value ────────────────────────────────────────────────────────────

    #[test]
    fn node_value_is_zero_for_empty_stats() {
        let n = node(1, "Empty", vec![], vec![], false);
        assert_eq!(node_value(&n, "RFInquisitor"), 0.0);
    }

    #[test]
    fn rf_values_dot_node_more_than_life_node_of_same_raw_value() {
        let dot_node  = node(1, "DoT",  vec![], vec![dot_stat(20.0)],  false);
        let life_node = node(2, "Life", vec![], vec![life_stat(20.0)], false);
        assert!(node_value(&dot_node, "RFInquisitor") > node_value(&life_node, "RFInquisitor"));
    }

    // ── recommend_next_points ─────────────────────────────────────────────────

    #[test]
    fn recommend_excludes_travel_nodes() {
        let nodes = small_tree();
        let tree = PassiveTree { allocated_nodes: vec![1], ..Default::default() };
        let recs = recommend_next_points(&tree, &nodes, "RFInquisitor", 10);
        let ids: Vec<u32> = recs.iter().map(|r| r.node_id).collect();
        assert!(!ids.contains(&2), "travel node 2 must not appear in recommendations");
    }

    #[test]
    fn recommend_returns_at_most_top_n() {
        let nodes = small_tree();
        let tree = PassiveTree { allocated_nodes: vec![1], ..Default::default() };
        let recs = recommend_next_points(&tree, &nodes, "RFInquisitor", 1);
        assert!(recs.len() <= 1);
    }

    #[test]
    fn recommend_rf_prefers_dot_over_life() {
        let nodes = small_tree();
        let tree = PassiveTree { allocated_nodes: vec![1], ..Default::default() };
        let recs = recommend_next_points(&tree, &nodes, "RFInquisitor", 10);
        let ids: Vec<u32> = recs.iter().map(|r| r.node_id).collect();
        if ids.contains(&3) && ids.contains(&4) {
            let pos3 = ids.iter().position(|&id| id == 3).unwrap();
            let pos4 = ids.iter().position(|&id| id == 4).unwrap();
            assert!(pos3 < pos4, "DoT node(3) should rank above life node(4) for RF archetype");
        }
    }

    #[test]
    fn recommend_sorted_by_efficiency_descending() {
        let nodes = small_tree();
        let tree = PassiveTree { allocated_nodes: vec![1], ..Default::default() };
        let recs = recommend_next_points(&tree, &nodes, "RFInquisitor", 10);
        for w in recs.windows(2) {
            assert!(w[0].efficiency >= w[1].efficiency,
                "recommendations must be sorted by efficiency desc");
        }
    }

    // ── tree_overlap_pct ──────────────────────────────────────────────────────

    #[test]
    fn overlap_identical_is_100() {
        let n = vec![1u32, 2, 3];
        assert!((tree_overlap_pct(&n, &n) - 100.0).abs() < 0.001);
    }

    #[test]
    fn overlap_disjoint_is_zero() {
        assert_eq!(tree_overlap_pct(&[1, 2], &[3, 4]), 0.0);
    }

    #[test]
    fn overlap_partial_correct() {
        // shared={3,4}=2, union={1,2,3,4,5,6}=6 → 33.33%
        let pct = tree_overlap_pct(&[1, 2, 3, 4], &[3, 4, 5, 6]);
        assert!((pct - 33.333).abs() < 0.1, "got {pct}");
    }

    #[test]
    fn overlap_empty_inputs_is_zero() {
        assert_eq!(tree_overlap_pct(&[], &[]), 0.0);
    }
}

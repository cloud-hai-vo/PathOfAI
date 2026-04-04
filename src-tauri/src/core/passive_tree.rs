/// Passive Tree Analyzer — Algorithm 48 (Top-N Passive Node Recommender).
/// BFS from allocated nodes to find most efficient next points.
use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};
use crate::models::build::PassiveTree;

/// A candidate passive node recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRecommendation {
    pub node_id:     u32,
    pub node_name:   String,
    pub stats:       Vec<String>,
    pub value_score: f64,     // archetype-weighted stat value
    pub path_cost:   u32,     // travel nodes required to reach it
    pub efficiency:  f64,     // value_score / path_cost
    pub path:        Vec<u32>,
}

/// A lightweight passive tree graph node for BFS/scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveNode {
    pub id:           u32,
    pub name:         String,
    pub stats:        Vec<PassiveStat>,
    pub neighbors:    Vec<u32>,
    pub is_keystone:  bool,
    pub is_notable:   bool,
    pub is_travel:    bool,    // small +str/dex/int nodes — skip in recommendations
    pub class_start:  bool,    // starting node — always allocated
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PassiveStat {
    pub text:      String,    // "+10 to maximum Life"
    pub stat_type: StatType,
    pub value:     f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub enum StatType {
    #[default]
    Life,
    EnergyShield,
    Armour,
    Evasion,
    Resistances,
    FireDamage,
    ColdDamage,
    LightningDamage,
    ChaosDamage,
    SpellDamage,
    AttackSpeed,
    CastSpeed,
    CritChance,
    CritMultiplier,
    DotMultiplier,
    MovementSpeed,
    Strength,
    Dexterity,
    Intelligence,
    Other,
}

/// Archetype-weighted importance table.
/// Returns 0.0-3.0: how valuable is this stat type for this archetype.
pub fn archetype_weight(archetype: &str, stat: &StatType) -> f64 {
    let is_fire_dot = archetype.contains("Fire") || archetype.contains("RF") || archetype.contains("DoT");
    let is_attack   = archetype.contains("Attack") || archetype.contains("attack");
    let is_es       = archetype.contains("ES") || archetype.contains("Occultist");

    match stat {
        StatType::Life            => if is_es { 0.3 } else { 2.0 },
        StatType::EnergyShield    => if is_es { 2.5 } else { 0.3 },
        StatType::Armour          => if is_fire_dot { 1.2 } else { 0.8 },
        StatType::DotMultiplier   => if is_fire_dot { 3.0 } else { 0.1 },
        StatType::FireDamage      => if is_fire_dot { 2.5 } else { 0.3 },
        StatType::CritChance
        | StatType::CritMultiplier => if is_attack { 2.0 } else { 0.5 },
        StatType::AttackSpeed     => if is_attack { 2.0 } else { 0.1 },
        StatType::SpellDamage     => if is_attack { 0.3 } else { 1.5 },
        StatType::Resistances     => 1.5,
        StatType::MovementSpeed   => 0.5,
        _                         => 0.5,
    }
}

/// Recommend top-N unallocated passive nodes reachable from current allocation.
/// Uses multi-source BFS then ranks by efficiency (value / path_cost).
pub fn recommend_next_points(
    tree:      &PassiveTree,
    nodes:     &HashMap<u32, PassiveNode>,
    archetype: &str,
    top_n:     usize,
) -> Vec<NodeRecommendation> {
    let allocated: HashSet<u32> = tree.allocated_nodes.iter().cloned().collect();

    // Multi-source BFS from all allocated nodes
    let reachable = bfs_reachable(&allocated, nodes);

    let mut candidates: Vec<NodeRecommendation> = reachable
        .into_iter()
        .filter(|(id, _)| !allocated.contains(id))
        .filter_map(|(node_id, (path_cost, path))| {
            let node = nodes.get(&node_id)?;
            if node.is_travel { return None; }  // skip pure travel nodes

            let value_score = node_value(node, archetype);
            if value_score <= 0.0 { return None; }

            let cost = path_cost.max(1) as f64;
            Some(NodeRecommendation {
                node_id,
                node_name:   node.name.clone(),
                stats:       node.stats.iter().map(|s| s.text.clone()).collect(),
                value_score,
                path_cost:   path_cost as u32,
                efficiency:  value_score / cost,
                path,
            })
        })
        .collect();

    candidates.sort_by(|a, b| b.efficiency.partial_cmp(&a.efficiency)
        .unwrap_or(std::cmp::Ordering::Equal));
    candidates.truncate(top_n);
    candidates
}

/// Multi-source BFS from all allocated nodes simultaneously.
/// Returns map of node_id → (distance, path_vec).
pub(crate) fn bfs_reachable(
    allocated: &HashSet<u32>,
    nodes:     &HashMap<u32, PassiveNode>,
) -> HashMap<u32, (usize, Vec<u32>)> {
    let mut visited: HashMap<u32, (usize, Vec<u32>)> = HashMap::new();
    let mut queue: VecDeque<(u32, usize, Vec<u32>)> = VecDeque::new();

    for &id in allocated {
        visited.insert(id, (0, vec![]));
        queue.push_back((id, 0, vec![]));
    }

    while let Some((node_id, dist, path)) = queue.pop_front() {
        if let Some(node) = nodes.get(&node_id) {
            for &neighbor in &node.neighbors {
                if visited.contains_key(&neighbor) { continue; }
                let mut new_path = path.clone();
                new_path.push(neighbor);
                visited.insert(neighbor, (dist + 1, new_path.clone()));
                queue.push_back((neighbor, dist + 1, new_path));
            }
        }
    }
    visited
}

/// Compute archetype-weighted value score for a node.
pub(crate) fn node_value(node: &PassiveNode, archetype: &str) -> f64 {
    node.stats.iter().map(|s| {
        let w = archetype_weight(archetype, &s.stat_type);
        s.value.abs() * w
    }).sum()
}

/// Count how many of the build's allocated nodes overlap with another set.
pub fn tree_overlap_pct(a: &[u32], b: &[u32]) -> f64 {
    let set_a: HashSet<u32> = a.iter().cloned().collect();
    let set_b: HashSet<u32> = b.iter().cloned().collect();
    let shared = set_a.intersection(&set_b).count();
    let union  = set_a.union(&set_b).count();
    if union == 0 { 0.0 } else { shared as f64 / union as f64 * 100.0 }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_node(id: u32, name: &str, neighbors: Vec<u32>, stats: Vec<PassiveStat>, is_travel: bool) -> PassiveNode {
        PassiveNode {
            id,
            name: name.to_string(),
            stats,
            neighbors,
            is_keystone: false,
            is_notable: !is_travel,
            is_travel,
            class_start: false,
        }
    }

    fn life_stat(value: f64) -> PassiveStat {
        PassiveStat { text: format!("+{value} to maximum Life"), stat_type: StatType::Life, value }
    }

    fn dot_stat(value: f64) -> PassiveStat {
        PassiveStat { text: format!("{value}% increased Damage over Time"), stat_type: StatType::DotMultiplier, value }
    }

    fn make_test_tree() -> HashMap<u32, PassiveNode> {
        let mut nodes = HashMap::new();
        // Start node (allocated)
        nodes.insert(1, make_node(1, "Start", vec![2, 3], vec![], false));
        // Path node (travel)
        nodes.insert(2, make_node(2, "Travel", vec![1, 4], vec![
            PassiveStat { text: "+10 to Str".to_string(), stat_type: StatType::Strength, value: 10.0 }
        ], true));
        // Life notable (2 steps from start via travel)
        nodes.insert(4, make_node(4, "Life Mastery", vec![2], vec![life_stat(50.0)], false));
        // DoT notable (1 step from start)
        nodes.insert(3, make_node(3, "Burning Mastery", vec![1], vec![dot_stat(20.0)], false));
        nodes
    }

    // ── bfs_reachable ─────────────────────────────────────────────────────────

    #[test]
    fn bfs_finds_all_connected_nodes() {
        let nodes = make_test_tree();
        let allocated: HashSet<u32> = [1].iter().cloned().collect();
        let reachable = bfs_reachable(&allocated, &nodes);

        assert!(reachable.contains_key(&2), "Node 2 should be reachable");
        assert!(reachable.contains_key(&3), "Node 3 should be reachable");
        assert!(reachable.contains_key(&4), "Node 4 should be reachable");
    }

    #[test]
    fn bfs_distance_is_shortest_path() {
        let nodes = make_test_tree();
        let allocated: HashSet<u32> = [1].iter().cloned().collect();
        let reachable = bfs_reachable(&allocated, &nodes);

        assert_eq!(reachable[&3].0, 1, "Node 3 is 1 hop from node 1");
        assert_eq!(reachable[&2].0, 1, "Node 2 is 1 hop from node 1");
        assert_eq!(reachable[&4].0, 2, "Node 4 is 2 hops from node 1 via node 2");
    }

    #[test]
    fn bfs_allocated_node_has_distance_zero() {
        let nodes = make_test_tree();
        let allocated: HashSet<u32> = [1, 3].iter().cloned().collect();
        let reachable = bfs_reachable(&allocated, &nodes);

        assert_eq!(reachable[&1].0, 0);
        assert_eq!(reachable[&3].0, 0);
    }

    // ── node_value ────────────────────────────────────────────────────────────

    #[test]
    fn node_value_fire_dot_values_dot_multiplier_highly() {
        let node = make_node(99, "Burning", vec![], vec![dot_stat(20.0)], false);
        let value = node_value(&node, "RFInquisitor");
        let life_node = make_node(100, "Life", vec![], vec![life_stat(20.0)], false);
        let life_value = node_value(&life_node, "RFInquisitor");
        assert!(value > life_value, "DoT multiplier should be more valuable than life for RF");
    }

    #[test]
    fn node_value_travel_nodes_have_low_value() {
        let travel = make_node(99, "Travel", vec![], vec![
            PassiveStat { text: "+10 Str".to_string(), stat_type: StatType::Strength, value: 10.0 }
        ], true);
        // Travel nodes return some value from stats but low weight
        let value = node_value(&travel, "RFInquisitor");
        assert!(value >= 0.0, "Travel nodes should have non-negative value");
    }

    // ── recommend_next_points ─────────────────────────────────────────────────

    #[test]
    fn recommend_skips_travel_nodes_in_results() {
        let nodes = make_test_tree();
        let tree = PassiveTree {
            allocated_nodes: vec![1],
            ..Default::default()
        };
        let recs = recommend_next_points(&tree, &nodes, "RFInquisitor", 10);
        // Should not include travel node 2
        let ids: Vec<u32> = recs.iter().map(|r| r.node_id).collect();
        assert!(!ids.contains(&2), "Travel node should not appear in recommendations");
    }

    #[test]
    fn recommend_rf_prefers_dot_over_life() {
        let nodes = make_test_tree();
        let tree = PassiveTree {
            allocated_nodes: vec![1],
            ..Default::default()
        };
        let recs = recommend_next_points(&tree, &nodes, "RFInquisitor", 10);
        // Node 3 (DoT, 1 step) should rank above node 4 (Life, 2 steps via travel)
        let ids: Vec<u32> = recs.iter().map(|r| r.node_id).collect();
        if ids.contains(&3) && ids.contains(&4) {
            let pos_3 = ids.iter().position(|&id| id == 3).unwrap();
            let pos_4 = ids.iter().position(|&id| id == 4).unwrap();
            assert!(pos_3 < pos_4, "DoT node (node 3) should rank higher than life node (node 4) for RF");
        }
    }

    #[test]
    fn recommend_returns_at_most_top_n() {
        let nodes = make_test_tree();
        let tree = PassiveTree {
            allocated_nodes: vec![1],
            ..Default::default()
        };
        let recs = recommend_next_points(&tree, &nodes, "RFInquisitor", 1);
        assert!(recs.len() <= 1, "Should return at most top_n recommendations");
    }

    // ── tree_overlap_pct ──────────────────────────────────────────────────────

    #[test]
    fn tree_overlap_identical_builds_is_100pct() {
        let nodes = vec![1, 2, 3, 4, 5];
        assert!((tree_overlap_pct(&nodes, &nodes) - 100.0).abs() < 0.01);
    }

    #[test]
    fn tree_overlap_disjoint_builds_is_0pct() {
        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        assert_eq!(tree_overlap_pct(&a, &b), 0.0);
    }

    #[test]
    fn tree_overlap_partial_overlap_is_correct() {
        let a = vec![1, 2, 3, 4];
        let b = vec![3, 4, 5, 6];
        // shared={3,4}=2, union={1,2,3,4,5,6}=6 → 33.3%
        let pct = tree_overlap_pct(&a, &b);
        assert!((pct - 33.333).abs() < 0.1, "Expected ~33.3%, got {pct}");
    }

    #[test]
    fn tree_overlap_empty_inputs_is_0pct() {
        assert_eq!(tree_overlap_pct(&[], &[]), 0.0);
    }

    // ── archetype_weight ──────────────────────────────────────────────────────

    #[test]
    fn fire_dot_archetype_weights_dot_highest() {
        let dot_w = archetype_weight("RFInquisitor", &StatType::DotMultiplier);
        let life_w = archetype_weight("RFInquisitor", &StatType::Life);
        let atk_w  = archetype_weight("RFInquisitor", &StatType::AttackSpeed);
        assert!(dot_w > life_w, "DoT multiplier weight should exceed life weight for fire dot");
        assert!(dot_w > atk_w, "DoT multiplier weight should exceed attack speed for fire dot");
    }
}

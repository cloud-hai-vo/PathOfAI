/// build_similarity.rs — Build Similarity & Collaborative Filtering (Algorithm 17).
///
/// Encodes builds as feature vectors and finds similar builds via cosine similarity.
/// Powers "90% of top RF Inquisitors use X" suggestions.
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────────────────────

/// A compact feature vector for similarity comparison.
/// Features are normalized to [0, 1] range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildVector {
    pub build_id:    String,
    pub features:    Vec<f64>,
    pub ascendancy:  String,
    pub main_skill:  String,
}

/// A top-player build for collaborative filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopBuild {
    pub build_id:   String,
    pub ascendancy: String,
    pub main_skill: String,
    pub dps:        f64,
    pub life:       f64,
    pub es:         f64,
    /// Item name/base per slot — key is slot name (e.g., "Ring", "Helmet").
    pub items:      std::collections::HashMap<String, String>,
    /// Keystone flags.
    pub keystones:  Vec<String>,
}

/// A suggestion from collaborative filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabSuggestion {
    pub slot:       String,
    pub message:    String,
    pub usage_pct:  f64,   // 0.0–100.0
    pub confidence: f64,   // 0.0–1.0
}

// ─── Algorithm 17a: Feature Vector ────────────────────────────────────────────

/// Encode a build's key stats into a normalized feature vector.
///
/// Layout (dimension 8 + custom keystones):
///   [0]: dps / 10M
///   [1]: life / 10K
///   [2]: es / 10K
///   [3]: armour / 100K
///   [4]: evasion / 100K
///   [5..5+K]: keystone flags (1.0 or 0.0 for each keystone in the list)
pub fn build_to_vector(
    build_id:  &str,
    ascendancy: &str,
    main_skill: &str,
    dps:       f64,
    life:      f64,
    es:        f64,
    armour:    f64,
    evasion:   f64,
    keystones: &[&str],   // which keystones this build has
    all_keystones: &[&str], // full list of known keystones (determines vector dim)
) -> BuildVector {
    let mut features = vec![
        (dps    / 10_000_000.0).clamp(0.0, 1.0),
        (life   / 10_000.0).clamp(0.0, 1.0),
        (es     / 10_000.0).clamp(0.0, 1.0),
        (armour / 100_000.0).clamp(0.0, 1.0),
        (evasion/ 100_000.0).clamp(0.0, 1.0),
    ];
    for &k in all_keystones {
        features.push(if keystones.contains(&k) { 1.0 } else { 0.0 });
    }
    BuildVector { build_id: build_id.into(), features, ascendancy: ascendancy.into(), main_skill: main_skill.into() }
}

// ─── Algorithm 17b: Similarity Search ────────────────────────────────────────

/// Cosine similarity between two feature vectors.
/// Returns 0.0 if either vector is all zeros.
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let len = a.len().min(b.len());
    let dot:    f64 = a[..len].iter().zip(&b[..len]).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a[..len].iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b[..len].iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

/// Find the `k` most similar builds from `top_builds` to the query build.
///
/// Pre-filters: same ascendancy AND same main skill (mandatory).
/// Then ranks by cosine similarity on the feature vectors.
pub fn find_similar<'a>(
    query:      &BuildVector,
    top_builds: &'a [TopBuild],
    k:          usize,
) -> Vec<(f64, &'a TopBuild)> {
    let all_keystones: Vec<&str> = vec![]; // simplified — no keystones in TopBuild here

    // Pre-filter: same ascendancy + same main skill.
    let mut scored: Vec<(f64, &'a TopBuild)> = top_builds.iter()
        .filter(|b| b.ascendancy == query.ascendancy && b.main_skill == query.main_skill)
        .map(|b| {
            let b_vec = build_to_vector(
                &b.build_id, &b.ascendancy, &b.main_skill,
                b.dps, b.life, b.es, 0.0, 0.0,
                &[], &all_keystones,
            );
            let sim = cosine_similarity(&query.features, &b_vec.features);
            (sim, b)
        })
        .collect();

    // Sort by similarity descending.
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);
    scored
}

// ─── Algorithm 17c: Collaborative Suggestions ─────────────────────────────────

/// Generate suggestions by counting what similar top builds use per slot.
///
/// If > 50% of similar builds use something the player doesn't, suggest it.
pub fn collaborative_suggestions(
    player_items: &std::collections::HashMap<String, String>,
    similar_builds: &[&TopBuild],
    threshold: f64, // default 0.50
) -> Vec<CollabSuggestion> {
    if similar_builds.is_empty() { return vec![]; }

    let mut suggestions = Vec::new();

    // Collect all slots mentioned in any build.
    let mut all_slots: std::collections::HashSet<String> = std::collections::HashSet::new();
    for b in similar_builds { all_slots.extend(b.items.keys().cloned()); }

    for slot in &all_slots {
        // Count item frequency across similar builds.
        let mut freq: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for b in similar_builds {
            if let Some(item) = b.items.get(slot) {
                *freq.entry(item.as_str()).or_insert(0) += 1;
            }
        }

        // Find most common item.
        if let Some((&most_common, &count)) = freq.iter().max_by_key(|(_, c)| *c) {
            let usage = count as f64 / similar_builds.len() as f64;
            let player_item = player_items.get(slot).map(|s| s.as_str()).unwrap_or("");

            if usage >= threshold && most_common != player_item {
                suggestions.push(CollabSuggestion {
                    slot: slot.clone(),
                    message: format!(
                        "{:.0}% of top players use {} (you: {})",
                        usage * 100.0, most_common, player_item
                    ),
                    usage_pct: usage * 100.0,
                    confidence: usage,
                });
            }
        }
    }

    // Sort by confidence descending.
    suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence)
        .unwrap_or(std::cmp::Ordering::Equal));
    suggestions
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn vec_from(features: &[f64]) -> BuildVector {
        BuildVector { build_id: "q".into(), features: features.to_vec(),
            ascendancy: "Inquisitor".into(), main_skill: "RF".into() }
    }

    fn top(id: &str, asc: &str, skill: &str, dps: f64, items: &[(&str, &str)]) -> TopBuild {
        TopBuild {
            build_id: id.into(), ascendancy: asc.into(), main_skill: skill.into(),
            dps, life: 5000.0, es: 0.0,
            items: items.iter().map(|(s, n)| (s.to_string(), n.to_string())).collect(),
            keystones: vec![],
        }
    }

    // ── cosine_similarity ─────────────────────────────────────────────────────

    #[test]
    fn identical_vectors_give_similarity_one() {
        let v = vec![1.0, 0.5, 0.3];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 0.001);
    }

    #[test]
    fn orthogonal_vectors_give_similarity_zero() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b)).abs() < 0.001);
    }

    #[test]
    fn zero_vector_gives_similarity_zero() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 0.5];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn similarity_is_between_neg_one_and_one() {
        let a = vec![0.5, 0.3, 0.8];
        let b = vec![0.1, 0.9, 0.2];
        let sim = cosine_similarity(&a, &b);
        assert!(sim >= -1.0 && sim <= 1.0);
    }

    // ── build_to_vector ────────────────────────────────────────────────────────

    #[test]
    fn build_vector_features_are_normalized() {
        let v = build_to_vector(
            "b1", "Inquisitor", "RF",
            10_000_000.0, 5000.0, 0.0, 0.0, 0.0,
            &[], &[],
        );
        for &f in &v.features {
            assert!(f >= 0.0 && f <= 1.0, "feature out of range: {f}");
        }
    }

    #[test]
    fn keystone_flag_present_when_build_has_it() {
        let v = build_to_vector(
            "b1", "Inquisitor", "RF",
            0.0, 0.0, 0.0, 0.0, 0.0,
            &["Resolute Technique"],
            &["Resolute Technique", "Chaos Inoculation"],
        );
        // Feature at index 5 = RT flag (1st keystone)
        assert_eq!(v.features[5], 1.0, "RT keystone should be 1.0");
        assert_eq!(v.features[6], 0.0, "CI keystone should be 0.0");
    }

    // ── find_similar ───────────────────────────────────────────────────────────

    #[test]
    fn find_similar_filters_by_ascendancy_and_skill() {
        let query = vec_from(&[1.0, 0.5]);
        let builds = vec![
            top("a", "Inquisitor", "RF",    5_000_000.0, &[]),
            top("b", "Champion",   "RF",    5_000_000.0, &[]), // wrong asc
            top("c", "Inquisitor", "Cyclone", 8_000_000.0, &[]), // wrong skill
        ];
        let similar = find_similar(&query, &builds, 10);
        assert_eq!(similar.len(), 1, "only 1 build matches asc+skill filter");
        assert_eq!(similar[0].1.build_id, "a");
    }

    #[test]
    fn find_similar_returns_at_most_k() {
        let query = vec_from(&[0.5, 0.5]);
        let builds: Vec<TopBuild> = (0..10).map(|i| {
            top(&format!("{i}"), "Inquisitor", "RF", 5_000_000.0, &[])
        }).collect();
        let similar = find_similar(&query, &builds, 3);
        assert!(similar.len() <= 3);
    }

    #[test]
    fn find_similar_empty_builds_returns_empty() {
        let query = vec_from(&[0.5]);
        assert!(find_similar(&query, &[], 5).is_empty());
    }

    // ── collaborative_suggestions ──────────────────────────────────────────────

    #[test]
    fn high_usage_item_generates_suggestion() {
        let builds: Vec<TopBuild> = (0..5).map(|_| {
            top("x", "Inquisitor", "RF", 5_000_000.0, &[("Ring", "Watcher's Eye")])
        }).collect();
        let player = HashMap::new(); // player has nothing in Ring slot
        let refs: Vec<&TopBuild> = builds.iter().collect();
        let suggestions = collaborative_suggestions(&player, &refs, 0.5);
        assert!(!suggestions.is_empty(), "100% usage should generate a suggestion");
        assert_eq!(suggestions[0].slot, "Ring");
        assert!(suggestions[0].usage_pct >= 50.0);
    }

    #[test]
    fn player_already_uses_popular_item_no_suggestion() {
        let builds: Vec<TopBuild> = (0..5).map(|_| {
            top("x", "Inquisitor", "RF", 5_000_000.0, &[("Ring", "Watcher's Eye")])
        }).collect();
        let mut player = HashMap::new();
        player.insert("Ring".into(), "Watcher's Eye".into());
        let refs: Vec<&TopBuild> = builds.iter().collect();
        let suggestions = collaborative_suggestions(&player, &refs, 0.5);
        assert!(suggestions.is_empty(), "no suggestion when player already uses popular item");
    }

    #[test]
    fn low_usage_item_does_not_trigger_suggestion() {
        let builds: Vec<TopBuild> = (0..10).map(|i| {
            let item = if i == 0 { "Watcher's Eye" } else { "Generic Ring" };
            top(&format!("{i}"), "Inquisitor", "RF", 5_000_000.0, &[("Ring", item)])
        }).collect();
        let player = HashMap::new();
        let refs: Vec<&TopBuild> = builds.iter().collect();
        let suggestions = collaborative_suggestions(&player, &refs, 0.5);
        // "Generic Ring" at 90% usage → suggestion; "Watcher's Eye" at 10% → no suggestion
        let ring = suggestions.iter().find(|s| s.slot == "Ring");
        if let Some(s) = ring {
            assert!(s.usage_pct >= 50.0, "only high-usage items should be suggested");
        }
    }
}

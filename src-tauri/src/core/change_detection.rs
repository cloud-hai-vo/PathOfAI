/// change_detection.rs — Hash-Based Lazy Recalculation (Algorithm 24).
///
/// Computes structural hashes of build sections and determines what needs
/// to be recalculated when the build changes.
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ─── Types ────────────────────────────────────────────────────────────────────

/// Per-section hash snapshot of a build.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BuildHash {
    pub items_hash:  u64,
    pub tree_hash:   u64,
    pub gems_hash:   u64,
    pub config_hash: u64,
}

/// What needs to be recalculated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecalcScope {
    /// Nothing changed — use cached results.
    None,
    /// Only offensive stats need recalculation (gem change).
    OffenseOnly,
    /// Only defensive stats need recalculation (flask change).
    DefenseOnly,
    /// Both offense and defense need recalculation.
    Full,
}

/// Cached calculator results with associated build hash.
#[derive(Debug, Clone, Default)]
pub struct CalcCache {
    pub last_hash:    BuildHash,
    pub has_result:   bool,
    /// Opaque result data (serialized JSON or a struct — caller manages this).
    pub offense_valid: bool,
    pub defense_valid: bool,
}

// ─── Hashing Helpers ──────────────────────────────────────────────────────────

/// Hash a sequence of string slices using the standard hasher.
pub fn hash_strings(values: &[&str]) -> u64 {
    let mut h = DefaultHasher::new();
    for s in values { s.hash(&mut h); }
    h.finish()
}

/// Hash a sequence of (key, value) f64 pairs.
pub fn hash_f64_pairs(pairs: &[(&str, f64)]) -> u64 {
    let mut h = DefaultHasher::new();
    for (k, v) in pairs {
        k.hash(&mut h);
        v.to_bits().hash(&mut h);
    }
    h.finish()
}

/// Hash a u32 slice (e.g., allocated passive node IDs).
pub fn hash_u32_slice(values: &[u32]) -> u64 {
    let mut h = DefaultHasher::new();
    values.hash(&mut h);
    h.finish()
}

// ─── Algorithm ────────────────────────────────────────────────────────────────

/// Determine what recalculation is needed by comparing old and new hashes.
pub fn should_recalculate(cache: &CalcCache, new_hash: &BuildHash) -> RecalcScope {
    if !cache.has_result {
        return RecalcScope::Full;
    }
    if new_hash == &cache.last_hash {
        return RecalcScope::None;
    }

    let items_changed  = new_hash.items_hash  != cache.last_hash.items_hash;
    let tree_changed   = new_hash.tree_hash   != cache.last_hash.tree_hash;
    let gems_changed   = new_hash.gems_hash   != cache.last_hash.gems_hash;
    let config_changed = new_hash.config_hash != cache.last_hash.config_hash;

    match (items_changed || tree_changed || config_changed, gems_changed) {
        (true, _)      => RecalcScope::Full,
        (false, true)  => RecalcScope::OffenseOnly,
        (false, false) => RecalcScope::None, // unreachable (hashes differ above)
    }
}

/// Update the cache after a recalculation.
pub fn update_cache(cache: &mut CalcCache, new_hash: BuildHash, scope: &RecalcScope) {
    cache.last_hash = new_hash;
    cache.has_result = true;
    match scope {
        RecalcScope::Full       => { cache.offense_valid = true; cache.defense_valid = true; }
        RecalcScope::OffenseOnly => { cache.offense_valid = true; }
        RecalcScope::DefenseOnly => { cache.defense_valid = true; }
        RecalcScope::None       => {}
    }
}

/// Compute a combined hash from separate build sections.
///
/// Callers are expected to call `hash_strings` / `hash_u32_slice` for each
/// section and pass the results here.
pub fn compute_build_hash(
    items_hash:  u64,
    tree_hash:   u64,
    gems_hash:   u64,
    config_hash: u64,
) -> BuildHash {
    BuildHash { items_hash, tree_hash, gems_hash, config_hash }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_cache() -> CalcCache { CalcCache::default() }

    fn hash(items: u64, tree: u64, gems: u64, config: u64) -> BuildHash {
        BuildHash { items_hash: items, tree_hash: tree, gems_hash: gems, config_hash: config }
    }

    #[test]
    fn no_result_in_cache_requires_full_recalc() {
        let cache = empty_cache();
        let new = hash(1, 2, 3, 4);
        assert_eq!(should_recalculate(&cache, &new), RecalcScope::Full);
    }

    #[test]
    fn identical_hashes_require_no_recalc() {
        let mut cache = empty_cache();
        let h = hash(1, 2, 3, 4);
        update_cache(&mut cache, h.clone(), &RecalcScope::Full);
        assert_eq!(should_recalculate(&cache, &h), RecalcScope::None);
    }

    #[test]
    fn item_change_requires_full_recalc() {
        let mut cache = empty_cache();
        let old = hash(1, 2, 3, 4);
        update_cache(&mut cache, old.clone(), &RecalcScope::Full);
        let new = hash(99, 2, 3, 4); // items changed
        assert_eq!(should_recalculate(&cache, &new), RecalcScope::Full);
    }

    #[test]
    fn tree_change_requires_full_recalc() {
        let mut cache = empty_cache();
        let old = hash(1, 2, 3, 4);
        update_cache(&mut cache, old.clone(), &RecalcScope::Full);
        let new = hash(1, 99, 3, 4); // tree changed
        assert_eq!(should_recalculate(&cache, &new), RecalcScope::Full);
    }

    #[test]
    fn gem_only_change_requires_offense_only_recalc() {
        let mut cache = empty_cache();
        let old = hash(1, 2, 3, 4);
        update_cache(&mut cache, old.clone(), &RecalcScope::Full);
        let new = hash(1, 2, 99, 4); // only gems changed
        assert_eq!(should_recalculate(&cache, &new), RecalcScope::OffenseOnly);
    }

    #[test]
    fn config_change_requires_full_recalc() {
        let mut cache = empty_cache();
        let old = hash(1, 2, 3, 4);
        update_cache(&mut cache, old.clone(), &RecalcScope::Full);
        let new = hash(1, 2, 3, 99); // config changed
        assert_eq!(should_recalculate(&cache, &new), RecalcScope::Full);
    }

    #[test]
    fn hash_strings_is_deterministic() {
        let h1 = hash_strings(&["item1", "item2"]);
        let h2 = hash_strings(&["item1", "item2"]);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_strings_differs_for_different_input() {
        let h1 = hash_strings(&["item1"]);
        let h2 = hash_strings(&["item2"]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_u32_slice_is_order_sensitive() {
        let h1 = hash_u32_slice(&[1, 2, 3]);
        let h2 = hash_u32_slice(&[3, 2, 1]);
        assert_ne!(h1, h2, "different orderings should produce different hashes");
    }

    #[test]
    fn hash_f64_pairs_is_deterministic() {
        let h1 = hash_f64_pairs(&[("dps", 1_000_000.0), ("life", 5000.0)]);
        let h2 = hash_f64_pairs(&[("dps", 1_000_000.0), ("life", 5000.0)]);
        assert_eq!(h1, h2);
    }

    #[test]
    fn update_cache_sets_has_result() {
        let mut cache = empty_cache();
        assert!(!cache.has_result);
        update_cache(&mut cache, hash(1, 2, 3, 4), &RecalcScope::Full);
        assert!(cache.has_result);
    }

    #[test]
    fn update_cache_offense_only_marks_only_offense_valid() {
        let mut cache = empty_cache();
        update_cache(&mut cache, hash(1, 2, 3, 4), &RecalcScope::OffenseOnly);
        assert!(cache.offense_valid);
        assert!(!cache.defense_valid);
    }
}

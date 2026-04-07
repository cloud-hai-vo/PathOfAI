/// change_history.rs — Change History Manager (Algorithm 33).
///
/// Ring-buffer undo/redo stack. Every build modification pushes a Snapshot.
/// Undo steps backward; Redo steps forward. Revert creates a new snapshot
/// so the revert itself is undoable.
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::models::build::BuildData;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeSource {
    UserEdit,
    FileWatcher,
    AiSuggestion(String), // suggestion UUID
    OAuthSync,
    ManualImport,
    Undo,
    Redo,
}

/// Lightweight stats snapshot stored with every history entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SnapshotStats {
    pub total_dps:     f64,
    pub effective_hp:  f64,
    pub fire_res:      f64,
    pub cold_res:      f64,
    pub lightning_res: f64,
    pub chaos_res:     f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id:          String,       // UUID string
    pub description: String,
    pub source:      ChangeSource,
    pub build_state: BuildData,
    /// Stats at the moment of capture (before calc runs again).
    pub stats_before: SnapshotStats,
    /// Populated after the calculator runs on this snapshot.
    pub stats_after:  SnapshotStats,
}

/// Diff between two consecutive snapshots — used for the "Apply & Simulate" panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatDiff {
    pub dps_before:  f64,
    pub dps_after:   f64,
    pub dps_pct:     f64,
    pub life_before: f64,
    pub life_after:  f64,
}

/// Single entry in the timeline view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub id:          String,
    pub description: String,
    pub source:      ChangeSource,
    pub is_current:  bool,
    pub is_redo:     bool,
}

// ─── Change History ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ChangeHistory {
    snapshots:     VecDeque<Snapshot>,
    cursor:        usize,
    max_snapshots: usize,
}

impl ChangeHistory {
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots:     VecDeque::new(),
            cursor:        0,
            max_snapshots: max_snapshots.max(1),
        }
    }

    // ── Push ──────────────────────────────────────────────────────────────────

    /// Record a new snapshot. Clears the redo stack (entries after cursor).
    /// Returns the ID of the new snapshot.
    pub fn push(&mut self, mut snap: Snapshot) -> String {
        // Trim redo stack: discard everything after cursor
        if !self.snapshots.is_empty() {
            self.snapshots.truncate(self.cursor + 1);
        }

        // Inherit stats_before from previous current snapshot's stats_after
        if let Some(prev) = self.snapshots.back() {
            snap.stats_before = prev.stats_after.clone();
        }

        // Enforce ring-buffer limit: drop oldest
        if self.snapshots.len() >= self.max_snapshots {
            self.snapshots.pop_front();
        }

        let id = snap.id.clone();
        self.snapshots.push_back(snap);
        self.cursor = self.snapshots.len() - 1;
        id
    }

    // ── Undo / Redo ───────────────────────────────────────────────────────────

    /// Step backward one snapshot. Returns a reference if possible.
    pub fn undo(&mut self) -> Option<&Snapshot> {
        if self.cursor == 0 || self.snapshots.is_empty() {
            return None;
        }
        self.cursor -= 1;
        Some(&self.snapshots[self.cursor])
    }

    /// Step forward if a redo stack exists.
    pub fn redo(&mut self) -> Option<&Snapshot> {
        if self.cursor + 1 >= self.snapshots.len() {
            return None;
        }
        self.cursor += 1;
        Some(&self.snapshots[self.cursor])
    }

    // ── Revert ────────────────────────────────────────────────────────────────

    /// Revert to ANY earlier snapshot by ID. Creates a new snapshot so the
    /// revert itself is undoable. Returns the cloned target snapshot on success.
    pub fn revert_to(&mut self, target_id: &str) -> Option<Snapshot> {
        let target = self.snapshots.iter().find(|s| s.id == target_id)?.clone();
        let revert_snap = Snapshot {
            id:          uuid_v4_string(),
            description: format!("Reverted to: {}", target.description),
            source:      ChangeSource::Undo,
            build_state: target.build_state.clone(),
            stats_before: SnapshotStats::default(),
            stats_after:  SnapshotStats::default(),
        };
        self.push(revert_snap);
        Some(target)
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn current(&self) -> Option<&Snapshot> {
        if self.snapshots.is_empty() { return None; }
        self.snapshots.get(self.cursor)
    }

    pub fn len(&self) -> usize { self.snapshots.len() }
    pub fn is_empty(&self) -> bool { self.snapshots.is_empty() }

    pub fn can_undo(&self) -> bool { !self.snapshots.is_empty() && self.cursor > 0 }
    pub fn can_redo(&self) -> bool {
        !self.snapshots.is_empty() && self.cursor + 1 < self.snapshots.len()
    }

    /// Full timeline ordered oldest→newest.
    pub fn timeline(&self) -> Vec<TimelineEntry> {
        self.snapshots.iter().enumerate().map(|(i, s)| TimelineEntry {
            id:          s.id.clone(),
            description: s.description.clone(),
            source:      s.source.clone(),
            is_current:  i == self.cursor,
            is_redo:     i > self.cursor,
        }).collect()
    }

    /// Update the stats_after for the current snapshot (called after calc runs).
    pub fn update_stats_after(&mut self, stats: SnapshotStats) {
        if let Some(snap) = self.snapshots.get_mut(self.cursor) {
            snap.stats_after = stats;
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Simple pseudo-UUID for use without the `uuid` crate.
fn uuid_v4_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("snap-{nanos:08x}")
}

/// Compute a stat diff between two snapshots.
pub fn stat_diff(before: &SnapshotStats, after: &SnapshotStats) -> StatDiff {
    let dps_pct = if before.total_dps.abs() > 0.001 {
        (after.total_dps - before.total_dps) / before.total_dps * 100.0
    } else {
        0.0
    };
    StatDiff {
        dps_before:  before.total_dps,
        dps_after:   after.total_dps,
        dps_pct,
        life_before: before.effective_hp,
        life_after:  after.effective_hp,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::build::{BuildData, BuildSource};

    fn empty_build(name: &str) -> BuildData {
        BuildData {
            id:           name.to_string(),
            name:         name.to_string(),
            class_name:   "Witch".to_string(),
            ascendancy:   "".to_string(),
            level:        1,
            items:        vec![],
            gems:         vec![],
            passive_tree: Default::default(),
            config:       Default::default(),
            source:       BuildSource::Unknown,
        }
    }

    fn snap(id: &str, desc: &str, build_name: &str) -> Snapshot {
        Snapshot {
            id:          id.to_string(),
            description: desc.to_string(),
            source:      ChangeSource::UserEdit,
            build_state: empty_build(build_name),
            stats_before: SnapshotStats::default(),
            stats_after:  SnapshotStats::default(),
        }
    }

    fn history(max: usize) -> ChangeHistory { ChangeHistory::new(max) }

    // ── push ─────────────────────────────────────────────────────────────────

    #[test]
    fn push_first_snapshot_makes_it_current() {
        let mut h = history(10);
        h.push(snap("1", "First", "A"));
        assert_eq!(h.current().unwrap().id, "1");
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn push_multiple_advances_cursor() {
        let mut h = history(10);
        h.push(snap("1", "A", "A"));
        h.push(snap("2", "B", "B"));
        h.push(snap("3", "C", "C"));
        assert_eq!(h.current().unwrap().id, "3");
        assert_eq!(h.len(), 3);
    }

    #[test]
    fn push_respects_ring_buffer_limit() {
        let mut h = history(3);
        h.push(snap("1", "A", "A"));
        h.push(snap("2", "B", "B"));
        h.push(snap("3", "C", "C"));
        h.push(snap("4", "D", "D")); // should evict "1"
        assert_eq!(h.len(), 3);
        assert_eq!(h.current().unwrap().id, "4");
    }

    #[test]
    fn push_clears_redo_stack() {
        let mut h = history(10);
        h.push(snap("1", "A", "A"));
        h.push(snap("2", "B", "B"));
        h.undo();
        // Now cursor is at "1", redo stack has "2"
        assert!(h.can_redo());
        // Push new snapshot — redo stack should be cleared
        h.push(snap("3", "C", "C"));
        assert!(!h.can_redo());
        assert_eq!(h.len(), 2); // "1" and "3"
    }

    // ── undo / redo ───────────────────────────────────────────────────────────

    #[test]
    fn undo_on_empty_returns_none() {
        let mut h = history(10);
        assert!(h.undo().is_none());
    }

    #[test]
    fn undo_single_snapshot_returns_none() {
        let mut h = history(10);
        h.push(snap("1", "A", "A"));
        assert!(h.undo().is_none()); // cursor already at 0
    }

    #[test]
    fn undo_returns_previous_snapshot() {
        let mut h = history(10);
        h.push(snap("1", "A", "A"));
        h.push(snap("2", "B", "B"));
        let prev = h.undo().unwrap();
        assert_eq!(prev.id, "1");
        assert_eq!(h.current().unwrap().id, "1");
    }

    #[test]
    fn redo_after_undo_returns_next() {
        let mut h = history(10);
        h.push(snap("1", "A", "A"));
        h.push(snap("2", "B", "B"));
        h.undo();
        let next = h.redo().unwrap();
        assert_eq!(next.id, "2");
    }

    #[test]
    fn redo_at_end_returns_none() {
        let mut h = history(10);
        h.push(snap("1", "A", "A"));
        assert!(h.redo().is_none());
    }

    #[test]
    fn can_undo_can_redo_flags() {
        let mut h = history(10);
        assert!(!h.can_undo());
        assert!(!h.can_redo());
        h.push(snap("1", "A", "A"));
        h.push(snap("2", "B", "B"));
        assert!(h.can_undo());
        assert!(!h.can_redo());
        h.undo();
        assert!(!h.can_undo());
        assert!(h.can_redo());
    }

    // ── revert_to ────────────────────────────────────────────────────────────

    #[test]
    fn revert_to_unknown_id_returns_none() {
        let mut h = history(10);
        h.push(snap("1", "A", "A"));
        assert!(h.revert_to("nonexistent").is_none());
    }

    #[test]
    fn revert_creates_new_undoable_snapshot() {
        let mut h = history(10);
        h.push(snap("1", "A", "A"));
        h.push(snap("2", "B", "B"));
        h.push(snap("3", "C", "C"));
        // Revert to "1"
        let target = h.revert_to("1").unwrap();
        assert_eq!(target.id, "1");
        // New snapshot at cursor should say "Reverted to: A"
        assert!(h.current().unwrap().description.contains("Reverted to: A"));
        // Revert is itself undoable
        assert!(h.can_undo());
        h.undo();
        assert_eq!(h.current().unwrap().id, "3");
    }

    // ── timeline ─────────────────────────────────────────────────────────────

    #[test]
    fn timeline_marks_current_and_redo() {
        let mut h = history(10);
        h.push(snap("1", "A", "A"));
        h.push(snap("2", "B", "B"));
        h.push(snap("3", "C", "C"));
        h.undo(); // cursor now at index 1 ("2")
        let tl = h.timeline();
        assert_eq!(tl.len(), 3);
        assert!(!tl[0].is_current && !tl[0].is_redo);
        assert!(tl[1].is_current);
        assert!(tl[2].is_redo);
    }

    // ── update_stats_after ───────────────────────────────────────────────────

    #[test]
    fn update_stats_after_patches_current_snapshot() {
        let mut h = history(10);
        h.push(snap("1", "A", "A"));
        let new_stats = SnapshotStats { total_dps: 1_000_000.0, ..Default::default() };
        h.update_stats_after(new_stats.clone());
        assert_eq!(h.current().unwrap().stats_after.total_dps, 1_000_000.0);
    }

    // ── stat_diff ─────────────────────────────────────────────────────────────

    #[test]
    fn stat_diff_positive_dps_gain() {
        let before = SnapshotStats { total_dps: 1_000.0, ..Default::default() };
        let after  = SnapshotStats { total_dps: 1_100.0, ..Default::default() };
        let d = stat_diff(&before, &after);
        assert!((d.dps_pct - 10.0).abs() < 0.01);
    }

    #[test]
    fn stat_diff_zero_base_dps_does_not_divide_by_zero() {
        let before = SnapshotStats { total_dps: 0.0, ..Default::default() };
        let after  = SnapshotStats { total_dps: 500.0, ..Default::default() };
        let d = stat_diff(&before, &after);
        assert_eq!(d.dps_pct, 0.0);
    }
}

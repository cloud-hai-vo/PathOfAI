/// PoB File Watcher — Algorithm 44b
/// Watches a directory for *.xml / *.pob changes with 500ms debounce.
/// On change: spawns re-analysis task and emits `pob-file-changed` event.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Tracks per-file last-event time for debouncing.
pub struct PobFileWatcher {
    debounce_ms: u64,
    last_seen:   HashMap<PathBuf, Instant>,
}

impl PobFileWatcher {
    pub fn new(debounce_ms: u64) -> Self {
        Self { debounce_ms, last_seen: HashMap::new() }
    }

    /// Returns true if enough time has passed since the last event for this path.
    /// Calling this function updates the internal timestamp.
    pub fn should_process(&mut self, path: &Path) -> bool {
        let now = Instant::now();
        let entry = self.last_seen.entry(path.to_path_buf()).or_insert(
            now.checked_sub(std::time::Duration::from_secs(10)).unwrap_or(now)
        );
        let elapsed = entry.elapsed().as_millis() as u64;
        if elapsed >= self.debounce_ms {
            *entry = now;
            true
        } else {
            false
        }
    }

    /// Returns true if the path is a PoB file (xml or pob extension).
    pub fn is_pob_file(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("xml" | "pob")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn is_pob_file_accepts_xml() {
        assert!(PobFileWatcher::is_pob_file(Path::new("build.xml")));
    }

    #[test]
    fn is_pob_file_accepts_pob() {
        assert!(PobFileWatcher::is_pob_file(Path::new("build.pob")));
    }

    #[test]
    fn is_pob_file_rejects_other() {
        assert!(!PobFileWatcher::is_pob_file(Path::new("notes.txt")));
        assert!(!PobFileWatcher::is_pob_file(Path::new("data.json")));
    }

    #[test]
    fn first_event_is_processed() {
        let mut w = PobFileWatcher::new(500);
        assert!(w.should_process(Path::new("build.xml")));
    }

    #[test]
    fn rapid_second_event_is_debounced() {
        let mut w = PobFileWatcher::new(500);
        w.should_process(Path::new("build.xml"));
        // Second call immediately after — should be debounced
        assert!(!w.should_process(Path::new("build.xml")));
    }

    #[test]
    fn different_files_tracked_independently() {
        let mut w = PobFileWatcher::new(500);
        assert!(w.should_process(Path::new("build_a.xml")));
        assert!(w.should_process(Path::new("build_b.xml")));
    }

    #[test]
    fn event_processed_after_debounce_window() {
        let mut w = PobFileWatcher::new(50); // 50ms for fast test
        w.should_process(Path::new("build.xml"));
        std::thread::sleep(Duration::from_millis(60));
        assert!(w.should_process(Path::new("build.xml")));
    }
}

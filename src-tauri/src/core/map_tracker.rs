/// map_tracker.rs — Map run tracker (Algorithms 41 + 53).
/// Tests written FIRST (TDD RED). Run `cargo test map_tracker` → all FAIL → then implement.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ZoneKind {
    Map,
    HideoutOrTown,
    Area,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneEvent {
    pub timestamp_secs: u64,
    pub zone_name:      String,
    pub kind:           ZoneKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapRun {
    pub zone_name:      String,
    pub started_at:     u64,
    pub ended_at:       u64,
    pub duration_secs:  u64,
    pub loot_chaos:     f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MapStats {
    pub total_runs:       u32,
    pub total_time_secs:  u64,
    pub avg_duration:     f64,
    pub total_loot_chaos: f64,
    pub chaos_per_hour:   f64,
    pub most_run_map:     String,
    pub by_zone:          HashMap<String, u32>,
}

// ─── Stubs → unimplemented!() → RED ──────────────────────────────────────────

/// Parse a single Client.txt log line into a ZoneEvent, or None.
/// Log format: `2024/01/15 14:23:45 [INFO Client 12345] : You have entered Lookout.`
pub fn parse_log_line(line: &str) -> Option<ZoneEvent> {
    // Match "You have entered <zone>."
    let entry_marker = ": You have entered ";
    let pos = line.find(entry_marker)?;
    let rest = &line[pos + entry_marker.len()..];
    let zone_name = rest.trim_end_matches('.').trim().to_string();
    if zone_name.is_empty() { return None; }

    // Parse timestamp from the start of the line "YYYY/MM/DD HH:MM:SS"
    let ts_str = line.get(..19)?;
    let timestamp_secs = parse_poe_timestamp(ts_str)?;
    let kind = classify_zone(&zone_name);

    Some(ZoneEvent { timestamp_secs, zone_name, kind })
}

pub fn parse_poe_timestamp(ts: &str) -> Option<u64> {
    // Expected: "YYYY/MM/DD HH:MM:SS"
    let parts: Vec<&str> = ts.split_whitespace().collect();
    if parts.len() != 2 { return None; }
    let date_parts: Vec<u64> = parts[0].split('/').filter_map(|p| p.parse().ok()).collect();
    let time_parts: Vec<u64> = parts[1].split(':').filter_map(|p| p.parse().ok()).collect();
    if date_parts.len() != 3 || time_parts.len() != 3 { return None; }

    let (y, m, d) = (date_parts[0], date_parts[1], date_parts[2]);
    let (h, min, s) = (time_parts[0], time_parts[1], time_parts[2]);

    // Simple Unix approximation (not accounting for leap seconds/DST, good enough for ordering)
    let days_since_epoch = (y - 1970) * 365 + (y - 1969) / 4 + month_days(y, m) + d - 1;
    let secs = days_since_epoch * 86400 + h * 3600 + min * 60 + s;
    Some(secs)
}

fn month_days(year: u64, month: u64) -> u64 {
    let days_in_months = [0u64, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let base = if month <= 12 { days_in_months[(month - 1) as usize] } else { 0 };
    let leap = if month > 2 && year % 4 == 0 { 1 } else { 0 };
    base + leap
}

const TOWNS: &[&str] = &[
    "Lioneye's Watch", "The Forest Encampment", "The Sarn Encampment",
    "Highgate", "Overseer's Tower", "The Bridge Encampment",
    "Oriath", "Karui Shores", "The Rogue Harbour",
    "Oriath Docks", "The Twilight Strand",
];

pub fn classify_zone(name: &str) -> ZoneKind {
    if TOWNS.iter().any(|&t| t.eq_ignore_ascii_case(name)) {
        ZoneKind::HideoutOrTown
    } else if name.to_lowercase().contains("hideout") {
        ZoneKind::HideoutOrTown
    } else {
        ZoneKind::Map
    }
}

pub fn build_map_runs(events: &[ZoneEvent]) -> Vec<MapRun> {
    let mut runs = Vec::new();
    let mut current_map: Option<&ZoneEvent> = None;

    for ev in events {
        match ev.kind {
            ZoneKind::Map => {
                if let Some(prev) = current_map {
                    runs.push(MapRun {
                        zone_name:     prev.zone_name.clone(),
                        started_at:    prev.timestamp_secs,
                        ended_at:      ev.timestamp_secs,
                        duration_secs: ev.timestamp_secs.saturating_sub(prev.timestamp_secs),
                        loot_chaos:    0.0,
                    });
                }
                current_map = Some(ev);
            }
            ZoneKind::HideoutOrTown | ZoneKind::Area => {
                if let Some(prev) = current_map.take() {
                    runs.push(MapRun {
                        zone_name:     prev.zone_name.clone(),
                        started_at:    prev.timestamp_secs,
                        ended_at:      ev.timestamp_secs,
                        duration_secs: ev.timestamp_secs.saturating_sub(prev.timestamp_secs),
                        loot_chaos:    0.0,
                    });
                }
            }
        }
    }
    runs
}

pub fn accumulate_stats(runs: &[MapRun]) -> MapStats {
    if runs.is_empty() {
        return MapStats::default();
    }
    let total_runs = runs.len() as u32;
    let total_time_secs: u64 = runs.iter().map(|r| r.duration_secs).sum();
    let total_loot_chaos: f64 = runs.iter().map(|r| r.loot_chaos).sum();
    let avg_duration = total_time_secs as f64 / total_runs as f64;
    let chaos_per_hour = if total_time_secs > 0 {
        total_loot_chaos / (total_time_secs as f64 / 3600.0)
    } else { 0.0 };

    let mut by_zone: HashMap<String, u32> = HashMap::new();
    for r in runs { *by_zone.entry(r.zone_name.clone()).or_insert(0) += 1; }
    let most_run_map = by_zone.iter().max_by_key(|(_, &c)| c)
        .map(|(k, _)| k.clone()).unwrap_or_default();

    MapStats { total_runs, total_time_secs, avg_duration, total_loot_chaos, chaos_per_hour, most_run_map, by_zone }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── classify_zone ─────────────────────────────────────────────────────────

    #[test]
    fn hideout_is_classified_as_hideout() {
        assert_eq!(classify_zone("Karui Shores"), ZoneKind::HideoutOrTown);
        assert_eq!(classify_zone("The Rogue Harbour"), ZoneKind::HideoutOrTown);
    }

    #[test]
    fn town_names_classified_as_hideout_or_town() {
        assert_eq!(classify_zone("Lioneye's Watch"), ZoneKind::HideoutOrTown);
        assert_eq!(classify_zone("The Forest Encampment"), ZoneKind::HideoutOrTown);
    }

    #[test]
    fn map_zones_classified_as_map() {
        assert_eq!(classify_zone("Lookout"), ZoneKind::Map);
        assert_eq!(classify_zone("Jungle Valley"), ZoneKind::Map);
        assert_eq!(classify_zone("Burial Chambers"), ZoneKind::Map);
    }

    // ── parse_poe_timestamp ───────────────────────────────────────────────────

    #[test]
    fn parse_timestamp_returns_some_for_valid_input() {
        let ts = parse_poe_timestamp("2024/01/15 14:23:45");
        assert!(ts.is_some(), "valid timestamp must return Some");
    }

    #[test]
    fn parse_timestamp_returns_none_for_garbage() {
        assert!(parse_poe_timestamp("not a timestamp").is_none());
        assert!(parse_poe_timestamp("").is_none());
    }

    #[test]
    fn parse_timestamp_ordering_preserved() {
        // Later date must produce larger value
        let t1 = parse_poe_timestamp("2024/01/15 00:00:00").unwrap();
        let t2 = parse_poe_timestamp("2024/01/16 00:00:00").unwrap();
        assert!(t2 > t1, "later date should produce larger timestamp");
    }

    // ── parse_log_line ────────────────────────────────────────────────────────

    #[test]
    fn parse_valid_zone_entry_line() {
        let line = "2024/01/15 14:23:45 239384 ae [INFO Client 12345] : You have entered Lookout.";
        let ev = parse_log_line(line);
        assert!(ev.is_some(), "should parse valid entry line");
        let ev = ev.unwrap();
        assert_eq!(ev.zone_name, "Lookout");
    }

    #[test]
    fn parse_log_line_returns_none_for_non_entry() {
        let line = "2024/01/15 14:23:45 239384 ae [INFO Client 12345] : Some other message.";
        assert!(parse_log_line(line).is_none());
    }

    #[test]
    fn parse_log_line_returns_none_for_empty() {
        assert!(parse_log_line("").is_none());
    }

    #[test]
    fn parse_log_line_classifies_map_zone() {
        let line = "2024/01/15 14:23:45 239384 ae [INFO Client 12345] : You have entered Lookout.";
        let ev = parse_log_line(line).unwrap();
        assert_eq!(ev.kind, ZoneKind::Map);
    }

    #[test]
    fn parse_log_line_classifies_town() {
        let line = "2024/01/15 14:23:45 239384 ae [INFO Client 12345] : You have entered Karui Shores.";
        let ev = parse_log_line(line).unwrap();
        assert_eq!(ev.kind, ZoneKind::HideoutOrTown);
    }

    // ── build_map_runs ────────────────────────────────────────────────────────

    fn zone_event(secs: u64, name: &str, kind: ZoneKind) -> ZoneEvent {
        ZoneEvent { timestamp_secs: secs, zone_name: name.to_string(), kind }
    }

    #[test]
    fn single_map_then_town_produces_one_run() {
        let events = vec![
            zone_event(0,    "Lookout",      ZoneKind::Map),
            zone_event(300,  "Karui Shores", ZoneKind::HideoutOrTown),
        ];
        let runs = build_map_runs(&events);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].zone_name, "Lookout");
        assert_eq!(runs[0].duration_secs, 300);
    }

    #[test]
    fn two_maps_produce_two_runs() {
        let events = vec![
            zone_event(0,    "Lookout",        ZoneKind::Map),
            zone_event(300,  "Jungle Valley",  ZoneKind::Map),
            zone_event(600,  "Karui Shores",   ZoneKind::HideoutOrTown),
        ];
        let runs = build_map_runs(&events);
        assert_eq!(runs.len(), 2);
    }

    #[test]
    fn town_only_produces_no_runs() {
        let events = vec![
            zone_event(0, "Karui Shores", ZoneKind::HideoutOrTown),
            zone_event(100, "The Rogue Harbour", ZoneKind::HideoutOrTown),
        ];
        assert_eq!(build_map_runs(&events).len(), 0);
    }

    #[test]
    fn empty_events_produces_empty_runs() {
        assert!(build_map_runs(&[]).is_empty());
    }

    // ── accumulate_stats ──────────────────────────────────────────────────────

    fn run(name: &str, dur: u64, loot: f64) -> MapRun {
        MapRun { zone_name: name.to_string(), started_at: 0, ended_at: dur, duration_secs: dur, loot_chaos: loot }
    }

    #[test]
    fn stats_total_runs_correct() {
        let runs = vec![run("A", 300, 10.0), run("B", 200, 5.0)];
        assert_eq!(accumulate_stats(&runs).total_runs, 2);
    }

    #[test]
    fn stats_avg_duration_correct() {
        let runs = vec![run("A", 300, 0.0), run("B", 100, 0.0)];
        let stats = accumulate_stats(&runs);
        assert!((stats.avg_duration - 200.0).abs() < 0.001);
    }

    #[test]
    fn stats_chaos_per_hour_correct() {
        // 1 run, 3600s, 100c → 100c/h
        let runs = vec![run("A", 3600, 100.0)];
        let stats = accumulate_stats(&runs);
        assert!((stats.chaos_per_hour - 100.0).abs() < 0.001);
    }

    #[test]
    fn stats_most_run_map_is_most_frequent() {
        let runs = vec![run("A", 300, 0.0), run("B", 300, 0.0), run("A", 300, 0.0)];
        assert_eq!(accumulate_stats(&runs).most_run_map, "A");
    }

    #[test]
    fn stats_empty_runs_returns_defaults() {
        let stats = accumulate_stats(&[]);
        assert_eq!(stats.total_runs, 0);
        assert_eq!(stats.chaos_per_hour, 0.0);
    }
}

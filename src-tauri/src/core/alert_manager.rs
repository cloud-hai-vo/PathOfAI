/// alert_manager.rs — Price alert manager (Algorithm 50).
/// Tests written FIRST (TDD RED). Run `cargo test alert_manager` → all FAIL → then implement.
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertCondition {
    Below(f64),      // trigger when price drops below threshold
    Above(f64),      // trigger when price rises above threshold
    ChangePercent { pct: f64, baseline: f64 }, // trigger when |change| > pct%
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAlert {
    pub id:         String,
    pub item_name:  String,
    pub condition:  AlertCondition,
    pub active:     bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertFired {
    pub alert_id:    String,
    pub item_name:   String,
    pub current_price: f64,
    pub message:     String,
}

// ─── Stubs → unimplemented!() → RED ──────────────────────────────────────────

/// Check whether an alert fires given the current price.
/// Returns Some(AlertFired) if triggered, None otherwise.
pub fn check_alert(alert: &PriceAlert, current_price: f64) -> Option<AlertFired> {
    if !alert.active { return None; }

    let triggered = match &alert.condition {
        AlertCondition::Below(thresh)  => current_price < *thresh,
        AlertCondition::Above(thresh)  => current_price > *thresh,
        AlertCondition::ChangePercent { pct, baseline } => {
            if *baseline == 0.0 { false }
            else { ((current_price - baseline) / baseline * 100.0).abs() > *pct }
        }
    };

    if !triggered { return None; }

    let message = match &alert.condition {
        AlertCondition::Below(t)  => format!("{} is now {:.1}c (below {:.1}c)", alert.item_name, current_price, t),
        AlertCondition::Above(t)  => format!("{} is now {:.1}c (above {:.1}c)", alert.item_name, current_price, t),
        AlertCondition::ChangePercent { pct, baseline } => {
            let change = (current_price - baseline) / baseline * 100.0;
            format!("{} changed {:.1}% (was {:.1}c, now {:.1}c, threshold ±{:.1}%)",
                alert.item_name, change, baseline, current_price, pct)
        }
    };

    Some(AlertFired { alert_id: alert.id.clone(), item_name: alert.item_name.clone(), current_price, message })
}

pub fn check_alerts(alerts: &[PriceAlert], prices: &std::collections::HashMap<String, f64>) -> Vec<AlertFired> {
    alerts.iter()
        .filter_map(|a| {
            let price = prices.get(&a.item_name)?;
            check_alert(a, *price)
        })
        .collect()
}

pub fn deactivate_alert(alerts: &mut Vec<PriceAlert>, id: &str) -> bool {
    if let Some(a) = alerts.iter_mut().find(|a| a.id == id) {
        a.active = false;
        true
    } else {
        false
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn alert(id: &str, name: &str, cond: AlertCondition) -> PriceAlert {
        PriceAlert { id: id.to_string(), item_name: name.to_string(), condition: cond, active: true }
    }

    // ── check_alert — Below ───────────────────────────────────────────────────

    #[test]
    fn below_fires_when_price_drops_below_threshold() {
        let a = alert("1", "Chaos Orb", AlertCondition::Below(5.0));
        let result = check_alert(&a, 4.0);
        assert!(result.is_some(), "should fire when price (4) < threshold (5)");
    }

    #[test]
    fn below_does_not_fire_at_threshold() {
        let a = alert("1", "Chaos Orb", AlertCondition::Below(5.0));
        assert!(check_alert(&a, 5.0).is_none());
    }

    #[test]
    fn below_does_not_fire_above_threshold() {
        let a = alert("1", "Chaos Orb", AlertCondition::Below(5.0));
        assert!(check_alert(&a, 6.0).is_none());
    }

    // ── check_alert — Above ───────────────────────────────────────────────────

    #[test]
    fn above_fires_when_price_rises_above_threshold() {
        let a = alert("2", "Divine Orb", AlertCondition::Above(200.0));
        let result = check_alert(&a, 201.0);
        assert!(result.is_some(), "should fire when price (201) > threshold (200)");
    }

    #[test]
    fn above_does_not_fire_at_threshold() {
        let a = alert("2", "Divine Orb", AlertCondition::Above(200.0));
        assert!(check_alert(&a, 200.0).is_none());
    }

    #[test]
    fn above_does_not_fire_below_threshold() {
        let a = alert("2", "Divine Orb", AlertCondition::Above(200.0));
        assert!(check_alert(&a, 199.0).is_none());
    }

    // ── check_alert — ChangePercent ───────────────────────────────────────────

    #[test]
    fn change_percent_fires_on_large_drop() {
        let a = alert("3", "Item", AlertCondition::ChangePercent { pct: 10.0, baseline: 100.0 });
        // 100 → 85 = 15% drop > 10%
        let result = check_alert(&a, 85.0);
        assert!(result.is_some());
    }

    #[test]
    fn change_percent_fires_on_large_rise() {
        let a = alert("3", "Item", AlertCondition::ChangePercent { pct: 10.0, baseline: 100.0 });
        // 100 → 115 = 15% rise > 10%
        let result = check_alert(&a, 115.0);
        assert!(result.is_some());
    }

    #[test]
    fn change_percent_does_not_fire_on_small_change() {
        let a = alert("3", "Item", AlertCondition::ChangePercent { pct: 10.0, baseline: 100.0 });
        // 100 → 105 = 5% change < 10%
        assert!(check_alert(&a, 105.0).is_none());
    }

    // ── check_alert — inactive ────────────────────────────────────────────────

    #[test]
    fn inactive_alert_never_fires() {
        let mut a = alert("1", "Test", AlertCondition::Below(100.0));
        a.active = false;
        assert!(check_alert(&a, 1.0).is_none(), "inactive alert must never fire");
    }

    // ── check_alert — fired message ───────────────────────────────────────────

    #[test]
    fn fired_alert_contains_item_name_and_price() {
        let a = alert("1", "Mirror of Kalandra", AlertCondition::Below(50000.0));
        let fired = check_alert(&a, 40000.0).unwrap();
        assert!(fired.message.contains("Mirror of Kalandra"),
            "message should mention item name");
        assert_eq!(fired.current_price, 40000.0);
    }

    // ── check_alerts batch ────────────────────────────────────────────────────

    #[test]
    fn batch_check_returns_only_fired() {
        let alerts = vec![
            alert("a", "Chaos Orb",  AlertCondition::Below(2.0)),   // fires at 1
            alert("b", "Divine Orb", AlertCondition::Below(100.0)),  // no fire at 150
        ];
        let mut prices = HashMap::new();
        prices.insert("Chaos Orb".to_string(),  1.0);
        prices.insert("Divine Orb".to_string(), 150.0);

        let fired = check_alerts(&alerts, &prices);
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].alert_id, "a");
    }

    #[test]
    fn batch_check_empty_prices_returns_empty() {
        let alerts = vec![alert("a", "X", AlertCondition::Below(10.0))];
        let fired = check_alerts(&alerts, &HashMap::new());
        assert!(fired.is_empty(), "no prices → nothing fires");
    }

    // ── deactivate_alert ──────────────────────────────────────────────────────

    #[test]
    fn deactivate_sets_active_false() {
        let mut alerts = vec![alert("x", "Test", AlertCondition::Above(100.0))];
        let ok = deactivate_alert(&mut alerts, "x");
        assert!(ok);
        assert!(!alerts[0].active);
    }

    #[test]
    fn deactivate_returns_false_for_missing_id() {
        let mut alerts = vec![alert("x", "Test", AlertCondition::Above(100.0))];
        assert!(!deactivate_alert(&mut alerts, "does-not-exist"));
    }
}

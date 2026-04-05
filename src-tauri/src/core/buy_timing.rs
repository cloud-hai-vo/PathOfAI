/// Buy Timing Advisor — Algorithm 52.
/// Recommends when to buy an item based on 7-day price trend and league phase.
use serde::{Deserialize, Serialize};

// ── Types ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuyAction {
    Wait,           // price falling — don't buy yet
    BuySoon,        // nearing floor — enter in 2-3 days
    BuyNow,         // price rising or late-league supply squeeze
    BuyNowOrWait,   // sharp spike — could correct or continue
    BuyWhenReady,   // stable — timing doesn't matter
    Monitor,        // insufficient data
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Urgency { None, Low, Medium, High }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence { Low, Medium, High }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    DroppingFast,
    DroppingSlow,
    Stable,
    RisingSlow,
    RisingFast,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaguePhase {
    LaunchFrenzy,   // day 1-3
    CrashPeriod,    // day 4-14
    Stabilization,  // day 15-30
    PeakEconomy,    // day 31-60
    LateLeague,     // day 61+
}

/// A single historical price data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub price_divine: f64,
}

/// Full buy recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyRecommendation {
    pub action:       BuyAction,
    pub reason:       String,
    pub urgency:      Urgency,
    pub confidence:   Confidence,
    pub current_div:  f64,
    pub trend:        TrendDirection,
    pub change_7d:    f64,           // percent change over last 7 data points
    pub league_phase: LeaguePhase,
    pub sparkline:    Vec<f64>,      // last 14 price points, oldest-first
}

// ── Algorithm ─────────────────────────────────────────────────────────────────

pub fn generate_buy_recommendation(
    item_key: &str,
    history:  &[PricePoint],
    phase:    LeaguePhase,
) -> BuyRecommendation {
    // Need at least 2 points for a trend
    let recent: Vec<&PricePoint> = history.iter().rev().take(7).collect();
    let (oldest_price, newest_price) = match (recent.last(), recent.first()) {
        (Some(o), Some(n)) => (o.price_divine, n.price_divine),
        _ => return unknown(item_key, phase),
    };

    let change_7d = if oldest_price > 0.0 {
        (newest_price - oldest_price) / oldest_price * 100.0
    } else {
        0.0
    };

    let trend = classify_trend(change_7d);

    let confidence = match recent.len() {
        n if n >= 5 => Confidence::High,
        n if n >= 3 => Confidence::Medium,
        _           => Confidence::Low,
    };

    let (action, urgency) = decide(phase.clone(), &trend);

    let reason = format_reason(&action, item_key, newest_price, change_7d, &phase);
    let sparkline: Vec<f64> = history.iter().rev().take(14).rev()
        .map(|p| p.price_divine)
        .collect();

    BuyRecommendation {
        action, reason, urgency, confidence,
        current_div: newest_price,
        trend, change_7d, league_phase: phase, sparkline,
    }
}

fn classify_trend(change_7d: f64) -> TrendDirection {
    match change_7d {
        c if c < -20.0 => TrendDirection::DroppingFast,
        c if c <  -5.0 => TrendDirection::DroppingSlow,
        c if c >  20.0 => TrendDirection::RisingFast,
        c if c >   5.0 => TrendDirection::RisingSlow,
        _              => TrendDirection::Stable,
    }
}

fn decide(phase: LeaguePhase, trend: &TrendDirection) -> (BuyAction, Urgency) {
    use LeaguePhase::*;
    use TrendDirection::*;
    use BuyAction::*;
    use Urgency::*;

    match (&phase, trend) {
        // Early league — prices crash regardless of trend (except fast rise)
        (LaunchFrenzy | CrashPeriod, t) if *t != RisingFast => (Wait, Urgency::None),

        // Dropping fast — always wait
        (_, DroppingFast) => (Wait, Urgency::None),

        // Slow drop in mature phase — nearing floor
        (Stabilization | PeakEconomy | LateLeague, DroppingSlow) => (BuySoon, Low),

        // Rising in late league — supply squeeze
        (PeakEconomy | LateLeague, RisingSlow) => (BuyNow, High),

        // Sharp spike — uncertain
        (_, RisingFast) => (BuyNowOrWait, Medium),

        // Stable in mature phase — timing irrelevant
        (Stabilization | PeakEconomy | LateLeague, Stable) => (BuyWhenReady, Urgency::None),

        _ => (Monitor, Urgency::None),
    }
}

fn format_reason(
    action:    &BuyAction,
    item_key:  &str,
    price:     f64,
    change_7d: f64,
    phase:     &LeaguePhase,
) -> String {
    match action {
        BuyAction::Wait         => format!("{item_key} is dropping ({change_7d:.1}% in 7d) — wait for the floor"),
        BuyAction::BuySoon      => format!("{item_key} is nearing floor at {price:.1} div — consider buying in 2-3 days"),
        BuyAction::BuyNow       => format!("{item_key} is rising in {:?} — prices may increase further", phase),
        BuyAction::BuyNowOrWait => format!("{item_key} spiked +{change_7d:.1}% — could correct; buy only if you need it now"),
        BuyAction::BuyWhenReady => format!("{item_key} is stable at {price:.1} div — buy whenever your budget allows"),
        BuyAction::Monitor      => format!("Insufficient price history for {item_key}"),
    }
}

fn unknown(item_key: &str, phase: LeaguePhase) -> BuyRecommendation {
    BuyRecommendation {
        action:       BuyAction::Monitor,
        reason:       format!("Insufficient price history for {item_key}"),
        urgency:      Urgency::None,
        confidence:   Confidence::Low,
        current_div:  0.0,
        trend:        TrendDirection::Stable,
        change_7d:    0.0,
        league_phase: phase,
        sparkline:    vec![],
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_history(prices: &[f64]) -> Vec<PricePoint> {
        prices.iter().map(|&p| PricePoint { price_divine: p }).collect()
    }

    #[test]
    fn empty_history_returns_monitor() {
        let rec = generate_buy_recommendation("Watcher's Eye", &[], LeaguePhase::PeakEconomy);
        assert_eq!(rec.action, BuyAction::Monitor);
        assert_eq!(rec.confidence, Confidence::Low);
    }

    #[test]
    fn dropping_fast_always_wait() {
        // 7-day drop of >20%
        let history = make_history(&[100.0, 95.0, 90.0, 80.0, 75.0, 70.0, 60.0]);
        let rec = generate_buy_recommendation("Mageblood", &history, LeaguePhase::PeakEconomy);
        assert_eq!(rec.action, BuyAction::Wait);
    }

    #[test]
    fn stable_in_peak_economy_gives_buy_when_ready() {
        // Prices stable ±3%
        let history = make_history(&[100.0, 101.0, 99.0, 100.0, 101.0, 100.0, 100.0]);
        let rec = generate_buy_recommendation("Aegis Aurora", &history, LeaguePhase::PeakEconomy);
        assert_eq!(rec.action, BuyAction::BuyWhenReady);
    }

    #[test]
    fn slow_drop_in_stabilization_gives_buy_soon() {
        // 8% drop over 7 days
        let history = make_history(&[50.0, 49.0, 48.0, 47.0, 46.0, 46.0, 46.0]);
        let rec = generate_buy_recommendation("Bottled Faith", &history, LeaguePhase::Stabilization);
        assert_eq!(rec.action, BuyAction::BuySoon);
    }

    #[test]
    fn rising_in_late_league_gives_buy_now() {
        // 10% rise over 7 days
        let history = make_history(&[40.0, 41.0, 42.0, 43.0, 44.0, 44.0, 44.0]);
        let rec = generate_buy_recommendation("Melding", &history, LeaguePhase::LateLeague);
        assert_eq!(rec.action, BuyAction::BuyNow);
    }

    #[test]
    fn sharp_spike_gives_buy_now_or_wait() {
        // 30% spike
        let history = make_history(&[20.0, 20.0, 20.0, 20.0, 20.0, 20.0, 26.0]);
        let rec = generate_buy_recommendation("Forbidden Jewel", &history, LeaguePhase::PeakEconomy);
        assert_eq!(rec.action, BuyAction::BuyNowOrWait);
    }

    #[test]
    fn early_league_non_spike_gives_wait() {
        let history = make_history(&[200.0, 190.0, 180.0, 170.0, 160.0, 150.0, 140.0]);
        let rec = generate_buy_recommendation("Mageblood", &history, LeaguePhase::LaunchFrenzy);
        assert_eq!(rec.action, BuyAction::Wait);
    }

    #[test]
    fn confidence_high_with_five_or_more_points() {
        let history = make_history(&[100.0, 100.0, 100.0, 100.0, 100.0]);
        let rec = generate_buy_recommendation("Item", &history, LeaguePhase::PeakEconomy);
        assert_eq!(rec.confidence, Confidence::High);
    }

    #[test]
    fn confidence_medium_with_three_points() {
        let history = make_history(&[100.0, 100.0, 100.0]);
        let rec = generate_buy_recommendation("Item", &history, LeaguePhase::PeakEconomy);
        assert_eq!(rec.confidence, Confidence::Medium);
    }

    #[test]
    fn confidence_low_with_two_points() {
        let history = make_history(&[100.0, 100.0]);
        let rec = generate_buy_recommendation("Item", &history, LeaguePhase::PeakEconomy);
        assert_eq!(rec.confidence, Confidence::Low);
    }

    #[test]
    fn sparkline_contains_up_to_14_points_oldest_first() {
        let prices: Vec<f64> = (1..=20).map(|x| x as f64).collect();
        let history = make_history(&prices);
        let rec = generate_buy_recommendation("Item", &history, LeaguePhase::PeakEconomy);
        assert_eq!(rec.sparkline.len(), 14);
        // Oldest should be less than newest (prices went from 1 to 20)
        assert!(rec.sparkline[0] < rec.sparkline[13]);
    }

    #[test]
    fn change_7d_calculated_correctly() {
        // oldest=100, newest=110 → +10%
        let history = make_history(&[100.0, 101.0, 102.0, 103.0, 105.0, 108.0, 110.0]);
        let rec = generate_buy_recommendation("Item", &history, LeaguePhase::PeakEconomy);
        assert!((rec.change_7d - 10.0).abs() < 0.1,
            "expected ~10% change, got {}", rec.change_7d);
    }
}

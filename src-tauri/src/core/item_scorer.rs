use crate::models::build::{BuildData, Item};
use crate::core::build_detector::Archetype;

pub struct ItemComparison {
    pub dps_delta: f64,
    pub life_delta: i32,
    pub res_delta: i32,
    pub verdict: String,
}

pub fn score_item(item: &Item, archetype: Archetype) -> u8 {
    crate::core::build_analyzer::score_items(
        &BuildData { items: vec![item.clone()], ..Default::default() },
        archetype,
    )
    .first()
    .map(|s| s.score)
    .unwrap_or(0)
}

pub fn compare_to_equipped(new_item: &Item, build: &BuildData, archetype: Archetype) -> ItemComparison {
    let current = build.items.iter().find(|i| i.slot == new_item.slot);
    let new_score = score_item(new_item, archetype) as i32;
    let old_score = current.map(|i| score_item(i, archetype) as i32).unwrap_or(0);

    let verdict = match new_score - old_score {
        d if d > 10 => "Upgrade".to_string(),
        d if d >= -5 => "Sidegrade".to_string(),
        _ => "Downgrade".to_string(),
    };

    ItemComparison { dps_delta: 0.0, life_delta: 0, res_delta: 0, verdict }
}

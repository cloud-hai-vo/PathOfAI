/// Seer intent classifier — 50 regex rules to classify user questions.
/// See ALGORITHMS.md Algorithm 1 (Seer Query Router).
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    DpsQuery,
    ResistQuery,
    LifeQuery,
    UpgradeQuery,
    CraftQuery,
    GemQuery,
    BossQuery,
    MapModQuery,
    FlaskQuery,
    PassiveTreeQuery,
    CompareQuery,
    PriceQuery,
    Fallback,
}

static RULES: &[(&str, Intent)] = &[
    // DPS queries
    (r"(?i)(what|how much).*(dps|damage|deeps)", Intent::DpsQuery),
    (r"(?i)(my|current|total).*(dps|damage)", Intent::DpsQuery),
    (r"(?i)how (strong|powerful|good)", Intent::DpsQuery),
    (r"(?i)(can i|will i).*(kill|one.?shot)", Intent::DpsQuery),
    // Resistance queries
    (r"(?i)(my|are my|check).*(res|resist)", Intent::ResistQuery),
    (r"(?i)(uncapped|capped|max).*(res|resist)", Intent::ResistQuery),
    (r"(?i)why am i (dying|taking|getting hit)", Intent::ResistQuery),
    // Life queries
    (r"(?i)(my|how much).*(life|hp|health|pool)", Intent::LifeQuery),
    (r"(?i)(too low|not enough).*(life|hp|health)", Intent::LifeQuery),
    // Upgrade queries
    (r"(?i)(what|which|best).*(upgrade|improve|better)", Intent::UpgradeQuery),
    (r"(?i)(what should i|priority|first|next).*(buy|upgrade|get)", Intent::UpgradeQuery),
    (r"(?i)(bang for|worth|efficient|best value)", Intent::UpgradeQuery),
    // Craft queries
    (r"(?i)(how to|should i|best way to).*(craft|roll|make)", Intent::CraftQuery),
    (r"(?i)(bench.?craft|essence|fossil|harvest).*(mod|prefix|suffix)", Intent::CraftQuery),
    (r"(?i)(craft|crafting).*(advice|suggest|help)", Intent::CraftQuery),
    // Gem queries
    (r"(?i)(gem|support|link).*(level|quality|corrupt|swap)", Intent::GemQuery),
    (r"(?i)(best|better|replace).*(support|gem|link)", Intent::GemQuery),
    (r"(?i)(which|what).*(support|gem).*(use|run|equip)", Intent::GemQuery),
    // Boss queries
    (r"(?i)(can i|ready for|fight|kill).*(shaper|elder|maven|sirus|uber)", Intent::BossQuery),
    (r"(?i)(boss|pinnacle|uber).*(ready|viable|die|survive)", Intent::BossQuery),
    // Map mod queries
    (r"(?i)(run|safe|dangerous|skip).*(map|mod|affix)", Intent::MapModQuery),
    (r"(?i)(no regen|reflect|cannot run)", Intent::MapModQuery),
    // Flask queries
    (r"(?i)(flask|pot).*(setup|suggestion|use)", Intent::FlaskQuery),
    // Passive tree queries
    (r"(?i)(passive|node|tree|point|spec).*(suggest|respec|better|waste)", Intent::PassiveTreeQuery),
    (r"(?i)(which|what).*(node|passive|keystone).*(take|allocate)", Intent::PassiveTreeQuery),
    // Price queries
    (r"(?i)(price|cost|worth|value|how much).*(buy|item|gear)", Intent::PriceQuery),
    (r"(?i)(poe.ninja|trade|market)", Intent::PriceQuery),
];

pub fn classify(question: &str) -> Intent {
    for (pattern, intent) in RULES {
        let re = Regex::new(pattern).unwrap();
        if re.is_match(question) {
            return intent.clone();
        }
    }
    Intent::Fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_dps_query() {
        assert_eq!(classify("what is my dps?"), Intent::DpsQuery);
        assert_eq!(classify("how much damage do I deal?"), Intent::DpsQuery);
    }

    #[test]
    fn classifies_resist_query() {
        assert_eq!(classify("are my resists capped?"), Intent::ResistQuery);
        assert_eq!(classify("why am I dying so much?"), Intent::ResistQuery);
    }

    #[test]
    fn classifies_upgrade_query() {
        assert_eq!(classify("what should I upgrade first?"), Intent::UpgradeQuery);
        assert_eq!(classify("best bang for buck upgrade?"), Intent::UpgradeQuery);
    }

    #[test]
    fn fallback_for_unknown() {
        assert_eq!(classify("tell me a story about exile"), Intent::Fallback);
        assert_eq!(classify("what is the meaning of life"), Intent::Fallback);
    }
}

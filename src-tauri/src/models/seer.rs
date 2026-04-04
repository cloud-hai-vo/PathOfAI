use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeerResponse {
    pub answer: String,
    pub engine: SeerEngine,
    pub confidence: f64,            // 0.0-1.0
    pub follow_up_questions: Vec<String>,
    pub related_suggestions: Vec<String>, // suggestion IDs from analysis
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeerEngine {
    Calculator,     // answered from our Rust calculator (85% of queries)
    Knowledge,      // answered from embedded knowledge base (12%)
    Cloud,          // answered from Claude/GPT API (3%)
    Fallback,       // could not answer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeAnalysis {
    pub total_allocated: u32,
    pub by_category: Vec<NodeCategory>,
    pub top_recommendations: Vec<NodeRecommendation>,
    pub inefficient_nodes: Vec<InefficientNode>,
    pub next_keystone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCategory {
    pub name: String,   // "Life", "Fire Damage", "Resistance"
    pub count: u32,
    pub total_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRecommendation {
    pub node_id: u32,
    pub node_name: String,
    pub path_cost: u32,             // passives to spend to reach it
    pub value_score: f64,
    pub efficiency: f64,            // value_score / path_cost
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InefficientNode {
    pub node_id: u32,
    pub node_name: String,
    pub value_score: f64,
    pub reason: String,             // "Only +5 Str — no synergy with build"
}

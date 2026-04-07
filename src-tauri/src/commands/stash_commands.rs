/// stash_commands.rs — Stash tab IPC commands.
///
/// `fetch_stash_tabs` and `fetch_stash_items` call the PoE API via OAuth token.
/// `find_stash_upgrades` and `get_currency_totals` use local pure-logic helpers.
use tauri::State;
use crate::AppState;
use crate::core::stash::{StashItem, StashUpgradeSuggestion, WealthSummary, tally_currency, find_stash_upgrades};
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────────────────────

/// A single stash tab entry returned by the PoE API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashTab {
    pub id:       String,
    pub name:     String,
    pub tab_type: String,
    pub index:    u32,
    pub colour:   Option<StashTabColour>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashTabColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

// ─── Commands ─────────────────────────────────────────────────────────────────

/// Fetch the list of stash tabs for the authenticated account.
/// Requires a valid OAuth token stored in the app's auth store.
#[tauri::command]
pub async fn fetch_stash_tabs(
    state: State<'_, AppState>,
) -> Result<Vec<StashTab>, String> {
    // Retrieve stored token
    let token = state.db.load_oauth_token()
        .map_err(|_| "Not authenticated — connect your PoE account first".to_string())?;

    // GET https://api.pathofexile.com/character-window/get-stash-items
    // (tab list is returned from list-characters or dedicated endpoint)
    let url = "https://api.pathofexile.com/character-window/get-stash-items?tabs=1&tabIndex=0&accountName=";
    let resp = state.http
        .get(url)
        .bearer_auth(&token)
        .send().await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("PoE API error: {}", resp.status()));
    }

    // The API returns tabs in a "tabs" array; we extract just the metadata
    #[derive(Deserialize)]
    struct ApiResp { tabs: Vec<ApiTab> }
    #[derive(Deserialize)]
    struct ApiTab { id: String, n: String, #[serde(rename = "type")] tab_type: String, i: u32 }

    let body: ApiResp = resp.json().await.map_err(|e| e.to_string())?;
    Ok(body.tabs.into_iter().map(|t| StashTab {
        id:       t.id,
        name:     t.n,
        tab_type: t.tab_type,
        index:    t.i,
        colour:   None,
    }).collect())
}

/// Fetch items in a specific stash tab by its index.
#[tauri::command]
pub async fn fetch_stash_items(
    tab_id: String,
    state:  State<'_, AppState>,
) -> Result<Vec<StashItem>, String> {
    let token = state.db.load_oauth_token()
        .map_err(|_| "Not authenticated — connect your PoE account first".to_string())?;

    let url = format!(
        "https://api.pathofexile.com/character-window/get-stash-items?tabIndex={tab_id}"
    );
    let resp = state.http
        .get(&url)
        .bearer_auth(&token)
        .send().await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("PoE API error: {}", resp.status()));
    }

    #[derive(Deserialize)]
    struct ApiResp { items: Vec<ApiItem> }
    #[derive(Deserialize)]
    struct ApiItem {
        id: String, #[serde(rename = "typeLine")] type_line: String,
        #[serde(rename = "name")] name: String,
        #[serde(rename = "stackSize", default)] stack_size: u32,
    }

    let body: ApiResp = resp.json().await.map_err(|e| e.to_string())?;
    Ok(body.items.into_iter().map(|it| StashItem {
        id:          it.id,
        name:        it.name,
        type_line:   it.type_line,
        chaos_value: 0.0,  // caller enriches with prices
        stack_size:  it.stack_size.max(1),
        tab_name:    tab_id.clone(),
    }).collect())
}

/// Find upgrade candidates already in the stash for a given build.
/// Items are scored by chaos value vs potential DPS gain.
#[tauri::command]
pub async fn find_stash_upgrades_cmd(
    items_json: String,
    min_gain:   Option<f64>,
    _state: State<'_, AppState>,
) -> Result<Vec<StashUpgradeSuggestion>, String> {
    let items: Vec<StashItem> = serde_json::from_str(&items_json)
        .map_err(|e| e.to_string())?;
    let threshold = min_gain.unwrap_or(1.0);
    // Simple heuristic score: items worth >200c with positive chaos_value get a score
    let score_fn  = |item: &StashItem| item.chaos_value;
    let upgrade_fn = |item: &StashItem| {
        if item.chaos_value > 200.0 { Some(item.chaos_value * 1.1) } else { None }
    };
    Ok(find_stash_upgrades(&items, &score_fn, &upgrade_fn, threshold))
}

/// Tally total stash wealth across all items.
#[tauri::command]
pub async fn get_currency_totals(
    items_json:     String,
    divine_price_c: f64,
    _state: State<'_, AppState>,
) -> Result<WealthSummary, String> {
    let items: Vec<StashItem> = serde_json::from_str(&items_json)
        .map_err(|e| e.to_string())?;
    Ok(tally_currency(&items, divine_price_c))
}

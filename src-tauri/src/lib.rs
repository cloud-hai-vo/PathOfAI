use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

mod commands;
mod models;
pub mod core;
pub mod calculator;
pub mod data;
pub mod market;
pub mod seer;
pub mod services;
pub mod db;

/// Shared application state — injected into every Tauri command via `State<'_, AppState>`.
pub struct AppState {
    pub db:          db::Database,
    pub game_data:   Arc<data::GameData>,
    pub ai_keys:     Mutex<core::cloud_ai::ApiKeyStore>,
    pub http:        reqwest::Client,
    /// Keep the notify watcher alive for the lifetime of the app.
    #[allow(dead_code)]
    pub _file_watcher: Option<notify::RecommendedWatcher>,
    /// Price poller state — shared with the background task.
    pub price_poller: Arc<Mutex<services::PricePoller>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            // Initialize portable storage (PathOfAI_Data/ next to exe)
            let data_dir = core::storage::init_storage(app.handle())?;
            log::info!("Storage initialized at: {}", data_dir.display());

            // Open SQLite database
            let db_path = data_dir.join("path-of-ai.db");
            let database = db::Database::open(&db_path)?;
            database.run_migrations()?;

            // Load game data (JSON files in PathOfAI_Data/game/)
            let game_data = Arc::new(data::load_game_data(&data_dir));
            if game_data.is_empty() {
                log::warn!("Game data files not found in {:?}/game/ — craft probability and mod scoring will use defaults", data_dir);
            } else {
                log::info!("Game data loaded: {} mods, {} gems, {} base items",
                    game_data.mods.len(), game_data.gems.len(), game_data.base_items.len());
            }

            // Load AI provider API keys from config dir
            let config_dir = data_dir.join("config");
            let ai_keys = Mutex::new(core::cloud_ai::ApiKeyStore::new(&config_dir));

            // Shared HTTP client (connection-pooled, reused across all requests)
            let http = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .user_agent("PathOfAI/0.1.0")
                .build()
                .unwrap_or_default();

            // Build price poller (5-min TTL)
            let price_poller = Arc::new(Mutex::new(services::PricePoller::new(300)));

            // Start file watcher for PoB XML changes (Algorithm 44b)
            let file_watcher = start_file_watcher(app.handle().clone(), &data_dir);

            // Start price poller background task (Algorithm 21)
            start_price_poller(app.handle().clone(), Arc::clone(&price_poller));

            app.manage(AppState {
                db: database,
                game_data,
                ai_keys,
                http,
                _file_watcher: file_watcher,
                price_poller,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Info
            commands::info::get_version,
            commands::info::get_app_info,
            // Build
            commands::build_commands::analyze_build,
            commands::build_commands::refresh_analysis,
            commands::build_commands::undo_last_change,
            commands::build_commands::redo_change,
            // Character (OAuth)
            commands::character_commands::load_character,
            commands::character_commands::list_characters,
            commands::character_commands::switch_character,
            commands::character_commands::start_oauth,
            commands::character_commands::get_auth_status,
            commands::character_commands::list_builds,
            // Seer
            commands::seer_commands::ask_seer,
            commands::seer_commands::get_tree_analysis,
            commands::seer_commands::get_craft_suggestions,
            commands::seer_commands::compare_craft_vs_buy,
            // Market
            commands::market_commands::get_prices,
            commands::market_commands::search_upgrades,
            commands::market_commands::get_buy_recommendation,
            // Items
            commands::item_commands::parse_clipboard_item,
            commands::item_commands::score_item,
            commands::item_commands::apply_upgrade,
            commands::item_commands::estimate_item_swap,
            commands::item_commands::resolve_item_image,
            // Simulation & Analysis
            commands::simulation_commands::run_simulation,
            commands::simulation_commands::compare_builds_cmd,
            commands::simulation_commands::tally_stash_wealth,
            commands::simulation_commands::get_map_stats,
            commands::simulation_commands::check_price_alerts,
            commands::simulation_commands::deactivate_price_alert,
            commands::simulation_commands::analyze_map_mods,
            commands::simulation_commands::set_price_alert,
            commands::simulation_commands::list_price_alerts,
            commands::simulation_commands::remove_price_alert,
            commands::simulation_commands::calculate_mana_reservation,
            commands::simulation_commands::generate_share_code,
            commands::simulation_commands::import_share_code,
            commands::simulation_commands::check_stat_requirements,
            commands::simulation_commands::detect_vendor_recipes,
            commands::simulation_commands::calc_ignite_cmd,
            commands::simulation_commands::calc_chill_cmd,
            commands::simulation_commands::calc_freeze_cmd,
            commands::simulation_commands::calc_shock_cmd,
            commands::simulation_commands::calc_poison_cmd,
            commands::simulation_commands::calc_bleed_cmd,
            commands::simulation_commands::calc_charge_bonuses_cmd,
            commands::simulation_commands::apply_charge_gain_cmd,
            commands::simulation_commands::tick_es_recharge_cmd,
            commands::simulation_commands::simulate_boss,
            commands::simulation_commands::simulate_map_clear,
            // Stash
            commands::stash_commands::fetch_stash_tabs,
            commands::stash_commands::fetch_stash_items,
            commands::stash_commands::find_stash_upgrades_cmd,
            commands::stash_commands::get_currency_totals,
            // Settings / AI providers
            commands::settings_commands::test_cloud_ai,
            commands::settings_commands::save_ai_key,
            commands::settings_commands::remove_ai_key,
            commands::settings_commands::get_configured_providers,
            commands::settings_commands::get_pob_watch_dir,
            commands::settings_commands::save_settings,
            commands::settings_commands::load_settings,
            commands::settings_commands::watch_pob_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Path of AI");
}

// ── File Watcher ──────────────────────────────────────────────────────────────

fn start_file_watcher(
    handle: tauri::AppHandle,
    data_dir: &std::path::Path,
) -> Option<notify::RecommendedWatcher> {
    use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
    use std::time::Duration;

    // Watch PathOfAI_Data/builds/ by default.
    // Users can point to their PoB folder by setting POE_POB_DIR env var.
    let watch_path = std::env::var("POE_POB_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| data_dir.join("builds"));

    // Ensure the watch directory exists
    if let Err(e) = std::fs::create_dir_all(&watch_path) {
        log::warn!("Could not create watch dir {:?}: {e}", watch_path);
        return None;
    }

    let mut pob_watcher = services::PobFileWatcher::new(500);

    let result = RecommendedWatcher::new(
        move |event: notify::Result<notify::Event>| {
            let event = match event {
                Ok(e) => e,
                Err(e) => { log::warn!("FS watch error: {e}"); return; }
            };

            // Only process Modify/Create events on PoB files
            let is_write = matches!(event.kind,
                EventKind::Modify(_) | EventKind::Create(_));
            if !is_write { return; }

            for path in &event.paths {
                if services::PobFileWatcher::is_pob_file(path)
                    && pob_watcher.should_process(path)
                {
                    log::info!("PoB file changed: {:?}", path);
                    let path_str = path.to_string_lossy().to_string();
                    if let Err(e) = handle.emit("pob-file-changed", path_str) {
                        log::warn!("Failed to emit pob-file-changed: {e}");
                    }
                }
            }
        },
        notify::Config::default().with_poll_interval(Duration::from_millis(500)),
    );

    match result {
        Ok(mut watcher) => {
            if let Err(e) = watcher.watch(&watch_path, RecursiveMode::NonRecursive) {
                log::warn!("Failed to start file watcher on {:?}: {e}", watch_path);
                None
            } else {
                log::info!("PoB file watcher started on {:?}", watch_path);
                Some(watcher)
            }
        }
        Err(e) => {
            log::warn!("Could not create file watcher: {e}");
            None
        }
    }
}

// ── Price Poller ──────────────────────────────────────────────────────────────

fn start_price_poller(
    handle: tauri::AppHandle,
    poller: Arc<Mutex<services::PricePoller>>,
) {
    tokio::spawn(async move {
        // Initial delay — let the app finish loading before hitting poe.ninja
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        loop {
            // Check circuit breaker — extract bool before any await
            let circuit_open = poller.lock().unwrap().circuit_is_open();
            if circuit_open {
                log::warn!("Price poller: circuit open — skipping poll");
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                continue;
            }

            // Refresh prices for commonly-tracked unique items
            let items_to_check = [
                "Watcher's Eye", "Bottled Faith", "Mageblood", "Melding of the Flesh",
                "Aegis Aurora", "Ashes of the Stars", "Forbidden Jewel",
            ];

            match market::price_cache::get_prices(
                &items_to_check.iter().map(|s| s.to_string()).collect::<Vec<_>>()
            ).await {
                Ok(prices) => {
                    {
                        let mut guard = poller.lock().unwrap();
                        for p in &prices {
                            guard.insert(&p.item_name, p.price_div);
                        }
                    } // guard dropped here, before the await-free emit
                    if let Err(e) = handle.emit("price-updated", &prices) {
                        log::warn!("Failed to emit price-updated: {e}");
                    }
                    log::info!("Price poll complete: {} items refreshed", prices.len());
                }
                Err(e) => {
                    log::warn!("Price poll failed: {e}");
                    poller.lock().unwrap().record_failure();
                    // guard dropped immediately — no await after this
                }
            }

            // Poll every 5 minutes
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
        }
    });
}

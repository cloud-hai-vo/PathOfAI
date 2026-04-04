use tauri::Manager;

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
    pub db: db::Database,
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

            app.manage(AppState { db: database });

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
            // Items
            commands::item_commands::parse_clipboard_item,
            commands::item_commands::score_item,
            commands::item_commands::apply_upgrade,
            // Simulation & Analysis
            commands::simulation_commands::run_simulation,
            commands::simulation_commands::compare_builds_cmd,
            commands::simulation_commands::tally_stash_wealth,
            commands::simulation_commands::get_map_stats,
            commands::simulation_commands::check_price_alerts,
            commands::simulation_commands::deactivate_price_alert,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Path of AI");
}

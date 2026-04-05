use std::path::PathBuf;
use anyhow::Result;

/// Initialize portable storage directory next to the executable.
/// See ALGORITHMS.md — Algorithm 44a.
///
/// Priority order:
///   1. PORTABLE_PATH env var (for development / testing)
///   2. PathOfAI_Data/ directory next to the executable
pub fn init_storage(_app: &tauri::AppHandle) -> Result<PathBuf> {
    let data_dir = if let Ok(env_path) = std::env::var("PORTABLE_PATH") {
        PathBuf::from(env_path)
    } else {
        // Next to executable
        let exe_dir = std::env::current_exe()?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine exe directory"))?
            .to_path_buf();
        exe_dir.join("PathOfAI_Data")
    };

    // Create subdirectories
    let dirs = [
        data_dir.as_path(),
        &data_dir.join("cache"),
        &data_dir.join("images"),
        &data_dir.join("builds"),
        &data_dir.join("logs"),
    ];

    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    Ok(data_dir)
}

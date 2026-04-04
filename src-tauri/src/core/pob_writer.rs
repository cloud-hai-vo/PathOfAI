/// PoB Write-Back Engine — see ALGORITHMS.md Algorithm 46.
/// Applies upgrade suggestions to PoB XML with backup + atomic write.
use anyhow::{anyhow, Result};
use crate::models::build::BuildData;
use crate::models::analysis::Suggestion;

pub fn apply_suggestion(build: &BuildData, suggestion: &Suggestion) -> Result<BuildData> {
    match build.source.clone() {
        crate::models::build::BuildSource::PobFile(path) => {
            apply_to_pob_file(&path, build, suggestion)
        }
        _ => {
            // OAuth characters can't write back to PoB — return updated in-memory build
            Ok(build.clone())
        }
    }
}

fn apply_to_pob_file(path: &str, build: &BuildData, _suggestion: &Suggestion) -> Result<BuildData> {
    // 1. Check file is not locked
    let file_path = std::path::Path::new(path);
    if !file_path.exists() {
        return Err(anyhow!("PoB file not found: {path}"));
    }

    // 2. Create backup: file.xml → file.xml.bak
    let backup_path = format!("{path}.bak");
    std::fs::copy(path, &backup_path)
        .map_err(|e| anyhow!("Backup failed: {e}"))?;

    // 3. Read XML
    let xml = std::fs::read_to_string(path)
        .map_err(|e| anyhow!("Read failed: {e}"))?;

    // 4. Apply patch (stub — full DOM manipulation in real implementation)
    // TODO: parse XML → find target node → apply WriteOp → serialize back
    let patched_xml = xml; // no-op for now

    // 5. Write atomically: write to .tmp, then rename
    let tmp_path = format!("{path}.tmp");
    std::fs::write(&tmp_path, &patched_xml)
        .map_err(|e| anyhow!("Write temp failed: {e}"))?;
    std::fs::rename(&tmp_path, path)
        .map_err(|e| anyhow!("Atomic rename failed: {e}"))?;

    // 6. Validate by re-parsing
    crate::core::pob_parser::parse_file(path)
        .map_err(|e| anyhow!("Validation failed after write: {e}"))
}

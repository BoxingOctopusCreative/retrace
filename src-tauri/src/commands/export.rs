use crate::export::{convert, ExportFormat};

#[tauri::command]
pub async fn export_vector(
    svg: String,
    format: ExportFormat,
    file_path: String,
) -> Result<(), String> {
    let content = convert(&svg, &format)?;
    std::fs::write(&file_path, content).map_err(|e| e.to_string())
}

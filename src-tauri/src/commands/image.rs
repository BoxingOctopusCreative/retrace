/// Returns raw file bytes so the frontend can construct a blob URL for preview.
/// This avoids needing the asset protocol or fs plugin scope configuration.
#[tauri::command]
pub async fn load_image_bytes(file_path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&file_path).map_err(|e| e.to_string())
}

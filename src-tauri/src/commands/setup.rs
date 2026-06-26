use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::setup::{installer, system};
use crate::state::{AppState, BackendId, BackendState, BackendStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum InstallStage {
    PythonEnv,
    ModelWeights(BackendId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub stage: InstallStage,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
}

#[tauri::command]
pub fn detect_gpu() -> system::GpuInfo {
    system::detect_gpu()
}

#[tauri::command]
pub fn get_disk_space(path: String) -> u64 {
    system::get_disk_space(&path)
}

#[tauri::command]
pub fn get_backend_statuses(state: tauri::State<'_, AppState>) -> Vec<BackendStatus> {
    state.backend_statuses.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_python_env_installed(app: tauri::AppHandle) -> bool {
    installer::is_python_env_installed(&app)
}

#[tauri::command]
pub async fn install_python_env(
    _app: tauri::AppHandle,
    _state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // TODO: invoke uv to create venv + install ML deps (milestone 2)
    Err("Python environment installation not yet implemented — coming in milestone 2.".to_string())
}

#[tauri::command]
pub fn uninstall_python_env(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let dir = installer::python_env_dir(&app);
    for sub in ["venv", "python"] {
        let path = dir.join(sub);
        if path.exists() {
            std::fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
        }
    }
    let mut statuses = state.backend_statuses.lock().unwrap();
    for s in statuses.iter_mut() {
        if !matches!(s.id, BackendId::Vtracer) {
            s.state = BackendState::NotInstalled;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn download_model(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    backend: BackendId,
) -> Result<(), String> {
    {
        let mut statuses = state.backend_statuses.lock().unwrap();
        if let Some(s) = statuses.iter_mut().find(|s| s.id == backend) {
            s.state = BackendState::Installing(0.0);
        }
    }
    // Emit a placeholder progress event so the frontend pipeline can be tested
    let _ = app.emit(
        "install:progress",
        InstallProgress {
            stage: InstallStage::ModelWeights(backend.clone()),
            bytes_downloaded: 0,
            total_bytes: 1,
        },
    );
    {
        let mut statuses = state.backend_statuses.lock().unwrap();
        if let Some(s) = statuses.iter_mut().find(|s| s.id == backend) {
            s.state = BackendState::NotInstalled;
        }
    }
    // TODO: Implement actual model download (milestone 4)
    Err("Model download not yet implemented — coming in milestone 4.".to_string())
}

#[tauri::command]
pub fn cancel_download(state: tauri::State<'_, AppState>, backend: BackendId) {
    let mut statuses = state.backend_statuses.lock().unwrap();
    if let Some(s) = statuses.iter_mut().find(|s| s.id == backend) {
        if matches!(s.state, BackendState::Installing(_)) {
            s.state = BackendState::NotInstalled;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_stage_python_env_serializes_without_value_field() {
        let json = serde_json::to_string(&InstallStage::PythonEnv).unwrap();
        assert_eq!(json, r#"{"kind":"python_env"}"#);
    }

    #[test]
    fn install_stage_model_weights_includes_backend_id() {
        let json = serde_json::to_string(&InstallStage::ModelWeights(BackendId::Live)).unwrap();
        assert_eq!(json, r#"{"kind":"model_weights","value":"live"}"#);
    }

    #[test]
    fn install_stage_model_weights_starvector() {
        let json =
            serde_json::to_string(&InstallStage::ModelWeights(BackendId::StarVector1B)).unwrap();
        assert_eq!(json, r#"{"kind":"model_weights","value":"starvector-1b"}"#);
    }

    #[test]
    fn install_progress_round_trips() {
        let original = InstallProgress {
            stage: InstallStage::ModelWeights(BackendId::StarVector8B),
            bytes_downloaded: 1024,
            total_bytes: 4096,
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: InstallProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.bytes_downloaded, 1024);
        assert_eq!(parsed.total_bytes, 4096);
        assert!(matches!(
            parsed.stage,
            InstallStage::ModelWeights(BackendId::StarVector8B)
        ));
    }

    #[test]
    fn install_stage_deserializes_python_env() {
        let parsed: InstallStage = serde_json::from_str(r#"{"kind":"python_env"}"#).unwrap();
        assert!(matches!(parsed, InstallStage::PythonEnv));
    }

    #[test]
    fn install_stage_deserializes_model_weights() {
        let parsed: InstallStage =
            serde_json::from_str(r#"{"kind":"model_weights","value":"live"}"#).unwrap();
        assert!(matches!(parsed, InstallStage::ModelWeights(BackendId::Live)));
    }
}

#[tauri::command]
pub fn uninstall_backend(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    backend: BackendId,
) -> Result<(), String> {
    let dir = installer::python_env_dir(&app);
    let model_dir = match &backend {
        BackendId::StarVector1B => Some(dir.join("models").join("starvector-1b")),
        BackendId::StarVector8B => Some(dir.join("models").join("starvector-8b")),
        BackendId::Live => Some(dir.join("models").join("live")),
        BackendId::Vtracer => None,
    };
    if let Some(path) = model_dir {
        if path.exists() {
            std::fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
        }
    }
    let mut statuses = state.backend_statuses.lock().unwrap();
    if let Some(s) = statuses.iter_mut().find(|s| s.id == backend) {
        s.state = BackendState::NotInstalled;
    }
    Ok(())
}

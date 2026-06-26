use anyhow::Context;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::setup::{installer, system, uv};
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
pub fn get_disk_space(app: tauri::AppHandle) -> u64 {
    system::get_disk_space(&installer::python_env_dir(&app))
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
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let app_task = app.clone();
    tokio::task::spawn_blocking(move || install_env_blocking(&app_task))
        .await
        .map_err(|e| format!("install task panicked: {e}"))?
        .map_err(|e| e.to_string())?;

    *state.backend_statuses.lock().unwrap() = installer::probe_backend_statuses(&app);
    Ok(())
}

fn install_env_blocking(app: &tauri::AppHandle) -> anyhow::Result<()> {
    use tauri::Manager;

    let env_dir = installer::python_env_dir(app);
    let venv_dir = env_dir.join("venv");
    let venv_str = venv_dir
        .to_str()
        .context("venv path contains non-UTF-8 characters")?
        .to_owned();

    let _ = app.emit(
        "install:progress",
        InstallProgress {
            stage: InstallStage::PythonEnv,
            bytes_downloaded: 0,
            total_bytes: 0,
        },
    );

    // Step 1: create the isolated venv; uv downloads Python 3.11 automatically.
    uv::run_uv_streaming(app, &["venv", &venv_str, "--python", "3.11"], |_| {})?;

    // Step 2: install base ML deps from the bundled requirements file.
    let requirements = app
        .path()
        .resource_dir()
        .context("could not resolve resource directory")?
        .join("requirements.txt");
    let req_str = requirements
        .to_str()
        .context("requirements.txt path contains non-UTF-8 characters")?
        .to_owned();

    uv::run_uv_streaming(
        app,
        &["pip", "install", "--python", &venv_str, "-r", &req_str],
        |_| {},
    )?;

    Ok(())
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

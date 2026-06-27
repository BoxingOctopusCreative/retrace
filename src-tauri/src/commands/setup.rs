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
    let _ = app.emit(
        "install:progress",
        InstallProgress {
            stage: InstallStage::ModelWeights(backend.clone()),
            bytes_downloaded: 0,
            total_bytes: 0,
        },
    );

    let app_task = app.clone();
    let backend_task = backend.clone();
    let result = tokio::task::spawn_blocking(move || {
        download_model_blocking(&app_task, &backend_task)
    })
    .await
    .map_err(|e| format!("download task panicked: {e}"))?;

    {
        let mut statuses = state.backend_statuses.lock().unwrap();
        if let Some(s) = statuses.iter_mut().find(|s| s.id == backend) {
            s.state = match &result {
                Ok(_) => BackendState::Ready,
                Err(_) => BackendState::NotInstalled,
            };
        }
    }

    result.map_err(|e| e.to_string())
}

fn download_model_blocking(app: &tauri::AppHandle, backend: &BackendId) -> anyhow::Result<()> {
    use anyhow::Context;
    use std::io::BufRead;
    use tauri::Manager;

    let env_dir = installer::python_env_dir(app);
    let venv_dir = env_dir.join("venv");

    if !venv_dir.exists() {
        anyhow::bail!(
            "Python environment not installed — set it up via Settings → Enhanced Backends first"
        );
    }

    // LIVE is purely algorithmic; just create the marker directory.
    if matches!(backend, BackendId::Live) {
        std::fs::create_dir_all(env_dir.join("models").join("live"))
            .context("failed to create LIVE model directory")?;
        let _ = app.emit(
            "install:progress",
            InstallProgress {
                stage: InstallStage::ModelWeights(backend.clone()),
                bytes_downloaded: 1,
                total_bytes: 1,
            },
        );
        return Ok(());
    }

    let model_subdir = match backend {
        BackendId::StarVector1B => "starvector-1b",
        BackendId::StarVector8B => "starvector-8b",
        BackendId::Live | BackendId::Vtracer => return Ok(()),
    };
    let model_dir = env_dir.join("models").join(model_subdir);
    std::fs::create_dir_all(&model_dir).context("failed to create model directory")?;

    // Step 1: install backend-specific pip packages (transformers pulls in huggingface_hub).
    let venv_str = venv_dir
        .to_str()
        .context("venv path is not valid UTF-8")?
        .to_owned();
    uv::run_uv_streaming(
        app,
        &[
            "pip",
            "install",
            "--python",
            &venv_str,
            "transformers>=4.35.0",
            "accelerate>=0.27.0",
            "huggingface_hub>=0.20.0",
        ],
        |_| {},
    )?;

    // Step 2: download weights via the bundled download_model.py script.
    let download_script = app
        .path()
        .resource_dir()
        .context("could not resolve resource directory")?
        .join("download_model.py");

    #[cfg(target_os = "windows")]
    let python = venv_dir.join("Scripts").join("python.exe");
    #[cfg(not(target_os = "windows"))]
    let python = venv_dir.join("bin").join("python3");

    let model_dir_str = model_dir
        .to_str()
        .context("model dir path is not valid UTF-8")?
        .to_owned();

    let mut child = std::process::Command::new(&python)
        .arg(&download_script)
        .arg("--backend")
        .arg(model_subdir)
        .arg("--model-dir")
        .arg(&model_dir_str)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("failed to spawn download_model.py")?;

    let stderr = child.stderr.take().expect("stderr was piped");
    let app_clone = app.clone();
    let backend_clone = backend.clone();

    let stderr_thread = std::thread::spawn(move || {
        let mut total_files: u64 = 0;
        let mut lines = Vec::<String>::new();

        for line in std::io::BufReader::new(stderr).lines().flatten() {
            if let Some(rest) = line.strip_prefix("progress:start:") {
                total_files = rest.parse().unwrap_or(1);
                let _ = app_clone.emit(
                    "install:progress",
                    InstallProgress {
                        stage: InstallStage::ModelWeights(backend_clone.clone()),
                        bytes_downloaded: 0,
                        total_bytes: total_files,
                    },
                );
            } else if let Some(rest) = line.strip_prefix("progress:file:") {
                let completed: u64 = rest.parse().unwrap_or(0);
                let _ = app_clone.emit(
                    "install:progress",
                    InstallProgress {
                        stage: InstallStage::ModelWeights(backend_clone.clone()),
                        bytes_downloaded: completed,
                        total_bytes: total_files.max(1),
                    },
                );
            }
            lines.push(line);
        }
        lines
    });

    let status = child.wait().context("failed to wait for download process")?;
    let stderr_lines = stderr_thread.join().unwrap_or_default();

    if !status.success() {
        anyhow::bail!("model download failed:\n{}", stderr_lines.join("\n").trim());
    }

    Ok(())
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

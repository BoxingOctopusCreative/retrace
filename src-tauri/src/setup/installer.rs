use crate::state::{BackendId, BackendState, BackendStatus};

/// Root directory for all Python environment data: venv, python runtime, models.
pub fn python_env_dir(app: &tauri::AppHandle) -> std::path::PathBuf {
    use tauri::Manager;
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
}

pub fn is_python_env_installed(app: &tauri::AppHandle) -> bool {
    is_env_installed_at(&python_env_dir(app))
}

pub fn is_model_installed(app: &tauri::AppHandle, backend: &BackendId) -> bool {
    is_model_installed_at(&python_env_dir(app), backend)
}

pub fn default_backend_statuses() -> Vec<BackendStatus> {
    vec![
        BackendStatus { id: BackendId::Vtracer, state: BackendState::Ready },
        BackendStatus { id: BackendId::Live, state: BackendState::NotInstalled },
        BackendStatus { id: BackendId::StarVector1B, state: BackendState::NotInstalled },
        BackendStatus { id: BackendId::StarVector8B, state: BackendState::NotInstalled },
    ]
}

pub fn probe_backend_statuses(app: &tauri::AppHandle) -> Vec<BackendStatus> {
    probe_statuses_at(&python_env_dir(app))
}

// ── Pure helpers (path-only, no AppHandle) ───────────────────────────────────

fn venv_python(base: &std::path::Path) -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    return base.join("venv").join("Scripts").join("python.exe");
    #[cfg(not(target_os = "windows"))]
    base.join("venv").join("bin").join("python3")
}

fn is_env_installed_at(dir: &std::path::Path) -> bool {
    venv_python(dir).exists()
}

fn is_model_installed_at(dir: &std::path::Path, backend: &BackendId) -> bool {
    match backend {
        BackendId::Vtracer => true,
        BackendId::Live => dir.join("models").join("live").exists(),
        BackendId::StarVector1B => dir.join("models").join("starvector-1b").exists(),
        BackendId::StarVector8B => dir.join("models").join("starvector-8b").exists(),
    }
}

fn probe_statuses_at(dir: &std::path::Path) -> Vec<BackendStatus> {
    let env_ok = is_env_installed_at(dir);
    vec![
        BackendStatus { id: BackendId::Vtracer, state: BackendState::Ready },
        BackendStatus {
            id: BackendId::Live,
            state: if env_ok && is_model_installed_at(dir, &BackendId::Live) {
                BackendState::Ready
            } else {
                BackendState::NotInstalled
            },
        },
        BackendStatus {
            id: BackendId::StarVector1B,
            state: if env_ok && is_model_installed_at(dir, &BackendId::StarVector1B) {
                BackendState::Ready
            } else {
                BackendState::NotInstalled
            },
        },
        BackendStatus {
            id: BackendId::StarVector8B,
            state: if env_ok && is_model_installed_at(dir, &BackendId::StarVector8B) {
                BackendState::Ready
            } else {
                BackendState::NotInstalled
            },
        },
    ]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn default_statuses_has_four_entries() {
        assert_eq!(default_backend_statuses().len(), 4);
    }

    #[test]
    fn vtracer_is_ready_by_default() {
        let s = default_backend_statuses();
        let vt = s.iter().find(|b| b.id == BackendId::Vtracer).unwrap();
        assert!(matches!(vt.state, BackendState::Ready));
    }

    #[test]
    fn ml_backends_not_installed_by_default() {
        let s = default_backend_statuses();
        for id in [BackendId::Live, BackendId::StarVector1B, BackendId::StarVector8B] {
            let b = s.iter().find(|b| b.id == id).unwrap();
            assert!(matches!(b.state, BackendState::NotInstalled), "{:?} should be NotInstalled", id);
        }
    }

    #[test]
    fn venv_python_path_has_expected_structure() {
        let base = std::path::Path::new("/fake/base");
        let p = venv_python(base);
        #[cfg(target_os = "windows")]
        assert!(p.ends_with("venv/Scripts/python.exe"));
        #[cfg(not(target_os = "windows"))]
        assert!(p.ends_with("venv/bin/python3"));
    }

    #[test]
    fn env_not_installed_when_venv_missing() {
        let dir = std::env::temp_dir().join("retrace_test_no_venv");
        let _ = fs::remove_dir_all(&dir);
        assert!(!is_env_installed_at(&dir));
    }

    #[test]
    fn env_installed_when_venv_python_exists() {
        let dir = std::env::temp_dir().join("retrace_test_venv");
        let python = venv_python(&dir);
        fs::create_dir_all(python.parent().unwrap()).unwrap();
        fs::write(&python, b"").unwrap();

        assert!(is_env_installed_at(&dir));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn vtracer_model_always_installed() {
        let dir = std::env::temp_dir().join("retrace_test_vtracer_model");
        assert!(is_model_installed_at(&dir, &BackendId::Vtracer));
    }

    #[test]
    fn ml_model_not_installed_when_dir_missing() {
        let dir = std::env::temp_dir().join("retrace_test_no_model");
        let _ = fs::remove_dir_all(&dir);
        assert!(!is_model_installed_at(&dir, &BackendId::Live));
        assert!(!is_model_installed_at(&dir, &BackendId::StarVector1B));
        assert!(!is_model_installed_at(&dir, &BackendId::StarVector8B));
    }

    #[test]
    fn ml_model_installed_when_dir_present() {
        let dir = std::env::temp_dir().join("retrace_test_with_model");
        fs::create_dir_all(dir.join("models").join("live")).unwrap();
        assert!(is_model_installed_at(&dir, &BackendId::Live));
        assert!(!is_model_installed_at(&dir, &BackendId::StarVector1B));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn probe_all_not_installed_when_no_env() {
        let dir = std::env::temp_dir().join("retrace_test_probe_no_env");
        let _ = fs::remove_dir_all(&dir);
        let s = probe_statuses_at(&dir);
        let vt = s.iter().find(|b| b.id == BackendId::Vtracer).unwrap();
        assert!(matches!(vt.state, BackendState::Ready));
        for id in [BackendId::Live, BackendId::StarVector1B, BackendId::StarVector8B] {
            let b = s.iter().find(|b| b.id == id).unwrap();
            assert!(matches!(b.state, BackendState::NotInstalled));
        }
    }

    #[test]
    fn probe_live_ready_when_env_and_model_present() {
        let dir = std::env::temp_dir().join("retrace_test_probe_live_ready");
        let python = venv_python(&dir);
        fs::create_dir_all(python.parent().unwrap()).unwrap();
        fs::write(&python, b"").unwrap();
        fs::create_dir_all(dir.join("models").join("live")).unwrap();

        let s = probe_statuses_at(&dir);
        let live = s.iter().find(|b| b.id == BackendId::Live).unwrap();
        assert!(matches!(live.state, BackendState::Ready));

        fs::remove_dir_all(&dir).unwrap();
    }
}

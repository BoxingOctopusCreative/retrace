use anyhow::{anyhow, Context};
use image::ColorType;
use serde::Serialize;
use tauri::Emitter;
use tauri::Manager;

use super::{RasterImage, VectorOutput};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceProgress {
    pub backend: String,
    /// Raw progress line from the sidecar (e.g. "progress:layer:3/8").
    pub message: String,
}

/// Invoke sidecar.py via the uv-managed venv Python interpreter.
///
/// Stderr is read in a background thread and forwarded as `trace:progress`
/// Tauri events so the frontend can display live progress. SVG is read
/// from stdout after the process exits.
pub fn call_sidecar(
    app: &tauri::AppHandle,
    backend: &str,
    input: &RasterImage,
) -> anyhow::Result<VectorOutput> {
    use crate::setup::installer;
    use std::io::Read;

    let env_dir = installer::python_env_dir(app);

    #[cfg(windows)]
    let python = env_dir.join("venv").join("Scripts").join("python.exe");
    #[cfg(not(windows))]
    let python = env_dir.join("venv").join("bin").join("python3");

    if !python.exists() {
        return Err(anyhow!(
            "Python environment not installed — set it up via Settings → Enhanced Backends"
        ));
    }

    let sidecar_path = app
        .path()
        .resource_dir()
        .context("could not resolve resource directory")?
        .join("sidecar.py");

    let input_path =
        std::env::temp_dir().join(format!("retrace_in_{}.png", std::process::id()));
    image::save_buffer(
        &input_path,
        &input.data,
        input.width,
        input.height,
        ColorType::Rgba8,
    )
    .context("failed to write temp input image")?;

    // Tell the sidecar where locally downloaded model weights live so backends
    // like StarVector can load from disk without hitting the HuggingFace hub.
    let model_dir = env_dir.join("models").join(backend);

    let mut child = std::process::Command::new(&python)
        .arg(&sidecar_path)
        .arg("--backend")
        .arg(backend)
        .arg("--input")
        .arg(&input_path)
        .env("RETRACE_MODEL_DIR", &model_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("failed to spawn sidecar process")?;

    // Drain stderr in a thread, emitting each line as a `trace:progress` event.
    let stderr = child.stderr.take().expect("stderr was piped");
    let app_clone = app.clone();
    let backend_str = backend.to_string();
    let stderr_thread = std::thread::spawn(move || {
        use std::io::BufRead;
        let mut lines = Vec::<String>::new();
        for line in std::io::BufReader::new(stderr).lines().flatten() {
            let _ = app_clone.emit(
                "trace:progress",
                TraceProgress {
                    backend: backend_str.clone(),
                    message: line.clone(),
                },
            );
            lines.push(line);
        }
        lines
    });

    // Read stdout (SVG) concurrently so the process is never blocked on a full
    // stdout pipe buffer.
    let mut stdout_bytes = Vec::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_to_end(&mut stdout_bytes).ok();
    }

    let status = child.wait().context("failed to wait for sidecar")?;
    let stderr_lines = stderr_thread.join().unwrap_or_default();

    let _ = std::fs::remove_file(&input_path);

    if !status.success() {
        return Err(anyhow!(
            "sidecar error: {}",
            stderr_lines.join("\n").trim()
        ));
    }

    let svg =
        String::from_utf8(stdout_bytes).context("sidecar output was not valid UTF-8")?;

    if svg.trim().is_empty() {
        return Err(anyhow!("sidecar returned empty output"));
    }

    Ok(VectorOutput { svg })
}

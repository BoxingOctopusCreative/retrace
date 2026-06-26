use anyhow::{anyhow, Context};
use image::ColorType;
use tauri::Manager;

use super::{RasterImage, VectorOutput};

/// Invoke sidecar.py via the uv-managed venv Python interpreter.
/// The sidecar writes SVG to stdout; stderr carries error messages.
pub fn call_sidecar(
    app: &tauri::AppHandle,
    backend: &str,
    input: &RasterImage,
) -> anyhow::Result<VectorOutput> {
    use crate::setup::installer;

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

    // Write input to a per-process temp file to avoid collisions
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

    let result = std::process::Command::new(&python)
        .arg(&sidecar_path)
        .arg("--backend")
        .arg(backend)
        .arg("--input")
        .arg(&input_path)
        .output()
        .context("failed to spawn sidecar process");

    let _ = std::fs::remove_file(&input_path);

    let output = result?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("sidecar error: {}", stderr.trim()));
    }

    let svg =
        String::from_utf8(output.stdout).context("sidecar output was not valid UTF-8")?;

    if svg.trim().is_empty() {
        return Err(anyhow!("sidecar returned empty output"));
    }

    Ok(VectorOutput { svg })
}

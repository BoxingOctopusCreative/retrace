/// uv invocation scaffold — milestone 2 implementation.
///
/// uv is a Rust-native Python environment manager (~15 MB) bundled as a
/// Tauri sidecar binary. It installs an isolated Python runtime and venv
/// without requiring any system Python.
use anyhow::{bail, Context};

pub fn run_uv(app: &tauri::AppHandle, args: &[&str]) -> anyhow::Result<std::process::Output> {
    use tauri::Manager;

    let resource_dir = app
        .path()
        .resource_dir()
        .context("could not resolve resource directory")?;

    let triple = target_triple();
    #[cfg(target_os = "windows")]
    let binary_name = format!("uv-{}.exe", triple);
    #[cfg(not(target_os = "windows"))]
    let binary_name = format!("uv-{}", triple);

    let binary = resource_dir.join("binaries").join(&binary_name);
    if !binary.exists() {
        bail!(
            "uv binary not found at {:?} — place it in src-tauri/binaries/ (see spec)",
            binary
        );
    }

    std::process::Command::new(&binary)
        .args(args)
        .output()
        .with_context(|| format!("failed to run uv with args {:?}", args))
}

fn target_triple() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { "aarch64-apple-darwin" }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { "x86_64-apple-darwin" }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    { "x86_64-unknown-linux-gnu" }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    { "x86_64-pc-windows-msvc" }
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86_64"),
    )))]
    { "unknown" }
}

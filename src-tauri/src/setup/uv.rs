use anyhow::Context;

/// Invoke uv with `args`, calling `on_line` for each line written to stderr.
/// Blocks until uv exits; returns an error if the exit status is non-zero.
pub fn run_uv_streaming<F>(
    app: &tauri::AppHandle,
    args: &[&str],
    mut on_line: F,
) -> anyhow::Result<()>
where
    F: FnMut(&str),
{
    use std::io::{BufRead, BufReader};

    let binary = uv_binary(app)?;
    let mut child = std::process::Command::new(&binary)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn uv with args {:?}", args))?;

    let mut stderr_lines: Vec<String> = Vec::new();
    if let Some(stderr) = child.stderr.take() {
        for line in BufReader::new(stderr).lines().flatten() {
            on_line(&line);
            stderr_lines.push(line);
        }
    }

    let status = child.wait().context("failed to wait for uv")?;
    if !status.success() {
        anyhow::bail!(
            "uv {:?} exited with status {:?}:\n{}",
            args,
            status.code(),
            stderr_lines.join("\n").trim()
        );
    }
    Ok(())
}

// In dev builds `resource_dir` resolves to `CARGO_MANIFEST_DIR` (src-tauri/),
// which is where we keep the binary during development. In release builds the
// sidecar lives next to the executable, so we walk up from `current_exe()`.
fn uv_binary(app: &tauri::AppHandle) -> anyhow::Result<std::path::PathBuf> {
    use tauri::Manager;

    #[cfg(target_os = "windows")]
    let name = format!("uv-{}.exe", target_triple());
    #[cfg(not(target_os = "windows"))]
    let name = format!("uv-{}", target_triple());

    #[cfg(dev)]
    let binary = {
        let dir = app
            .path()
            .resource_dir()
            .context("could not resolve resource directory")?;
        dir.join("binaries").join(&name)
    };

    #[cfg(not(dev))]
    let binary = {
        let _ = app; // not needed outside dev
        // In production Tauri places sidecars next to the executable,
        // with no binaries/ subdirectory prefix.
        let dir = std::env::current_exe()
            .context("could not get current executable path")?
            .parent()
            .context("executable has no parent directory")?
            .to_path_buf();
        dir.join(&name)
    };
    if !binary.exists() {
        anyhow::bail!("uv binary not found at {:?}", binary);
    }
    Ok(binary)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_triple_is_known_platform() {
        assert_ne!(
            target_triple(),
            "unknown",
            "add a cfg branch for this build target"
        );
    }
}

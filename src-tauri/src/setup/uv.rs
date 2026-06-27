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

fn uv_binary(app: &tauri::AppHandle) -> anyhow::Result<std::path::PathBuf> {
    #[cfg(dev)]
    let binary = {
        let _ = app;
        // Dev: binary lives in src-tauri/binaries/ with the triple suffix so
        // Tauri can select the right platform binary during cargo tauri dev.
        // CARGO_MANIFEST_DIR is baked in at compile time and always points to
        // src-tauri/, regardless of where the dev process runs from.
        #[cfg(target_os = "windows")]
        let name = format!("uv-{}.exe", target_triple());
        #[cfg(not(target_os = "windows"))]
        let name = format!("uv-{}", target_triple());
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join(name)
    };

    #[cfg(not(dev))]
    let binary = {
        let _ = app;
        // Production: Tauri strips the triple suffix when it installs sidecars,
        // so the installed file is just "uv" / "uv.exe" next to the executable.
        let dir = std::env::current_exe()
            .context("could not get current executable path")?
            .parent()
            .context("executable has no parent directory")?
            .to_path_buf();
        #[cfg(target_os = "windows")]
        { dir.join("uv.exe") }
        #[cfg(not(target_os = "windows"))]
        { dir.join("uv") }
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

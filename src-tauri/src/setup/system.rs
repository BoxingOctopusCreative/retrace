use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComputeBackend {
    Metal,
    Cuda,
    Rocm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: Option<u64>,
    pub compute_backend: Option<ComputeBackend>,
}

pub fn detect_gpu() -> GpuInfo {
    if let Some(info) = try_nvidia() {
        return info;
    }
    #[cfg(target_os = "macos")]
    if let Some(info) = try_metal() {
        return info;
    }
    if let Some(info) = try_rocm() {
        return info;
    }
    GpuInfo { name: "No GPU detected".into(), vram_mb: None, compute_backend: None }
}

fn try_nvidia() -> Option<GpuInfo> {
    let out = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total", "--format=csv,noheader,nounits"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.trim().lines().next()?;
    let mut parts = line.splitn(2, ", ");
    let name = parts.next()?.trim().to_string();
    let vram_mb = parts.next().and_then(|v| v.trim().parse().ok());
    Some(GpuInfo { name, vram_mb, compute_backend: Some(ComputeBackend::Cuda) })
}

#[cfg(target_os = "macos")]
fn try_metal() -> Option<GpuInfo> {
    let out = std::process::Command::new("system_profiler")
        .arg("SPDisplaysDataType")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    // First non-empty line after "Graphics/Displays:" is the GPU model name
    let name = text
        .lines()
        .skip_while(|l| !l.contains("Graphics"))
        .skip(1)
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim().trim_end_matches(':').to_string())
        .unwrap_or_else(|| "Apple GPU".to_string());
    let vram_mb: Option<u64> = text
        .lines()
        .find(|l| l.contains("VRAM") && l.contains("MB"))
        .and_then(|l| l.split_whitespace().find(|s| s.parse::<u64>().is_ok()))
        .and_then(|s| s.parse().ok())
        .filter(|&v| v > 0); // M-series reports 0 (unified memory)
    Some(GpuInfo { name, vram_mb, compute_backend: Some(ComputeBackend::Metal) })
}

fn try_rocm() -> Option<GpuInfo> {
    let out = std::process::Command::new("rocm-smi")
        .arg("--showproductname")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let name = text
        .lines()
        .find(|l| l.contains("Card") || l.contains("GPU"))
        .map(|l| l.trim().to_string())
        .unwrap_or_else(|| "AMD GPU".to_string());
    Some(GpuInfo { name, vram_mb: None, compute_backend: Some(ComputeBackend::Rocm) })
}

pub fn get_disk_space(path: &str) -> u64 {
    use sysinfo::Disks;
    let check = std::path::Path::new(path);
    let disks = Disks::new_with_refreshed_list();
    let mut best_space = 0u64;
    let mut best_len = 0usize;
    for disk in &disks {
        let mount = disk.mount_point();
        if check.starts_with(mount) && mount.as_os_str().len() > best_len {
            best_len = mount.as_os_str().len();
            best_space = disk.available_space();
        }
    }
    best_space
}

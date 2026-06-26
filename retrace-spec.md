# Re:Trace — Project Specification

> Open-source desktop image tracing application  
> A Boxing Octopus Creative project  
> License: MPL 2.0

---

## Overview

Re:Trace is a cross-platform desktop application for converting raster images (PNG, JPEG, etc.) into scalable vector graphics (SVG). It ships with a fast, zero-setup default backend (vtracer) and supports optional enhanced ML-powered backends, installable via a built-in setup tool. No system Python is required — all ML dependencies are managed via a bundled `uv` binary into a fully isolated environment.

---

## Goals

- Lightweight, native-feeling desktop app — no Electron
- Zero-friction default experience via vtracer — no Python, no setup
- Optional ML-powered backends for users who want them
- Clean abstraction layer — all backends implement one Rust trait
- No system Python dependency — fully isolated ML environment via `uv`
- Cross-platform: macOS, Windows, Linux
- Open source under MPL 2.0

## Non-Goals

- Web or mobile versions (at this time)
- Cloud processing or account systems
- Bundling ML model weights in the base install

---

## Tech Stack

| Layer | Technology |
|---|---|
| UI Framework | Svelte + TypeScript + Vite |
| Desktop Shell | Tauri v2 |
| Core / Backend | Rust |
| Default Tracing Backend | vtracer (Rust crate, zero Python) |
| Optional ML Backends | LIVE, StarVector-1B, StarVector-8B |
| Python Environment Manager | `uv` (bundled ~15 MB binary) |
| ML Runtime | Isolated Python venv managed by `uv` |
| Tracing Abstraction | `ImageTracer` trait |

---

## Backends

### Default: vtracer
- **Zero setup** — ships with Re:Trace as a Rust crate, no downloads, no Python
- **O(n) algorithm**, handles full-colour images, fast on any hardware
- **Best for:** general use, flat graphics, photos, illustrations
- **License:** MIT — no conflicts

### Optional: LIVE (Layer-wise Image Vectorization)
- **Requires:** uv-managed Python environment (~4 GB)
- **No model weights download** — optimization-based, not a pretrained model
- **Notably slower** — iterative process, not single-pass inference. Surfaced clearly in UI.
- **Best for:** artistic images, complex illustrations
- **License:** MIT

### Optional: StarVector
- **Requires:** uv-managed Python environment + model weights download
- **Two quality tiers** (user selects one or both):

| Tier | Disk (FP16) | Disk (Q4) | Min VRAM |
|---|---|---|---|
| StarVector-1B | ~2 GB | ~0.5 GB | 4 GB |
| StarVector-8B | ~16 GB | ~4 GB | 8 GB |

- **Best for:** icons, logos, technical diagrams — NOT natural images or illustrations
- **License:** Apache 2.0

### Backend Comparison

| Backend | Setup | Speed | Best for |
|---|---|---|---|
| vtracer | Zero | Fast | General use, photos, flat graphics |
| LIVE | Python env only | Slow | Artistic, illustrative |
| StarVector-1B | Python env + ~2 GB | Medium | Icons, logos, diagrams |
| StarVector-8B | Python env + ~16 GB | Slower | Icons, logos, diagrams (best quality) |

---

## Python Environment — `uv` Architecture

### Why uv

The standard Tauri sidecar approach (PyInstaller) would produce a 3–5 GB binary just for the PyTorch runtime — before any model weights. Instead, Re:Trace bundles `uv`, a ~15 MB Rust-native Python environment manager. `uv` installs an isolated Python runtime and manages all ML dependencies without touching the system Python or PATH.

### How it works

```
User clicks "Install Sidecar Environment"
  → Re:Trace invokes bundled uv binary
  → uv downloads and installs an isolated Python runtime
  → uv creates a venv in the app data directory
  → uv installs PyTorch + ML dependencies into that venv
  → sidecar.py (bundled Tauri resource) runs against that venv
```

### App Data Layout

Platform-specific app data directory (`~/.local/share/retrace/` on Linux, `~/Library/Application Support/retrace/` on macOS, `%APPDATA%\retrace\` on Windows):

```
retrace/
├── python/                         # uv-managed Python runtime
├── venv/                           # isolated virtual environment
│   └── lib/site-packages/          # PyTorch, LIVE, StarVector deps
└── models/
    ├── starvector-1b/              # StarVector-1B weights
    └── starvector-8b/              # StarVector-8B weights
```

### Sidecar Interface

`sidecar.py` is bundled as a Tauri resource. The Rust backend always calls the venv Python binary directly — no system Python, no PATH dependency.

```
<venv>/bin/python sidecar.py --backend live --input image.png
<venv>/bin/python sidecar.py --backend starvector-1b --input image.png
<venv>/bin/python sidecar.py --backend starvector-8b --input image.png
```

All optional `ImageTracer` backends call the same sidecar with different args — one process, no duplication.

---

## Project Structure

```
retrace/
├── src/                                  # Svelte frontend
│   ├── lib/
│   │   ├── components/
│   │   │   ├── Canvas.svelte             # Preview pane (raster in, SVG out)
│   │   │   ├── Controls.svelte           # Trace options (sliders, dropdowns)
│   │   │   ├── Toolbar.svelte            # Open file, export, backend switcher
│   │   │   └── settings/
│   │   │       ├── Settings.svelte       # Settings root
│   │   │       └── BackendSetup.svelte   # Enhanced backends setup panel
│   │   ├── stores/
│   │   │   ├── tracing.ts               # Reactive state (image, SVG output, options)
│   │   │   └── backends.ts              # Backend status, install progress
│   │   └── types.ts                     # Shared TS types mirroring Rust structs
│   ├── App.svelte
│   └── main.ts
│
├── src-tauri/                            # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── trace.rs                 # Core tracing commands
│   │   │   └── setup.rs                 # System detection, install, download
│   │   ├── tracer/
│   │   │   ├── mod.rs                   # ImageTracer trait + shared types
│   │   │   ├── vtracer.rs               # vtracer backend (default, pure Rust)
│   │   │   ├── sidecar.rs               # Shared sidecar launcher + venv path resolution
│   │   │   ├── live.rs                  # LIVE backend (invokes sidecar)
│   │   │   └── starvector.rs            # StarVector backend (invokes sidecar, 1B/8B)
│   │   ├── setup/
│   │   │   ├── mod.rs                   # Setup orchestration
│   │   │   ├── system.rs                # GPU, disk detection
│   │   │   ├── uv.rs                    # uv invocation (Python install, venv, deps)
│   │   │   └── installer.rs             # Model weight download management
│   │   └── state.rs                     # AppState
│   ├── binaries/
│   │   ├── uv-x86_64-unknown-linux-gnu  # uv binary per target triple
│   │   ├── uv-aarch64-apple-darwin
│   │   └── uv-x86_64-pc-windows-msvc
│   ├── resources/
│   │   └── sidecar.py                   # Python sidecar (bundled resource)
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── sidecar/                              # Python sidecar source
│   ├── sidecar.py                        # Entry point, CLI arg routing
│   ├── backends/
│   │   ├── live.py                       # LIVE inference wrapper
│   │   └── starvector.py                 # StarVector inference wrapper
│   └── requirements.txt                  # Deps installed by uv at setup time
│
├── package.json
└── vite.config.ts
```

---

## Tracer Abstraction Layer

### Shared Types (`tracer/mod.rs`)

```rust
pub struct RasterImage {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct TraceOptions {
    pub color_precision: u8,
    pub filter_speckle: u32,
    pub corner_threshold: f64,
    // extend as needed
}

pub struct VectorOutput {
    pub svg: String,
}

pub trait ImageTracer: Send + Sync {
    fn trace(&self, input: &RasterImage, opts: &TraceOptions) -> anyhow::Result<VectorOutput>;
    fn name(&self) -> &str;
}
```

### App State (`state.rs`)

```rust
pub struct AppState {
    pub tracer: Mutex<Box<dyn ImageTracer>>,
    pub backend_statuses: Mutex<Vec<BackendStatus>>,
}

pub struct BackendStatus {
    pub id: BackendId,
    pub state: BackendState,
}

pub enum BackendId {
    Vtracer,
    Live,
    StarVector1B,
    StarVector8B,
}

pub enum BackendState {
    Ready,
    NotInstalled,
    Incompatible(String), // reason, e.g. "Insufficient VRAM (8 GB required)"
    Installing(f32),      // 0.0–1.0 progress
    Error(String),
}
```

Swapping the active backend is a single mutex lock and box replacement — nothing else changes.

---

## Tauri Command Surface

### Tracing

```rust
#[tauri::command]
pub async fn trace_image(
    state: tauri::State<'_, AppState>,
    image_data: Vec<u8>,
    width: u32,
    height: u32,
    opts: TraceOptions,
) -> Result<String, String>

#[tauri::command]
pub fn set_backend(
    state: tauri::State<'_, AppState>,
    backend: BackendId,
) -> Result<(), String>
```

### System Detection

```rust
#[tauri::command] fn detect_gpu() -> GpuInfo        // VRAM, vendor, Metal/CUDA/ROCm support
#[tauri::command] fn get_disk_space(path: String) -> u64
#[tauri::command] fn get_backend_statuses() -> Vec<BackendStatus>
```

### Install Lifecycle

```rust
#[tauri::command] async fn install_python_env() -> Result<(), String>  // runs uv
#[tauri::command] async fn download_model(backend: BackendId) -> Result<(), String>
#[tauri::command] fn cancel_download(backend: BackendId)
#[tauri::command] fn uninstall_backend(backend: BackendId)             // removes weights
#[tauri::command] fn uninstall_python_env() -> Result<(), String>      // removes venv
```

Progress streams via Tauri events — no polling:

```rust
app.emit("install:progress", InstallProgress {
    stage: InstallStage::PythonEnv,
    bytes_downloaded: 800_000_000,
    total_bytes: 3_800_000_000,
})

app.emit("install:progress", InstallProgress {
    stage: InstallStage::ModelWeights(BackendId::StarVector1B),
    bytes_downloaded: 1_200_000_000,
    total_bytes: 2_100_000_000,
})
```

---

## Frontend State (Svelte)

### Tracing Store (`stores/tracing.ts`)

```typescript
import { invoke } from '@tauri-apps/api/core';
import { writable } from 'svelte/store';

export const svgOutput = writable<string | null>(null);
export const isTracing = writable(false);

export async function runTrace(
    imageData: Uint8Array,
    width: number,
    height: number,
    opts: TraceOptions
) {
    isTracing.set(true);
    const svg = await invoke<string>('trace_image', {
        imageData: Array.from(imageData),
        width,
        height,
        opts,
    });
    svgOutput.set(svg);
    isTracing.set(false);
}
```

### Backend Store (`stores/backends.ts`)

```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { writable } from 'svelte/store';

export const backendStatuses = writable<BackendStatus[]>([]);
export const installProgress = writable<InstallProgress | null>(null);

await listen<InstallProgress>('install:progress', ({ payload }) => {
    installProgress.set(payload);
});
```

---

## Setup Tool (Settings Panel)

Accessible at **Settings → Enhanced Backends**. Runs a system check automatically on first open. No Python check needed — `uv` handles the runtime.

### System Check

```
🖥️ System
GPU                 ✓ Apple M1 (32 GB unified)
Available Disk      142 GB free
```

### Panel Layout

```
Settings → Enhanced Backends
┌──────────────────────────────────────────────────┐
│ 🖥️ System                                         │
│ GPU: Apple M1 (32 GB unified) ✓                  │
│ Available disk: 142 GB                            │
├──────────────────────────────────────────────────┤
│ Python Environment                   [Install]   │
│ ~4 GB · Managed automatically · No Python needed │
│ Powers all enhanced backends                     │
├──────────────────────────────────────────────────┤
│ LIVE                              ● Not Installed │
│ Best for artistic / illustrative images           │
│ ⚠ Significantly slower than vtracer              │
│ No model download required                       │
│ Requires: Python Environment      [Install]      │
├──────────────────────────────────────────────────┤
│ StarVector                        ● Not Installed │
│ Best for icons, logos, technical diagrams         │
│ Not suitable for photos or illustrations          │
│                                                  │
│ ○ StarVector-1B   ~2 GB    [Download]            │
│ ○ StarVector-8B   ~16 GB   [Download]            │
│   ⚠ Requires 8 GB VRAM — grayed if incompatible │
│                                                  │
│ Requires: Python Environment + 4 GB VRAM         │
└──────────────────────────────────────────────────┘
```

### UX Rules

- **LIVE and StarVector install buttons are disabled** until the Python environment is installed. Dependency is visually indicated, not hidden.
- **StarVector-8B is greyed out** if VRAM check fails, with tooltip explaining why. Visible but not clickable.
- **Disk space warning** fires before any install if available disk is within 10% of the download size.
- **Progress shown inline** per install stage — bytes downloaded, total, estimated time, Cancel button.
- **LIVE shows a speed warning** before install — not to discourage, just to set expectations.
- **Each component is individually uninstallable** — model weights, Python environment, or everything. Users are never locked in.
- **Q4 quantization option** offered at StarVector download time with a brief quality tradeoff note.
- **StarVector quality toggle** (1B / 8B) appears in the main Toolbar backend switcher once either model is installed. Does not require returning to Settings.

---

## Backend Switcher (Toolbar)

Default state (no optional backends installed):

```
Backend: Standard (vtracer) ▾
```

With optional backends installed:

```
Backend: StarVector ▾  Quality: 1B ○  8B ●
```

Quality toggle only appears when StarVector is the active backend and both models are installed.

---

## Storage Footprint

| Component | Size |
|---|---|
| Re:Trace base app | < 50 MB |
| `uv` binary (bundled) | ~15 MB |
| `sidecar.py` (bundled) | < 1 MB |
| Python runtime (uv-managed) | ~100 MB |
| PyTorch + ML deps (venv) | ~3–4 GB |
| CUDA toolkit (NVIDIA only) | ~4 GB |
| LIVE dependencies | ~1 GB |
| StarVector-1B weights (FP16) | ~2 GB |
| StarVector-1B weights (Q4) | ~0.5 GB |
| StarVector-8B weights (FP16) | ~16 GB |
| StarVector-8B weights (Q4) | ~4 GB |

All optional components live under the platform app data directory and are individually removable.

---

## Code Signing

Required for trusted distribution on macOS and Windows.

### macOS
- **Apple Developer Program** — $99/year
- **Developer ID Application certificate** — for distribution outside the App Store
- **Notarization** — required in addition to signing for Gatekeeper approval
- **Entitlements** — `com.apple.security.cs.allow-jit` and `com.apple.security.cs.allow-unsigned-executable-memory` required for Tauri's WebView

Environment variables (stored as GitHub Actions secrets):
```
APPLE_CERTIFICATE
APPLE_CERTIFICATE_PASSWORD
APPLE_ID
APPLE_PASSWORD          # app-specific password
APPLE_TEAM_ID
APPLE_SIGNING_IDENTITY
```

### Windows
- **OV certificate** (~$150–300/year) — signs the binary; SmartScreen reputation builds over time. Can be accelerated via Microsoft's manual file submission portal.
- **EV certificate** (~$300–500/year) — immediate SmartScreen trust. Requires cloud HSM (Azure Key Vault recommended).

Environment variables:
```
AZURE_CLIENT_ID
AZURE_CLIENT_SECRET
AZURE_TENANT_ID
```

Recommend starting with an OV cert and moving to EV post-1.0 when budget allows.

---

## Scaffold

```bash
npm create tauri-app@latest retrace -- --template svelte-ts
cd retrace
cargo add vtracer anyhow --manifest-path src-tauri/Cargo.toml
```

Download the appropriate `uv` binaries for each target triple and place them in `src-tauri/binaries/`.

---

## Milestones

### Pre-1.0
1. **Core** — vtracer backend, `ImageTracer` trait, Tauri command surface, Svelte UI
2. **uv integration** — bundle uv binary per target triple, venv creation, sidecar.py resource
3. **Setup Tool** — GPU/disk detection, Python env installer, backend status panel, progress events
4. **LIVE backend** — sidecar integration, speed warning UX
5. **StarVector backend** — model download, Q4 option, 1B/8B toggle, VRAM gating
6. **Code signing** — macOS notarization, Windows OV cert, GitHub Actions release pipeline

### Post-1.0
- Custom tracing algorithm (`tracer/custom.rs`) — novel geometric approach, better curve fitting
- Remote backend — StarVector via hosted inference endpoint, user-supplied API key
- Additional export formats (PDF, EPS, DXF)
- Batch processing mode
- EV certificate for Windows

---

## Future Considerations

- **Custom algorithm** — drop into `tracer/custom.rs`, implement `ImageTracer`. No other changes required by design.
- **Remote StarVector** — HTTP `ImageTracer` backend; users supply a HuggingFace Inference Endpoint or Replicate API key.
- **Theming** — UI should avoid default/bland aesthetics from day one. Full CSS control via Svelte + Tauri.

---

## License

Mozilla Public License 2.0 (MPL 2.0)

Modifications to Re:Trace's own source files must remain open source. The project may be combined with proprietary code under MPL's file-level copyleft terms.

**Dependency license notes:**
- vtracer: MIT — no conflicts
- LIVE: MIT — no conflicts
- StarVector: Apache 2.0 — no conflicts
- uv: MIT/Apache 2.0 — no conflicts
- Python sidecar dependencies: mixed, all permissive

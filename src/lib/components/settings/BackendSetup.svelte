<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { backendStatuses, installProgress, refreshStatuses } from "../../stores/backends";
  import type { BackendStatus, GpuInfo, InstallProgress } from "../../types";

  let gpu: GpuInfo | null = null;
  let diskFreeBytes = 0;
  let pythonEnvInstalled = false;
  let loading = true;

  let statuses: BackendStatus[] = [];
  backendStatuses.subscribe((v) => (statuses = v));

  let progress: InstallProgress | null = null;
  installProgress.subscribe((v) => (progress = v));

  onMount(async () => {
    [gpu, pythonEnvInstalled] = await Promise.all([
      invoke<GpuInfo>("detect_gpu"),
      invoke<boolean>("get_python_env_installed"),
    ]);
    diskFreeBytes = await invoke<number>("get_disk_space", { path: "/" });
    await refreshStatuses();
    loading = false;
  });

  function statusFor(id: string): BackendStatus | undefined {
    return statuses.find((s) => s.id === id);
  }

  function isReady(id: string) {
    return statusFor(id)?.state.type === "ready";
  }

  function isInstalling(id: string) {
    return statusFor(id)?.state.type === "installing";
  }

  function progressFor(backendId: string): number {
    if (!progress) return 0;
    const s = progress.stage;
    if (s.kind === "model_weights" && s.value === backendId) {
      return progress.bytes_downloaded / progress.total_bytes;
    }
    return 0;
  }

  function pythonEnvProgress(): number {
    if (!progress || progress.stage.kind !== "python_env") return 0;
    return progress.bytes_downloaded / progress.total_bytes;
  }

  let installError: string | null = null;
  let installingEnv = false;

  async function installPythonEnv() {
    installingEnv = true;
    installError = null;
    try {
      await invoke("install_python_env");
      pythonEnvInstalled = await invoke<boolean>("get_python_env_installed");
      await refreshStatuses();
    } catch (e) {
      installError = String(e);
    } finally {
      installingEnv = false;
    }
  }

  async function uninstallPythonEnv() {
    installError = null;
    try {
      await invoke("uninstall_python_env");
      pythonEnvInstalled = false;
      await refreshStatuses();
    } catch (e) {
      installError = String(e);
    }
  }

  async function downloadModel(backend: string) {
    installError = null;
    try {
      await invoke("download_model", { backend });
      await refreshStatuses();
    } catch (e) {
      installError = String(e);
    }
  }

  async function uninstall(backend: string) {
    try {
      await invoke("uninstall_backend", { backend });
      await refreshStatuses();
    } catch (e) {
      installError = String(e);
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes >= 1e12) return `${(bytes / 1e12).toFixed(1)} TB`;
    if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(0)} GB`;
    if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(0)} MB`;
    return `${bytes} B`;
  }

  function vramOk(minGb: number): boolean {
    if (!gpu?.vram_mb) return gpu?.compute_backend === "metal";
    return gpu.vram_mb >= minGb * 1024;
  }
</script>

{#if loading}
  <div class="loading">Checking system…</div>
{:else}
  <!-- System info -->
  <section class="card">
    <h3 class="card-title">System</h3>
    <div class="sys-row">
      <span class="sys-label">GPU</span>
      {#if gpu}
        <span class="sys-val ok">
          ✓ {gpu.name}{gpu.vram_mb ? ` (${gpu.vram_mb} MB VRAM)` : ""}
          {gpu.compute_backend ? ` · ${gpu.compute_backend.toUpperCase()}` : ""}
        </span>
      {:else}
        <span class="sys-val muted">No GPU detected</span>
      {/if}
    </div>
    <div class="sys-row">
      <span class="sys-label">Available disk</span>
      <span class="sys-val">{formatBytes(diskFreeBytes)} free</span>
    </div>
  </section>

  <!-- Python Environment -->
  <section class="card">
    <div class="card-row">
      <div>
        <h3 class="card-title">Python Environment</h3>
        <p class="card-desc">~4 GB · Managed automatically · No Python needed</p>
      </div>
      {#if pythonEnvInstalled}
        <span class="badge-ok">Installed</span>
        <button class="btn-remove" on:click={uninstallPythonEnv}>Remove</button>
      {:else if installingEnv}
        <progress value={pythonEnvProgress()} max="1" class="prog"></progress>
        <span class="badge-busy">Installing…</span>
      {:else}
        <button class="btn-install" on:click={installPythonEnv}>Install</button>
      {/if}
    </div>
  </section>

  <!-- LIVE -->
  <section class="card">
    <div class="card-row">
      <div>
        <h3 class="card-title">LIVE</h3>
        <p class="card-desc">Best for artistic / illustrative images · No model download</p>
      </div>
      {#if isReady("live")}
        <span class="badge-ok">Ready</span>
        <button class="btn-remove" on:click={() => uninstall("live")}>Remove</button>
      {:else if isInstalling("live")}
        <progress value={progressFor("live")} max="1" class="prog"></progress>
        <button class="btn-cancel" on:click={() => invoke("cancel_download", { backend: "live" })}>
          Cancel
        </button>
      {:else}
        <button
          class="btn-install"
          disabled={!pythonEnvInstalled}
          on:click={() => downloadModel("live")}
        >
          Install
        </button>
      {/if}
    </div>
    {#if !pythonEnvInstalled}
      <p class="dep-note">Requires: Python Environment</p>
    {/if}
    <p class="speed-note">
      Note: LIVE is significantly slower than vtracer. Expect longer wait times.
    </p>
  </section>

  <!-- StarVector -->
  <section class="card">
    <div class="card-row">
      <div>
        <h3 class="card-title">StarVector</h3>
        <p class="card-desc">Best for icons, logos, diagrams · Not for photos</p>
      </div>
      {#if isReady("starvector-1b") || isReady("starvector-8b")}
        <span class="badge-ok">Ready</span>
      {:else}
        <span class="badge-no">Not Installed</span>
      {/if}
    </div>

    <div class="model-row">
      <span class="model-label">StarVector-1B</span>
      <span class="model-size">~2 GB</span>
      {#if isReady("starvector-1b")}
        <span class="badge-ok small">Installed</span>
        <button class="btn-remove" on:click={() => uninstall("starvector-1b")}>Remove</button>
      {:else if isInstalling("starvector-1b")}
        <progress value={progressFor("starvector-1b")} max="1" class="prog"></progress>
        <button
          class="btn-cancel"
          on:click={() => invoke("cancel_download", { backend: "starvector-1b" })}
        >Cancel</button>
      {:else}
        <button
          class="btn-install"
          disabled={!pythonEnvInstalled || !vramOk(4)}
          title={!vramOk(4) ? "Requires 4 GB VRAM" : ""}
          on:click={() => downloadModel("starvector-1b")}
        >
          Download
        </button>
      {/if}
    </div>

    <div class="model-row">
      <span class="model-label">StarVector-8B</span>
      <span class="model-size">~16 GB</span>
      {#if isReady("starvector-8b")}
        <span class="badge-ok small">Installed</span>
        <button class="btn-remove" on:click={() => uninstall("starvector-8b")}>Remove</button>
      {:else if isInstalling("starvector-8b")}
        <progress value={progressFor("starvector-8b")} max="1" class="prog"></progress>
        <button
          class="btn-cancel"
          on:click={() => invoke("cancel_download", { backend: "starvector-8b" })}
        >Cancel</button>
      {:else}
        <button
          class="btn-install"
          disabled={!pythonEnvInstalled || !vramOk(8)}
          title={!vramOk(8) ? "Requires 8 GB VRAM" : ""}
          on:click={() => downloadModel("starvector-8b")}
        >
          Download
        </button>
      {/if}
    </div>

    {#if !pythonEnvInstalled}
      <p class="dep-note">Requires: Python Environment + 4 GB VRAM (8B requires 8 GB VRAM)</p>
    {/if}
  </section>

  {#if installError}
    <p class="error-msg">{installError}</p>
  {/if}
{/if}

<style>
  .loading {
    color: var(--text-3);
    font-size: 13px;
    padding: 32px;
    text-align: center;
  }

  .card {
    background: var(--bg-3);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .card-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-1);
    margin: 0;
  }

  .card-desc {
    font-size: 12px;
    color: var(--text-3);
    margin: 2px 0 0;
  }

  .card-row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    justify-content: space-between;
  }

  .sys-row {
    display: flex;
    gap: 12px;
    font-size: 13px;
  }

  .sys-label {
    width: 120px;
    flex-shrink: 0;
    color: var(--text-3);
  }

  .sys-val { color: var(--text-1); }
  .sys-val.ok { color: #4ade80; }
  .sys-val.muted { color: var(--text-4); }

  .badge-ok {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 4px;
    background: rgba(74, 222, 128, 0.15);
    color: #4ade80;
    white-space: nowrap;
  }

  .badge-ok.small { align-self: center; }

  .badge-no {
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 4px;
    background: var(--bg-4);
    color: var(--text-4);
    white-space: nowrap;
  }

  .badge-busy {
    font-size: 11px;
    color: var(--accent);
  }

  .btn-install,
  .btn-remove,
  .btn-cancel {
    padding: 4px 12px;
    border-radius: 5px;
    font-size: 12px;
    cursor: pointer;
    border: 1px solid var(--border);
    background: var(--bg-4);
    color: var(--text-1);
    white-space: nowrap;
    transition: background 0.15s;
  }

  .btn-install { background: var(--accent); border-color: var(--accent); color: #fff; }
  .btn-install:hover:not(:disabled) { filter: brightness(1.1); }
  .btn-install:disabled { opacity: 0.4; cursor: default; }
  .btn-remove { background: transparent; color: var(--text-3); }
  .btn-remove:hover { color: var(--error); border-color: var(--error); }
  .btn-cancel { background: transparent; color: var(--text-3); }

  .model-row {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
  }

  .model-label { flex: 1; color: var(--text-2); }
  .model-size { color: var(--text-4); min-width: 48px; }

  .prog {
    flex: 1;
    height: 6px;
    border-radius: 3px;
    accent-color: var(--accent);
  }

  .dep-note {
    font-size: 11px;
    color: var(--text-4);
    margin: 0;
  }

  .speed-note {
    font-size: 11px;
    color: var(--text-4);
    margin: 0;
    font-style: italic;
  }

  .error-msg {
    font-size: 12px;
    color: var(--error);
    background: rgba(248, 113, 113, 0.08);
    border-radius: 6px;
    padding: 10px 14px;
  }
</style>

<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import { imagePath, svgOutput, traceOptions, runTrace } from "../stores/tracing";
  import { activeBackend, backendStatuses, setBackend } from "../stores/backends";
  import type { BackendId, BackendStatus } from "../types";

  export let onOpenSettings: () => void;

  type ExportFormat = "svg" | "eps" | "ai";

  const FORMAT_META: Record<ExportFormat, { label: string; ext: string; filter: string }> = {
    svg: { label: "SVG", ext: "svg", filter: "SVG" },
    eps: { label: "EPS", ext: "eps", filter: "EPS" },
    ai: { label: "AI", ext: "ai", filter: "Adobe Illustrator" },
  };

  let currentSvg: string | null = null;
  let currentPath: string | null = null;
  let currentOpts = $traceOptions;
  let exportFormat: ExportFormat = "svg";
  let showFormatMenu = false;
  let showBackendMenu = false;

  let statuses: BackendStatus[] = [];
  let backend: BackendId = "vtracer";

  svgOutput.subscribe((v) => (currentSvg = v));
  imagePath.subscribe((v) => (currentPath = v));
  traceOptions.subscribe((v) => (currentOpts = v));
  activeBackend.subscribe((v) => (backend = v));
  backendStatuses.subscribe((v) => (statuses = v));

  $: readyStatuses = statuses.filter((s) => s.state.type === "ready");
  $: hasStarVector1B = readyStatuses.some((s) => s.id === "starvector-1b");
  $: hasStarVector8B = readyStatuses.some((s) => s.id === "starvector-8b");
  $: showQualityToggle =
    (backend === "starvector-1b" || backend === "starvector-8b") &&
    hasStarVector1B &&
    hasStarVector8B;

  function backendLabel(id: BackendId): string {
    switch (id) {
      case "vtracer": return "Standard (vtracer)";
      case "live": return "LIVE";
      case "starvector-1b":
      case "starvector-8b": return "StarVector";
    }
  }

  async function openFile() {
    const selected = await open({
      multiple: false,
      filters: [
        { name: "Images", extensions: ["png", "jpg", "jpeg", "bmp", "gif", "tiff", "webp"] },
      ],
    });
    if (typeof selected === "string") {
      imagePath.set(selected);
      svgOutput.set(null);
    }
  }

  async function exportAs() {
    if (!currentSvg) return;
    const meta = FORMAT_META[exportFormat];
    const path = await save({
      filters: [{ name: meta.filter, extensions: [meta.ext] }],
      defaultPath: `output.${meta.ext}`,
    });
    if (!path) return;
    await invoke("export_vector", { svg: currentSvg, format: exportFormat, filePath: path });
  }

  async function trace() {
    if (!currentPath) return;
    await runTrace(currentPath, currentOpts);
  }

  function selectFormat(f: string) {
    exportFormat = f as ExportFormat;
    showFormatMenu = false;
  }

  async function switchBackend(id: BackendId) {
    showBackendMenu = false;
    if (id === backend) return;
    try {
      await setBackend(id);
      svgOutput.set(null);
    } catch (e) {
      console.error("Failed to switch backend:", e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      showFormatMenu = false;
      showBackendMenu = false;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<header class="toolbar">
  <span class="logo">Re:Trace</span>

  <div class="actions">
    <button class="btn" on:click={openFile}>Open Image</button>
    <button class="btn btn-primary" on:click={trace} disabled={!currentPath}>Trace</button>

    <div class="export-group" class:disabled={!currentSvg}>
      <button class="btn export-btn" on:click={exportAs} disabled={!currentSvg}>
        Export {FORMAT_META[exportFormat].label}
      </button>
      <button
        class="btn fmt-toggle"
        on:click={() => (showFormatMenu = !showFormatMenu)}
        disabled={!currentSvg}
        aria-label="Select export format"
      >▾</button>

      {#if showFormatMenu}
        <div class="fmt-menu">
          {#each Object.entries(FORMAT_META) as [fmt, meta]}
            <button
              class="fmt-item"
              class:active={exportFormat === fmt}
              on:click={() => selectFormat(fmt)}
            >
              {meta.label}
              <span class="fmt-ext">.{meta.ext}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <div class="right-group">
    <!-- Backend switcher -->
    <div class="backend-group">
      <button
        class="btn backend-btn"
        on:click={() => (showBackendMenu = !showBackendMenu)}
      >
        Backend: {backendLabel(backend)} ▾
      </button>

      {#if showBackendMenu}
        <div class="backend-menu">
          {#each readyStatuses as s}
            {#if s.id !== "starvector-8b" || !readyStatuses.some((x) => x.id === "starvector-1b")}
              <button
                class="backend-item"
                class:active={s.id === backend || (s.id === "starvector-1b" && backend === "starvector-8b")}
                on:click={() => switchBackend(s.id)}
              >
                {#if s.id === "starvector-1b" && hasStarVector8B}
                  StarVector
                {:else}
                  {backendLabel(s.id)}
                {/if}
              </button>
            {/if}
          {/each}
          <div class="menu-divider"></div>
          <button
            class="backend-item settings-item"
            on:click={() => { showBackendMenu = false; onOpenSettings(); }}
          >
            Get Enhanced Backends…
          </button>
        </div>
      {/if}
    </div>

    <!-- StarVector quality toggle -->
    {#if showQualityToggle}
      <div class="quality-group">
        <span class="quality-label">Quality:</span>
        <button
          class="quality-btn"
          class:active={backend === "starvector-1b"}
          on:click={() => switchBackend("starvector-1b")}
        >1B</button>
        <button
          class="quality-btn"
          class:active={backend === "starvector-8b"}
          on:click={() => switchBackend("starvector-8b")}
        >8B</button>
      </div>
    {/if}

    <!-- Settings button -->
    <button class="btn settings-gear" on:click={onOpenSettings} aria-label="Open settings">
      ⚙
    </button>
  </div>
</header>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 16px;
    height: 48px;
    background: var(--bg-2);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .logo {
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-1);
    margin-right: 8px;
  }

  .actions {
    display: flex;
    gap: 8px;
    flex: 1;
    align-items: center;
  }

  .right-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn {
    padding: 5px 14px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg-3);
    color: var(--text-1);
    font-size: 13px;
    cursor: pointer;
    transition: background 0.15s;
    white-space: nowrap;
  }

  .btn:hover:not(:disabled) { background: var(--bg-4); }
  .btn:disabled { opacity: 0.4; cursor: default; }

  .btn-primary {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }
  .btn-primary:hover:not(:disabled) { filter: brightness(1.1); background: var(--accent); }

  /* Export split button */
  .export-group {
    position: relative;
    display: flex;
  }

  .export-btn {
    border-radius: 6px 0 0 6px;
    border-right: none;
  }

  .fmt-toggle {
    padding: 5px 8px;
    border-radius: 0 6px 6px 0;
    font-size: 11px;
  }

  .fmt-menu {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    background: var(--bg-3);
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
    z-index: 100;
    min-width: 140px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  }

  .fmt-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 8px 12px;
    background: none;
    border: none;
    color: var(--text-1);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    gap: 16px;
  }

  .fmt-item:hover { background: var(--bg-4); }
  .fmt-item.active { color: var(--accent); }

  .fmt-ext {
    font-size: 11px;
    color: var(--text-4);
  }

  /* Backend switcher */
  .backend-group {
    position: relative;
  }

  .backend-btn {
    font-size: 12px;
    padding: 4px 10px;
    color: var(--text-2);
  }

  .backend-menu {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    background: var(--bg-3);
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
    z-index: 100;
    min-width: 200px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  }

  .backend-item {
    display: block;
    width: 100%;
    padding: 8px 14px;
    background: none;
    border: none;
    color: var(--text-1);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
  }

  .backend-item:hover { background: var(--bg-4); }
  .backend-item.active { color: var(--accent); }
  .settings-item { color: var(--text-3); font-size: 12px; }
  .settings-item:hover { color: var(--text-1); }

  .menu-divider {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }

  /* Quality toggle */
  .quality-group {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .quality-label {
    font-size: 12px;
    color: var(--text-3);
  }

  .quality-btn {
    padding: 3px 8px;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: var(--bg-3);
    color: var(--text-3);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }

  .quality-btn.active {
    background: var(--accent-dim);
    border-color: var(--accent);
    color: var(--accent);
  }

  .quality-btn:hover:not(.active) { background: var(--bg-4); color: var(--text-1); }

  /* Settings gear */
  .settings-gear {
    padding: 5px 9px;
    font-size: 14px;
    color: var(--text-3);
    border-color: transparent;
    background: transparent;
  }

  .settings-gear:hover { color: var(--text-1); background: var(--bg-3); border-color: var(--border); }
</style>

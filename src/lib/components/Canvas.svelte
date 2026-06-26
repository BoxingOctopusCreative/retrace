<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { imagePath, svgOutput, isTracing, traceError } from "../stores/tracing";

  let currentPath: string | null = null;
  let currentSvg: string | null = null;
  let tracing = false;
  let error: string | null = null;
  let imageUrl: string | null = null;
  let prevImageUrl: string | null = null;

  // SVG zoom / pan state
  let isFit = true;
  let svgZoom = 1.0;
  let svgNatW = 0;
  let svgNatH = 0;
  let svgImgEl: HTMLImageElement;
  let svgScrollEl: HTMLElement;

  // Saved scroll position to restore after a re-trace (src change resets scroll).
  let pendingScrollRestore: { x: number; y: number } | null = null;

  imagePath.subscribe((path) => {
    currentPath = path;
    if (prevImageUrl) { URL.revokeObjectURL(prevImageUrl); prevImageUrl = null; }
    imageUrl = null;
    if (!path) return;
    invoke<number[]>("load_image_bytes", { filePath: path })
      .then((bytes) => {
        const blob = new Blob([new Uint8Array(bytes)]);
        const url = URL.createObjectURL(blob);
        imageUrl = url;
        prevImageUrl = url;
      })
      .catch((e) => console.error("Failed to load image:", e));
  });

  svgOutput.subscribe((v) => {
    const isFirstResult = !currentSvg && v !== null;
    const isRetrace = currentSvg !== null && v !== null;
    if (isRetrace && svgScrollEl) {
      // Snapshot scroll position now, before the src change resets the container.
      pendingScrollRestore = { x: svgScrollEl.scrollLeft, y: svgScrollEl.scrollTop };
    }
    currentSvg = v;
    if (isFirstResult || v === null) {
      isFit = true;
      svgZoom = 1.0;
      svgNatW = 0;
      svgNatH = 0;
    }
  });
  isTracing.subscribe((v) => (tracing = v));
  traceError.subscribe((v) => (error = v));

  $: svgBlob = currentSvg
    ? URL.createObjectURL(new Blob([currentSvg], { type: "image/svg+xml" }))
    : null;

  function onSvgLoad() {
    if (svgImgEl) {
      svgNatW = svgImgEl.naturalWidth;
      svgNatH = svgImgEl.naturalHeight;
    }
    if (pendingScrollRestore && svgScrollEl) {
      svgScrollEl.scrollLeft = pendingScrollRestore.x;
      svgScrollEl.scrollTop  = pendingScrollRestore.y;
      pendingScrollRestore = null;
    }
  }

  function clamp(v: number, lo: number, hi: number) {
    return Math.max(lo, Math.min(hi, v));
  }

  function enterZoomMode(container: HTMLElement) {
    if (isFit && svgNatW && svgNatH) {
      // Start from the current visual size so zoom feels continuous
      svgZoom = clamp(
        Math.min(
          (container.clientWidth - 32) / svgNatW,
          (container.clientHeight - 32) / svgNatH,
        ),
        0.05, 20,
      );
      isFit = false;
    }
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    enterZoomMode(e.currentTarget as HTMLElement);
    // Normalize to pixels so trackpad (many small events) and mouse wheel (one big event) feel the same.
    const px = e.deltaMode === 1 ? e.deltaY * 20 : e.deltaMode === 2 ? e.deltaY * 300 : e.deltaY;
    svgZoom = clamp(svgZoom * Math.pow(0.999, px), 0.05, 20);
  }

  function zoomIn()  { enterZoomMode(svgImgEl?.parentElement!); svgZoom = clamp(svgZoom * 1.25, 0.05, 20); }
  function zoomOut() { enterZoomMode(svgImgEl?.parentElement!); svgZoom = clamp(svgZoom / 1.25, 0.05, 20); }
  function setFit()  { isFit = true; svgZoom = 1.0; }

  // ── Pan (hand tool) ──────────────────────────────────────────────────────
  let isPanning = false;
  let panStartX = 0;
  let panStartY = 0;
  let panScrollX = 0;
  let panScrollY = 0;

  function onPanStart(e: MouseEvent) {
    if (isFit || e.button !== 0) return;
    isPanning = true;
    panStartX = e.clientX;
    panStartY = e.clientY;
    const el = e.currentTarget as HTMLElement;
    panScrollX = el.scrollLeft;
    panScrollY = el.scrollTop;
    e.preventDefault();
  }

  function onPanMove(e: MouseEvent) {
    if (!isPanning) return;
    const el = e.currentTarget as HTMLElement;
    el.scrollLeft = panScrollX - (e.clientX - panStartX);
    el.scrollTop  = panScrollY - (e.clientY - panStartY);
  }

  function onPanEnd() {
    isPanning = false;
  }

  // ── Drag-and-drop ────────────────────────────────────────────────────────
  const RASTER_EXTS = new Set([".png", ".jpg", ".jpeg", ".bmp", ".gif", ".tiff", ".tif", ".webp"]);
  const isRaster = (p: string) => RASTER_EXTS.has(p.toLowerCase().slice(p.lastIndexOf(".")));

  let isDragOver = false;
  let unlistenDrag: (() => void) | null = null;

  onMount(async () => {
    unlistenDrag = await getCurrentWebview().onDragDropEvent((e) => {
      const p = e.payload;
      if (p.type === "enter") {
        isDragOver = p.paths.some(isRaster);
      } else if (p.type === "drop") {
        isDragOver = false;
        const path = p.paths.find(isRaster);
        if (path) { imagePath.set(path); svgOutput.set(null); }
      } else if (p.type === "leave") {
        isDragOver = false;
      }
    });
  });

  onDestroy(() => { unlistenDrag?.(); });

  $: zoomLabel = isFit ? "Fit" : `${Math.round(svgZoom * 100)}%`;
  $: svgImgStyle = isFit
    ? "max-width:100%;max-height:100%;object-fit:contain;"
    : `width:${svgNatW * svgZoom}px;height:${svgNatH * svgZoom}px;flex-shrink:0;display:block;`;
  $: paneCursor = isFit ? "default" : isPanning ? "grabbing" : "grab";
</script>

<div class="canvas-area">
  {#if isDragOver}
    <div class="drop-overlay">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M12 4v12M7 9l5-5 5 5" stroke="var(--accent)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M4 17v2a1 1 0 001 1h14a1 1 0 001-1v-2" stroke="var(--accent)" stroke-width="2" stroke-linecap="round"/>
      </svg>
      <span class="drop-label">Drop to open</span>
    </div>
  {/if}

  {#if !currentPath}
    <div class="empty">
      <div class="empty-icon">⬡</div>
      <p class="empty-text">Open an image to get started</p>
      <p class="empty-sub">PNG, JPEG, BMP, GIF, TIFF, WebP</p>
    </div>
  {:else}
    <div class="split" class:single={!currentSvg && !tracing && !error}>
      <!-- Original image pane -->
      <div class="pane">
        <div class="pane-header">
          <span class="pane-label">Original</span>
        </div>
        <div class="pane-content">
          <img src={imageUrl} alt="Input" class="preview-img" />
        </div>
      </div>

      <!-- Vector output pane -->
      {#if currentSvg}
        <div class="pane">
          <div class="pane-header">
            <span class="pane-label">Vector</span>
            <div class="zoom-bar">
              {#if tracing}
                <span class="updating-badge">Updating…</span>
              {/if}
              <button class="zoom-btn" on:click={zoomOut} title="Zoom out">−</button>
              <button class="zoom-btn zoom-label" on:click={setFit} title="Reset to fit">
                {zoomLabel}
              </button>
              <button class="zoom-btn" on:click={zoomIn} title="Zoom in">+</button>
            </div>
          </div>
          <!-- svelte-ignore a11y-no-static-element-interactions -->
          <div
            class="pane-content svg-scroll"
            class:svg-overflow={!isFit}
            class:updating={tracing}
            style="cursor:{paneCursor}"
            bind:this={svgScrollEl}
            on:wheel={handleWheel}
            on:mousedown={onPanStart}
            on:mousemove={onPanMove}
            on:mouseup={onPanEnd}
            on:mouseleave={onPanEnd}
          >
            <img
              src={svgBlob}
              alt="SVG output"
              bind:this={svgImgEl}
              on:load={onSvgLoad}
              style={svgImgStyle}
            />
          </div>
        </div>
      {:else if tracing}
        <div class="pane">
          <div class="pane-header"><span class="pane-label">Vector</span></div>
          <div class="pane-content loading-content">
            <div class="spinner" />
            <span class="loading-text">Tracing…</span>
          </div>
        </div>
      {:else if error}
        <div class="pane">
          <div class="pane-header"><span class="pane-label">Vector</span></div>
          <div class="pane-content error-content">
            <p class="error-msg">{error}</p>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .canvas-area {
    flex: 1;
    overflow: hidden;
    display: flex;
    background: var(--bg-1);
    position: relative;
  }

  .drop-overlay {
    position: absolute;
    inset: 0;
    z-index: 10;
    pointer-events: none;
    background: var(--accent-dim);
    border: 2px solid var(--accent);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
  }

  .drop-label {
    font-size: 14px;
    font-weight: 600;
    color: var(--accent);
    user-select: none;
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    user-select: none;
  }

  .empty-icon {
    font-size: 48px;
    opacity: 0.15;
    line-height: 1;
  }

  .empty-text { font-size: 15px; color: var(--text-3); margin: 0; }
  .empty-sub  { font-size: 12px; color: var(--text-4); margin: 0; }

  .split {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1px;
    flex: 1;
    background: var(--border);
  }

  .split.single { grid-template-columns: 1fr; }

  .pane {
    display: flex;
    flex-direction: column;
    background: var(--bg-1);
    overflow: hidden;
    min-width: 0;
  }

  /* ── Pane header ── */
  .pane-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    height: 32px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .pane-label {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-4);
  }

  /* ── Zoom controls ── */
  .zoom-bar {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .zoom-btn {
    padding: 2px 6px;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: var(--bg-3);
    color: var(--text-2);
    font-size: 12px;
    line-height: 1.4;
    cursor: pointer;
    transition: background 0.1s;
  }

  .zoom-btn:hover { background: var(--bg-4); color: var(--text-1); }

  .zoom-label {
    min-width: 40px;
    text-align: center;
    font-variant-numeric: tabular-nums;
    font-size: 11px;
  }

  /* ── Pane content ── */
  .pane-content {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
    overflow: hidden;
  }

  .svg-overflow {
    overflow: auto;
    align-items: flex-start;
    justify-content: flex-start;
    cursor: default;
  }

  .preview-img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }

  .updating-badge {
    font-size: 11px;
    color: var(--accent);
    opacity: 0.8;
  }

  .updating img {
    opacity: 0.5;
    transition: opacity 0.15s;
  }

  .loading-content {
    flex-direction: column;
    gap: 12px;
    color: var(--text-3);
  }

  .spinner {
    width: 28px;
    height: 28px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .loading-text { font-size: 13px; }

  .error-content { padding: 24px; }
  .error-msg {
    font-size: 12px;
    color: var(--error);
    font-family: monospace;
    white-space: pre-wrap;
    word-break: break-all;
  }

  @keyframes spin { to { transform: rotate(360deg); } }
</style>

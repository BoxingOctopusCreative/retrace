<script lang="ts">
  import { activeBackend } from "../stores/backends";
  import { traceOptions } from "../stores/tracing";

  let opts = $traceOptions;
  let backend = $activeBackend;

  traceOptions.subscribe((v) => (opts = { ...v }));
  activeBackend.subscribe((v) => (backend = v));

  function update() {
    traceOptions.set({ ...opts });
  }
</script>

<aside class="controls">
  {#if backend === "vtracer"}
    <h2 class="section-title">Trace Options</h2>

    <div class="field">
      <label for="color-precision">
        Color Precision
        <span class="value">{opts.color_precision}</span>
      </label>
      <input
        id="color-precision"
        type="range"
        min="1"
        max="8"
        step="1"
        bind:value={opts.color_precision}
        on:input={update}
      />
      <div class="hint-row"><span>fewer</span><span>more</span></div>
    </div>

    <div class="field">
      <label for="filter-speckle">
        Filter Speckle
        <span class="value">{opts.filter_speckle}px</span>
      </label>
      <input
        id="filter-speckle"
        type="range"
        min="1"
        max="100"
        step="1"
        bind:value={opts.filter_speckle}
        on:input={update}
      />
      <div class="hint-row"><span>fine</span><span>coarse</span></div>
    </div>

    <div class="field">
      <label for="corner-threshold">
        Corner Threshold
        <span class="value">{opts.corner_threshold}°</span>
      </label>
      <input
        id="corner-threshold"
        type="range"
        min="0"
        max="180"
        step="1"
        bind:value={opts.corner_threshold}
        on:input={update}
      />
      <div class="hint-row"><span>sharp</span><span>smooth</span></div>
    </div>
  {:else}
    <div class="no-opts">
      <p class="no-opts-label">No options</p>
      <p class="no-opts-desc">
        {backend === "live"
          ? "LIVE uses its own internal parameters."
          : "StarVector parameters are set via the quality toggle in the toolbar."}
      </p>
    </div>
  {/if}
</aside>

<style>
  .controls {
    width: 220px;
    flex-shrink: 0;
    padding: 16px;
    background: var(--bg-2);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .section-title {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-3);
    margin: 0;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  label {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
    color: var(--text-2);
  }

  .value {
    font-variant-numeric: tabular-nums;
    color: var(--text-1);
    font-weight: 600;
  }

  input[type="range"] {
    width: 100%;
    accent-color: var(--accent);
  }

  .hint-row {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: var(--text-4);
  }

  .no-opts {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 4px;
  }

  .no-opts-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-3);
    margin: 0;
  }

  .no-opts-desc {
    font-size: 12px;
    color: var(--text-4);
    line-height: 1.5;
    margin: 0;
  }
</style>

<script lang="ts">
  import { slide } from "svelte/transition";
  import { activeBackend } from "../stores/backends";
  import { traceOptions } from "../stores/tracing";

  let backend = $activeBackend;
  let opts = $traceOptions;
  let collapsed = true;

  activeBackend.subscribe((v) => (backend = v));
  traceOptions.subscribe((v) => (opts = { ...v }));

  function update() {
    traceOptions.set({ ...opts });
  }
</script>

{#if backend === "vtracer"}
  <div class="ribbon">
    <div class="handle">
      <button
        class="toggle"
        on:click={() => (collapsed = !collapsed)}
        aria-expanded={!collapsed}
        aria-label={collapsed ? "Show trace options" : "Hide trace options"}
      >
        <svg width="14" height="10" viewBox="0 0 14 10" aria-hidden="true">
          <line x1="0" y1="1"   x2="14" y2="1"   stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          <line x1="0" y1="5"   x2="14" y2="5"   stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          <line x1="0" y1="9"   x2="14" y2="9"   stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
    </div>

    {#if !collapsed}
      <div class="body" transition:slide={{ duration: 180 }}>
        <div class="fields">
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
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .ribbon {
    flex-shrink: 0;
    border-top: 1px solid var(--border);
    background: var(--bg-2);
  }

  /* ── Collapsed handle ── */
  .handle {
    height: 32px;
    display: flex;
    align-items: center;
  }

  .toggle {
    width: 44px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-right: 1px solid var(--border);
    color: var(--text-3);
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
    flex-shrink: 0;
  }

  .toggle:hover {
    background: var(--bg-3);
    color: var(--text-1);
  }

  /* ── Expanded body ── */
  .body {
    border-top: 1px solid var(--border);
    padding: 10px 16px 12px;
    overflow: hidden;
  }

  .fields {
    display: flex;
    gap: 0;
  }

  .field {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 0 20px;
    border-right: 1px solid var(--border);
  }

  .field:first-child { padding-left: 0; }
  .field:last-child  { border-right: none; padding-right: 0; }

  label {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
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
</style>

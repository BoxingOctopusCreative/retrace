<script lang="ts">
  import { preferences } from "../../stores/preferences";
  import { pendingUpdate, isChecking, checkError, checkForUpdate } from "../../stores/updater";

  let checkDone = false;

  async function handleCheckNow() {
    checkDone = false;
    await checkForUpdate();
    checkDone = true;
  }
</script>

<div class="section">
  <h2 class="section-title">Updates</h2>

  <div class="row">
    <div class="row-info">
      <span class="row-label">Check for updates automatically</span>
      <span class="row-desc">Re:Trace checks for new releases on launch</span>
    </div>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <label class="toggle" on:click|stopPropagation>
      <input
        type="checkbox"
        checked={$preferences.autoCheckUpdates}
        on:change={(e) =>
          preferences.update((p) => ({
            ...p,
            autoCheckUpdates: (e.target as HTMLInputElement).checked,
          }))}
      />
      <span class="track" />
    </label>
  </div>

  <div class="check-row">
    <button class="check-btn" disabled={$isChecking} on:click={handleCheckNow}>
      {$isChecking ? "Checking…" : "Check Now"}
    </button>
    {#if $pendingUpdate}
      <span class="result update">Re:Trace {$pendingUpdate.version} is available</span>
    {:else if checkDone && !$isChecking && !$checkError}
      <span class="result ok">Re:Trace is up to date.</span>
    {:else if $checkError}
      <span class="result error">{$checkError}</span>
    {/if}
  </div>
</div>

<style>
  .section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .section-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-4);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    margin: 0;
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .row-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .row-label {
    font-size: 13px;
    color: var(--text-1);
  }

  .row-desc {
    font-size: 11px;
    color: var(--text-4);
  }

  .toggle {
    position: relative;
    display: inline-flex;
    align-items: center;
    cursor: pointer;
    flex-shrink: 0;
  }

  .toggle input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .track {
    width: 36px;
    height: 20px;
    border-radius: 10px;
    background: var(--bg-4);
    border: 1px solid var(--border);
    transition: background 0.15s;
    position: relative;
  }

  .track::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--text-3);
    transition: transform 0.15s, background 0.15s;
  }

  .toggle input:checked + .track {
    background: var(--accent);
    border-color: transparent;
  }

  .toggle input:checked + .track::after {
    transform: translateX(16px);
    background: #fff;
  }

  .check-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .check-btn {
    padding: 6px 14px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg-3);
    color: var(--text-2);
    font-size: 12px;
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
  }

  .check-btn:hover:not(:disabled) {
    background: var(--bg-4);
    color: var(--text-1);
  }

  .check-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .result {
    font-size: 12px;
  }

  .result.ok {
    color: var(--text-3);
  }

  .result.update {
    color: var(--accent);
    font-weight: 500;
  }

  .result.error {
    color: var(--error);
  }
</style>

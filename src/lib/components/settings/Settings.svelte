<script lang="ts">
  import BackendSetup from "./BackendSetup.svelte";

  export let onClose: () => void;
</script>

<!-- svelte-ignore a11y-no-static-element-interactions a11y-click-events-have-key-events -->
<div class="overlay" on:click|self={onClose}>
  <div class="panel">
    <div class="panel-header">
      <div class="header-left">
        <h1 class="panel-title">Settings</h1>
        <span class="panel-section">Enhanced Backends</span>
      </div>
      <button class="close-btn" on:click={onClose} aria-label="Close settings">✕</button>
    </div>
    <div class="panel-body">
      <BackendSetup />
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .panel {
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 10px;
    width: 540px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }

  .panel-title {
    font-size: 16px;
    font-weight: 700;
    color: var(--text-1);
    margin: 0;
  }

  .panel-section {
    font-size: 12px;
    color: var(--text-4);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-4);
    font-size: 14px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    line-height: 1;
    transition: color 0.15s;
  }

  .close-btn:hover {
    color: var(--text-1);
    background: var(--bg-4);
  }

  .panel-body {
    padding: 16px 20px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>

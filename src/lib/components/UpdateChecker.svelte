<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { preferences } from "../stores/preferences";
  import { pendingUpdate, isChecking, checkError, checkForUpdate } from "../stores/updater";

  let installing = false;
  let installError: string | null = null;

  async function install() {
    installing = true;
    installError = null;
    try {
      await invoke("install_update");
      // app restarts automatically after a successful install
    } catch (e) {
      installError = String(e);
      installing = false;
    }
  }

  onMount(async () => {
    const unlisten = await listen("menu:check-updates", () => checkForUpdate());
    if ($preferences.autoCheckUpdates) checkForUpdate();
    return unlisten;
  });
</script>

{#if $isChecking || $pendingUpdate || $checkError}
  <div class="overlay" role="dialog" aria-modal="true" aria-label="Software Update">
    <div class="card">
      {#if $isChecking}
        <p class="status">Checking for updates…</p>
      {:else if $checkError}
        <h3 class="title">Update check failed</h3>
        <p class="body-text error">{$checkError}</p>
        <div class="actions">
          <button class="btn" on:click={() => checkError.set(null)}>Dismiss</button>
        </div>
      {:else if $pendingUpdate}
        <h3 class="title">Update Available</h3>
        <p class="version">Re:Trace {$pendingUpdate.version} is ready to install.</p>
        {#if $pendingUpdate.body}
          <div class="notes">{$pendingUpdate.body}</div>
        {/if}
        {#if installError}
          <p class="error">{installError}</p>
        {/if}
        {#if installing}
          <p class="status">Downloading and installing…</p>
        {:else}
          <div class="actions">
            <button class="btn primary" on:click={install}>Install &amp; Restart</button>
            <button class="btn" on:click={() => pendingUpdate.set(null)}>Later</button>
          </div>
        {/if}
      {/if}
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 3000;
  }

  .card {
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 28px 32px;
    width: 380px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.7);
  }

  .title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-1);
    margin: 0;
  }

  .version {
    font-size: 13px;
    color: var(--text-2);
    margin: 0;
  }

  .notes {
    font-size: 12px;
    color: var(--text-3);
    background: var(--bg-3);
    border-radius: 6px;
    padding: 10px 12px;
    max-height: 160px;
    overflow-y: auto;
    white-space: pre-wrap;
    line-height: 1.5;
  }

  .status {
    font-size: 13px;
    color: var(--text-3);
    margin: 0;
  }

  .error {
    font-size: 12px;
    color: var(--error);
    margin: 0;
  }

  .body-text {
    font-size: 13px;
    color: var(--text-2);
    margin: 0;
  }

  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 4px;
  }

  .btn {
    padding: 7px 14px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg-3);
    color: var(--text-2);
    font-size: 13px;
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
  }

  .btn:hover {
    background: var(--bg-4);
    color: var(--text-1);
  }

  .btn.primary {
    background: var(--accent);
    border-color: transparent;
    color: #fff;
    font-weight: 500;
  }

  .btn.primary:hover {
    filter: brightness(1.1);
  }
</style>

<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  const isMacOS = navigator.userAgent.includes("Mac OS");

  async function minimize() {
    await getCurrentWindow().minimize();
  }

  async function toggleMaximize() {
    const win = getCurrentWindow();
    await (await win.isMaximized() ? win.unmaximize() : win.maximize());
  }

  async function closeWindow() {
    await getCurrentWindow().close();
  }
</script>

<div class="titlebar" data-tauri-drag-region>
  <!-- Absolutely centered so it stays centred regardless of traffic-light / button widths -->
  <div class="center">
    <svg
      width="16"
      height="16"
      viewBox="0 0 48 48"
      fill="none"
      aria-hidden="true"
      class="icon"
    >
      <circle cx="10" cy="38" r="5" fill="var(--accent)" opacity="0.9" />
      <circle cx="38" cy="10" r="5" fill="var(--accent)" opacity="0.9" />
      <circle cx="36" cy="36" r="3.5" stroke="var(--accent)" stroke-width="1.5" fill="none" opacity="0.6" />
      <path d="M10 38 C 12 20 22 10 38 10" stroke="var(--accent)" stroke-width="2" fill="none" stroke-linecap="round" />
      <line x1="36" y1="36" x2="38" y2="10" stroke="var(--border)" stroke-width="1.5" stroke-dasharray="3 3" />
    </svg>
    <span class="app-name">Re:Trace</span>
  </div>

  {#if !isMacOS}
    <!-- Stop mousedown from bubbling into the drag region so buttons stay clickable -->
    <div class="controls" on:mousedown|stopPropagation>
      <button class="ctrl" on:click={minimize} aria-label="Minimize">
        <svg width="10" height="1" viewBox="0 0 10 1" aria-hidden="true">
          <line x1="0" y1=".5" x2="10" y2=".5" stroke="currentColor" />
        </svg>
      </button>
      <button class="ctrl" on:click={toggleMaximize} aria-label="Maximize">
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <rect x=".5" y=".5" width="9" height="9" rx="0.5" stroke="currentColor" fill="none" />
        </svg>
      </button>
      <button class="ctrl close" on:click={closeWindow} aria-label="Close">
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <line x1="1" y1="1" x2="9" y2="9" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" />
          <line x1="9" y1="1" x2="1" y2="9" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  {/if}
</div>

<style>
  .titlebar {
    height: 36px;
    background: var(--bg-2);
    position: relative;
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  /* Icon + title centred in the full window width */
  .center {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 6px;
    pointer-events: none; /* let drag-region events pass through */
    user-select: none;
  }

  .icon {
    flex-shrink: 0;
  }

  .app-name {
    font-family: var(--font-heading);
    font-size: 12px;
    font-weight: 600;
    color: var(--text-3);
    letter-spacing: -0.01em;
  }

  /* Windows / Linux window controls — right-aligned */
  .controls {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    display: flex;
  }

  .ctrl {
    width: 46px;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: var(--text-3);
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }

  .ctrl:hover {
    background: rgba(255, 255, 255, 0.07);
    color: var(--text-1);
  }

  .ctrl.close:hover {
    background: #c42b1c;
    color: #fff;
  }
</style>

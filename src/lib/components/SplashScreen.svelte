<script lang="ts">
  import { onMount } from "svelte";

  export let onclose: (() => void) | undefined = undefined;

  let visible = false;

  onMount(() => {
    requestAnimationFrame(() => {
      visible = true;
    });
  });

  function dismiss() {
    visible = false;
    setTimeout(() => onclose?.(), 220);
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === "Escape") dismiss();
  }
</script>

<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
<div
  class="overlay"
  class:visible
  on:click={dismiss}
  on:keydown={handleKey}
  role="dialog"
  aria-modal="true"
  aria-label="About Re:Trace"
  tabindex="-1"
>
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="card" on:click|stopPropagation on:keydown|stopPropagation>
    <div class="hero">
      <svg class="icon" width="48" height="48" viewBox="0 0 48 48" fill="none" aria-hidden="true">
        <circle cx="10" cy="38" r="5" fill="var(--accent)" opacity="0.9" />
        <circle cx="38" cy="10" r="5" fill="var(--accent)" opacity="0.9" />
        <circle cx="36" cy="36" r="3.5" stroke="var(--accent)" stroke-width="1.5" fill="none" opacity="0.6" />
        <path d="M10 38 C 12 20 22 10 38 10" stroke="var(--accent)" stroke-width="2" fill="none" stroke-linecap="round" />
        <line x1="36" y1="36" x2="38" y2="10" stroke="var(--border)" stroke-width="1.5" stroke-dasharray="3 3" />
      </svg>
      <span class="wordmark">Re:Trace</span>
      <span class="tagline">Raster to vector, beautifully.</span>
    </div>

    <hr class="rule" />

    <dl class="meta">
      <div class="row"><dt>Version</dt><dd>0.1.0</dd></div>
      <div class="row"><dt>Studio</dt><dd>Boxing Octopus Creative</dd></div>
      <div class="row"><dt>License</dt><dd>MPL 2.0</dd></div>
    </dl>

    <button class="close-btn" on:click={dismiss}>Close</button>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.72);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
    opacity: 0;
    transition: opacity 0.2s ease;
  }

  .overlay.visible {
    opacity: 1;
  }

  .card {
    background: var(--bg-2);
    border: 1px solid var(--border);
    border-radius: 14px;
    padding: 44px 48px 36px;
    width: 360px;
    display: flex;
    flex-direction: column;
    align-items: center;
    box-shadow: 0 32px 80px rgba(0, 0, 0, 0.7);
    transform: scale(0.96);
    transition: transform 0.2s ease;
  }

  .overlay.visible .card {
    transform: scale(1);
  }

  .hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    text-align: center;
  }

  .icon {
    margin-bottom: 2px;
    flex-shrink: 0;
  }

  .wordmark {
    font-size: 40px;
    font-weight: 800;
    letter-spacing: -0.04em;
    color: var(--accent);
    line-height: 1;
  }

  .tagline {
    font-size: 13px;
    color: var(--text-3);
    letter-spacing: 0.01em;
  }

  .rule {
    width: 100%;
    border: none;
    border-top: 1px solid var(--border);
    margin: 28px 0 22px;
  }

  .meta {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-bottom: 32px;
  }

  .row {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
  }

  dt {
    color: var(--text-4);
  }

  dd {
    color: var(--text-2);
    font-weight: 500;
    margin: 0;
  }

  .close-btn {
    width: 100%;
    padding: 9px;
    border-radius: 7px;
    border: 1px solid var(--border);
    background: var(--bg-3);
    color: var(--text-2);
    font-size: 13px;
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
  }

  .close-btn:hover {
    background: var(--bg-4);
    color: var(--text-1);
  }
</style>

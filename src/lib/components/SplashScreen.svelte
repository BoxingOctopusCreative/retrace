<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";

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
      <svg class="icon" width="72" height="72" viewBox="0 0 48 48" fill="none" aria-hidden="true">
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

    <div class="meta">
      <p class="meta-line">
        Version 0.4.0 from <!-- svelte-ignore a11y-invalid-attribute -->
        <a href="#" class="link" on:click|preventDefault={() => openUrl("https://boxingoctop.us")}>BOC Engineering</a>
      </p>
      <p class="meta-line">
        Licensed under the <!-- svelte-ignore a11y-invalid-attribute -->
        <a href="#" class="link" on:click|preventDefault={() => openUrl("https://www.mozilla.org/en-US/MPL/2.0/")}>Mozilla Public License 2.0</a>
      </p>
    </div>

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
    background: #000;
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 52px 56px 44px;
    width: 460px;
    display: flex;
    flex-direction: column;
    align-items: center;
    box-shadow: 0 40px 100px rgba(0, 0, 0, 0.9);
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
    font-family: var(--font-heading);
    font-size: 52px;
    font-weight: 800;
    letter-spacing: -0.04em;
    color: var(--accent);
    line-height: 1;
  }

  .tagline {
    font-size: 14px;
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
    gap: 6px;
    margin-bottom: 32px;
    text-align: center;
  }

  .meta-line {
    font-size: 13px;
    color: var(--text-4);
    margin: 0;
    line-height: 1.5;
  }

  .link {
    color: var(--text-3);
    text-decoration: underline;
    text-underline-offset: 2px;
    cursor: pointer;
    transition: color 0.1s;
  }

  .link:hover {
    color: var(--text-1);
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

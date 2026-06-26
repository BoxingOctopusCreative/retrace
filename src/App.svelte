<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Canvas from "./lib/components/Canvas.svelte";
  import Controls from "./lib/components/Controls.svelte";
  import Settings from "./lib/components/settings/Settings.svelte";
  import SplashScreen from "./lib/components/SplashScreen.svelte";
  import Titlebar from "./lib/components/Titlebar.svelte";
  import Toolbar from "./lib/components/Toolbar.svelte";

  let showSettings = false;
  let showSplash = true;

  onMount(() => {
    listen("menu:about", () => {
      showSplash = true;
    });
  });
</script>

<div class="app">
  <Titlebar />
  <Toolbar onOpenSettings={() => (showSettings = true)} />
  <main class="workspace">
    <Canvas />
    <Controls />
  </main>
</div>

{#if showSettings}
  <Settings onClose={() => (showSettings = false)} />
{/if}

{#if showSplash}
  <SplashScreen onclose={() => (showSplash = false)} />
{/if}

<style>
  @font-face {
    font-family: 'Elms Sans';
    font-style: normal;
    font-weight: 300 700;
    font-display: swap;
    src: url('/fonts/elms-sans-latin.woff2') format('woff2');
  }

  @font-face {
    font-family: 'Elms Sans';
    font-style: italic;
    font-weight: 300 700;
    font-display: swap;
    src: url('/fonts/elms-sans-latin-italic.woff2') format('woff2');
  }

  @font-face {
    font-family: 'Quicksand';
    font-style: normal;
    font-weight: 300 700;
    font-display: swap;
    src: url('/fonts/quicksand-latin.woff2') format('woff2');
  }

  :global(*) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(:root) {
    --font-body: 'Quicksand', -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif;
    --font-heading: 'Elms Sans', -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif;
    --bg-1: #111113;
    --bg-2: #18181b;
    --bg-3: #222226;
    --bg-4: #2e2e34;
    --border: #2e2e34;
    --text-1: #ededef;
    --text-2: #b4b4bb;
    --text-3: #7e7e8a;
    --text-4: #52525e;
    --accent: #7c6af7;
    --accent-dim: rgba(124, 106, 247, 0.15);
    --error: #f87171;
    color-scheme: dark;
  }

  :global(body) {
    background: var(--bg-1);
    color: var(--text-1);
    font-family: var(--font-body);
    height: 100vh;
    overflow: hidden;
  }

  :global(h1, h2, h3, h4, h5, h6) {
    font-family: var(--font-heading);
  }

  :global(#app) {
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .workspace {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }
</style>

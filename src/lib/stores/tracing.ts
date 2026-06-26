import { invoke } from "@tauri-apps/api/core";
import { get, writable } from "svelte/store";
import type { TraceOptions } from "../types";
import { defaultTraceOptions } from "../types";
import { activeBackend } from "./backends";

export const imagePath = writable<string | null>(null);
export const svgOutput = writable<string | null>(null);
export const isTracing = writable(false);
export const traceError = writable<string | null>(null);
export const traceOptions = writable<TraceOptions>({ ...defaultTraceOptions });

let pendingTrace: { path: string; opts: TraceOptions } | null = null;

export async function runTrace(path: string, opts: TraceOptions): Promise<void> {
  if (get(isTracing)) {
    pendingTrace = { path, opts };
    return;
  }
  isTracing.set(true);
  traceError.set(null);
  try {
    const svg = await invoke<string>("trace_image", { filePath: path, opts });
    svgOutput.set(svg);
  } catch (e) {
    traceError.set(String(e));
    svgOutput.set(null);
  } finally {
    isTracing.set(false);
    if (pendingTrace) {
      const next = pendingTrace;
      pendingTrace = null;
      runTrace(next.path, next.opts);
    }
  }
}

// Live preview: only auto-retrace when using vtracer (ML backends are slow).
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

traceOptions.subscribe(() => {
  const path = get(imagePath);
  if (!path) return;
  if (get(activeBackend) !== "vtracer") return;
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => runTrace(path, get(traceOptions)), 500);
});

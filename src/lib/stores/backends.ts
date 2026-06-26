import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import type { BackendId, BackendStatus, InstallProgress } from "../types";

export const backendStatuses = writable<BackendStatus[]>([]);
export const activeBackend = writable<BackendId>("vtracer");
export const installProgress = writable<InstallProgress | null>(null);

export async function refreshStatuses(): Promise<void> {
  const statuses = await invoke<BackendStatus[]>("get_backend_statuses");
  backendStatuses.set(statuses);
}

export async function setBackend(backend: BackendId): Promise<void> {
  await invoke("set_backend", { backend });
  activeBackend.set(backend);
}

listen<InstallProgress>("install:progress", ({ payload }) => {
  installProgress.set(payload);
  refreshStatuses();
}).catch(console.error);

refreshStatuses();

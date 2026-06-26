import { invoke } from "@tauri-apps/api/core";
import { writable } from "svelte/store";

export interface UpdateInfo {
  version: string;
  body: string | null;
}

export const pendingUpdate = writable<UpdateInfo | null>(null);
export const isChecking = writable(false);
export const checkError = writable<string | null>(null);

export async function checkForUpdate(): Promise<void> {
  isChecking.set(true);
  checkError.set(null);
  try {
    const update = await invoke<UpdateInfo | null>("check_for_update");
    pendingUpdate.set(update);
  } catch (e) {
    checkError.set(String(e));
  } finally {
    isChecking.set(false);
  }
}

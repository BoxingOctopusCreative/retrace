import { writable } from "svelte/store";

const KEY = "retrace:prefs";

interface Preferences {
  autoCheckUpdates: boolean;
}

const DEFAULTS: Preferences = { autoCheckUpdates: true };

function load(): Preferences {
  try {
    const raw = localStorage.getItem(KEY);
    if (raw) return { ...DEFAULTS, ...JSON.parse(raw) };
  } catch {}
  return { ...DEFAULTS };
}

function createStore() {
  const { subscribe, set, update } = writable<Preferences>(load());
  return {
    subscribe,
    set(v: Preferences) {
      localStorage.setItem(KEY, JSON.stringify(v));
      set(v);
    },
    update(fn: (p: Preferences) => Preferences) {
      update((p) => {
        const next = fn(p);
        localStorage.setItem(KEY, JSON.stringify(next));
        return next;
      });
    },
  };
}

export const preferences = createStore();

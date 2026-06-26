import { vi, describe, it, expect, beforeEach } from "vitest";
import { get } from "svelte/store";

// Mocks must be declared before the module under test is imported.
// vi.mock is hoisted so the mock factory runs before any imports.
const mockInvoke = vi.fn();
const mockListen = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mockListen }));

// Default: listen resolves (doesn't crash), invoke returns empty array for statuses
mockListen.mockResolvedValue(() => {});
mockInvoke.mockResolvedValue([]);

// Import AFTER mocks are in place
const { backendStatuses, activeBackend, installProgress, refreshStatuses, setBackend } =
  await import("./backends");

import type { BackendStatus } from "../types";

beforeEach(() => {
  vi.clearAllMocks();
  mockListen.mockResolvedValue(() => {});
  mockInvoke.mockResolvedValue([]);
});

const MOCK_STATUSES: BackendStatus[] = [
  { id: "vtracer", state: { type: "ready" } },
  { id: "live", state: { type: "not_installed" } },
];

describe("refreshStatuses", () => {
  it("invokes get_backend_statuses and updates the store", async () => {
    mockInvoke.mockResolvedValueOnce(MOCK_STATUSES);

    await refreshStatuses();

    expect(mockInvoke).toHaveBeenCalledWith("get_backend_statuses");
    expect(get(backendStatuses)).toEqual(MOCK_STATUSES);
  });

  it("replaces existing statuses on each refresh", async () => {
    mockInvoke.mockResolvedValueOnce(MOCK_STATUSES);
    await refreshStatuses();

    const updated: BackendStatus[] = [{ id: "vtracer", state: { type: "ready" } }];
    mockInvoke.mockResolvedValueOnce(updated);
    await refreshStatuses();

    expect(get(backendStatuses)).toEqual(updated);
  });
});

describe("setBackend", () => {
  it("invokes set_backend with the correct backend id", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await setBackend("vtracer");

    expect(mockInvoke).toHaveBeenCalledWith("set_backend", { backend: "vtracer" });
  });

  it("updates activeBackend store after a successful switch", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await setBackend("live");

    expect(get(activeBackend)).toBe("live");
  });
});

describe("installProgress store", () => {
  it("starts as null", () => {
    expect(get(installProgress)).toBeNull();
  });
});

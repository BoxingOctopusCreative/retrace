import { vi, describe, it, expect, beforeEach } from "vitest";
import { render, fireEvent, act } from "@testing-library/svelte";
import type { BackendStatus } from "../../types";

// vi.hoisted creates values before vi.mock hoisting runs, so they're accessible
// inside the factory closures without TDZ issues.
const mockInvoke = vi.hoisted(() => vi.fn());
const mockListen = vi.hoisted(() => vi.fn().mockResolvedValue(() => {}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mockListen }));

import BackendSetup from "./BackendSetup.svelte";

const STATUSES: BackendStatus[] = [
  { id: "vtracer", state: { type: "ready" } },
  { id: "live", state: { type: "not_installed" } },
  { id: "starvector-1b", state: { type: "not_installed" } },
  { id: "starvector-8b", state: { type: "not_installed" } },
];

const GPU_METAL = {
  name: "Apple M2",
  vram_mb: null,
  compute_backend: "metal",
};

const GPU_NVIDIA = {
  name: "NVIDIA RTX 4080",
  vram_mb: 16384,
  compute_backend: "cuda",
};

function setupInvoke(overrides: Record<string, unknown> = {}) {
  const defaults: Record<string, unknown> = {
    detect_gpu: GPU_METAL,
    get_python_env_installed: false,
    get_disk_space: 100_000_000_000,
    get_backend_statuses: STATUSES,
    ...overrides,
  };
  mockInvoke.mockImplementation((cmd: string) =>
    Promise.resolve(defaults[cmd] ?? undefined)
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  mockListen.mockResolvedValue(() => {});
  setupInvoke();
});

// Render the component and wait for async onMount to complete.
// act() calls the callback, then calls Svelte.tick() to flush reactive updates.
// Using setTimeout(50) inside gives all pending microtasks time to resolve.
async function renderLoaded(overrides: Record<string, unknown> = {}) {
  if (Object.keys(overrides).length > 0) setupInvoke(overrides);
  const result = render(BackendSetup);
  await act(async () => {
    await new Promise<void>((r) => setTimeout(r, 50));
  });
  return result;
}

describe("loading state", () => {
  it("shows loading spinner initially (synchronous check before onMount)", () => {
    const { container } = render(BackendSetup);
    expect(container.querySelector(".loading")).toBeTruthy();
  });

  it("hides loading spinner after onMount completes", async () => {
    const { container } = await renderLoaded();
    expect(container.querySelector(".loading")).toBeNull();
  });
});

describe("System section", () => {
  it("shows GPU name from detect_gpu", async () => {
    const { getByText } = await renderLoaded();
    expect(getByText(/Apple M2/)).toBeTruthy();
  });

  it("shows NVIDIA GPU with VRAM", async () => {
    const { getByText } = await renderLoaded({ detect_gpu: GPU_NVIDIA });
    expect(getByText(/NVIDIA RTX 4080/)).toBeTruthy();
    expect(getByText(/16384 MB VRAM/)).toBeTruthy();
  });

  it("shows available disk space", async () => {
    const { getByText } = await renderLoaded();
    expect(getByText(/100 GB free/)).toBeTruthy();
  });

  it("does NOT show a Python row", async () => {
    const { container } = await renderLoaded();
    const labels = Array.from(container.querySelectorAll(".sys-label")).map(
      (el) => el.textContent
    );
    expect(labels).not.toContain("Python");
  });
});

describe("Python Environment section", () => {
  it("renders the section header", async () => {
    const { getByText } = await renderLoaded();
    expect(getByText("Python Environment")).toBeTruthy();
  });

  it("shows 'Managed automatically · No Python needed' description", async () => {
    const { getByText } = await renderLoaded();
    expect(getByText(/Managed automatically/)).toBeTruthy();
  });

  it("renders Install button when env is not installed", async () => {
    const { getAllByRole } = await renderLoaded({ get_python_env_installed: false });
    const buttons = getAllByRole("button");
    const pythonInstall = buttons.find(
      (b) => b.textContent?.trim() === "Install" && !b.hasAttribute("disabled")
    );
    expect(pythonInstall).toBeTruthy();
  });

  it("shows Installed badge and Remove button when env is installed", async () => {
    const { getAllByText, getByRole } = await renderLoaded({ get_python_env_installed: true });
    const badges = getAllByText("Installed");
    expect(badges.length).toBeGreaterThanOrEqual(1);
    const removeBtn = getByRole("button", { name: "Remove" });
    expect(removeBtn).toBeTruthy();
  });

  it("calls install_python_env when Install is clicked", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      const defaults: Record<string, unknown> = {
        detect_gpu: GPU_METAL,
        get_python_env_installed: false,
        get_disk_space: 100_000_000_000,
        get_backend_statuses: STATUSES,
        install_python_env: undefined,
      };
      return Promise.resolve(defaults[cmd] ?? undefined);
    });

    const { getAllByRole } = await renderLoaded();
    const buttons = getAllByRole("button");
    const installBtn = buttons.find(
      (b) => b.textContent?.trim() === "Install" && !b.hasAttribute("disabled")
    )!;
    await fireEvent.click(installBtn);

    expect(mockInvoke).toHaveBeenCalledWith("install_python_env");
  });

  it("calls uninstall_python_env when Remove is clicked", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      const defaults: Record<string, unknown> = {
        detect_gpu: GPU_METAL,
        get_python_env_installed: true,
        get_disk_space: 100_000_000_000,
        get_backend_statuses: STATUSES,
        uninstall_python_env: undefined,
      };
      return Promise.resolve(defaults[cmd] ?? undefined);
    });

    const { getByRole } = await renderLoaded({ get_python_env_installed: true });
    const removeBtn = getByRole("button", { name: "Remove" });
    await fireEvent.click(removeBtn);

    expect(mockInvoke).toHaveBeenCalledWith("uninstall_python_env");
  });
});

describe("LIVE backend section", () => {
  it("renders LIVE section", async () => {
    const { getByText } = await renderLoaded();
    expect(getByText("LIVE")).toBeTruthy();
  });

  it("disables Install button when python env is not installed", async () => {
    const { container } = await renderLoaded({ get_python_env_installed: false });
    const disabledButtons = Array.from(
      container.querySelectorAll<HTMLButtonElement>("button[disabled]")
    );
    const liveInstall = disabledButtons.find((b) => b.textContent?.trim() === "Install");
    expect(liveInstall).toBeTruthy();
  });

  it("enables Install button when python env is installed", async () => {
    const { getAllByRole } = await renderLoaded({ get_python_env_installed: true });
    const buttons = getAllByRole("button");
    const enabledInstall = buttons.find(
      (b) => b.textContent?.trim() === "Install" && !b.hasAttribute("disabled")
    );
    expect(enabledInstall).toBeTruthy();
  });

  it("shows dependency note when python env is missing", async () => {
    const { getAllByText } = await renderLoaded({ get_python_env_installed: false });
    const notes = getAllByText(/Requires: Python Environment/);
    expect(notes.length).toBeGreaterThanOrEqual(1);
  });

  it("shows LIVE speed note", async () => {
    const { getByText } = await renderLoaded();
    expect(getByText(/LIVE is significantly slower/)).toBeTruthy();
  });
});

describe("StarVector section", () => {
  it("renders StarVector section header", async () => {
    const { getByText } = await renderLoaded();
    expect(getByText("StarVector")).toBeTruthy();
  });

  it("shows model size labels", async () => {
    const { getByText } = await renderLoaded();
    expect(getByText("~2 GB")).toBeTruthy();
    expect(getByText("~16 GB")).toBeTruthy();
  });

  it("StarVector-1B Download button is enabled when VRAM is sufficient", async () => {
    const { container } = await renderLoaded({
      detect_gpu: { name: "NVIDIA RTX 3060", vram_mb: 8192, compute_backend: "cuda" },
      get_python_env_installed: true,
    });
    const downloadBtns = Array.from(
      container.querySelectorAll<HTMLButtonElement>("button")
    ).filter((b) => b.textContent?.trim() === "Download");
    // With 8192 MB (>= 4 GB), 1B button should NOT be disabled for VRAM
    const disabledForVram = downloadBtns.find(
      (b) => b.hasAttribute("disabled") && b.title?.includes("4 GB")
    );
    expect(disabledForVram).toBeUndefined();
  });
});

describe("error display", () => {
  it("shows error message when an invoke rejects", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "install_python_env")
        return Promise.reject("Python environment installation not yet implemented");
      const defaults: Record<string, unknown> = {
        detect_gpu: GPU_METAL,
        get_python_env_installed: false,
        get_disk_space: 100_000_000_000,
        get_backend_statuses: STATUSES,
      };
      return Promise.resolve(defaults[cmd] ?? undefined);
    });

    const { getAllByRole, getByText } = await renderLoaded();
    const buttons = getAllByRole("button");
    const installBtn = buttons.find(
      (b) => b.textContent?.trim() === "Install" && !b.hasAttribute("disabled")
    )!;

    await fireEvent.click(installBtn);
    await act(async () => {
      await new Promise<void>((r) => setTimeout(r, 10));
    });

    expect(
      getByText(/Python environment installation not yet implemented/)
    ).toBeTruthy();
  });
});

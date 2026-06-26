import { vi, describe, it, expect, beforeEach, afterEach } from "vitest";
import { get } from "svelte/store";

// @tauri-apps/api/core must be mocked before any import that touches it.
// backends.ts calls invoke/listen at module-init; by mocking core+event here,
// the real backends store still loads cleanly (no actual Tauri calls made).
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

import { invoke } from "@tauri-apps/api/core";
import {
  runTrace,
  imagePath,
  svgOutput,
  isTracing,
  traceError,
  traceOptions,
} from "./tracing";
import { activeBackend } from "./backends";
import { defaultTraceOptions } from "../types";

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  vi.clearAllMocks();
  // Resolve get_backend_statuses so the backends store doesn't error on init
  mockInvoke.mockResolvedValue([]);

  imagePath.set(null);
  svgOutput.set(null);
  isTracing.set(false);
  traceError.set(null);
  traceOptions.set({ ...defaultTraceOptions });
  activeBackend.set("vtracer");
});

afterEach(() => {
  vi.useRealTimers();
});

describe("runTrace", () => {
  it("calls trace_image and sets svgOutput on success", async () => {
    mockInvoke.mockResolvedValueOnce("<svg>...</svg>");

    await runTrace("/path/to/image.png", defaultTraceOptions);

    expect(mockInvoke).toHaveBeenCalledWith("trace_image", {
      filePath: "/path/to/image.png",
      opts: defaultTraceOptions,
    });
    expect(get(svgOutput)).toBe("<svg>...</svg>");
    expect(get(traceError)).toBeNull();
  });

  it("sets traceError and clears svgOutput on failure", async () => {
    svgOutput.set("<svg>previous</svg>");
    mockInvoke.mockRejectedValueOnce("vtracer failed");

    await runTrace("/bad/path.png", defaultTraceOptions);

    expect(get(traceError)).toBe("vtracer failed");
    expect(get(svgOutput)).toBeNull();
  });

  it("clears traceError before invoking", async () => {
    traceError.set("old error");
    mockInvoke.mockResolvedValueOnce("<svg/>");

    await runTrace("/img.png", defaultTraceOptions);

    expect(get(traceError)).toBeNull();
  });

  it("sets isTracing true while running and false after", async () => {
    let resolve!: (v: string) => void;
    mockInvoke.mockReturnValueOnce(new Promise((r) => (resolve = r)));

    const promise = runTrace("/img.png", defaultTraceOptions);
    expect(get(isTracing)).toBe(true);

    resolve("<svg/>");
    await promise;
    expect(get(isTracing)).toBe(false);
  });

  it("queues a concurrent call and runs it after the first completes", async () => {
    let resolve1!: (v: string) => void;
    let resolve2!: (v: string) => void;
    mockInvoke
      .mockReturnValueOnce(new Promise((r) => (resolve1 = r)))
      .mockReturnValueOnce(new Promise((r) => (resolve2 = r)));

    // Start first trace
    const p1 = runTrace("/img1.png", defaultTraceOptions);
    // Second call while first is in progress — should queue, not call invoke yet
    runTrace("/img2.png", defaultTraceOptions);

    expect(mockInvoke).toHaveBeenCalledTimes(1);

    // Complete first
    resolve1("<svg>1</svg>");
    await p1;

    // Pending trace should now have fired
    expect(mockInvoke).toHaveBeenCalledTimes(2);
    expect(mockInvoke).toHaveBeenNthCalledWith(2, "trace_image", {
      filePath: "/img2.png",
      opts: defaultTraceOptions,
    });

    // Resolve second; use a macrotask boundary so all pending microtasks
    // (including the second runTrace's continuation) complete before we check.
    resolve2("<svg>2</svg>");
    await new Promise<void>((r) => setTimeout(r, 0));

    expect(get(svgOutput)).toBe("<svg>2</svg>");
  });
});

describe("live preview debounce", () => {
  it("triggers runTrace after 500 ms for vtracer backend", async () => {
    vi.useFakeTimers();
    mockInvoke.mockResolvedValue("<svg/>");

    imagePath.set("/img.png");
    activeBackend.set("vtracer");

    traceOptions.update((o) => ({ ...o, filter_speckle: 8 }));
    expect(mockInvoke).not.toHaveBeenCalled();

    vi.advanceTimersByTime(500);
    // Let the async runTrace start
    await vi.runAllTimersAsync();

    expect(mockInvoke).toHaveBeenCalledWith("trace_image", expect.anything());
  });

  it("does not trigger debounce when no image is loaded", async () => {
    vi.useFakeTimers();

    imagePath.set(null);
    activeBackend.set("vtracer");
    traceOptions.update((o) => ({ ...o, filter_speckle: 8 }));

    vi.advanceTimersByTime(600);
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("does not trigger debounce for non-vtracer backends", async () => {
    vi.useFakeTimers();

    imagePath.set("/img.png");
    activeBackend.set("live");
    traceOptions.update((o) => ({ ...o, filter_speckle: 8 }));

    vi.advanceTimersByTime(600);
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});

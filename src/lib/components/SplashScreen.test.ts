import { vi, describe, it, expect, beforeEach } from "vitest";
import { render, fireEvent } from "@testing-library/svelte";
import SplashScreen from "./SplashScreen.svelte";

describe("SplashScreen", () => {
  it("renders the wordmark", () => {
    const { getByText } = render(SplashScreen);
    expect(getByText("Re:Trace")).toBeTruthy();
  });

  it("renders the tagline", () => {
    const { getByText } = render(SplashScreen);
    expect(getByText("Raster to vector, beautifully.")).toBeTruthy();
  });

  it("renders studio and license metadata", () => {
    const { getByText } = render(SplashScreen);
    expect(getByText("Boxing Octopus Creative")).toBeTruthy();
    expect(getByText("MPL 2.0")).toBeTruthy();
  });

  it("renders a Close button", () => {
    const { getByRole } = render(SplashScreen);
    expect(getByRole("button", { name: "Close" })).toBeTruthy();
  });

  it("renders the dialog with correct aria attributes", () => {
    const { getByRole } = render(SplashScreen);
    const dialog = getByRole("dialog");
    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(dialog).toHaveAttribute("aria-label", "About Re:Trace");
  });

  it("dispatches close event when Close button is clicked", async () => {
    vi.useFakeTimers();
    const { getByRole, component } = render(SplashScreen);

    const closeHandler = vi.fn();
    component.$on("close", closeHandler);

    await fireEvent.click(getByRole("button", { name: "Close" }));

    // close is dispatched after a 220 ms animation delay
    expect(closeHandler).not.toHaveBeenCalled();
    vi.advanceTimersByTime(220);
    expect(closeHandler).toHaveBeenCalledTimes(1);

    vi.useRealTimers();
  });

  it("dispatches close event when overlay is clicked", async () => {
    vi.useFakeTimers();
    const { getByRole, component } = render(SplashScreen);

    const closeHandler = vi.fn();
    component.$on("close", closeHandler);

    await fireEvent.click(getByRole("dialog"));

    vi.advanceTimersByTime(220);
    expect(closeHandler).toHaveBeenCalledTimes(1);

    vi.useRealTimers();
  });

  it("dispatches close event on Escape key", async () => {
    vi.useFakeTimers();
    const { getByRole, component } = render(SplashScreen);

    const closeHandler = vi.fn();
    component.$on("close", closeHandler);

    await fireEvent.keyDown(getByRole("dialog"), { key: "Escape" });

    vi.advanceTimersByTime(220);
    expect(closeHandler).toHaveBeenCalledTimes(1);

    vi.useRealTimers();
  });

  it("does not dispatch close on non-Escape key", async () => {
    vi.useFakeTimers();
    const { getByRole, component } = render(SplashScreen);

    const closeHandler = vi.fn();
    component.$on("close", closeHandler);

    await fireEvent.keyDown(getByRole("dialog"), { key: "Enter" });

    vi.advanceTimersByTime(500);
    expect(closeHandler).not.toHaveBeenCalled();

    vi.useRealTimers();
  });
});

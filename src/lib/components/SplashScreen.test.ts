import { vi, describe, it, expect } from "vitest";
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
    expect(getByText("BOC Engineering")).toBeTruthy();
    expect(getByText("Mozilla Public License 2.0")).toBeTruthy();
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

  it("calls onclose after animation delay when Close button is clicked", async () => {
    vi.useFakeTimers();
    const onclose = vi.fn();
    const { getByRole } = render(SplashScreen, { onclose });

    await fireEvent.click(getByRole("button", { name: "Close" }));

    expect(onclose).not.toHaveBeenCalled();
    vi.advanceTimersByTime(220);
    expect(onclose).toHaveBeenCalledTimes(1);

    vi.useRealTimers();
  });

  it("calls onclose after animation delay when overlay is clicked", async () => {
    vi.useFakeTimers();
    const onclose = vi.fn();
    const { getByRole } = render(SplashScreen, { onclose });

    await fireEvent.click(getByRole("dialog"));

    vi.advanceTimersByTime(220);
    expect(onclose).toHaveBeenCalledTimes(1);

    vi.useRealTimers();
  });

  it("calls onclose after animation delay on Escape key", async () => {
    vi.useFakeTimers();
    const onclose = vi.fn();
    const { getByRole } = render(SplashScreen, { onclose });

    await fireEvent.keyDown(getByRole("dialog"), { key: "Escape" });

    vi.advanceTimersByTime(220);
    expect(onclose).toHaveBeenCalledTimes(1);

    vi.useRealTimers();
  });

  it("does not call onclose on non-Escape key", async () => {
    vi.useFakeTimers();
    const onclose = vi.fn();
    const { getByRole } = render(SplashScreen, { onclose });

    await fireEvent.keyDown(getByRole("dialog"), { key: "Enter" });

    vi.advanceTimersByTime(500);
    expect(onclose).not.toHaveBeenCalled();

    vi.useRealTimers();
  });
});

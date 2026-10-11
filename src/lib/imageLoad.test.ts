import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { loadImage } from "./imageLoad";
import { extractSpectrumColorsFromImage } from "./spectrumColors";
class FakeImage {
  static all: FakeImage[] = [];
  onload: (() => void) | null = null;
  onerror: (() => void) | null = null;
  src = ""; crossOrigin = "";
  constructor() { FakeImage.all.push(this); }
  removeAttribute() { this.src = ""; }
}
beforeEach(() => { vi.useFakeTimers(); FakeImage.all = []; vi.stubGlobal("Image", FakeImage); });
afterEach(() => { vi.useRealTimers(); vi.unstubAllGlobals(); });
describe("bounded image lifecycle", () => {
  it.each(["error", "timeout", "abort"])("clears handlers, timeout and source on %s, ignores captured late callbacks", async terminal => {
    const controller = new AbortController(); const remove = vi.spyOn(controller.signal, "removeEventListener");
    const request = loadImage("https://test/cover", controller.signal);
    const rejected = expect(request).rejects.toThrow();
    const image = FakeImage.all[0], late = image.onload!;
    if (terminal === "error") image.onerror!();
    if (terminal === "timeout") vi.advanceTimersByTime(5000);
    if (terminal === "abort") controller.abort();
    late(); await rejected;
    expect(image.onload).toBeNull(); expect(image.onerror).toBeNull(); expect(image.src).toBe("");
    expect(vi.getTimerCount()).toBe(0); expect(remove).toHaveBeenCalledTimes(1);
  });
  it("success removes callbacks and abort subscription while retaining pixels for the caller", async () => {
    const controller = new AbortController(), request = loadImage("data:image/png,test", controller.signal);
    const image = FakeImage.all[0]; image.onload!(); await request; controller.abort();
    expect(image.src).toBe("data:image/png,test"); expect(image.onload).toBeNull(); expect(image.onerror).toBeNull();
    expect(vi.getTimerCount()).toBe(0);
  });
  it("does not allocate for an already destroyed owner", async () => {
    const controller = new AbortController(); controller.abort();
    await expect(loadImage("cover", controller.signal)).rejects.toMatchObject({name:"AbortError"});
    expect(FakeImage.all).toHaveLength(0);
  });
  it("spectrum timeout removes callbacks without creating a sampling canvas", async () => {
    const createElement = vi.fn(); vi.stubGlobal("document", {createElement});
    const rejected = expect(extractSpectrumColorsFromImage("cover")).rejects.toThrow("timed out");
    vi.advanceTimersByTime(5000); await rejected;
    expect(FakeImage.all[0].onload).toBeNull(); expect(createElement).not.toHaveBeenCalled();
  });
  it("abort between image success and sampling prevents DOM work", async () => {
    const createElement = vi.fn(); vi.stubGlobal("document", {createElement});
    const controller = new AbortController(); const result = extractSpectrumColorsFromImage("cover", controller.signal);
    FakeImage.all[0].onload!(); controller.abort();
    expect(await result).toBeNull(); expect(FakeImage.all[0].src).toBe(""); expect(createElement).not.toHaveBeenCalled();
  });
});

import { describe, expect, it, vi } from "vitest";
import { createCanvasArtworkRenderer } from "./canvasArtwork";

function fixture() {
  const canvas = { isConnected: true, width: 100, height: 100 } as HTMLCanvasElement;
  const images: HTMLImageElement[] = [];
  const paint = vi.fn();
  const process = vi.fn(async (url: string) => `processed:${url}`);
  const renderer = createCanvasArtworkRenderer(canvas, {
    paint, process,
    createImage: () => {
      const image = { src: "", onload: null, onerror: null } as HTMLImageElement;
      images.push(image);
      return image;
    },
  });
  const update = (url: string, pixelated = false, track = url) => renderer.update({ url, pixelated, track });
  const complete = (image: HTMLImageElement) => image.onload?.call(image, {} as Event);
  return { canvas, images, paint, process, renderer, update, complete };
}

describe("Canvas artwork ownership", () => {
  it("rejects a delayed old load even when the URL is reused for another track", () => {
    const f = fixture();
    f.update("same-cover", false, "track-a");
    const lateLoad = f.images[0].onload!;
    f.update("same-cover", false, "track-b");
    lateLoad.call(f.images[0], {} as Event);
    expect(f.paint).not.toHaveBeenCalled();
    f.complete(f.images[1]);
    expect(f.paint).toHaveBeenCalledExactlyOnceWith(f.canvas, f.images[1]);
    expect(f.images[0].src).toBe("");
    expect(f.images[1].onload).toBeNull();
  });

  it("rejects superseded backend processing and pixel mode changes", async () => {
    const f = fixture();
    let resolve!: (url: string) => void;
    f.process.mockImplementationOnce(() => new Promise<string>(done => resolve = done));
    f.update("old", true);
    f.update("new", false);
    resolve("processed:old");
    await Promise.resolve();
    expect(f.images).toHaveLength(1);
    expect(f.images[0].src).toBe("new");
    f.complete(f.images[0]);
    expect(f.paint).toHaveBeenCalledOnce();
  });

  it("cancels empty covers and prevents detached or destroyed nodes from drawing", () => {
    const f = fixture();
    f.update("old");
    const late = f.images[0].onload!;
    f.update("");
    late.call(f.images[0], {} as Event);
    expect(f.canvas.width).toBe(0);
    f.update("new");
    Object.defineProperty(f.canvas, "isConnected", { value: false });
    f.complete(f.images[1]);
    expect(f.paint).not.toHaveBeenCalled();
    f.renderer.destroy();
    expect(f.canvas.height).toBe(0);
    f.update("after-destroy");
    expect(f.images).toHaveLength(2);
  });

  it("keeps latest-generation fallback working without resurrecting stale errors", async () => {
    const f = fixture();
    f.update("original", true);
    await Promise.resolve();
    const staleError = f.images[0].onerror!;
    staleError.call(f.images[0], {} as Event);
    expect(f.images[1].src).toBe("original");
    f.update("latest");
    staleError.call(f.images[0], {} as Event);
    expect(f.images).toHaveLength(3);
    f.complete(f.images[2]);
    expect(f.paint).toHaveBeenCalledExactlyOnceWith(f.canvas, f.images[2]);
  });

  it("releases backing stores and image callbacks over 100 remount cycles", () => {
    for (let index = 0; index < 100; index++) {
      const f = fixture();
      f.update("cover");
      const late = f.images[0].onload!;
      f.renderer.destroy();
      late.call(f.images[0], {} as Event);
      expect(f.paint).not.toHaveBeenCalled();
      expect(f.canvas.width + f.canvas.height).toBe(0);
      expect(f.images[0]).toMatchObject({ src: "", onload: null, onerror: null });
    }
  });
});

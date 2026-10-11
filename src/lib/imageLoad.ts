/** A bounded Image request. Every terminal path releases handlers and source. */
export function loadImage(src: string, signal?: AbortSignal, timeoutMs = 5_000): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) { reject(new DOMException("Image cancelled", "AbortError")); return; }
    const image = new Image();
    let settled = false;
    const cleanup = () => {
      clearTimeout(timer);
      image.onload = null;
      image.onerror = null;
      signal?.removeEventListener("abort", abort);
    };
    const finish = (error?: Error) => {
      if (settled) return;
      settled = true;
      cleanup();
      if (error) { image.removeAttribute("src"); reject(error); }
      else resolve(image);
    };
    const abort = () => finish(new DOMException("Image cancelled", "AbortError"));
    const timer = setTimeout(() => finish(new Error("Artwork load timed out")), timeoutMs);
    image.onload = () => finish();
    image.onerror = () => finish(new Error("Artwork load failed"));
    signal?.addEventListener("abort", abort, { once: true });
    if (src.startsWith("http") && !src.includes("asset.localhost")) image.crossOrigin = "Anonymous";
    try { image.src = src; } catch (error) { finish(error instanceof Error ? error : new Error(String(error))); }
  });
}

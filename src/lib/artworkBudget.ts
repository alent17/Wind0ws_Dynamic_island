export const MAX_ARTWORK_EDGE = 1280;
export const MAX_ARTWORK_URL_CHARS = 16 * 1024 * 1024 + 128;

export function artworkCanvasSize(width: number, height: number, logicalEdge: number, dpr: number) {
  const edge = Math.min(MAX_ARTWORK_EDGE, Math.max(1, Math.ceil(logicalEdge * dpr)));
  const scale = Math.min(1, edge / Math.max(1, width, height));
  return { width: Math.max(1, Math.round(width * scale)), height: Math.max(1, Math.round(height * scale)) };
}

/** Conservative UTF-16 accounting, including keys; oversized values bypass retention. */
export class ArtworkResultCache {
  private entries = new Map<string, string>();
  private bytes = 0;
  constructor(private maxBytes = 8 * 1024 * 1024, private maxEntries = 12) {}
  get byteSize() { return this.bytes; }
  get size() { return this.entries.size; }
  get(key: string) {
    const value = this.entries.get(key);
    if (value !== undefined) { this.entries.delete(key); this.entries.set(key, value); }
    return value;
  }
  set(key: string, value: string) {
    const previous = this.entries.get(key);
    if (previous !== undefined) {
      this.entries.delete(key); this.bytes -= 2 * (key.length + previous.length);
    }
    const cost = 2 * (key.length + value.length);
    if (cost > this.maxBytes) return;
    while (this.entries.size && (this.bytes + cost > this.maxBytes || this.entries.size >= this.maxEntries)) {
      const oldest = this.entries.keys().next().value!;
      this.bytes -= 2 * (oldest.length + this.entries.get(oldest)!.length);
      this.entries.delete(oldest);
    }
    this.entries.set(key, value); this.bytes += cost;
  }
  clear() { this.entries.clear(); this.bytes = 0; }
}

export async function artworkCacheKey(url: string, pixelated: boolean) {
  if (url.length > MAX_ARTWORK_URL_CHARS) throw new Error("Artwork input exceeds the byte budget");
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(url));
  return `${pixelated ? "p" : "n"}:${Array.from(new Uint8Array(digest), byte => byte.toString(16).padStart(2, "0")).join("")}`;
}

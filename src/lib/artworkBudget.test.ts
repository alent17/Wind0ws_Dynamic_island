import { describe, expect, it } from "vitest";
import { artworkCanvasSize, ArtworkResultCache, artworkCacheKey } from "./artworkBudget";

describe("artwork byte and raster budgets", () => {
  it("uses the visible edge at 1x, 1.5x and 2x while bounding higher DPI", () => {
    for (const [dpr, edge] of [[1,640],[1.5,960],[2,1280],[3,1280]]) {
      expect(artworkCanvasSize(4096,2048,640,dpr)).toEqual({width:edge,height:edge/2});
    }
    expect(artworkCanvasSize(108,108,640,2)).toEqual({width:108,height:108});
  });
  it("evicts by retained bytes and recency, including replacement keys", () => {
    const cache = new ArtworkResultCache(24,12);
    cache.set("a","1111"); cache.set("b","2222"); cache.get("a");
    cache.set("c","3333");
    expect(cache.get("b")).toBeUndefined();
    expect(cache.byteSize).toBe(20);
    cache.set("a","x".repeat(100));
    expect(cache.get("a")).toBeUndefined();
    expect(cache.byteSize).toBe(10);
    cache.clear(); expect(cache.byteSize).toBe(0);
  });
  it("retains short content keys rather than Base64 source strings", async () => {
    const key = await artworkCacheKey("data:image/png;base64,AQID",true);
    expect(key).toMatch(/^p:[a-f0-9]{64}$/);
    expect(await artworkCacheKey("data:image/png;base64,AQIE",true)).not.toBe(key);
  });
});

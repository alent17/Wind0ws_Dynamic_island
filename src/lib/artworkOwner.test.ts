import { describe, expect, it, vi, beforeEach } from "vitest";
const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
import { createArtworkOwner } from "./artworkOwner";

beforeEach(() => invoke.mockReset());
describe("artwork consumer lifetime", () => {
  it("clear rejects a delayed result and releases its native lease", async () => {
    let finish!: (value: string) => void;
    invoke.mockImplementation((command: string) => command === "request_artwork" ? new Promise(resolve => finish = resolve) : Promise.resolve());
    const owner = createArtworkOwner("color");
    const request = owner.request<string>({ imagePath: "old" });
    const rejection = expect(request).rejects.toMatchObject({ name: "AbortError" });
    owner.cancel(); finish("old-color"); await rejection;
    expect(invoke.mock.calls[1][0]).toBe("cancel_artwork");
    expect(invoke.mock.calls[1][1].revision).toBe(invoke.mock.calls[0][1].revision);
  });
  it("same track replacement carries different cover identity and newer revision", async () => {
    invoke.mockResolvedValue(null);
    const owner = createArtworkOwner("cover");
    const args = { title: "same", artist: "artist", source: "source" };
    await owner.request(args, "data:image/png;base64,AQID");
    await owner.request(args, "data:image/png;base64,AQIE");
    const requests = invoke.mock.calls.filter(call => call[0] === "request_artwork");
    expect(requests[1][1].coverVersion).not.toBe(requests[0][1].coverVersion);
    expect(requests[1][1].revision).toBeGreaterThan(requests[0][1].revision);
  });
  it("aborted pixel work cannot return after a new mode or canvas owner", async () => {
    let finish!: (value: string) => void;
    invoke.mockImplementation((command: string) => command === "request_artwork" ? new Promise(resolve => finish = resolve) : Promise.resolve());
    const signal = new AbortController(), owner = createArtworkOwner("pixel");
    const request = owner.request<string>({ imagePath: "cover", enablePixelArt: true }, undefined, signal.signal);
    const rejection = expect(request).rejects.toMatchObject({ name: "AbortError" });
    signal.abort(); finish("stale-pixels"); await rejection;
  });
  it("failure remains retryable and a failed lease does not cancel its replacement", async () => {
    invoke.mockRejectedValueOnce(new Error("network")).mockResolvedValueOnce("new");
    const owner = createArtworkOwner("pixel");
    await expect(owner.request({ imagePath: "cover" })).rejects.toThrow("network");
    await expect(owner.request({ imagePath: "cover" })).resolves.toBe("new");
    expect(invoke.mock.calls.map(call => call[0])).toEqual(["request_artwork", "request_artwork"]);
  });
});

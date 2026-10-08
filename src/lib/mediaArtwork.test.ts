import { expect, it } from "vitest";
import { artworkValue, snapshotDecision, withoutArtwork } from "./mediaArtwork";

it("distinguishes a delta without artwork from explicit clears and replacements", () => {
  expect(artworkValue({ title: "same" })).toBeUndefined();
  expect(artworkValue({ albumArt: "" })).toBe("");
  expect(artworkValue({ albumArt: null })).toBe("");
  expect(artworkValue({ albumArt: "new" })).toBe("new");
  expect(artworkValue({ albumArt: "", thumbnail: "old" })).toBe("");
  expect(artworkValue({ thumbnail: "new" })).toBe("new");
});

it("accepts a full same-track snapshot after an artwork-less delta", () => {
  expect(snapshotDecision({events:0,artwork:0},{events:2,artwork:0},true)).toBe("accept");
});

it("does not let an older snapshot revert a track, clear, or newer cover", () => {
  expect(snapshotDecision({events:0,artwork:0},{events:1,artwork:1},false)).toBe("discard");
  expect(snapshotDecision({events:0,artwork:0},{events:1,artwork:1},true)).toBe("ignore-artwork");
  expect(artworkValue(withoutArtwork({albumArt:"old",thumbnail:"old",title:"same"}))).toBeUndefined();
});

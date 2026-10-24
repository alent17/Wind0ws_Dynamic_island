import { expect, it } from "vitest";
import { appOverlayDecision, artworkValue, mergeProgressAnchorFields, nativeStampDecision, readNativeMediaStamp, snapshotDecision, withoutArtwork } from "./mediaArtwork";

it("distinguishes a delta without artwork from explicit clears and replacements", () => {
  expect(artworkValue({ title: "same" })).toBeUndefined();
  expect(artworkValue({ albumArt: "" })).toBe("");
  expect(artworkValue({ albumArt: null })).toBe("");
  expect(artworkValue({ albumArt: "new" })).toBe("new");
  expect(artworkValue({ albumArt: "", thumbnail: "old" })).toBe("");
  expect(artworkValue({ thumbnail: "new" })).toBe("new");
});

it("accepts confirmed same-track progress fields without losing omitted anchor state", () => {
  const pending = { progressKnown: false, neteaseProgressAvailable: false };
  expect(mergeProgressAnchorFields(pending, { progressKnown: true, neteaseProgressAvailable: true })).toEqual({
    progressKnown: true,
    neteaseProgressAvailable: true,
  });
  expect(mergeProgressAnchorFields(pending, { isPlaying: true })).toEqual(pending);
});

it("limits a same-track snapshot after newer events to artwork only", () => {
  expect(snapshotDecision({events:0,artwork:0,trackGeneration:1,trackKey:"track"},{events:2,artwork:0,trackGeneration:1,trackKey:"track"},true)).toBe("artwork-only");
  expect(snapshotDecision({events:0,artwork:0,trackGeneration:0,trackKey:""},{events:1,artwork:0,trackGeneration:1,trackKey:"track"},true)).toBe("artwork-only");
});

it("does not let an older snapshot revert a track, clear, or newer cover", () => {
  expect(snapshotDecision({events:0,artwork:0,trackGeneration:1,trackKey:"track"},{events:1,artwork:1,trackGeneration:2,trackKey:"other"},false)).toBe("discard");
  expect(snapshotDecision({events:0,artwork:0,trackGeneration:1,trackKey:"track"},{events:1,artwork:1,trackGeneration:1,trackKey:"track"},true)).toBe("discard");
  expect(snapshotDecision({events:0,artwork:0,trackGeneration:1,trackKey:"track"},{events:1,artwork:0,trackGeneration:3,trackKey:"track"},true)).toBe("discard");
  expect(artworkValue(withoutArtwork({albumArt:"old",thumbnail:"old",title:"same"}))).toBeUndefined();
});

it("validates native stamps and orders only revisions inside an opaque epoch", () => {
  const current = { nativeEpoch: "epoch-z", nativeRevision: 8 };
  expect(readNativeMediaStamp({ nativeEpoch: "epoch-z", nativeRevision: 8 })).toEqual(current);
  expect(readNativeMediaStamp({ nativeEpoch: "epoch-z", nativeRevision: Number.MAX_SAFE_INTEGER + 1 })).toBeUndefined();
  expect(nativeStampDecision(current, true, new Set(), { nativeEpoch: "epoch-z", nativeRevision: 7 }, true)).toBe("older");
  expect(nativeStampDecision(current, true, new Set(), { nativeEpoch: "epoch-z", nativeRevision: 8 }, true)).toBe("same");
  expect(nativeStampDecision(current, true, new Set(), { nativeEpoch: "epoch-z", nativeRevision: 9 }, true)).toBe("newer");
  expect(nativeStampDecision(current, true, new Set(), { nativeEpoch: "epoch-a", nativeRevision: 0 }, false)).toBe("retired");
  expect(nativeStampDecision(current, true, new Set(), { nativeEpoch: "epoch-a", nativeRevision: 0 }, true)).toBe("switch");
  expect(nativeStampDecision(undefined, false, new Set(), undefined, false)).toBe("legacy");
  expect(nativeStampDecision(current, true, new Set(), undefined, true)).toBe("invalid");
  expect(nativeStampDecision(undefined, false, new Set(["old"]), { nativeEpoch: "old", nativeRevision: 99 }, true)).toBe("retired");
});

it("accepts App overlays only from the active session, in sequence, on the current basis and track", () => {
  const base = {
    incomingSession:"session-new", currentSession:"session-new", retiredSessions:new Set(["session-old"]),
    incomingSequence:3, currentSequence:1, basisMatches:true, trackMatches:true,
  };
  expect(appOverlayDecision(base)).toBe("accept"); // gaps are fine after sender coalescing
  expect(appOverlayDecision({...base,incomingSession:"session-old"})).toBe("retired-session");
  expect(appOverlayDecision({...base,currentSession:"session-new",incomingSequence:1})).toBe("old-sequence");
  expect(appOverlayDecision({...base,incomingSequence:Number.MAX_SAFE_INTEGER+1})).toBe("invalid-sequence");
  expect(appOverlayDecision({...base,basisMatches:false})).toBe("wrong-basis");
  expect(appOverlayDecision({...base,trackMatches:false})).toBe("wrong-track");
});

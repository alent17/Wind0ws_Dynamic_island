import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import test from "node:test";

const rust = readFileSync(fileURLToPath(new URL("../src-tauri/src/services/netease_cdp.rs", import.meta.url)), "utf8");
function rawConstant(name) {
  const match = rust.match(new RegExp(`const ${name}: &str = r#"([\\s\\S]*?)"#;`));
  assert.ok(match, `missing Rust constant ${name}`);
  return match[1];
}
const capture = rawConstant("CLOUD_MUSIC_CAPTURE_TOOL");
const progress = rawConstant("CLOUD_MUSIC_PROGRESS_EXPRESSION");
const seek = rawConstant("CLOUD_MUSIC_SEEK_EXPRESSION");

function fixture({ video = false, loading = false, restore = false, changeTrack = false } = {}) {
  const store = { playing: {
    playId: "play-1", onlineResourceId: "song-1", resourceDuration: 180,
    resourceName: "Song One", resourceArtist: "Artist One", curPlaying: { track: { name: "Song One", artists: [{ name: "Artist One" }] } },
    isPlayingVideo: video, isLoadingFirst: loading, restoreResource: restore,
  } };
  const progress = { playId: "play-1", current: 12 };
  let action;
  let dispatchCount = 0;
  const tool = {
    inited: true,
    app: { _store: { getState() {} } },
    getStore: () => store,
    getDispatch: () => async (received) => {
      dispatchCount++;
      action = received;
      if (changeTrack) {
        store.playing.onlineResourceId = "song-2";
        store.playing.playId = "play-2";
        progress.playId = "play-2";
      } else {
        progress.current = received.payload.duration + 0.25;
      }
    },
  };
  const queue = [[[], { 8: function () { this.app._store.getState(); } }, [[1424]]]];
  queue.push = function (chunk) {
    const id = chunk[2][0][0];
    const req = {
      c: { 8: { exports: { a: tool } }, 148: { exports: { b: () => progress } }, 1424: { exports: {} } },
      m: { [id]: chunk[1][id] },
      s: 1424,
    };
    chunk[1][id]({}, {}, req);
    Array.prototype.push.call(this, chunk);
    return this.length;
  };
  const context = vm.createContext({ Date, Math, Number, String, Array, Promise, setTimeout });
  context.globalThis = context;
  context.webpackJsonp = queue;
  return { context, get action() { return action; }, get dispatchCount() { return dispatchCount; }, progress };
}

async function execute(fx, requested, title = "Song One", artist = "Artist One") {
  const expression = seek
    .replaceAll("__CAPTURE_TOOL__", capture)
    .replaceAll("__REQUESTED_SECONDS__", JSON.stringify(requested))
    .replaceAll("__EXPECTED_IDENTITY__", JSON.stringify({ title, artist }));
  return vm.runInContext(expression, fx.context, { timeout: 6000 });
}

async function readProgress(fx) {
  const expression = progress.replaceAll("__CAPTURE_TOOL__", capture);
  return vm.runInContext(expression, fx.context, { timeout: 2000 });
}

test("production progress expression returns current identity and live position", async () => {
  const fx = fixture();
  const first = await readProgress(fx);
  assert.deepEqual(JSON.parse(JSON.stringify(first)), {
    ready: true, trackId: "song-1", playId: "play-1", title: "Song One", artist: "Artist One",
    positionMs: 12_000, durationMs: 180_000,
  });
  // Model an external NetEase seek after the previous read; the next expression
  // reads the provider's current progress rather than advancing a stale anchor.
  fx.progress.current = 91.5;
  const second = await readProgress(fx);
  assert.equal(second.positionMs, 91_500);
});

test("production seek expression dispatches seconds and verifies actual progress", async () => {
  const fx = fixture();
  const result = await execute(fx, 35.5);
  assert.equal(JSON.stringify(fx.action), JSON.stringify({
    type: "playing/setPlayingPosition",
    payload: { duration: 35.5 },
  }));
  assert.equal(result.verified, true);
  assert.equal(result.trackId, "song-1");
  assert.equal(result.positionMs, 35_750);
});

test("ordinary audio guards reject video, loading, and restore modes", async () => {
  for (const options of [{ video: true }, { loading: true }, { restore: true }]) {
    const result = await execute(fixture(options), 35.5);
    assert.equal(result.unsupported, true);
  }
});

test("rejects out-of-range targets and a track change during seek", async () => {
  assert.equal((await execute(fixture(), 180)).invalidPosition, true);
  assert.equal((await execute(fixture({ changeTrack: true }), 35.5)).trackChanged, true);
});

test("rejects a mismatched expected UI identity before dispatch", async () => {
  const fx = fixture();
  const result = await execute(fx, 35.5, "Song Two", "Artist One");
  assert.equal(result.identityMismatch, true);
  assert.equal(fx.dispatchCount, 0);
});

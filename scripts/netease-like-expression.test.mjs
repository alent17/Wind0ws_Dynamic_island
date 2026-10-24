import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import test from 'node:test';

const rust = readFileSync(new URL('../src-tauri/src/services/netease_cdp.rs', import.meta.url), 'utf8');
const raw = name => rust.match(new RegExp(`const ${name}: &str = r#"([\\s\\S]*?)"#;`))[1];
const read = raw('CLOUD_MUSIC_LIKE_READ_EXPRESSION');
function fixture({ liked = false, fail = false, changed = false, offline = false } = {}) {
  const store = { app: { isOnline: !offline }, playing: { resourceType: 'track', onlineResourceId: '123', playId: '123_play', resourceName: 'Song', resourceArtist: 'Artist', curPlaying: { track: { id: '123' } } }, 'async:hostResource': { starPlaylistId: '456', likeTracksMap: { 123: liked }, likeTrackIds: liked ? ['123'] : [] } };
  const actions = [];
  const tool = { getStore: () => store, getDispatch: () => async action => {
    actions.push(action);
    if (changed) store.playing.playId = 'new-play';
    if (!fail) store['async:hostResource'].likeTracksMap[123] = action.payload.isLike;
  } };
  const context = vm.createContext({ tool, setTimeout: resolve => resolve() });
  const execute = expression => vm.runInContext(expression.replaceAll('__CAPTURE_TOOL__', 'function(){return {tool:globalThis.tool}}'), context);
  return { store, actions, execute, set(liked, expected = {}) {
    return execute(raw('CLOUD_MUSIC_LIKE_SET_EXPRESSION').replace('__LIKE_READ__', read).replace('__EXPECTED__', JSON.stringify({ trackId: '123', playId: '123_play', title: 'Song', artist: 'Artist', ...expected })).replace('__LIKED__', JSON.stringify(liked)));
  } };
}
test('reads liked and unliked state from provider', () => {
  for (const liked of [true, false]) assert.equal(fixture({ liked }).execute(read).liked, liked);
});
test('uses explicit like and unlike actions and verified readback', async () => {
  for (const liked of [true, false]) {
    const fx = fixture({ liked: !liked });
    const result = await fx.set(liked);
    assert.equal(result.verified, true);
    assert.equal(result.liked, liked);
    assert.equal(fx.actions[0].type, 'async:hostResource/setLikeTrack');
    assert.equal(fx.actions[0].payload.isLike, liked);
    assert.equal(fx.actions[0].payload.resource.id, '123');
  }
});
test('does not act on stale song identities or already desired state', async () => {
  const fx = fixture();
  assert.equal((await fx.set(true, { trackId: 'other' })).verified, false);
  assert.equal((await fx.set(false)).verified, true);
  assert.equal(fx.actions.length, 0);
});
test('failed writes and track changes never verify success', async () => {
  assert.equal((await fixture({ fail: true }).set(true)).verified, false);
  assert.equal((await fixture({ changed: true }).set(true)).verified, false);
});
test('offline and unsupported resources have no editable state', async () => {
  const fx = fixture({ offline: true });
  assert.equal(fx.execute(read).ready, undefined);
  assert.equal((await fx.set(true)).verified, false);
  assert.equal(fx.actions.length, 0);
  fx.store.app.isOnline = true;
  fx.store.playing.resourceType = 'voice';
  assert.equal(fx.execute(read).ready, undefined);
});

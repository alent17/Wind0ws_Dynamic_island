import { expect, test } from "@playwright/test";

test("VolumePanel submits the pointerup coordinate as its final write", async ({page}) => {
  const errors: string[] = []; page.on("pageerror", error => errors.push(String(error)));
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => (window as any).mountVolumePanel());
  const ruler = page.locator(".volume-ruler");
  await ruler.evaluate(element => {
    (window as any).volumePointerId = null;
    element.addEventListener("pointerdown", event => {
      (window as any).volumePointerId = (event as PointerEvent).pointerId;
    }, {once:true});
  });
  const box = await ruler.boundingBox();
  expect(box).not.toBeNull();
  const startX = box!.x + box!.width / 2;
  const y = box!.y + box!.height / 2;
  await page.mouse.move(startX,y);
  await page.mouse.down();
  await page.mouse.move(startX-10,y);
  const pointerId = await page.evaluate(() => (window as any).volumePointerId);
  await ruler.dispatchEvent("pointerup", {pointerId,clientX:startX-20,clientY:y});
  await page.mouse.up();
  await expect.poll(() => page.evaluate(() => (window as any).volumeWrites.at(-1))).toBe(52);
  const writesAfterUp = await page.evaluate(() => (window as any).volumeWrites.slice());
  await page.waitForTimeout(50);
  expect(await page.evaluate(() => (window as any).volumeWrites)).toEqual(writesAfterUp);
  expect(errors).toEqual([]);
});

test("VolumePanel pointercancel clears capture without submitting a terminal value", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => (window as any).mountVolumePanel());
  const ruler = page.locator(".volume-ruler");
  await ruler.evaluate(element => {
    (window as any).volumePointerId = null;
    element.addEventListener("pointerdown", event => {
      (window as any).volumePointerId = (event as PointerEvent).pointerId;
    }, {once:true});
  });
  const box = await ruler.boundingBox();
  expect(box).not.toBeNull();
  const startX = box!.x + box!.width / 2;
  const y = box!.y + box!.height / 2;
  await page.mouse.move(startX,y);
  await page.mouse.down();
  await page.mouse.move(startX-10,y);
  const beforeCancel = await page.evaluate(() => (window as any).volumeWrites.slice());
  const pointerId = await page.evaluate(() => (window as any).volumePointerId);
  await ruler.dispatchEvent("pointercancel", {pointerId,clientX:startX-30,clientY:y});
  await page.mouse.up();
  await page.waitForTimeout(50);
  expect(await page.evaluate(() => (window as any).volumeWrites)).toEqual(beforeCancel);
});

test("transport diagnostics stay scoped to ready media sends and one in-flight region command", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    (window as any).beginTransportDiagnostics();
    (window as any).holdRegionCommands = true;
    (window as any).mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).transportDiagnosticsSnapshot().routes.region.dispatchAttempts)).toBe(1);
  let snapshot = await page.evaluate(() => (window as any).transportDiagnosticsSnapshot());
  expect(snapshot.scope).toBe("this-webview");
  expect(snapshot.routes.region.durationCount).toBe(0);
  expect(snapshot.routes.mediaSync.dispatchAttempts).toBeUndefined();

  await page.evaluate(() => (window as any).media({
    title:"No consumer yet",artist:"Fixture",source:"fixture-player",isPlaying:false,positionMs:0,durationMs:60_000,
    albumArt:"fixture-art",nativeEpoch:"fixture-epoch",nativeRevision:1,
  }));
  await expect.poll(() => page.evaluate(() => (window as any).transportDiagnosticsSnapshot().routes.mediaSync.noConsumer)).toBeGreaterThan(0);

  await page.evaluate(() => (window as any).releaseRegionCommands());
  await expect.poll(() => page.evaluate(() => (window as any).transportDiagnosticsSnapshot().routes.region.dispatchSuccess)).toBe(1);
  expect((await page.evaluate(() => (window as any).transportDiagnosticsSnapshot().routes.region)).durationCount).toBe(1);
  await page.evaluate(() => (window as any).mountFloatingPeer({enableHdCover:false}));
  await page.evaluate(() => (window as any).activateFloatingConsumer());
  await page.evaluate(() => (window as any).media({
    title:"Fixture route",artist:"Fixture",source:"fixture-player",isPlaying:false,positionMs:0,durationMs:60_000,
    albumArt:"fixture-art",nativeEpoch:"fixture-epoch",nativeRevision:1,
  }));
  await expect.poll(() => page.evaluate(() => (window as any).transportDiagnosticsSnapshot().routes.mediaSync.received)).toBeGreaterThan(0);
  snapshot = await page.evaluate(() => (window as any).transportDiagnosticsSnapshot());
  expect(snapshot.routes.mediaSync.noConsumer).toBeGreaterThan(0);
  expect(snapshot.routes.mediaSync.dispatchAttempts).toBeGreaterThan(0);
  expect(snapshot.routes.mediaSync.dispatchSuccess).toBeGreaterThan(0);
  await page.evaluate(() => { (window as any).endTransportDiagnostics(); (window as any).destroy(); });
});

test("transport diagnostics count Timer snapshots on change, not local running seconds", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    (window as any).beginTransportDiagnostics();
    (window as any).mountApp();
    (window as any).mountTimerPeer();
  });
  await expect.poll(() => page.evaluate(() => (window as any).transportDiagnosticsSnapshot().routes.timerState.dispatchSuccess)).toBeGreaterThan(0);
  await page.evaluate(() => (window as any).timerAction("start", 60_000));
  await expect.poll(() => page.evaluate(() => (window as any).transportDiagnosticsSnapshot().routes.timerState.received)).toBeGreaterThan(0);
  const before = await page.evaluate(() => (window as any).transportDiagnosticsSnapshot().routes.timerState.dispatchAttempts);
  await page.waitForTimeout(1_200);
  const after = await page.evaluate(() => (window as any).transportDiagnosticsSnapshot().routes.timerState.dispatchAttempts);
  expect(after).toBe(before);
  await page.evaluate(() => { (window as any).endTransportDiagnostics(); (window as any).destroy(); });
});

test("transport diagnostics count actual volume dispatches after queue dedupe", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => { (window as any).beginTransportDiagnostics(); (window as any).mountVolumePanel(); });
  const ruler = page.locator(".volume-ruler");
  await expect(ruler).toBeVisible();
  await expect(ruler).toHaveAttribute("aria-valuenow", "50");
  await ruler.press("Home");
  await ruler.press("Home");
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((call:string) => call === "set_system_volume").length)).toBe(1);
  const volume = await page.evaluate(() => (window as any).transportDiagnosticsSnapshot().routes.volume);
  expect(volume.dispatchAttempts).toBe(1);
  expect(volume.dispatchSuccess).toBe(1);
  expect(volume.deduped).toBeGreaterThan(0);
  expect(volume.durationCount).toBe(1);
  await page.evaluate(() => { (window as any).endTransportDiagnostics(); (window as any).destroy(); });
});

test("App ignores a delayed manual hide after re-expansion and still applies a normal hide", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    (window as any).settingsOverrides = {
      enableAnimations: true,
      reduceAnimations: false,
      showCustomFunctionPanel: true,
      showHideTool: true,
    };
    (window as any).mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((call:string) => call === "animate_window_bounds").length)).toBeGreaterThan(0);
  await page.evaluate(() => (window as any).expand());
  await expect.poll(() => page.evaluate(() => (window as any).islandSnapshot().expanded)).toBe(true);
  await expect(page.locator(".feature-menu button")).toHaveCount(7);

  const hideButton = page.locator(".feature-menu button").nth(4);
  const baselineBoundsCalls = await page.evaluate(() => (window as any).calls.filter((call:string) => call === "animate_window_bounds").length);
  await hideButton.click({force:true});
  await expect.poll(() => page.evaluate(() => (window as any).islandSnapshot().expanded)).toBe(false);
  // Reopen while the production handler is inside its 360ms collapse wait.
  await page.evaluate(() => (window as any).expand());
  await expect.poll(() => page.evaluate(() => (window as any).islandSnapshot().expanded)).toBe(true);
  await page.waitForTimeout(500);
  expect(await page.evaluate(() => (window as any).calls.filter((call:string) => call === "animate_window_bounds").length)).toBe(baselineBoundsCalls);

  // A later hide with no intervening expansion must still reach native placement.
  await page.waitForTimeout(250);
  await hideButton.click({force:true});
  await expect.poll(() => page.evaluate(() => (window as any).islandSnapshot().expanded)).toBe(false);
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((call:string) => call === "animate_window_bounds").length), {timeout:1500}).toBeGreaterThan(baselineBoundsCalls);
  await page.evaluate(() => (window as any).destroy());
});

test("App applies the full monitor snapshot once after the built-in snake-case legacy marker", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    (window as any).useTwoMonitors();
    (window as any).mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((call:string) => call === "get_settings").length)).toBeGreaterThan(0);
  await expect.poll(() => page.evaluate(() => (window as any).calls.includes("show_main_window"))).toBe(true);
  await expect.poll(() => page.evaluate(() => (window as any).calls.includes("animate_window_bounds"))).toBe(true);
  await page.waitForTimeout(50);

  const before = await page.evaluate(() => ({
    reads:(window as any).calls.filter((call:string) => call === "get_settings").length,
    bounds:(window as any).calls.filter((call:string) => call === "animate_window_bounds").length,
    current:(window as any).getSettingsSnapshot(),
  }));
  const next = {...before.current,monitorIndex:1};
  await page.evaluate(async snapshot => {
    await (window as any).settingsChanged("monitor_index");
    await (window as any).settingsUpdated(snapshot);
    await (window as any).settingsCommitted(1);
  },next);
  await expect.poll(() => page.evaluate(() => (window as any).getSettingsSnapshot().monitorIndex)).toBe(1);
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((call:string) => call === "animate_window_bounds").length)).toBe(before.bounds+1);
  await page.waitForTimeout(75);
  expect(await page.evaluate(() => (window as any).calls.filter((call:string) => call === "get_settings").length)).toBe(before.reads);
  expect(await page.evaluate(() => (window as any).calls.filter((call:string) => call === "animate_window_bounds").length)).toBe(before.bounds+1);
  await page.evaluate(() => (window as any).destroy());
});

test("App keeps camelCase and unknown settings-changed fallbacks, but disposed App has no listener side effects", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    (window as any).useTwoMonitors();
    (window as any).nativeMonitorIndex=1;
    (window as any).mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((call:string) => call === "get_settings").length)).toBeGreaterThan(0);
  await expect.poll(() => page.evaluate(() => (window as any).calls.includes("show_main_window"))).toBe(true);

  const beforeMonitorRead = await page.evaluate(() => (window as any).calls.filter((call:string) => call === "get_current_monitor_index").length);
  await page.evaluate(() => (window as any).settingsChanged("monitorIndex"));
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((call:string) => call === "get_current_monitor_index").length)).toBe(beforeMonitorRead+1);

  const beforeFallback = await page.evaluate(() => (window as any).calls.filter((call:string) => call === "get_settings").length);
  await page.evaluate(() => (window as any).settingsChanged("legacy-plugin-setting"));
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((call:string) => call === "get_settings").length)).toBe(beforeFallback+1);

  const beforeLateResult = await page.evaluate(() => (window as any).getSettingsSnapshot());
  await page.evaluate(() => {
    (window as any).deferSettingsReads=true;
    return (window as any).settingsChanged("legacy-stale-setting");
  });
  await expect.poll(() => page.evaluate(() => (window as any).pendingSettingsReads.length)).toBe(1);
  await page.evaluate(() => (window as any).destroy());
  await page.evaluate(snapshot => (window as any).resolveSettingsRead(0,{...snapshot,monitorIndex:99}),beforeLateResult);
  await page.waitForTimeout(50);
  expect(await page.evaluate(() => (window as any).getSettingsSnapshot())).toEqual(beforeLateResult);
  const afterDispose = await page.evaluate(() => (window as any).calls.filter((call:string) => call === "get_settings").length);
  await page.evaluate(() => (window as any).settingsChanged("legacy-plugin-setting"));
  await page.waitForTimeout(50);
  expect(await page.evaluate(() => (window as any).calls.filter((call:string) => call === "get_settings").length)).toBe(afterDispose);
});

for (const stage of ["exit", "enter"]) test(`App cancels WAAPI on unmount during ${stage}, no late layout IPC`, async ({page}) => {
  const errors: string[] = []; page.on("pageerror", error => errors.push(String(error)));
  await page.emulateMedia({reducedMotion:"no-preference"});
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => (window as any).mountApp());
  await expect.poll(() => page.evaluate(() => (window as any).calls.includes("show_main_window"))).toBe(true);
  await page.evaluate(() => (window as any).changeEdge());
  await page.waitForFunction(opacity => (window as any).animations.at(-1)?.firstOpacity === opacity, stage === "exit" ? 1 : 0);
  const before = await page.evaluate(async () => {
    await (window as any).destroy();
    return (window as any).calls.filter((cmd: string) => cmd === "animate_window_bounds").length;
  });
  await page.waitForTimeout(800);
  expect(await page.evaluate(() => (window as any).calls.filter((cmd: string) => cmd === "animate_window_bounds").length)).toBe(before);
  expect(await page.evaluate(() => (window as any).animations.at(-1).cancelled)).toBeGreaterThan(0);
  await expect(page.locator(".fixed-island-host")).toHaveCount(0);
  expect(errors).toEqual([]);
});

for (const frames of [0,1]) test(`RollingDigit cancels pending frame after ${frames} recenter frames`, async ({page}) => {
  await page.emulateMedia({reducedMotion:"no-preference"});
  await page.goto("/ui-tests/lifecycle-fixture.html"); await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => (window as any).mountDigit());
  await expect(page.locator(".digit-strip")).toHaveClass(/ready/);
  await page.evaluate(() => (window as any).setDigit("9"));
  await expect(page.locator(".digit-strip")).toHaveCSS("transform", /matrix/);
  const result = await page.evaluate(async count => {
    let next = 100; const queue = new Map<number,FrameRequestCallback>(), cancelled: number[] = [];
    window.requestAnimationFrame = callback => { queue.set(++next,callback); return next; };
    window.cancelAnimationFrame = id => { cancelled.push(id); queue.delete(id); };
    document.querySelector(".digit-strip")!.dispatchEvent(new TransitionEvent("transitionend",{propertyName:"transform"}));
    if (count) { const [id,callback] = [...queue][0]; queue.delete(id); callback(0); }
    await (window as any).destroy();
    return {cancelled,pending:queue.size};
  }, frames);
  expect(result.pending).toBe(0); expect(result.cancelled.length).toBeGreaterThan(0);
  await expect(page.locator(".digit-strip")).toHaveCount(0);
});

test("FloatingWindow gates MV playback and progress work while capture-hidden", async ({page}) => {
  const errors: string[] = []; page.on("pageerror", error => errors.push(String(error)));
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.route("http://asset.localhost/**", route => route.abort());
  await page.evaluate(() => (window as any).mountFloating());
  await page.waitForTimeout(100);
  await page.evaluate(() => (window as any).resolveVideoPlay());

  const playing = {
    title: "C3 capture gate",
    artist: "Fixture",
    albumArt: "data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=",
    isPlaying: true,
    positionMs: 10_000,
    durationMs: 180_000,
    lastUpdatedTimestamp: Date.now(),
    source: "netease",
    sourceDisplay: "Fixture",
    capabilities: { previous: true, playPause: true, next: true, seek: true, shuffle: false, repeat: false },
  };
  await page.evaluate(payload => (window as any).media(payload), playing);
  await expect(page.locator(".mv-player")).toHaveCount(1);
  await expect.poll(() => page.evaluate(() => (window as any).videoCalls.filter((call: any) => call.kind === "play").length)).toBeGreaterThan(0);
  await expect.poll(() => page.evaluate(() => (window as any).live250Intervals())).toBe(1);

  await page.evaluate(() => (window as any).captureMode(true));
  await expect(page.locator(".player")).toHaveClass(/capture-hidden/);
  await expect.poll(() => page.evaluate(() => (window as any).videoIsPaused())).toBe(true);
  await expect.poll(() => page.evaluate(() => (window as any).live250Intervals())).toBe(0);
  const hiddenPlayCount = await page.evaluate(() => (window as any).videoCalls.filter((call: any) => call.kind === "play").length);

  await page.waitForTimeout(550); // MEDIA_UPDATE uses the production 500 ms throttle.
  await page.evaluate(async (payload) => {
    await (window as any).media(payload);
    (window as any).loadedData();
  }, playing);
  await expect.poll(() => page.evaluate(() => (window as any).videoCalls.filter((call: any) => call.kind === "play").length)).toBe(hiddenPlayCount);

  await page.evaluate(() => (window as any).captureMode(false));
  await expect.poll(() => page.evaluate(() => (window as any).videoIsPaused())).toBe(false);
  await page.evaluate(() => (window as any).resolvePendingVideoPlay());
  await expect.poll(() => page.evaluate(() => (window as any).videoIsPaused())).toBe(false);

  await page.waitForTimeout(550);
  await page.evaluate(payload => (window as any).media({...payload,isPlaying:false,lastUpdatedTimestamp:Date.now()}), playing);
  await page.waitForTimeout(100);
  const pauseState = await page.evaluate(() => ({
    paused:(window as any).videoIsPaused(),
    calls:(window as any).videoCalls.map((call: any) => call.kind),
    live250:(window as any).live250Intervals(),
  }));
  if (!pauseState.paused) throw new Error(`SMTC pause did not stop MV: ${JSON.stringify(pauseState)}`);
  const pausedPlayCount = await page.evaluate(() => (window as any).videoCalls.filter((call: any) => call.kind === "play").length);
  await page.evaluate(async () => {
    await (window as any).captureMode(true);
    await (window as any).captureMode(false);
  });
  await expect.poll(() => page.evaluate(() => (window as any).videoCalls.filter((call: any) => call.kind === "play").length)).toBe(pausedPlayCount);

  await page.evaluate(() => (window as any).resolveVideoPlay());
  await page.waitForTimeout(550);
  await page.evaluate(payload => (window as any).media({...payload,isPlaying:true,lastUpdatedTimestamp:Date.now()}), playing);
  await expect.poll(() => page.evaluate(() => (window as any).videoCalls.filter((call: any) => call.kind === "play").length)).toBe(pausedPlayCount + 1);
  await page.evaluate(() => (window as any).destroy());
  await page.evaluate(() => (window as any).resolvePendingVideoPlay());
  await expect.poll(() => page.evaluate(() => (window as any).lastVideoIsPaused())).toBe(true);
  expect(errors).toEqual([]);
});

test("FloatingWindow pauses circular artwork motion while capture-hidden", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => (window as any).mountFloating({floatingCircularAlbum:true,enableMvPlayback:false}));
  await page.waitForTimeout(100);
  const playing = {
    title: "C3 vinyl gate",
    artist: "Fixture",
    albumArt: "data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=",
    isPlaying: true,
    positionMs: 10_000,
    durationMs: 180_000,
    lastUpdatedTimestamp: Date.now(),
    source: "netease",
    sourceDisplay: "Fixture",
    capabilities: { previous: true, playPause: true, next: true, seek: true, shuffle: false, repeat: false },
  };
  await page.evaluate(payload => (window as any).media(payload), playing);
  await expect(page.locator(".player.circular-album.playing .album-wrapper")).toHaveCSS("animation-play-state", "running");
  await page.evaluate(() => (window as any).captureMode(true));
  await expect(page.locator(".player.circular-album")).not.toHaveClass(/playing/);
  await expect(page.locator(".player.circular-album .album-wrapper")).toHaveCSS("animation-play-state", "paused");
  await page.evaluate(() => (window as any).captureMode(false));
  await expect(page.locator(".player.circular-album.playing .album-wrapper")).toHaveCSS("animation-play-state", "running");
  await page.evaluate(() => (window as any).destroy());
});

const mediaFixtureState = (patch: Record<string, unknown> = {}) => ({
  title: "Snapshot race",
  artist: "Fixture",
  source: "fixture-player",
  sourceDisplay: "Fixture",
  isPlaying: false,
  positionMs: 42_000,
  durationMs: 120_000,
  lastUpdatedTimestamp: Date.now(),
  capabilities: {previous:true,playPause:true,next:true,seek:true,shuffle:false,repeat:false},
  ...patch,
});

async function prepareDeferredFloatingSnapshot(page: import("@playwright/test").Page) {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    (window as any).__ISLE_MEDIA_TRACE__ = [];
    (window as any).deferNextMediaInfo();
    (window as any).mountFloating({enableMvPlayback:false});
  });
  await page.waitForFunction(() => (window as any).pendingMediaInfo.length === 1);
}

test("FloatingWindow keeps same-track event fields and only fills a missing cover from its delayed snapshot", async ({page}) => {
  await prepareDeferredFloatingSnapshot(page);
  const eventState = mediaFixtureState();
  delete (eventState as Record<string, unknown>).albumArt;
  await page.evaluate(payload => (window as any).mediaSync(payload), eventState);
  await expect(page.locator(".track-title")).toHaveText("Snapshot race");
  await expect(page.locator('input[type="range"]')).toHaveAttribute("max", "120000");
  await expect(page.locator('input[type="range"]')).toHaveValue("42000");
  await expect(page.locator(".player")).not.toHaveClass(/playing/);

  await page.evaluate(payload => (window as any).resolveMediaInfo(0,payload), mediaFixtureState({
    isPlaying:true, positionMs:8_000, durationMs:90_000, albumArt:"data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=", artworkResolved:true,
  }));
  await expect(page.locator('input[type="range"]')).toHaveAttribute("max", "120000");
  await expect(page.locator('input[type="range"]')).toHaveValue("42000");
  await expect(page.locator(".player")).not.toHaveClass(/playing/);
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row:any) => row.outcome === "filled-missing-snapshot-artwork"))).toBe(true);
  await page.evaluate(() => (window as any).destroy());
});

for (const [label,clear] of [["empty string", ""], ["null", null]] as const) test(`FloatingWindow does not revive a same-track ${label} artwork clear`, async ({page}) => {
  await prepareDeferredFloatingSnapshot(page);
  const initial = mediaFixtureState({albumArt:"data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=",artworkResolved:true});
  await page.evaluate(payload => (window as any).mediaSync(payload), initial);
  await expect(page.locator(".track-title")).toHaveText("Snapshot race");
  await page.evaluate(payload => (window as any).mediaSync(payload), mediaFixtureState({albumArt:clear}));
  await page.evaluate(payload => (window as any).resolveMediaInfo(0,payload), mediaFixtureState({
    isPlaying:true, positionMs:8_000, durationMs:90_000, albumArt:"data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=", artworkResolved:true,
  }));
  await expect(page.locator(".player")).not.toHaveClass(/playing/);
  await expect(page.locator('input[type="range"]')).toHaveAttribute("max", "120000");
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row:any) => row.origin === "initial-snapshot" && row.outcome === "dropped-stale-snapshot"))).toBe(true);
  await page.evaluate(() => (window as any).destroy());
});

test("FloatingWindow rejects an old A snapshot after an A-to-B-to-A event sequence", async ({page}) => {
  await prepareDeferredFloatingSnapshot(page);
  const event = (title:string, patch:Record<string,unknown>) => ({...mediaFixtureState(patch),title});
  await page.evaluate(payload => (window as any).mediaSync(payload), event("Track A",{positionMs:10_000,isPlaying:false}));
  await page.evaluate(payload => (window as any).mediaSync(payload), event("Track B",{positionMs:20_000,isPlaying:false}));
  await page.evaluate(payload => (window as any).mediaSync(payload), event("Track A",{positionMs:35_000,isPlaying:false}));
  await expect(page.locator(".track-title")).toHaveText("Track A");
  await expect(page.locator('input[type="range"]')).toHaveValue("35000");
  await page.evaluate(payload => (window as any).resolveMediaInfo(0,payload), event("Track A",{isPlaying:true,positionMs:1_000,durationMs:80_000}));
  await expect(page.locator('input[type="range"]')).toHaveAttribute("max", "120000");
  await expect(page.locator('input[type="range"]')).toHaveValue("35000");
  await expect(page.locator(".player")).not.toHaveClass(/playing/);
  await page.evaluate(() => (window as any).destroy());
});

test("FloatingWindow uses native revisions across raw events and App relays", async ({page}) => {
  await prepareDeferredFloatingSnapshot(page);
  const state = (revision:number, patch:Record<string,unknown> = {}) => mediaFixtureState({nativeEpoch:"epoch-a",nativeRevision:revision,...patch});
  await page.evaluate(payload => (window as any).media(payload), state(2,{title:"Revision 2",positionMs:22_000}));
  await page.evaluate(payload => (window as any).resolveMediaInfo(0,payload),state(1,{title:"Late command",positionMs:1_000}));
  await page.evaluate(() => (window as any).mediaSessionStart("relay-session"));
  await expect.poll(() => page.evaluate(() => (window as any).floatingReadyMessages.some((m:any) => m.appSessionId === "relay-session"))).toBe(true);
  const relay = (revision:number,title:string,positionMs:number,overlaySeq:number) => ({...state(revision,{title,positionMs}),appSessionId:"relay-session",overlaySeq});
  await page.evaluate(payload => (window as any).mediaSync(payload), relay(1,"Revision 1 relay",1_000,1));
  await expect(page.locator(".track-title")).toHaveText("Revision 2");
  await expect(page.locator('input[type="range"]')).toHaveValue("22000");
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row:any) => row.outcome === "dropped-older-native-stamp"))).toBe(true);

  await page.evaluate(payload => (window as any).mediaSync(payload), relay(3,"Revision 3 relay",33_000,2));
  await page.evaluate(payload => (window as any).media(payload), state(2,{title:"Late revision 2",positionMs:2_000}));
  await expect(page.locator(".track-title")).toHaveText("Revision 3 relay");
  await expect(page.locator('input[type="range"]')).toHaveValue("33000");
  await page.evaluate(() => (window as any).destroy());
});

test("FloatingWindow makes duplicate stamps idempotent, accepts newer clears, and retires switched epochs", async ({page}) => {
  await prepareDeferredFloatingSnapshot(page);
  const state = (epoch:string, revision:number, patch:Record<string,unknown> = {}) => mediaFixtureState({nativeEpoch:epoch,nativeRevision:revision,...patch});
  await page.evaluate(payload => (window as any).media(payload), state("epoch-a",2,{title:"Current",positionMs:22_000,isPlaying:false}));
  await page.evaluate(payload => (window as any).media(payload), state("epoch-a",2,{title:"Duplicate rollback",positionMs:1_000,isPlaying:true}));
  await expect(page.locator(".track-title")).toHaveText("Current");
  await expect(page.locator('input[type="range"]')).toHaveValue("22000");
  await expect(page.locator(".player")).not.toHaveClass(/playing/);

  await page.evaluate(payload => (window as any).media(payload), state("epoch-a",3,{title:"",source:"",albumArt:""}));
  await expect(page.locator(".track-title")).not.toHaveText("Current");
  await page.evaluate(payload => (window as any).media(payload), state("epoch-b",0,{title:"New epoch",positionMs:9_000}));
  await expect(page.locator(".track-title")).toHaveText("New epoch");
  await page.evaluate(payload => (window as any).media(payload), state("epoch-a",99,{title:"Retired epoch",positionMs:99_000}));
  await expect(page.locator(".track-title")).toHaveText("New epoch");
  await expect(page.locator('input[type="range"]')).toHaveValue("9000");
  await page.evaluate(() => (window as any).destroy());
});

test("FloatingWindow rejects a lower-revision command cover and allows same-revision art-only fill", async ({page}) => {
  await prepareDeferredFloatingSnapshot(page);
  const base = mediaFixtureState({nativeEpoch:"snapshot-epoch",nativeRevision:4,positionMs:44_000,isPlaying:false});
  await page.evaluate(payload => (window as any).media(payload), base);
  await page.evaluate(payload => (window as any).resolveMediaInfo(0,payload), mediaFixtureState({
    ...base,nativeRevision:3,isPlaying:true,positionMs:3_000,albumArt:"late-old-cover",artworkResolved:true,
  }));
  await expect(page.locator('input[type="range"]')).toHaveValue("44000");
  await expect(page.locator(".player")).not.toHaveClass(/playing/);
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row:any) => row.origin === "initial-snapshot" && row.outcome === "dropped-older-native-stamp"))).toBe(true);
  await page.evaluate(() => (window as any).destroy());

  await prepareDeferredFloatingSnapshot(page);
  await page.evaluate(payload => (window as any).media(payload), base);
  await page.evaluate(payload => (window as any).resolveMediaInfo(0,payload), mediaFixtureState({
    ...base,isPlaying:true,positionMs:3_000,albumArt:"same-revision-cover",artworkResolved:true,
  }));
  await expect(page.locator('input[type="range"]')).toHaveValue("44000");
  await expect(page.locator(".player")).not.toHaveClass(/playing/);
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row:any) => row.outcome === "filled-missing-snapshot-artwork"))).toBe(true);
  await page.evaluate(() => (window as any).destroy());
});

test("A same-revision initial command snapshot cannot revive artwork after a stamped clear", async ({page}) => {
  await prepareDeferredFloatingSnapshot(page);
  const cleared = mediaFixtureState({nativeEpoch:"clear-epoch",nativeRevision:2,albumArt:"",positionMs:20_000});
  await page.evaluate(payload => (window as any).media(payload), cleared);
  await page.evaluate(payload => (window as any).resolveMediaInfo(0,payload), {
    ...cleared, albumArt:"late-cover", artworkResolved:true, isPlaying:true, positionMs:1_000,
  });
  await expect(page.locator('input[type="range"]')).toHaveValue("20000");
  await expect(page.locator(".compact-cover-image")).toHaveCount(0);
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row:any) => row.origin === "initial-snapshot" && row.outcome === "dropped-stale-snapshot"))).toBe(true);
  await page.evaluate(() => (window as any).destroy());
});

test("App relays the latest accepted native stamp and drops lower revisions", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => { (window as any).settingsOverrides = {enableHdCover:false}; (window as any).mountApp(); });
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBeGreaterThan(0);
  await page.evaluate(() => (window as any).activateFloatingConsumer());
  const state = (revision:number, title:string) => ({title,artist:"Fixture",source:"fixture-player",isPlaying:false,positionMs:revision*1000,durationMs:120000,nativeEpoch:"app-epoch",nativeRevision:revision});
  await page.evaluate(payload => (window as any).media(payload), state(2,"Accepted revision 2"));
  await page.evaluate(payload => (window as any).media(payload), state(1,"Rejected revision 1"));
  await expect.poll(() => page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBeGreaterThan(0);
  const latest = await page.evaluate(() => (window as any).mediaSyncDispatches.at(-1));
  expect(latest).toMatchObject({title:"Accepted revision 2",nativeEpoch:"app-epoch",nativeRevision:2});
  expect(latest.appSessionId).toEqual(expect.any(String));
  expect(latest.overlaySeq).toBeGreaterThan(0);
  expect(latest.appOverlay).toMatchObject({nativeEpoch:"app-epoch",nativeRevision:2,trackKey:"fixture-player|acceptedrevision2|fixture"});
  await page.evaluate(() => (window as any).destroy());
});

test("App gates media sync by Floating instance and replays full state after hide, reopen, and stale close", async ({page}) => {
  const cover = "data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=";
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => { (window as any).settingsOverrides = {enableHdCover:false}; (window as any).mountApp(); });
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBeGreaterThan(0);
  const state = {title:"Consumer gate",artist:"Fixture",source:"fixture",sourceDisplay:"Fixture",albumArt:cover,isPlaying:false,positionMs:1_000,durationMs:60_000,lastUpdatedTimestamp:Date.now()};
  await page.evaluate((payload) => (window as any).media(payload), state);
  await page.waitForTimeout(25);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBe(0);

  await page.evaluate(() => (window as any).activateFloatingConsumer("floating-one"));
  await expect.poll(() => page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBe(1);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.at(-1).albumArt)).toBe(cover);
  await page.evaluate(() => (window as any).floatingNotReady({instanceToken:"floating-one",reason:"hidden"}));
  await page.evaluate((payload) => (window as any).media({...payload,positionMs:2_000}), state);
  await page.waitForTimeout(25);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBe(1);

  await page.evaluate(() => (window as any).activateFloatingConsumer("floating-one"));
  await expect.poll(() => page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBe(2);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.at(-1).albumArt)).toBe(cover);
  await page.evaluate(() => (window as any).activateFloatingConsumer("floating-two"));
  await expect.poll(() => page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBe(3);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.at(-1).albumArt)).toBe(cover);
  await page.evaluate(() => (window as any).floatingClosed({instanceToken:"floating-one"}));
  await page.evaluate((payload) => (window as any).media({...payload,positionMs:3_000}), state);
  await expect.poll(() => page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBe(4);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.at(-1))).toMatchObject({positionMs:3_000});
  expect(await page.evaluate(() => Object.hasOwn((window as any).mediaSyncDispatches.at(-1),"albumArt"))).toBe(false);
  await page.evaluate(() => (window as any).destroy());
});

test("App assigns increasing overlay sequences to optimistic seek and rollback offers", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => { (window as any).settingsOverrides = {enableHdCover:false}; (window as any).mountApp(); });
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBeGreaterThan(0);
  await page.evaluate(() => (window as any).activateFloatingConsumer());
  await page.evaluate((payload) => (window as any).media(payload), mediaFixtureState({
    nativeEpoch:"seek-epoch",nativeRevision:1,positionMs:5_000,isPlaying:false,
    capabilities:{previous:true,playPause:true,next:true,seek:true,shuffle:false,repeat:false},
  }));
  await expect.poll(() => page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBeGreaterThan(0);
  await page.evaluate(() => { (window as any).expand(); (window as any).selectIslandPage("music"); });
  const range = page.locator('input[type="range"]');
  await expect(range).toBeVisible();
  const before = await page.evaluate(() => (window as any).mediaSyncDispatches.length);
  await page.evaluate(() => { (window as any).rejectNextSeek = true; });
  await range.evaluate((input:HTMLInputElement) => {
    input.value = "20000";
    input.dispatchEvent(new Event("input",{bubbles:true}));
    input.dispatchEvent(new Event("change",{bubbles:true}));
  });
  await expect.poll(() => page.evaluate((start) => (window as any).mediaSyncDispatches.length, before)).toBeGreaterThan(before + 1);
  const offers = await page.evaluate((start) => (window as any).mediaSyncDispatches.slice(start) as Array<any>, before);
  expect(offers[0].overlaySeq).toBeLessThan(offers.at(-1).overlaySeq);
  expect(offers.at(-1).appOverlay).toMatchObject({nativeEpoch:"seek-epoch",nativeRevision:1,positionMs:5_000});
  await expect(range).toHaveValue("5000");
  await page.evaluate(() => (window as any).destroy());
});

test("Floating fences App overlays by session, basis, and increasing sequence without letting duplicate raw art downgrade HD", async ({page}) => {
  await prepareDeferredFloatingSnapshot(page);
  const native = mediaFixtureState({nativeEpoch:"overlay-epoch",nativeRevision:2,albumArt:"native-cover",positionMs:22_000,isPlaying:false});
  await page.evaluate(payload => (window as any).media(payload), native);
  await page.evaluate(payload => (window as any).resolveMediaInfo(0,payload), {...native,nativeRevision:1,positionMs:1_000,albumArt:"stale-command-cover"});
  await expect.poll(() => page.evaluate(() => (window as any).floatingReadyQueryListenerCount())).toBe(1);
  await page.evaluate(() => (window as any).mediaSessionStart("app-session-a"));
  await expect.poll(() => page.evaluate(() => (window as any).floatingReadyMessages.some((m:any) => m.appSessionId === "app-session-a"))).toBe(true);

  const relay = (session:string, sequence:number, revision=2, albumArt="hd-cover", positionMs=45_000) => ({
    ...native, nativeRevision:revision, appSessionId:session, overlaySeq:sequence,
    appOverlay:{nativeEpoch:"overlay-epoch",nativeRevision:revision,trackKey:"fixture-player|snapshotrace|fixture",title:"Snapshot race",artist:"Fixture",albumArt,artworkResolved:true,positionMs,lastUpdatedTimestamp:Date.now()},
  });
  await page.evaluate(payload => (window as any).mediaSync(payload), relay("app-session-a",1));
  await expect(page.locator('input[type="range"]')).toHaveValue("45000");
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row:any) => row.outcome === "applied-app-overlay"))).toBe(true);
  const acceptedCoverChars = await page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.at(-1).displayCoverChars);
  expect(acceptedCoverChars).toBe("hd-cover".length);

  await page.evaluate(payload => (window as any).media(payload), native);
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.at(-1).displayCoverChars)).toBe(acceptedCoverChars);
  await page.evaluate(payload => (window as any).mediaSync(payload), relay("app-session-a",1,2,"stale-overlay",1_000));
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row:any) => row.outcome === "rejected-app-overlay-old-sequence"))).toBe(true);
  await expect(page.locator('input[type="range"]')).toHaveValue("45000");

  await page.evaluate(() => (window as any).mediaSessionStart("app-session-b"));
  await expect.poll(() => page.evaluate(() => (window as any).floatingReadyMessages.some((m:any) => m.appSessionId === "app-session-b"))).toBe(true);
  await page.evaluate(payload => (window as any).mediaSync(payload), relay("app-session-a",99,2,"retired-session-cover",9_000));
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row:any) => row.outcome === "dropped-retired-app-session"))).toBe(true);
  await page.evaluate(payload => (window as any).mediaSync(payload), relay("app-session-b",1,2,"new-session-cover",60_000));
  await expect(page.locator('input[type="range"]')).toHaveValue("60000");

  await page.evaluate(payload => (window as any).media(payload), {...native,nativeRevision:3,positionMs:30_000});
  await page.evaluate(payload => (window as any).mediaSync(payload), relay("app-session-b",2,2,"old-basis-cover",2_000));
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row:any) => row.outcome === "dropped-older-native-stamp"))).toBe(true);
  await page.evaluate(payload => (window as any).mediaSync(payload), relay("app-session-b",4,3,"new-basis-cover",31_000));
  await expect(page.locator('input[type="range"]')).toHaveValue("31000");
  await page.evaluate(() => (window as any).destroy());
});

test("FloatingWindow ignores a deferred initial snapshot after unmount", async ({page}) => {
  await prepareDeferredFloatingSnapshot(page);
  await page.evaluate(() => (window as any).destroy());
  await page.evaluate(payload => (window as any).resolveMediaInfo(0,payload), mediaFixtureState({albumArt:"late-cover"}));
  await expect(page.locator(".player")).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row:any) => row.origin === "initial-snapshot"))).toBe(false);
});

test("App stops idle weather polling while hidden and reuses then expires the existing cache", async ({page}) => {
  await page.clock.install({time:new Date("2026-10-20T00:00:00Z")});
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => (window as any).mountApp());
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((command: string) => command === "get_weather_snapshot").length)).toBe(1);
  expect(await page.evaluate(() => (window as any).calls.filter((command: string) => command === "get_idle_snapshot").length)).toBe(0);
  await expect.poll(() => page.evaluate(() => (window as any).live30000Intervals())).toBe(1);

  await page.evaluate(() => (window as any).captureMode(true));
  await expect(page.locator(".expanded-layer")).toHaveCount(0);
  await expect.poll(() => page.evaluate(() => (window as any).live30000Intervals())).toBe(0);
  await page.clock.runFor(10_000);
  expect(await page.evaluate(() => (window as any).calls.filter((command: string) => command === "get_weather_snapshot").length)).toBe(1);
  expect(await page.evaluate(() => (window as any).calls.filter((command: string) => command === "get_idle_snapshot").length)).toBe(0);

  await page.evaluate(() => (window as any).captureMode(false));
  await expect(page.locator(".expanded-layer")).toHaveCount(1);
  // The existing 30-second idleApi cache serves this immediate recovery.
  await page.waitForTimeout(0);
  expect(await page.evaluate(() => (window as any).calls.filter((command: string) => command === "get_weather_snapshot").length)).toBe(1);

  await page.evaluate(() => (window as any).captureMode(true));
  await expect(page.locator(".expanded-layer")).toHaveCount(0);
  await page.clock.runFor(31_000);
  expect(await page.evaluate(() => (window as any).calls.filter((command: string) => command === "get_weather_snapshot").length)).toBe(1);
  expect(await page.evaluate(() => (window as any).calls.filter((command: string) => command === "get_idle_snapshot").length)).toBe(0);
  await page.evaluate(() => (window as any).captureMode(false));
  await expect(page.locator(".expanded-layer")).toHaveCount(1);
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((command: string) => command === "get_weather_snapshot").length)).toBe(2);
  expect(await page.evaluate(() => (window as any).calls.filter((command: string) => command === "get_idle_snapshot").length)).toBe(0);
  await expect.poll(() => page.evaluate(() => (window as any).live30000Intervals())).toBe(1);
});

test("Studio preview clock refreshes on wall-minute boundaries and cancels on unmount", async ({page}) => {
  await page.clock.install({time:new Date("2026-10-20T00:00:00.250Z")});
  await page.clock.pauseAt(new Date("2026-10-20T00:00:00.250Z"));
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => (window as any).mountStudio());
  expect(await page.evaluate(() => (window as any).liveTimeoutDelays()[0])).toBe(59_750);
  expect(await page.evaluate(() => (window as any).intervalRows.some((row:any) => row.delay === 1_000 && row.active))).toBe(false);

  await page.clock.runFor(59_749);
  expect(await page.evaluate(() => (window as any).timeoutRows.filter((row:any) => row.delay === 59_750).reduce((n:number,row:any) => n + row.fired,0))).toBe(0);
  await page.clock.runFor(1);
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays())).toContain(60_000);

  await page.evaluate(() => (window as any).destroy());
  const firedAtUnmount = await page.evaluate(() => (window as any).firedTimeouts());
  expect(await page.evaluate(() => (window as any).liveTimeoutDelays())).toEqual([]);
  await page.clock.runFor(60_000);
  expect(await page.evaluate(() => (window as any).firedTimeouts())).toBe(firedAtUnmount);
});

test("Studio uses one listener-first media snapshot and keeps its demo preview until live data arrives", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture=window as any;
    fixture.deferNextMediaInfo();
    fixture.mountStudio();
  });
  await expect(page.getByRole("heading",{name:"Isle Studio"})).toBeVisible();
  await page.waitForFunction(() => (window as any).pendingMediaInfo.length === 1);
  expect(await page.evaluate(() => (window as any).calls.filter((command:string) => command === "get_media_info_cmd").length)).toBe(1);
  const beforeLive = await page.locator(".studio-status strong").textContent();

  await page.evaluate(() => (window as any).resolveMediaInfo(0,{
    title:"Studio live track",artist:"Fixture",albumArt:"",isPlaying:true,positionMs:8_000,durationMs:60_000,
    lastUpdatedTimestamp:Date.now(),source:"fixture",sourceDisplay:"Fixture",
    capabilities:{previous:false,playPause:true,next:false,seek:true,shuffle:false,repeat:false},
  }));
  await expect.poll(() => page.locator(".studio-status strong").textContent()).not.toBe(beforeLive);
  expect(await page.evaluate(() => (window as any).calls.filter((command:string) => command === "get_media_info_cmd").length)).toBe(1);
  await page.evaluate(() => (window as any).destroy());
});

test("Studio mediaStore rejects stale native events and a late actor snapshot", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => { (window as any).deferNextMediaInfo(); (window as any).mountStudio(); });
  await expect(page.getByRole("heading",{name:"Isle Studio"})).toBeVisible();
  await page.waitForFunction(() => (window as any).pendingMediaInfo.length === 1);
  await page.evaluate(payload => (window as any).media(payload),mediaFixtureState({nativeEpoch:"studio-epoch-a",nativeRevision:2,title:"Studio revision 2",positionMs:22_000}));
  await expect(page.locator(".marquee-title").filter({hasText:"Studio revision 2"})).toBeVisible();
  await page.evaluate(payload => (window as any).media(payload),mediaFixtureState({nativeEpoch:"studio-epoch-a",nativeRevision:1,title:"Studio stale revision 1",albumArt:"stale-cover",positionMs:1_000}));
  await page.evaluate(payload => (window as any).resolveMediaInfo(0,payload),mediaFixtureState({nativeEpoch:"studio-epoch-a",nativeRevision:1,title:"Studio stale command",albumArt:"stale-command-cover",positionMs:500}));
  await expect(page.locator(".marquee-title").filter({hasText:"Studio revision 2"})).toBeVisible();
  await expect(page.locator(".marquee-title").filter({hasText:/stale/})).toHaveCount(0);

  await page.evaluate(payload => (window as any).media(payload),mediaFixtureState({nativeEpoch:"studio-epoch-b",nativeRevision:0,title:"Studio new native epoch",positionMs:8_000}));
  await expect(page.locator(".marquee-title").filter({hasText:"Studio new native epoch"})).toBeVisible();
  await page.evaluate(payload => (window as any).media(payload),mediaFixtureState({nativeEpoch:"studio-epoch-a",nativeRevision:99,title:"Studio retired epoch",positionMs:99_000}));
  await expect(page.locator(".marquee-title").filter({hasText:"Studio new native epoch"})).toBeVisible();
  await expect(page.locator(".marquee-title").filter({hasText:/retired/})).toHaveCount(0);
  await page.evaluate(() => (window as any).destroy());
});

test("App timer fanout is state-change-only and sends absolute deadlines through pause/resume/adjust", async ({page}) => {
  await page.clock.install({time:new Date("2026-10-21T00:00:00.000Z")});
  await page.clock.pauseAt(new Date("2026-10-21T00:00:00.000Z"));
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => (window as any).mountApp());
  await page.clock.runFor(1_000);
  await expect.poll(() => page.evaluate(() => (window as any).timerStateDispatches.length)).toBeGreaterThan(0);
  const initialStateCount = await page.evaluate(() => (window as any).timerStateDispatches.length);
  await page.evaluate(() => (window as any).requestTimerSnapshot());
  await expect.poll(() => page.evaluate((count) => (window as any).timerStateDispatches.length,initialStateCount)).toBeGreaterThan(initialStateCount);
  const baseline = await page.evaluate(() => (window as any).timerStateDispatches.length);

  await page.evaluate(() => (window as any).timerAction("start",10_000));
  await expect.poll(() => page.evaluate((count) => (window as any).timerStateDispatches.length,baseline)).toBe(baseline+1);
  const started = await page.evaluate(() => (window as any).timerStateDispatches.at(-1));
  expect(started).toMatchObject({status:"running",remainingMs:10_000,endsAt:expect.any(Number),stateRevision:expect.any(Number)});
  const session = started.appSessionId;

  await page.clock.runFor(6_000);
  expect(await page.evaluate(() => (window as any).timerStateDispatches.length)).toBe(baseline+1);
  await page.evaluate(() => (window as any).timerAction("pause"));
  await expect.poll(() => page.evaluate(() => (window as any).timerStateDispatches.length)).toBe(baseline+2);
  const pausedState = await page.evaluate(() => (window as any).timerStateDispatches.at(-1));
  expect(pausedState).toMatchObject({status:"paused",endsAt:null,remainingMs:4_000,appSessionId:session});
  await page.clock.runFor(5_000);
  expect(await page.evaluate(() => (window as any).timerStateDispatches.length)).toBe(baseline+2);

  await page.evaluate(() => (window as any).timerAction("resume"));
  await expect.poll(() => page.evaluate(() => (window as any).timerStateDispatches.length)).toBe(baseline+3);
  const resumed = await page.evaluate(() => (window as any).timerStateDispatches.at(-1));
  expect(resumed).toMatchObject({status:"running",remainingMs:4_000,appSessionId:session});
  expect(resumed.endsAt).toBeGreaterThan(started.endsAt);
  await page.evaluate(() => (window as any).timerAction("adjust",2_000));
  await expect.poll(() => page.evaluate(() => (window as any).timerStateDispatches.length)).toBe(baseline+4);
  const adjusted = await page.evaluate(() => (window as any).timerStateDispatches.at(-1));
  expect(adjusted).toMatchObject({status:"running",remainingMs:6_000,appSessionId:session});
  expect(adjusted.endsAt).toBe(resumed.endsAt+2_000);
  expect(adjusted.stateRevision).toBeGreaterThan(resumed.stateRevision);
  await page.evaluate(() => (window as any).destroy());
});

test("TimerWindow uses endsAt after delayed delivery, hidden resume, and reopen", async ({page}) => {
  await page.clock.install({time:new Date("2026-10-21T00:00:00.000Z")});
  await page.clock.pauseAt(new Date("2026-10-21T00:00:00.000Z"));
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => (window as any).mountApp());
  await page.clock.runFor(1_000);
  await expect.poll(() => page.evaluate(() => (window as any).timerStateDispatches.length)).toBeGreaterThan(0);
  const initialStateCount = await page.evaluate(() => (window as any).timerStateDispatches.length);
  await page.evaluate(() => (window as any).requestTimerSnapshot());
  await expect.poll(() => page.evaluate((count) => (window as any).timerStateDispatches.length,initialStateCount)).toBeGreaterThan(initialStateCount);
  await page.evaluate(() => { (window as any).holdTimerStateEvents = true; (window as any).timerAction("start",10_000); });
  await expect.poll(() => page.evaluate(() => (window as any).pendingTimerStateEvents.length)).toBe(1);
  const startState = await page.evaluate(() => (window as any).pendingTimerStateEvents[0].payload);
  await page.evaluate(() => (window as any).mountTimer());
  await expect.poll(() => page.evaluate(() => (window as any).pendingTimerStateEvents.length)).toBe(2);
  await page.clock.runFor(6_000);
  await page.evaluate(() => (window as any).releaseTimerStateEvents());
  await expect(page.locator(".wheel-value.current")).toHaveText("00:04");
  expect(startState.endsAt).toBeGreaterThan(await page.evaluate(() => Date.now()));

  await page.evaluate(() => (window as any).setVisibility("hidden"));
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays().filter((delay:number) => delay <= 1_000))).toEqual([]);
  await page.clock.runFor(2_000);
  await page.evaluate(() => (window as any).setVisibility("visible"));
  await expect.poll(() => page.evaluate(() => (window as any).timerRequestMessages.length)).toBeGreaterThan(1);
  await expect(page.locator(".wheel-value.current")).toHaveText("00:02");

  await page.evaluate(() => (window as any).destroy());
  await page.clock.runFor(1_000);
  await page.evaluate(() => (window as any).mountTimer());
  await expect.poll(() => page.evaluate(() => (window as any).timerRequestMessages.length)).toBeGreaterThan(2);
  await expect.poll(() => page.evaluate(() => (window as any).pendingTimerStateEvents.length)).toBeGreaterThan(0);
  await page.evaluate(() => (window as any).releaseTimerStateEvents());
  await expect(page.locator(".wheel-value.current")).toHaveText("00:01");
  const shortTimeoutsBeforeUnmount = await page.evaluate(() => (window as any).liveTimeoutDelays().filter((delay:number) => delay <= 1_000).length);
  await page.evaluate(() => (window as any).destroy());
  const shortTimeoutsAfterUnmount = await page.evaluate(() => (window as any).liveTimeoutDelays().filter((delay:number) => delay <= 1_000).length);
  expect(shortTimeoutsAfterUnmount).toBeLessThan(shortTimeoutsBeforeUnmount);
});

test("TimerWindow schedules only display-second boundaries and stops while paused, hidden, zero, or unmounted", async ({page}) => {
  await page.clock.install({time:new Date("2026-10-20T00:00:00.000Z")});
  await page.clock.pauseAt(new Date("2026-10-20T00:00:00.000Z"));
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => (window as any).mountTimer());
  await expect.poll(() => page.evaluate(() => (window as any).timerRequestMessages.length)).toBeGreaterThan(0);
  const appSessionId = "timer-window-test-session";
  let revision = 0;
  const baseNow = await page.evaluate(() => Date.now());
  const state = (status:string,remainingMs:number,requestId?:string,now=baseNow) => ({
    status,durationMs:10_000,endsAt:status === "running" ? now+remainingMs : null,remainingMs,label:"Fixture",
    appSessionId,stateRevision:++revision,...(requestId ? {requestId} : {}),
  });

  const initialRequest = await page.evaluate(() => (window as any).timerRequestMessages.at(-1).requestId);
  await page.evaluate((value) => (window as any).timerState(value), state("running",5_500,initialRequest,baseNow));
  await expect(page.locator(".wheel-value.current")).toHaveText("00:06");
  const firstTimerDelay = await page.evaluate(() => (window as any).liveTimeoutDelays());
  expect(firstTimerDelay.some((delay:number) => delay > 0 && delay <= 500)).toBe(true);
  await page.clock.runFor(499);
  await expect(page.locator(".wheel-value.current")).toHaveText("00:06");
  await page.clock.runFor(1);
  await expect(page.locator(".wheel-value.current")).toHaveText("00:05");
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays())).toContain(1_000);

  await page.evaluate((value) => (window as any).timerState(value), state("paused",4_500));
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays())).toEqual([]);
  await page.evaluate((value) => (window as any).timerState(value), state("running",500,undefined,baseNow+500));
  await expect(page.locator(".wheel-value.current")).toHaveText("00:01");
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays())).toContain(500);

  await page.evaluate(() => {
    Object.defineProperty(document,"visibilityState",{configurable:true,value:"hidden"});
    document.dispatchEvent(new Event("visibilitychange"));
  });
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays())).toEqual([]);
  await page.clock.runFor(600);
  await page.evaluate(() => {
    Object.defineProperty(document,"visibilityState",{configurable:true,value:"visible"});
    document.dispatchEvent(new Event("visibilitychange"));
  });
  const visibleRequest = await page.evaluate(() => (window as any).timerRequestMessages.at(-1).requestId);
  await page.evaluate((value) => (window as any).timerState(value), state("idle",0,visibleRequest));
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays())).toEqual([]);

  await page.evaluate((value) => (window as any).timerState(value), state("running",5_500,undefined,baseNow+1_100));
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays())).toContain(500);

  await page.evaluate((payload) => (window as any).timerState(payload), {
    status:"paused",durationMs:10_000,endsAt:null,remainingMs:3_000,label:"New App session",
    appSessionId:"timer-window-new-session",stateRevision:1,sessionStart:true,
  });
  await expect(page.locator(".action-button.paused")).toBeVisible();
  await expect(page.locator(".wheel-value.current")).toHaveText("00:03");
  await page.evaluate((payload) => (window as any).timerState(payload), {
    status:"running",durationMs:10_000,endsAt:Date.now()+1_000,remainingMs:1_000,label:"Retired App session",
    appSessionId:"timer-window-test-session",stateRevision:99,
  });
  await expect(page.locator(".action-button.paused")).toBeVisible();
  await page.evaluate(() => (window as any).destroy());
  const firedAtUnmount = await page.evaluate(() => (window as any).firedTimeouts());
  expect(await page.evaluate(() => (window as any).liveTimeoutDelays())).toEqual([]);
  await page.clock.runFor(1_000);
  expect(await page.evaluate(() => (window as any).firedTimeouts())).toBe(firedAtUnmount);
});

test("App refreshes clock text by minute and ticks media progress only while visible and playing", async ({page}) => {
  await page.clock.install({time:new Date("2026-10-21T12:00:00.000Z")});
  await page.clock.pauseAt(new Date("2026-10-21T12:00:00.000Z"));
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture=window as any;
    fixture.settingsOverrides={clockTimeZone:"UTC",captureHideOnScreenshot:true};
    fixture.mountApp();
  });
  await page.clock.runFor(1_000);
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBeGreaterThan(0);
  const clock=page.locator(".idle-clock .rolling-number-sr");
  await expect(clock).toHaveText("12:00");
  const formatterCalls=await page.evaluate(() => (window as any).clockFormatterCalls());
  const minuteDelay=await page.evaluate(() => (window as any).liveTimeoutDelays().find((delay:number)=>delay>50_000&&delay<=60_000));
  expect(minuteDelay).toBeGreaterThan(50_000);
  await page.clock.runFor(minuteDelay-2_000);
  await expect(clock).toHaveText("12:00");
  expect(await page.evaluate(() => (window as any).clockFormatterCalls())).toBe(formatterCalls);
  await page.clock.runFor(2_000);
  await expect(clock).toHaveText("12:01");
  await expect.poll(() => page.evaluate(() => (window as any).clockFormatterCalls())).toBe(formatterCalls+1);

  await page.evaluate(() => (window as any).updateSettings({clockTimeZone:"Asia/Tokyo"}));
  await expect(clock).toHaveText("21:01");
  await expect.poll(() => page.evaluate(() => (window as any).clockFormatterCalls())).toBe(formatterCalls+2);
  const media=(isPlaying:boolean)=>({title:"Clock fixture",artist:"Fixture",albumArt:"",isPlaying,positionMs:0,durationMs:180_000,lastUpdatedTimestamp:Date.now(),source:"generic",sourceDisplay:"Fixture"});
  await page.evaluate((payload) => (window as any).media(payload),media(true));
  await page.evaluate(() => { (window as any).expand(); (window as any).hoverIsland(); });
  await expect.poll(() => page.evaluate(() => (window as any).islandSnapshot())).toMatchObject({expanded:true,hovering:true});
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays().includes(250))).toBe(true);
  await page.clock.runFor(2_500);
  await expect(page.locator(".progress-block .time.elapsed .rolling-number-sr")).toHaveText("0:02");
  expect(await page.evaluate(() => (window as any).clockFormatterCalls())).toBe(formatterCalls+2);

  await page.evaluate(() => (window as any).captureMode(true));
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays().includes(250))).toBe(false);
  await page.clock.runFor(5_000);
  await page.evaluate(() => (window as any).captureMode(false));
  await expect.poll(() => page.locator(".progress-block .time.elapsed .rolling-number-sr").textContent()).toBe("0:07");
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays().includes(250))).toBe(true);

  await page.evaluate(() => (window as any).setVisibility("hidden"));
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays().includes(250))).toBe(false);
  await page.clock.runFor(5_000);
  await page.evaluate(() => (window as any).setVisibility("visible"));
  await expect(page.locator(".progress-block .time.elapsed .rolling-number-sr")).toHaveText("0:12");
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays().includes(250))).toBe(true);
  await page.evaluate((payload) => (window as any).media(payload),media(false));
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays().includes(250))).toBe(false);
  await page.evaluate(() => (window as any).destroy());
  expect(await page.evaluate(() => (window as any).liveTimeoutDelays())).toEqual([]);
});

test("App completes a hidden countdown at its absolute deadline exactly once", async ({page}) => {
  await page.clock.install({time:new Date("2026-10-21T12:00:00.000Z")});
  await page.clock.pauseAt(new Date("2026-10-21T12:00:00.000Z"));
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture=window as any;
    fixture.settingsOverrides={captureHideOnScreenshot:true};
    fixture.mountApp();
  });
  await page.clock.runFor(1_000);
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBeGreaterThan(0);
  const timerStateBaseline = await page.evaluate(() => (window as any).timerStateDispatches.length);
  await page.evaluate(() => (window as any).timerAction("start",60_000));
  await expect.poll(() => page.evaluate(() => (window as any).timerStateDispatches.length)).toBe(timerStateBaseline+1);
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays().some((delay:number)=>delay>0&&delay<=1_000))).toBe(true);

  await page.evaluate(() => (window as any).captureMode(true));
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays().includes(250))).toBe(false);
  await expect.poll(() => page.evaluate(() => (window as any).liveTimeoutDelays().some((delay:number)=>delay>59_000&&delay<=60_000))).toBe(true);
  await page.clock.runFor(60_000);
  await expect.poll(() => page.evaluate(() => (window as any).timerStateDispatches.length)).toBe(timerStateBaseline+2);
  await page.evaluate(() => (window as any).captureMode(false));
  await expect(page.locator(".timer-complete-card")).toBeVisible();
  expect(await page.evaluate(() => (window as any).completionCardAdds())).toBe(1);
  await page.clock.runFor(10_000);
  expect(await page.evaluate(() => (window as any).timerStateDispatches.length)).toBe(timerStateBaseline+2);
  await expect(page.locator(".timer-complete-card")).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).completionCardAdds())).toBe(1);
  await page.clock.runFor(10_000);
  expect(await page.evaluate(() => (window as any).completionCardAdds())).toBe(1);
  await page.evaluate(() => (window as any).destroy());
  expect(await page.evaluate(() => (window as any).liveTimeoutDelays())).toEqual([]);
});

test("Studio polls media sessions only while Players is visible and serializes stale requests", async ({page}) => {
  await page.setViewportSize({width:900,height:280});
  await page.clock.install({time:new Date("2026-10-21T00:00:00.000Z")});
  await page.clock.pauseAt(new Date("2026-10-21T00:00:00.000Z"));
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture=window as any;
    fixture.holdMediaSessions=true;
    fixture.mountStudio();
  });
  await expect(page.getByRole("heading",{name:"Isle Studio"})).toBeVisible();
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((command:string)=>command==="get_settings").length)).toBeGreaterThan(0);
  const players=page.locator(".players-section");
  expect(await players.evaluate(el=>el.getBoundingClientRect().top>=innerHeight)).toBe(true);
  await page.clock.runFor(100);
  expect(await page.evaluate(() => (window as any).calls.filter((command:string)=>command==="list_media_sessions").length)).toBe(0);
  expect(await page.evaluate(() => (window as any).activeIntervals(10_000))).toBe(0);

  await players.scrollIntoViewIfNeeded();
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((command:string)=>command==="list_media_sessions").length)).toBe(1);
  expect(await page.evaluate(() => (window as any).activeIntervals(10_000))).toBe(1);
  await page.clock.runFor(10_000);
  expect(await page.evaluate(() => (window as any).calls.filter((command:string)=>command==="list_media_sessions").length)).toBe(1);
  await page.evaluate(() => (window as any).resolveMediaSessions(0,[{id:"stale-before-leave",displayName:"Stale before leave",source:"generic",isPlaying:false}]));
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((command:string)=>command==="list_media_sessions").length)).toBe(2);

  await page.evaluate(() => window.scrollTo(0,0));
  await page.setViewportSize({width:900,height:200});
  await expect.poll(() => page.evaluate(() => (window as any).activeIntervals(10_000))).toBe(0);
  await page.clock.runFor(20_000);
  expect(await page.evaluate(() => (window as any).calls.filter((command:string)=>command==="list_media_sessions").length)).toBe(2);
  await page.evaluate(() => (window as any).resolveMediaSessions(1,[{id:"stale-after-leave",displayName:"Stale after leave",source:"generic",isPlaying:false}]));
  await page.clock.runFor(0);
  await expect(players.getByText("Stale after leave")).toHaveCount(0);

  await players.scrollIntoViewIfNeeded();
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((command:string)=>command==="list_media_sessions").length)).toBe(3);
  await page.evaluate(() => (window as any).resolveMediaSessions(2,[{id:"fresh-session",displayName:"Fresh session",source:"generic",isPlaying:false}]));
  await expect(players.getByText("Fresh session")).toBeVisible();
  await page.evaluate(() => { (window as any).holdMediaSessions=false; });
  await players.locator(".icon-button").click();
  await expect.poll(() => page.evaluate(() => (window as any).calls.filter((command:string)=>command==="list_media_sessions").length)).toBe(4);

  await page.evaluate(() => (window as any).destroy());
  expect(await page.evaluate(() => (window as any).activeIntervals(10_000))).toBe(0);
  expect(await page.evaluate(() => (window as any).intersectionObserverRows.at(-1).disconnected)).toBe(true);
  await page.clock.runFor(20_000);
  expect(await page.evaluate(() => (window as any).calls.filter((command:string)=>command==="list_media_sessions").length)).toBe(4);
});

test("App keeps compact playing-dot cover colors and skips extraction with no visible consumer", async ({page}) => {
  const cover = (title:string, color:string) => ({
    title,artist:"Fixture",albumArt:`data:image/png;base64,${color}`,isPlaying:true,
    positionMs:1_000,durationMs:60_000,lastUpdatedTimestamp:Date.now(),source:"generic",sourceDisplay:"Fixture",
  });
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const f = window as any;
    f.settingsOverrides = {showSpectrum:false,enableHdCover:false,captureHideOnScreenshot:true};
    f.trackSpectrumExtraction();
    f.mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBeGreaterThan(0);
  await page.evaluate(() => (window as any).updateSettings({showSpectrum:false,enableHdCover:false}));
  await page.waitForTimeout(50);
  await expect(page.locator(".compact-layer")).toHaveCount(1);

  await page.evaluate((payload) => (window as any).media(payload), cover("Spectrum off","png-a"));
  await page.waitForTimeout(550);
  await expect.poll(() => page.evaluate(() => (window as any).spectrumLoadSources().length)).toBe(1);
  expect(await page.evaluate(() => (window as any).spectrumLoadSources())).toEqual([cover("Spectrum off","png-a").albumArt]);
  await expect(page.locator(".playing-dot")).toBeVisible();
  await page.evaluate(() => (window as any).completeSpectrumLoad(0));
  await expect.poll(() => page.locator(".playing-dot").evaluate(el => getComputedStyle(el).backgroundColor)).not.toBe("rgb(255, 255, 255)");

  await page.evaluate(() => (window as any).expand());
  await expect.poll(() => page.evaluate(() => (window as any).islandSnapshot())).toMatchObject({expanded:true,activePage:"music"});
  await page.evaluate((payload) => (window as any).media(payload), cover("No visible color consumer","png-b"));
  await page.waitForTimeout(550);
  expect(await page.evaluate(() => (window as any).spectrumLoadSources())).toEqual([cover("Spectrum off","png-a").albumArt]);

  await page.evaluate(() => { (window as any).selectIslandPage("music"); (window as any).collapse(); });
  await expect.poll(() => page.evaluate(() => (window as any).islandSnapshot())).toMatchObject({expanded:false,activePage:"music"});
  await page.evaluate((payload) => (window as any).media(payload), cover("Compact consumer","png-c"));
  await page.waitForTimeout(550);
  await expect.poll(() => page.evaluate(() => (window as any).spectrumLoadSources().filter((src:string) => src.endsWith("png-c")).length)).toBeGreaterThan(0);
  const compactLoadCount = await page.evaluate(() => (window as any).spectrumLoadSources().length);
  await page.evaluate(() => (window as any).captureMode(true));
  await expect(page.locator(".compact-layer")).toHaveCount(0);
  await page.evaluate((payload) => (window as any).media(payload), cover("Hidden cover","png-d"));
  await page.waitForTimeout(550);
  expect(await page.evaluate(() => (window as any).spectrumLoadSources().length)).toBe(compactLoadCount);

  await page.evaluate(() => (window as any).captureMode(false));
  await expect(page.locator(".compact-layer")).toHaveCount(1);
  await expect.poll(() => page.evaluate(() => (window as any).spectrumLoadSources().filter((src:string) => src.endsWith("png-d")).length)).toBe(1);
  const restoredSources = await page.evaluate(() => (window as any).spectrumLoadSources() as string[]);
  expect(restoredSources.at(-1)).toBe(cover("Hidden cover","png-d").albumArt);
  for (let index=1;index<compactLoadCount;index++) {
    if (restoredSources[index].endsWith("png-c")) await page.evaluate((i) => (window as any).completeSpectrumLoad(i),index);
  }
  expect(await page.evaluate(() => (window as any).spectrumDrawCount)).toBe(1);
  await page.evaluate((index) => (window as any).completeSpectrumLoad(index),restoredSources.length-1);
  await expect.poll(() => page.evaluate(() => (window as any).spectrumDrawCount)).toBe(2);

  await page.evaluate(() => (window as any).setVisibility("hidden"));
  await expect(page.locator(".compact-layer")).toHaveCount(0);
  await page.evaluate((payload) => (window as any).media(payload), cover("Page hidden","png-e"));
  await page.waitForTimeout(550);
  expect(await page.evaluate(() => (window as any).spectrumLoadSources().length)).toBe(restoredSources.length);
  await page.evaluate(() => (window as any).setVisibility("visible"));
  await expect(page.locator(".compact-layer")).toHaveCount(1);
  await expect.poll(() => page.evaluate(() => (window as any).spectrumLoadSources().filter((src:string) => src.endsWith("png-e")).length)).toBe(1);
  const visibleSources = await page.evaluate(() => (window as any).spectrumLoadSources() as string[]);
  expect(visibleSources.at(-1)).toBe(cover("Page hidden","png-e").albumArt);
  await page.evaluate((index) => (window as any).completeSpectrumLoad(index),visibleSources.length-1);
  await expect.poll(() => page.evaluate(() => (window as any).spectrumDrawCount)).toBe(3);
  await page.evaluate(() => (window as any).destroy());
});

test("App retries the full same-track cover after a media sync emit failure", async ({page}) => {
  const cover = "data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=";
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const fixture = window as any;
    fixture.settingsOverrides = { enableHdCover: false, captureHideOnScreenshot: true };
    fixture.rejectNextMediaSync = true;
    fixture.mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBeGreaterThan(0);
  await page.evaluate(() => (window as any).activateFloatingConsumer());

  await page.evaluate((albumArt) => (window as any).media({
    title: "Sync retry track", artist: "Fixture", albumArt, isPlaying: false,
    positionMs: 1_000, durationMs: 60_000, lastUpdatedTimestamp: Date.now(),
    source: "generic", sourceDisplay: "Fixture",
  }), cover);
  await expect.poll(() => page.evaluate(() => (window as any).mediaSyncFailures)).toBe(1);

  const sameTrackWithoutArt = {
    title: "Sync retry track", artist: "Fixture", isPlaying: false,
    positionMs: 1_500, durationMs: 60_000, lastUpdatedTimestamp: Date.now(),
    source: "generic", sourceDisplay: "Fixture",
  };
  await page.evaluate((payload) => (window as any).media(payload), sameTrackWithoutArt);
  await expect.poll(() => page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBe(2);
  const sent = await page.evaluate(() => (window as any).mediaSyncDispatches as Array<Record<string, unknown>>);
  expect(sent[0].albumArt).toBe(cover);
  expect(sent[1].albumArt).toBe(cover);
  expect(sent[1]).toMatchObject({ title: "Sync retry track", positionMs: 1_500 });
  await page.evaluate(() => (window as any).destroy());
});

test("Floating announces ready after an unavailable initial snapshot and receives a full same-track cover", async ({page}) => {
  const cover = "data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=";
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    (window as any).__ISLE_MEDIA_TRACE__ = [];
    (window as any).settingsOverrides = { enableHdCover: false, captureHideOnScreenshot: true };
    (window as any).mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBeGreaterThan(0);
  await expect.poll(() => page.evaluate(() => (window as any).floatingReadyListenerCount())).toBe(1);

  const state = {
    title: "Ready handshake track", artist: "Fixture", albumArt: cover,
    isPlaying: false, positionMs: 2_000, durationMs: 60_000,
    lastUpdatedTimestamp: Date.now(), source: "generic", sourceDisplay: "Fixture",
  };
  await page.evaluate((payload) => (window as any).media(payload), state);
  await page.waitForTimeout(25);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBe(0);

  await page.evaluate(() => {
    (window as any).failMediaInfoSnapshot = true;
    (window as any).mountFloatingPeer({ enableHDCover: false });
  });
  await expect.poll(() => page.evaluate(() => (window as any).floatingReadyMessages.length)).toBeGreaterThan(0);
  await expect.poll(() => page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBeGreaterThan(0);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.at(-1).albumArt)).toBe(cover);
  await expect(page.locator("#floating-peer-0 .track-title")).toHaveText("Ready handshake track");
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row: any) =>
    row.origin === "island-sync" && row.coverPresent && row.coverChars > 0,
  ))).toBe(true);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.at(-1).albumArt)).toBe(cover);
  const instanceToken = await page.evaluate(() => (window as any).floatingReadyMessages.at(-1).instanceToken);
  const beforeHide = await page.evaluate(() => (window as any).mediaSyncDispatches.length);
  await page.evaluate(() => (window as any).setVisibility("hidden"));
  await expect.poll(() => page.evaluate((token) => (window as any).floatingNotReadyMessages.some((m:any) => m.instanceToken === token && m.reason === "hidden"), instanceToken)).toBe(true);
  await page.evaluate((payload) => (window as any).media({...payload,positionMs:3_000}), state);
  await page.waitForTimeout(25);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBe(beforeHide);
  await page.evaluate(() => (window as any).setVisibility("visible"));
  await expect.poll(() => page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBeGreaterThan(beforeHide);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.at(-1).albumArt)).toBe(cover);
  await page.evaluate(async () => {
    await (window as any).destroyFloatingPeer(0);
    await (window as any).destroy();
  });
});

test("App reload waits for native media before replaying a ready Floating instance", async ({page}) => {
  const cover = "data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=";
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    (window as any).__ISLE_MEDIA_TRACE__ = [];
    (window as any).settingsOverrides = { enableHdCover: false, captureHideOnScreenshot: true };
    (window as any).mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).mediaListenerCount())).toBeGreaterThan(0);
  await page.evaluate(() => (window as any).mountFloatingPeer({ enableHDCover: false }));
  await expect.poll(() => page.evaluate(() => (window as any).floatingReadyMessages.length)).toBeGreaterThan(0);

  const state = {
    title: "Reload handshake track", artist: "Fixture", albumArt: cover,
    isPlaying: true, positionMs: 4_000, durationMs: 60_000,
    lastUpdatedTimestamp: Date.now(), source: "generic", sourceDisplay: "Fixture",
  };
  await page.evaluate((payload) => (window as any).media(payload), state);
  await expect.poll(() => page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBeGreaterThan(0);
  const oldSessionPayload = await page.evaluate(() => (window as any).mediaSyncDispatches.at(-1));

  await page.evaluate(() => (window as any).destroy());
  const queryCount = await page.evaluate(() => (window as any).floatingReadyQueries.length);
  const dispatchCount = await page.evaluate(() => (window as any).mediaSyncDispatches.length);
  await page.evaluate(() => {
    (window as any).holdReadyQueries = true;
    (window as any).mountApp();
  });
  await expect.poll(() => page.evaluate(() => (window as any).floatingReadyQueries.length)).toBeGreaterThan(queryCount);
  await page.evaluate(() => (window as any).releaseReadyQueries());
  await page.waitForTimeout(50);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBe(dispatchCount);
  const currentAppSession = await page.evaluate(() => (window as any).floatingReadyMessages.at(-1)?.appSessionId);
  expect(currentAppSession).toEqual(expect.any(String));
  expect(currentAppSession).not.toBe(oldSessionPayload.appSessionId);
  await page.evaluate((payload) => (window as any).mediaSync(payload), oldSessionPayload);
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row:any) => row.outcome === "dropped-retired-app-session"))).toBe(true);
  await expect(page.locator("#floating-peer-0 .track-title")).toHaveText("Reload handshake track");

  await page.evaluate((payload) => (window as any).media(payload), { ...state, positionMs: 5_000 });
  await expect.poll(() => page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBeGreaterThan(dispatchCount);
  const tail = await page.evaluate((count) => (window as any).mediaSyncDispatches.slice(count) as Array<Record<string, unknown>>, dispatchCount);
  expect(tail.at(-1)?.albumArt).toBe(cover);
  expect(tail.at(-1)?.positionMs).toBeGreaterThanOrEqual(5_000);
  expect(tail.at(-1)?.positionMs).toBeLessThan(5_100);
  await expect.poll(() => page.evaluate(() => (window as any).__ISLE_MEDIA_TRACE__.some((row: any) => row.origin === "island-sync" && row.coverPresent))).toBe(true);

  const beforeEmpty = await page.evaluate(() => (window as any).mediaSyncDispatches.length);
  await page.evaluate(() => (window as any).media({ source: "", title: "", artist: "", isPlaying: false, positionMs: 0, durationMs: 0 }));
  await expect.poll(() => page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBeGreaterThan(beforeEmpty);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.at(-1))).toMatchObject({ source: "", title: "", albumArt: "" });
  await expect(page.locator("#floating-peer-0 .track-title")).not.toHaveText("Reload handshake track");

  const afterEmpty = await page.evaluate(() => (window as any).mediaSyncDispatches.length);
  await page.evaluate(() => (window as any).destroy());
  await page.evaluate((payload) => (window as any).media(payload), state);
  await page.waitForTimeout(50);
  expect(await page.evaluate(() => (window as any).mediaSyncDispatches.length)).toBe(afterEmpty);
  await page.evaluate(async () => {
    await (window as any).destroyFloatingPeer(0);
  });
});

test("Floating unmount during its first snapshot never announces a stale ready token", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    (window as any).deferNextMediaInfo();
    (window as any).mountFloatingPeer({ enableHDCover: false });
  });
  await page.waitForFunction(() => (window as any).pendingMediaInfo.length === 1);
  await page.evaluate(async () => {
    await (window as any).destroyFloatingPeer(0);
    (window as any).resolveMediaInfo(0, { title: "Too late", artist: "Fixture", albumArt: "late", source: "fixture" });
  });
  await page.waitForTimeout(50);
  expect(await page.evaluate(() => (window as any).floatingReadyMessages.length)).toBe(0);
});

test("Studio preserves playing-dot cover colors and extracts spectrum colors only when visible", async ({page}) => {
  await page.goto("/ui-tests/lifecycle-fixture.html");
  await page.waitForFunction(() => (window as any).ready);
  await page.evaluate(() => {
    const f=window as any;
    f.mediaInfoSnapshot={title:"Studio preview",artist:"Fixture",albumArt:"data:image/png;base64,studio",isPlaying:true,positionMs:0,durationMs:60_000,lastUpdatedTimestamp:Date.now(),source:"generic",sourceDisplay:"Fixture"};
    f.trackSpectrumExtraction();
    f.mountStudio();
  });
  await expect(page.getByRole("heading",{name:"Isle Studio"})).toBeVisible();
  expect(await page.evaluate(() => (window as any).spectrumLoadSources().length)).toBe(0);

  const showSpectrum = page.getByRole("button",{name:/显示频谱|Show spectrum|スペクトラムを表示/});
  const previewStates = page.locator(".preview-state .segmented button");
  await previewStates.nth(0).click(); // Compact mode exposes the playing-dot color consumer.
  await expect.poll(() => page.evaluate(() => (window as any).spectrumLoadSources().length)).toBe(1);
  await expect(page.locator(".preview-host .playing-dot")).toBeVisible();
  await page.evaluate(() => (window as any).completeSpectrumLoad(0));
  await expect.poll(() => page.evaluate(() => (window as any).spectrumDrawCount)).toBe(1);
  await expect.poll(() => page.locator(".preview-host .playing-dot").evaluate(el => getComputedStyle(el).backgroundColor)).not.toBe("rgb(255, 255, 255)");

  await previewStates.nth(2).click(); // Expanded music has no color consumer while the spectrum is disabled.
  expect(await page.evaluate(() => (window as any).spectrumLoadSources().length)).toBe(1);
  await showSpectrum.click();
  await expect(showSpectrum).toHaveAttribute("aria-pressed","true");
  await expect.poll(() => page.evaluate(() => (window as any).spectrumLoadSources().length)).toBe(2);
  await previewStates.nth(3).click(); // A hidden preview aborts this spectrum request.
  await expect(previewStates.nth(3)).toHaveAttribute("aria-pressed","true");
  await page.evaluate(() => (window as any).completeSpectrumLoad(1));
  expect(await page.evaluate(() => (window as any).spectrumDrawCount)).toBe(1);

  await previewStates.nth(0).click();
  await expect.poll(() => page.evaluate(() => (window as any).spectrumLoadSources().length)).toBe(3);
  await page.evaluate(() => (window as any).completeSpectrumLoad(2));
  await expect.poll(() => page.evaluate(() => (window as any).spectrumDrawCount)).toBe(2);

  await page.evaluate(() => (window as any).setVisibility("hidden"));
  await expect(page.getByRole("heading",{name:"Isle Studio"})).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).spectrumLoadSources().length)).toBe(3);
  await page.evaluate(() => (window as any).setVisibility("visible"));
  await expect(page.getByRole("heading",{name:"Isle Studio"})).toBeVisible();
  expect(await page.evaluate(() => (window as any).spectrumLoadSources().length)).toBe(3); // Mocked settings remount expanded with the spectrum disabled.
  await page.evaluate(() => (window as any).destroy());
});

import { mount, unmount, tick } from "svelte";
import { mockConvertFileSrc, mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { DEFAULT_SETTINGS } from "../src/lib/api/types";
import { settingsStore } from "../src/lib/settingsStore";
import { get } from "svelte/store";
import { emit } from "@tauri-apps/api/event";
import { islandState, transitionIsland } from "../src/lib/islandStore";
import App from "../src/App.svelte";
import RollingFixture from "./RollingFixture.svelte";
import NeteaseLikeControl from "../src/lib/components/media/NeteaseLikeControl.svelte";
import FloatingWindow from "../src/FloatingWindow.svelte";
import Studio from "../src/Studio.svelte";
import TimerWindow from "../src/TimerWindow.svelte";
import VolumePanel from "../src/lib/components/volume/VolumePanel.svelte";
import VisibleWindow from "../src/lib/components/island/VisibleWindow.svelte";
import { eventManager } from "../src/utils/eventManager";
import "../src/app.css";
import { Events } from "../src/utils/eventConstants";
import { enableTransportDiagnostics, disableTransportDiagnostics, resetTransportDiagnostics, snapshotTransportDiagnostics } from "../src/lib/transportDiagnostics";
import { isTransportDiagnosticsEnabled, runTimedTransportCommand } from "../src/lib/transportDiagnostics";
import { LatestValueWriteQueue } from "../src/lib/latestValueWriteQueue";
const fixture = window as any;
fixture.calls = [];
fixture.likeState = { trackId: "123", playId: "123_play", title: "Like Song", artist: "Artist", liked: false };
fixture.likeActions = [];
fixture.likeWriteFail = false;
fixture.likeReadFailures = 0;
fixture.neteaseSetupCalls = [];
fixture.openApplicationCalls = [];
fixture.reopenedMedia = undefined;
let likeComponent: ReturnType<typeof mount> | undefined;
fixture.mountLike = () => { likeComponent = mount(NeteaseLikeControl, { target: document.body, props: { title: "Like Song", artist: "Artist" } }); };
fixture.unmountLike = async () => { if (likeComponent) await unmount(likeComponent); likeComponent = undefined; };
fixture.deferCoverArtwork = false;
fixture.pendingCoverArtwork = [];
fixture.animations = [];
fixture.videoCalls = [];
fixture.settingsOverrides = {};
fixture.deferSettingsReads = false;
fixture.pendingSettingsReads = [];
fixture.monitorList = [{width:1920,height:1080,x:0,y:0,workX:0,workY:0,workWidth:1920,workHeight:1080,scaleFactor:1}];
fixture.availableMonitorList = [{name:"test",position:{x:0,y:0},size:{width:1920,height:1080},scaleFactor:1}];
fixture.nativeMonitorIndex = 0;
fixture.mediaInfoSnapshot = undefined;
fixture.mediaSyncDispatches = [];
fixture.mediaSyncFailures = 0;
fixture.rejectNextMediaSync = false;
fixture.rejectNextSeek = false;
fixture.disableSongMetadataRecovery = false;
fixture.failMediaInfoSnapshot = false;
fixture.floatingReadyMessages = [];
fixture.floatingNotReadyMessages = [];
fixture.floatingReadyQueries = [];
fixture.holdReadyQueries = false;
fixture.pendingReadyQueries = [];
fixture.deferMediaInfo = false;
fixture.pendingMediaInfo = [];
fixture.holdMediaSessions = false;
fixture.pendingMediaSessions = [];
fixture.mediaSessionsResult = [];
fixture.intersectionObserverRows = [];
fixture.clockFormatterCount = 0;
fixture.completionCardCount = 0;
fixture.timerStateDispatches = [];
fixture.timerRequestMessages = [];
fixture.volumeWrites = [];
fixture.holdTimerStateEvents = false;
fixture.pendingTimerStateEvents = [];
fixture.holdRegionCommands = false;
fixture.pendingRegionCommands = [];
fixture.trackVideoCalls = false;
fixture.nextVideoPlay = undefined;
fixture.intervalRows = [];
fixture.timeoutRows = [];
fixture.spectrumImageRows = [];
fixture.spectrumCanvasCount = 0;
fixture.spectrumDrawCount = 0;
const intervals = new Map<number, { delay: number; active: boolean }>();
const timeouts = new Map<number, { delay: number; active: boolean; fired: number }>();
const nativeSetInterval = window.setInterval.bind(window);
const nativeClearInterval = window.clearInterval.bind(window);
const nativeSetTimeout = window.setTimeout.bind(window);
const nativeClearTimeout = window.clearTimeout.bind(window);
const NativeIntersectionObserver = window.IntersectionObserver;
window.IntersectionObserver = class extends NativeIntersectionObserver {
  private row:{target?:Element;disconnected:boolean}={disconnected:false};
  constructor(callback:IntersectionObserverCallback,options?:IntersectionObserverInit){
    super(callback,options);
    fixture.intersectionObserverRows.push(this.row);
  }
  observe(target:Element){this.row.target=target;super.observe(target)}
  disconnect(){this.row.disconnected=true;super.disconnect()}
} as typeof IntersectionObserver;
const NativeDateTimeFormat = Intl.DateTimeFormat;
Intl.DateTimeFormat = new Proxy(NativeDateTimeFormat,{
  construct(target,args,newTarget){
    const options=args[1] as Intl.DateTimeFormatOptions|undefined;
    if(options?.hour==="2-digit"&&options?.minute==="2-digit")fixture.clockFormatterCount++;
    return Reflect.construct(target,args,newTarget);
  },
}) as typeof Intl.DateTimeFormat;
const completionObserver = new MutationObserver(records=>{
  for(const record of records)for(const node of record.addedNodes){
    if(node instanceof HTMLElement&&(node.matches(".timer-complete-card")||node.querySelector(".timer-complete-card")))fixture.completionCardCount++;
  }
});
completionObserver.observe(document.getElementById("app")!,{childList:true,subtree:true});
window.setInterval = ((handler: TimerHandler, timeout?: number, ...args: any[]) => {
  const id = nativeSetInterval(handler, timeout, ...args);
  const row = { delay: Number(timeout ?? 0), active: true };
  fixture.intervalRows.push(row);
  intervals.set(id, row);
  return id;
}) as typeof window.setInterval;
window.clearInterval = ((id: number) => {
  const row = intervals.get(id);
  if (row) row.active = false;
  nativeClearInterval(id);
}) as typeof window.clearInterval;
window.setTimeout = ((handler: TimerHandler, timeout?: number, ...args: any[]) => {
  const row = { delay: Number(timeout ?? 0), active: true, fired: 0 };
  if (row.delay === 5_000) {
    const imageRow = [...fixture.spectrumImageRows].reverse().find((item:{pendingSource:boolean;loadAttempt:boolean}) => item.pendingSource);
    if (imageRow && !imageRow.visualOnly) imageRow.loadAttempt = true;
  }
  const wrapped = typeof handler === "function" ? (...callbackArgs: any[]) => {
    row.active = false;
    row.fired++;
    handler(...callbackArgs);
  } : handler;
  const id = nativeSetTimeout(wrapped, timeout, ...args);
  fixture.timeoutRows.push(row);
  timeouts.set(id, row);
  return id;
}) as typeof window.setTimeout;
window.clearTimeout = ((id: number) => {
  const row = timeouts.get(id);
  if (row) row.active = false;
  nativeClearTimeout(id);
}) as typeof window.clearTimeout;
const videoPaused = new WeakMap<HTMLMediaElement, boolean>();
const nativeVideoPlay = HTMLMediaElement.prototype.play;
const nativeVideoPause = HTMLMediaElement.prototype.pause;
const nativePausedGetter = Object.getOwnPropertyDescriptor(HTMLMediaElement.prototype, "paused")?.get;
HTMLMediaElement.prototype.play = function() {
  if (!fixture.trackVideoCalls) return nativeVideoPlay.call(this);
  fixture.lastVideo = this;
  fixture.videoCalls.push({ kind: "play", video: this });
  const pending: Promise<void> | undefined = fixture.nextVideoPlay;
  fixture.nextVideoPlay = undefined;
  return Promise.resolve(pending).then(() => { videoPaused.set(this, false); });
};
HTMLMediaElement.prototype.pause = function() {
  if (!fixture.trackVideoCalls) return nativeVideoPause.call(this);
  fixture.videoCalls.push({ kind: "pause", video: this });
  videoPaused.set(this, true);
};
Object.defineProperty(HTMLMediaElement.prototype, "paused", {
  configurable: true,
  get() {
    return fixture.trackVideoCalls ? videoPaused.get(this) ?? true : nativePausedGetter?.call(this) ?? true;
  },
});
const animate = Element.prototype.animate;
Element.prototype.animate = function(...args) {
  const animation = animate.apply(this, args);
  const row = {firstOpacity:(args[0] as Keyframe[])[0].opacity,cancelled:0};
  fixture.animations.push(row);
  const cancel = animation.cancel.bind(animation);
  animation.cancel = () => { row.cancelled++; cancel(); };
  return animation;
};
mockWindows("main");
mockConvertFileSrc("windows");
mockIPC((command, args) => {
  fixture.calls.push(command);
  if (command === "setup_netease_connection") {
    fixture.neteaseSetupCalls.push(args);
    return {connected:false,configured:true,executablePath:"C:\\NetEase\\cloudmusic.exe",shortcutPath:"",boundSessionId:null,bindingFailed:false,launchEntries:16};
  }
  if (command === "open_application") {
    fixture.openApplicationCalls.push(args);
    if (fixture.reopenedMedia) fixture.mediaInfoSnapshot = fixture.reopenedMedia;
    return null;
  }
  if (command === "get_netease_playback_mode") return "repeat_list";
  if (command === "netease_song_like") {
    if (typeof args?.liked !== "boolean" && fixture.likeReadFailures > 0) {
      fixture.likeReadFailures--;
      return Promise.reject(new Error("player starting"));
    }
    if (typeof args?.liked === "boolean") {
      fixture.likeActions.push(args);
      if (fixture.likeWriteFail) return Promise.reject(new Error("like failed"));
      fixture.likeState = { ...fixture.likeState, liked: args.liked };
    }
    return { ...fixture.likeState };
  }
  if (command === "get_settings") {
    const snapshot = {...DEFAULT_SETTINGS,showSpectrum:false,idleContentEnabled:false,...fixture.settingsOverrides};
    if (fixture.deferSettingsReads) {
      return new Promise(resolve => fixture.pendingSettingsReads.push((value:unknown) => resolve(value ?? snapshot)));
    }
    return snapshot;
  }
  if (command === "get_system_audio_state") return {deviceId:"fixture-device",deviceName:"Fixture",volumePercent:50,muted:false};
  if (command === "list_audio_output_devices") return [{id:"fixture-device",name:"Fixture",isDefault:true}];
  if (command === "set_island_interaction_region" && fixture.holdRegionCommands) {
    return new Promise(resolve => fixture.pendingRegionCommands.push(resolve));
  }
  if (command === "get_monitors") return fixture.monitorList;
  if (command === "get_current_monitor_index") return fixture.nativeMonitorIndex;
  if (command === "plugin:window|available_monitors") return fixture.availableMonitorList;
  if (command === "get_weather_snapshot") return {weatherTemperature:null,weatherCode:null,weatherUpdatedAt:null,weatherForecast:[]};
  if (command === "get_idle_snapshot") return {cpuPercent:12,memoryPercent:34,uploadBytesPerSecond:56,downloadBytesPerSecond:78,batteryPercent:90,batteryCharging:true,weatherTemperature:null,weatherCode:null,weatherUpdatedAt:null,weatherForecast:[]};
  if (command === "get_media_info_cmd") {
    if (fixture.failMediaInfoSnapshot) return Promise.reject(new Error("fixture media snapshot unavailable"));
    if (fixture.deferMediaInfo) return new Promise(resolve => fixture.pendingMediaInfo.push(resolve));
    return fixture.mediaInfoSnapshot ?? {title:"",artist:"",source:"",isPlaying:false};
  }
  if (command === "seek_media" && fixture.rejectNextSeek) {
    fixture.rejectNextSeek = false;
    return Promise.reject(new Error("fixture seek failure"));
  }
  if (command === "list_media_sessions") {
    if (fixture.holdMediaSessions) return new Promise(resolve => fixture.pendingMediaSessions.push(resolve));
    return fixture.mediaSessionsResult;
  }
  if (command === "request_artwork") {
    if (args?.slot === "cover" && fixture.deferCoverArtwork) {
      return new Promise(resolve => fixture.pendingCoverArtwork.push({ args, resolve }));
    }
    return null;
  }
  if (command === "get_netease_song_info_cmd") return fixture.disableSongMetadataRecovery ? null : {mvUrl:"https://fixture.invalid/video.mp4",duration:180_000};
  if (command === "get_cached_media") return null;
  if (command === "download_and_cache") return "C:\\fixture\\video.mp4";
}, {shouldMockEvents:true});
const tauriInvoke = window.__TAURI_INTERNALS__!.invoke;
window.__TAURI_INTERNALS__!.invoke = async (command, args, options) => {
  if (command === "plugin:event|emit") {
    if (args?.event === Events.ISLAND_MEDIA_SYNC) {
      fixture.mediaSyncDispatches.push(args.payload);
      if (fixture.rejectNextMediaSync) {
        fixture.rejectNextMediaSync = false;
        fixture.mediaSyncFailures++;
        throw new Error("fixture media sync emit failure");
      }
    } else if (args?.event === Events.FLOATING_MEDIA_READY) {
      fixture.floatingReadyMessages.push(args.payload);
    } else if (args?.event === Events.FLOATING_MEDIA_NOT_READY) {
      fixture.floatingNotReadyMessages.push(args.payload);
    } else if (args?.event === Events.FLOATING_MEDIA_READY_QUERY) {
      fixture.floatingReadyQueries.push(args.payload);
      if (fixture.holdReadyQueries) {
        fixture.pendingReadyQueries.push(args);
        return null;
      }
    } else if (args?.event === Events.TIMER_STATE_CHANGED) {
      fixture.timerStateDispatches.push(args.payload);
      if (fixture.holdTimerStateEvents) {
        fixture.pendingTimerStateEvents.push(args);
        return null;
      }
    } else if (args?.event === Events.TIMER_REQUEST_STATE) {
      fixture.timerRequestMessages.push(args.payload);
    }
  }
  return tauriInvoke(command, args, options);
};
let app: ReturnType<typeof mount>;
const floatingPeers: Array<{ app: ReturnType<typeof mount>; target: HTMLDivElement }> = [];
fixture.mountApp = () => { app = mount(App,{target:document.getElementById("app")!}); };
fixture.mountStudio = () => { app = mount(VisibleWindow,{target:document.getElementById("app")!,props:{component:Studio}}); };
fixture.mountTimer = () => { mockWindows("timer"); app = mount(TimerWindow,{target:document.getElementById("app")!}); };
fixture.mountTimerPeer = () => {
  mockWindows("timer");
  const target = document.createElement("div");
  target.id = `timer-peer-${floatingPeers.length}`;
  document.body.appendChild(target);
  const peer = mount(TimerWindow,{target});
  const index = floatingPeers.push({app:peer,target}) - 1;
  return index;
};
fixture.mountDigit = () => { app = mount(RollingFixture,{target:document.getElementById("app")!}); };
fixture.mountVolumePanel = () => {
  fixture.volumeWrites = [];
  const queue = new LatestValueWriteQueue<number>(value => {
    fixture.calls.push("set_system_volume");
    if (isTransportDiagnosticsEnabled()) {
      return runTimedTransportCommand("volume", () => Promise.resolve().then(() => { fixture.volumeWrites.push(value); }));
    }
    fixture.volumeWrites.push(value);
    return Promise.resolve();
  }, () => undefined);
  app = mount(VolumePanel,{
    target:document.getElementById("app")!,
    props:{volume:50,onVolume:(value:number)=>queue.submit(value,`fixture-device\0${value}`)},
  });
};
fixture.resolveMediaSessions = (index:number,sessions:unknown[]=[]) => fixture.pendingMediaSessions[index]?.(sessions);
fixture.activeIntervals = (delay:number) => fixture.intervalRows.filter((row:{delay:number;active:boolean})=>row.delay===delay&&row.active).length;
fixture.clockFormatterCalls = () => fixture.clockFormatterCount;
fixture.completionCardAdds = () => fixture.completionCardCount;
fixture.releaseTimerStateEvents = async () => {
  const pending = fixture.pendingTimerStateEvents.splice(0);
  await Promise.all(pending.map((args: any) => tauriInvoke("plugin:event|emit", args)));
};
fixture.mountFloating = (overrides: Record<string, unknown> = {}) => {
  fixture.settingsOverrides = {enableMvPlayback:true,captureHideOnScreenshot:true,captureHideOnRecording:true,captureHideOnFullscreen:true,captureHideOnScreenShare:true,...overrides};
  fixture.trackVideoCalls = true;
  fixture.videoCalls = [];
  mockWindows("floating_player");
  app = mount(FloatingWindow,{target:document.getElementById("app")!});
};
fixture.mountFloatingPeer = (overrides: Record<string, unknown> = {}) => {
  fixture.settingsOverrides = {enableMvPlayback:false,captureHideOnScreenshot:true,...overrides};
  mockWindows("floating_player");
  const target = document.createElement("div");
  target.id = `floating-peer-${floatingPeers.length}`;
  document.body.appendChild(target);
  const peer = mount(FloatingWindow,{target});
  const index = floatingPeers.push({app:peer,target}) - 1;
  return index;
};
fixture.destroyFloatingPeer = async (index: number) => {
  const peer = floatingPeers[index];
  if (!peer) return;
  await unmount(peer.app);
  peer.target.remove();
  await tick();
};
fixture.releaseRegionCommands = () => {
  const pending = fixture.pendingRegionCommands.splice(0);
  for (const resolve of pending) resolve();
};
fixture.beginTransportDiagnostics = () => {
  resetTransportDiagnostics();
  enableTransportDiagnostics();
};
fixture.endTransportDiagnostics = () => disableTransportDiagnostics();
fixture.transportDiagnosticsSnapshot = () => snapshotTransportDiagnostics();
fixture.releaseReadyQueries = async () => {
  const pending = fixture.pendingReadyQueries.splice(0);
  await Promise.all(pending.map((args: any) => tauriInvoke("plugin:event|emit", args)));
};
fixture.deferNextMediaInfo = () => { fixture.deferMediaInfo = true; fixture.pendingMediaInfo = []; };
fixture.resolveMediaInfo = (index: number, value: unknown) => fixture.pendingMediaInfo[index]?.(value);
fixture.destroy = async () => { await unmount(app); await tick(); };
fixture.changeEdge = () => settingsStore.update(settings => ({...settings,islandEdge:"left"}));
fixture.updateSettings = (patch: Record<string, unknown>) => settingsStore.update(settings => ({...settings,...patch}));
fixture.getSettingsSnapshot = () => get(settingsStore);
fixture.settingsChanged = (name: string) => emit(Events.SETTINGS_CHANGED,name);
fixture.settingsUpdated = (snapshot: unknown) => emit(Events.SETTINGS_UPDATED,snapshot);
fixture.settingsCommitted = (revision: number) => emit("settings-committed",{revision});
fixture.hdCoverChanged = (enabled: boolean) => emit(Events.HD_COVER_CHANGED,{enableHDCover:enabled});
fixture.resolveSettingsRead = (index: number, snapshot?: unknown) => fixture.pendingSettingsReads[index]?.(snapshot);
fixture.useTwoMonitors = () => {
  fixture.monitorList = [
    {width:1920,height:1080,x:0,y:0,workX:0,workY:0,workWidth:1920,workHeight:1080,scaleFactor:1},
    {width:1920,height:1080,x:1920,y:0,workX:1920,workY:0,workWidth:1920,workHeight:1080,scaleFactor:1},
  ];
  fixture.availableMonitorList = [
    {name:"test-0",position:{x:0,y:0},size:{width:1920,height:1080},scaleFactor:1},
    {name:"test-1",position:{x:1920,y:0},size:{width:1920,height:1080},scaleFactor:1},
  ];
};
fixture.expand = () => transitionIsland({type:"expand"});
fixture.hoverIsland = () => transitionIsland({type:"hover-enter"});
fixture.collapse = () => transitionIsland({type:"collapse"});
fixture.selectIslandPage = (page: "music"|"timer"|"volume"|"clock"|"weather") => transitionIsland({type:"select-page",page});
fixture.islandSnapshot = () => get(islandState);
fixture.media = (payload: unknown) => emit("media-update",payload);
fixture.mediaSync = (payload: unknown) => emit(Events.ISLAND_MEDIA_SYNC,payload);
fixture.mediaSessionStart = (appSessionId: string) => emit(Events.FLOATING_MEDIA_READY_QUERY,{appSessionId});
fixture.floatingReady = (payload: unknown) => emit(Events.FLOATING_MEDIA_READY,payload);
fixture.floatingNotReady = (payload: unknown) => emit(Events.FLOATING_MEDIA_NOT_READY,payload);
fixture.floatingClosed = (payload: unknown) => emit(Events.FLOATING_WINDOW_CLOSED,payload);
fixture.activateFloatingConsumer = async (instanceToken = "fixture-floating") => {
  const appSessionId = fixture.floatingReadyQueries.at(-1)?.appSessionId;
  if (appSessionId) await emit(Events.FLOATING_MEDIA_READY,{instanceToken,appSessionId});
  return {instanceToken,appSessionId};
};
fixture.mediaListenerCount = () => eventManager.getListenerCount(Events.MEDIA_UPDATE);
fixture.floatingReadyListenerCount = () => eventManager.getListenerCount(Events.FLOATING_MEDIA_READY);
fixture.floatingReadyQueryListenerCount = () => eventManager.getListenerCount(Events.FLOATING_MEDIA_READY_QUERY);
fixture.timerState = (payload: unknown) => emit(Events.TIMER_STATE_CHANGED,payload);
fixture.timerAction = (action:"start"|"pause"|"resume"|"adjust"|"reset",durationMs?:number) => emit(Events.TIMER_ACTION,{action,durationMs});
fixture.requestTimerSnapshot = (requestId = crypto.randomUUID()) => emit(Events.TIMER_REQUEST_STATE,{requestId});
fixture.setVisibility = (visibility: "visible" | "hidden") => {
  Object.defineProperty(document,"visibilityState",{configurable:true,value:visibility});
  Object.defineProperty(document,"hidden",{configurable:true,value:visibility === "hidden"});
  document.dispatchEvent(new Event("visibilitychange"));
};
fixture.trackSpectrumExtraction = () => {
  const NativeImage = window.Image;
  window.Image = function(this: unknown) {
    const image = new NativeImage();
    // CoverArt now shares the bounded loader. Its visual preload is not a
    // palette extraction; counting every 5s timeout conflates those owners.
    const visualOnly = /CoverArt\.svelte/.test(new Error().stack ?? "");
    const row = {image,src:"",pendingSource:true,loadAttempt:false,visualOnly};
    Object.defineProperty(image,"src",{
      configurable:true,
      get:()=>row.src,
      set:(value:string)=>{row.src=String(value);row.pendingSource=false;},
    });
    fixture.spectrumImageRows.push(row);
    return image;
  } as unknown as typeof Image;
  const nativeCreateElement = document.createElement.bind(document);
  document.createElement = ((tagName: string, options?: ElementCreationOptions) => {
    const element = nativeCreateElement(tagName,options);
    if (tagName.toLowerCase() === "canvas") {
      fixture.spectrumCanvasCount++;
      (element as HTMLCanvasElement).getContext = (() => ({
        drawImage:()=>{fixture.spectrumDrawCount++},
        getImageData:()=>({data:new Uint8ClampedArray(Array.from({length:24*24},()=>[220,30,30,255]).flat())}),
      })) as any;
    }
    return element;
  }) as typeof document.createElement;
};
fixture.spectrumImageSources = () => fixture.spectrumImageRows.map((row:{src:string}) => row.src);
fixture.spectrumLoadSources = () => fixture.spectrumImageRows.filter((row:{loadAttempt:boolean}) => row.loadAttempt).map((row:{src:string}) => row.src);
fixture.completeSpectrumImage = (index:number) => {
  const image = fixture.spectrumImageRows[index]?.image;
  image?.onload?.(new Event("load"));
};
fixture.completeSpectrumLoad = (index:number) => {
  const row = fixture.spectrumImageRows.filter((item:{loadAttempt:boolean}) => item.loadAttempt)[index];
  row?.image.onload?.(new Event("load"));
};
fixture.captureMode = (screenshot: boolean) => emit(Events.CAPTURE_MODE_CHANGED,{screenshot,recording:false,fullscreen:false,screenShare:false});
fixture.resolveVideoPlay = () => {
  let resolve!: () => void;
  fixture.nextVideoPlay = new Promise<void>((done) => { resolve = done; });
  fixture.resolvePendingVideoPlay = resolve;
};
fixture.videoIsPaused = () => {
  const video = document.querySelector(".mv-player") as HTMLVideoElement | null;
  if (video) fixture.lastVideo = video;
  return video ? video.paused : true;
};
fixture.lastVideoIsPaused = () => fixture.lastVideo ? (fixture.lastVideo as HTMLVideoElement).paused : true;
fixture.live250Intervals = () => fixture.intervalRows.filter((row: {delay:number;active:boolean}) => row.delay === 250 && row.active).length;
fixture.live30000Intervals = () => fixture.intervalRows.filter((row: {delay:number;active:boolean}) => row.delay === 30_000 && row.active).length;
fixture.liveTimeoutDelays = () => fixture.timeoutRows.filter((row: {delay:number;active:boolean}) => row.active).map((row: {delay:number}) => row.delay);
fixture.firedTimeouts = () => fixture.timeoutRows.reduce((total:number,row:{fired:number}) => total + row.fired,0);
fixture.loadedData = () => document.querySelector(".mv-player")?.dispatchEvent(new Event("loadeddata"));
fixture.ready = true;

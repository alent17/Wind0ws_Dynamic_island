<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { spring } from "svelte/motion";
  import { RotateCcw,Trash2,ExternalLink,Monitor,RefreshCw,ChevronUp,ChevronDown,CloudSun,Search,SlidersHorizontal,Music2,Settings2 } from "lucide-svelte";
  import { normalizeWidgets, type IslandWidgets } from "$lib/widgetLayout";
  import IslandSurface from "$lib/components/island/IslandSurface.svelte";
  import StudioSlider from "$lib/components/settings/StudioSlider.svelte";
  import StudioSelect from "$lib/components/settings/StudioSelect.svelte";
  import {  geometryFor, navigationHostFor as hostFor, type IslandMode, type IslandPage } from "$lib/islandGeometry";
  import { DEMO_MEDIA, media, connectMedia } from "$lib/mediaStore";
  import { extractSpectrumColorsFromImage } from "$lib/spectrumColors";
  import { settingsApi } from "$lib/api/settings";
  import { connectSettingsStore, settingsStore } from "$lib/settingsStore";
  import { windowApi } from "$lib/api/window";
  import { cacheApi } from "$lib/api/cache";
  import { mediaApi } from "$lib/api/media";
  import { playerNameFromId } from "$lib/playerNames";
  import { moveSelectedPlayer } from "$lib/playerOrder";
  import { idleApi } from "$lib/api/idle";
  import { applyAppFont } from "$lib/font";
  import { locale, resolveLocale, setLocale, translate, type TranslationKey } from "$lib/i18n";
  import { DEFAULT_SETTINGS, normalizeAppSettings, type AppLanguage, type AppPreferences, type MediaSessionInfo, type MediaState, type MonitorInfo, type WeatherForecastDay, type WeatherLocationCandidate, type WeatherSnapshot } from "$lib/api/types";
  import type { IslandTool } from "$lib/featureRail";
  import { formatClock } from "$lib/countdown";
  import { CLOCK_TIME_ZONES, clockTimeZoneOption } from "$lib/timeZones";
  let previewNow = $state(Date.now());

  type Scenario="playing"|"paused"|"no-art"|"long-title";
  type PreviewTarget="shape"|"compactLength"|"shoulder"|"radius"|"spectrum"|null;
  type AppearanceDraft=Pick<AppPreferences,"islandStyle"|"islandEdge"|"islandEdgePosition"|"floatingExpandedLength"|"edgeExpandedLength"|"floatingCompactLength"|"edgeCompactLength"|"floatingExpandedCornerRadius"|"edgeExpandedCornerRadius"|"collapsedEdgeShoulderRadius"|"expandedEdgeShoulderRadius"|"showSpectrum"|"spectrumMode">;

  function createAppearanceDraft(source:AppPreferences):AppearanceDraft{
    return {
      islandStyle:source.islandStyle,
      islandEdge:source.islandEdge,
      islandEdgePosition:source.islandEdgePosition,
      floatingExpandedLength:source.floatingExpandedLength,
      edgeExpandedLength:source.edgeExpandedLength,
      floatingCompactLength:source.floatingCompactLength,
      edgeCompactLength:source.edgeCompactLength,
      collapsedEdgeShoulderRadius:source.collapsedEdgeShoulderRadius,
      expandedEdgeShoulderRadius:source.expandedEdgeShoulderRadius,
      floatingExpandedCornerRadius:source.floatingExpandedCornerRadius,
      edgeExpandedCornerRadius:source.edgeExpandedCornerRadius,
      showSpectrum:source.showSpectrum,
      spectrumMode:source.spectrumMode,
    };
  }

  let mode=$state<IslandMode>("expanded"); let previewPage=$state<IslandPage>("music"); let scenario=$state<Scenario>("playing"); let progress=$state(50); let settings=$state<AppPreferences>({...DEFAULT_SETTINGS}); let monitors=$state<MonitorInfo[]>([]); let cacheMessage=$state(""); let settingsSaveFailed=$state(false);
  const previewClock = $derived(formatClock(settings.clockTimeZone, $locale, previewNow));
  let nativeRuntime=$state(false); let followingLive=$state(false); let liveMedia=$state<MediaState>(DEMO_MEDIA); let liveMediaDisconnect:undefined|(()=>void); let stageWidth=$state(0); let stageHeight=$state(0); let previewReady=$state(false);
  let weatherSnapshot=$state<WeatherSnapshot|null>(null);
  const demoForecast:WeatherForecastDay[]=[0,1,2,3].map((offset)=>({date:new Date(Date.now()+offset*86_400_000).toISOString().slice(0,10),weatherCode:[1,61,2,3][offset],temperatureMax:[33,34,34,32][offset],temperatureMin:[26,24,24,23][offset]}));
  let sessions=$state<MediaSessionInfo[]>([]); let sessionsLoading=$state(false); let sessionsLoaded=$state(false); let sessionRequest=0; let sessionInFlight=false; let sessionRefreshQueued=false; let sessionSettingsInitialized=false; let sessionSectionVisible=false; let weatherQuery=$state(""); let weatherResults=$state<WeatherLocationCandidate[]>([]); let weatherMessage=$state("");
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let persistInFlight: Promise<void> | undefined;
  let persistPending=false;
  let pendingSettingsPatch:Partial<AppPreferences>={};
  let disconnectSettingsStore:(()=>void)|undefined;
  let unsubscribeSettingsStore:(()=>void)|undefined;
  let appearanceDraft=$state<AppearanceDraft>(createAppearanceDraft(DEFAULT_SETTINGS));
  let previewTarget=$state<PreviewTarget>(null);

  let appearanceDirty=$derived.by(()=>{
    return appearanceDraft.islandStyle!==settings.islandStyle||
      appearanceDraft.islandEdge!==settings.islandEdge||
      appearanceDraft.islandEdgePosition!==settings.islandEdgePosition||
      appearanceDraft.floatingExpandedLength!==settings.floatingExpandedLength||
      appearanceDraft.edgeExpandedLength!==settings.edgeExpandedLength||
      appearanceDraft.floatingCompactLength!==settings.floatingCompactLength||
      appearanceDraft.edgeCompactLength!==settings.edgeCompactLength||
      appearanceDraft.collapsedEdgeShoulderRadius!==settings.collapsedEdgeShoulderRadius||
      appearanceDraft.expandedEdgeShoulderRadius!==settings.expandedEdgeShoulderRadius||
      appearanceDraft.floatingExpandedCornerRadius!==settings.floatingExpandedCornerRadius||
      appearanceDraft.edgeExpandedCornerRadius!==settings.edgeExpandedCornerRadius||
      appearanceDraft.showSpectrum!==settings.showSpectrum||
      appearanceDraft.spectrumMode!==settings.spectrumMode;
  });
  let studioDisposed=false;
  let neteaseSetupBusy=$state(false);
  let neteaseSetupMessage=$state("");
  let neteaseShortcut=$state("");
  async function setupNetease(restart:boolean,configureOnly=false,repairStartMenu=false){
    if(neteaseSetupBusy||!nativeRuntime||settings.neteaseConnectionMode!=="enhanced")return;
    neteaseSetupBusy=true;
    neteaseSetupMessage=t("neteaseStageSaving");
    neteaseShortcut="";
    try{
      await persist();
      const result=await mediaApi.setupNeteaseConnection(restart,true,configureOnly,stage=>{
        if(studioDisposed)return;
        const keys={waiting:"neteaseStageWaiting",detecting:"neteaseStageDetecting",configuring:"neteaseStageConfiguring",waitingPlayer:"neteaseStageWaitingPlayer",binding:"neteaseStageBinding",saving:"neteaseStageSaving"} as const;
        if(stage in keys)neteaseSetupMessage=t(keys[stage]);
      },repairStartMenu);
      if(studioDisposed)return;
      neteaseShortcut=result.configured?result.shortcutPath:"";
      neteaseSetupMessage=result.restartRequired?t("neteaseRestartRequired"):configureOnly&&result.configured&&result.launchEntries>0?t("neteaseLaunchEntriesReady",{count:result.launchEntries}):t(result.bindingFailed?"neteaseBindFailed":result.boundSessionId?"neteaseBound":result.connected?"neteaseBindPending":result.configured?"neteaseConfigured":"neteaseDisconnected");
      if((result.pendingStartMenuEntries??0)>0)neteaseSetupMessage+=" "+t("neteaseStartMenuPending",{count:result.pendingStartMenuEntries??0});
      // Binding has already committed. A transient refresh failure must not
      // replace that success with a misleading "setup failed" message.
      const refreshed=await Promise.allSettled([settingsApi.getSettings()]);
      if(studioDisposed)return;
      if(refreshed[0].status==="fulfilled")applyExternalSettings(normalizeAppSettings(refreshed[0].value));
      if(result.boundSessionId){
        await Promise.allSettled([refreshSessions(),mediaApi.getMediaInfo()]);
      }
    }catch(error){
      if(studioDisposed)return;
      const detail=String(error);
      neteaseSetupMessage=t(detail.includes("NETEASE_NOT_FOUND")?"neteaseNotFound":detail.includes("NETEASE_SETUP_BUSY")?"neteaseSetupBusy":detail.includes("NETEASE_SETUP_TIMEOUT")?"neteaseSetupTimeout":detail.includes("NETEASE_RESTART_FAILED")?"neteaseRestartFailed":"neteaseSetupFailed");
    }finally{
      neteaseSetupBusy=false;
    }
  }
  async function setNeteaseAutoConnect(enabled:boolean){
    updatePreference({neteaseAutoConnect:enabled});
    if(enabled)await setupNetease(false,true,true);
  }
  const previewBars=[.45,.78,.58,.96,.7,.38];
  let scenarioMedia=$derived.by<MediaState>(()=>{const base={...DEMO_MEDIA,positionMs:244000*progress/100,lastUpdatedTimestamp:Date.now()};if(scenario==="paused")return{...base,isPlaying:false};if(scenario==="no-art")return{...base,albumArt:""};if(scenario==="long-title")return{...base,title:"宇宙尽头的浪漫主义与一场不会结束的午夜公路旅行",artist:"The Extremely Long Artist Name · 特别长的专辑名称"};return base});
  let sample=$derived(nativeRuntime&&followingLive?liveMedia:scenarioMedia);
  let previewForecast=$derived(nativeRuntime?(weatherSnapshot?.weatherForecast??[]):demoForecast);
  let previewSpectrumTopColor=$state("#fff");
  let previewSpectrumBottomColor=$state("#888");
  $effect(()=>{
    const cover=sample.albumArt;
    if(!cover){previewSpectrumTopColor="#fff";previewSpectrumBottomColor="#888";return}
    const compactSurfaceVisible=mode==="compact"||mode==="hover";
    const compactColorConsumer=compactSurfaceVisible&&(!appearanceDraft.showSpectrum||sample.isPlaying);
    const spectrumConsumer=appearanceDraft.showSpectrum&&(compactSurfaceVisible||(mode==="expanded"&&previewPage==="music"));
    const shouldExtractSpectrumColors=previewReady&&(compactColorConsumer||spectrumConsumer);
    if(!shouldExtractSpectrumColors)return;
    const controller=new AbortController();
    void extractSpectrumColorsFromImage(cover,controller.signal).then(colors=>{
      const currentCompactSurfaceVisible=mode==="compact"||mode==="hover";
      const hasCurrentCompactColorConsumer=currentCompactSurfaceVisible&&(!appearanceDraft.showSpectrum||sample.isPlaying);
      const hasCurrentSpectrumConsumer=appearanceDraft.showSpectrum&&(currentCompactSurfaceVisible||(mode==="expanded"&&previewPage==="music"));
      if(controller.signal.aborted||sample.albumArt!==cover||!previewReady||(!hasCurrentCompactColorConsumer&&!hasCurrentSpectrumConsumer)||!colors)return;
      previewSpectrumTopColor=`rgb(${colors.top.join(",")})`;
      previewSpectrumBottomColor=`rgb(${colors.bottom.join(",")})`;
    }).catch(()=>{});
    return()=>controller.abort();
  });
  // Keep the island at the current preview scale while the surrounding stage changes width.
  const previewScaleFloor=.79;
  let previewTools=$derived.by<IslandTool[]>(()=>[
    ...(settings.showSettingsTool?["settings"]:[]),
    ...(settings.showFloatingTool?["floating"]:[]),
    ...(settings.showVolumeTool?["volume"]:[]),
    ...(settings.showTimerTool?["timer"]:[]),
    ...(settings.showHideTool?["hide"]:[]),
    ...(settings.showClockTool?["clock"]:[]),
    ...(settings.showWeatherTool?["weather"]:[]),
  ] as IslandTool[]);
  let compactLengthKey: "edgeCompactLength" | "floatingCompactLength" = $derived(appearanceDraft.islandStyle==="edge"?"edgeCompactLength":"floatingCompactLength");
  let expandedRadiusKey: "edgeExpandedCornerRadius" | "floatingExpandedCornerRadius" = $derived(appearanceDraft.islandStyle==="edge"?"edgeExpandedCornerRadius":"floatingExpandedCornerRadius");
  let previewCompactLength=$derived(appearanceDraft[compactLengthKey]);
  let previewExpandedRadius=$derived(appearanceDraft[expandedRadiusKey]);
  let previewExpandedLength=$derived(appearanceDraft.islandStyle==="edge"?appearanceDraft.edgeExpandedLength:appearanceDraft.floatingExpandedLength);
  let previewLayout=$state<IslandWidgets|null>(null);
  let previewEditing=$state(false);
  const previewWidgets=$derived(previewLayout ?? (settings.islandWidgetsEnabled ? normalizeWidgets(settings.islandWidgets) : {left:[],right:[]}));
  const previewWidgetCount=$derived(previewEditing ? 2 : Math.max(previewWidgets.left.length + previewWidgets.right.length, normalizeWidgets(settings.islandWidgets).left.length + normalizeWidgets(settings.islandWidgets).right.length));
  async function savePreviewWidgets(layout:IslandWidgets){
    const islandWidgets=normalizeWidgets(layout);
    if(nativeRuntime)await settingsApi.updateSettings({islandWidgets,islandWidgetsEnabled:true});
    settings={...settings,islandWidgets,islandWidgetsEnabled:true};
  }
  let previewHost=$derived(hostFor(appearanceDraft.islandStyle,appearanceDraft.islandEdge,previewCompactLength,1,previewExpandedLength,previewWidgetCount));
  let currentGeometry=$state(geometryFor("expanded"));
  let previewScale=$derived(Math.min(previewScaleFloor, Math.max(0.01, Math.min((stageWidth - 24) / previewHost.width, (stageHeight - 24) / previewHost.height))));
  const previewPosition=spring(50,{stiffness:.1,damping:.7,precision:.1});
  $effect(()=>{previewPosition.set(appearanceDraft.islandEdgePosition,{hard:!settings.enableAnimations||settings.reduceAnimations})});
  let previewPositionStyle=$derived.by(()=>{
    const p=$previewPosition/100;
    const scaledWidth=previewHost.width*previewScale;
    const scaledHeight=previewHost.height*previewScale;
    const horizontal=appearanceDraft.islandEdge==="top"||appearanceDraft.islandEdge==="bottom";
    const floating=appearanceDraft.islandStyle==="floating";
    const x=floating
      ? Math.max(0,(stageWidth-scaledWidth)/2)
      : horizontal
        ? Math.max(0,stageWidth-scaledWidth)*p
        : appearanceDraft.islandEdge==="right"?Math.max(0,stageWidth-scaledWidth):0;
    const y=floating
      ? Math.max(0,(stageHeight-scaledHeight)/2)
      : horizontal
        ? appearanceDraft.islandEdge==="bottom"?Math.max(0,stageHeight-scaledHeight):0
        : Math.max(0,stageHeight-scaledHeight)*p;
    return `transform:translate3d(${x}px,${y}px,0) scale(${previewScale})`;
  });
  function enableLivePreview(){
    if(!nativeRuntime)return;
    if(liveMediaDisconnect)return;
    const disconnect=connectMedia(()=>{
      if(!studioDisposed)followingLive=true;
    });
    if(studioDisposed)disconnect();
    else liveMediaDisconnect=disconnect;
  }
  async function refreshWeatherSnapshot(){
    if(!nativeRuntime)return;
    const locationKey=JSON.stringify(settings.weatherLocation);
    try{const snapshot=await idleApi.getWeatherSnapshot(locationKey);if(!studioDisposed&&locationKey===JSON.stringify(settings.weatherLocation))weatherSnapshot=snapshot}catch{}
  }
  const t=(key:TranslationKey,values:Record<string,string|number>={})=>translate(key,values,$locale);
  const clockTimeZoneOptions=$derived([{value:"system",label:t("systemTimeZone")},...CLOCK_TIME_ZONES.map((timeZone)=>clockTimeZoneOption(timeZone.value,resolveLocale(settings.language)))]);
  const enumLabel=(value:string)=>t(value as TranslationKey);
  const previewTargetLabel=(target:PreviewTarget)=>target? t(({shape:"shape",compactLength:"compactLength",shoulder:"shoulder",radius:"expandedRadius",spectrum:"spectrum"} as const)[target]):t("previewReady");
  const appearanceKeys=(Object.keys(createAppearanceDraft(DEFAULT_SETTINGS)) as (keyof AppearanceDraft)[]);
  function applyExternalSettings(next:AppPreferences){
    const dirtyAppearanceKeys=new Set(appearanceKeys.filter((key)=>appearanceDraft[key]!==settings[key]));
    const merged={...next,...pendingSettingsPatch};
    settings=merged;
    appearanceDraft={
      ...createAppearanceDraft(merged),
      ...Object.fromEntries(appearanceKeys.filter((key)=>dirtyAppearanceKeys.has(key)).map((key)=>[key,appearanceDraft[key]])),
    } as AppearanceDraft;
    applyAppFont(merged.fontId);
    setLocale(merged.language);
  }
  onMount(()=>{
    studioDisposed=false;
    let clockTimer:ReturnType<typeof setTimeout>;
    const refreshPreviewClock=()=>{
      const now=Date.now();
      previewNow=now;
      clockTimer=setTimeout(refreshPreviewClock,60_000-(now%60_000));
    };
    refreshPreviewClock();
    nativeRuntime=Boolean((window as any).__TAURI_INTERNALS__);
    const unsubscribe=media.subscribe(value=>liveMedia=value);
    let disposed=false;
    let deferredFrame=0;
    let deferredTimers:ReturnType<typeof setTimeout>[]=[];
    let sessionPollTimer:ReturnType<typeof setInterval>|undefined;
    let sessionObserver:IntersectionObserver|undefined;
    let closePromise:Promise<()=>void>|undefined;
    if(nativeRuntime){
      const appWindow=getCurrentWindow();
      closePromise=appWindow.onCloseRequested(async(event)=>{
        event.preventDefault();
        try{await persist()}catch{return}
        await appWindow.destroy();
      });
    }
    // Let the document paint before mounting the preview. The preview contains
    // canvases, motion stores, and a fairly large component tree; mounting it
    // in the same turn as a newly-created WebView can make the native window
    // look hung even though the settings UI itself is ready.
    deferredFrame=requestAnimationFrame(()=>{
      if(disposed)return;
      previewReady=true;
      if(!nativeRuntime)return;
      deferredTimers.push(
        setTimeout(()=>{
          if(disposed)return;
          void windowApi.getMonitors().then(value=>{if(!disposed)monitors=value}).catch(()=>{});
        },500),
      );
    });
    if(nativeRuntime){
      void enableLivePreview();
      void refreshWeatherSnapshot();
      const idleTimer=setInterval(()=>void refreshWeatherSnapshot(),30_000);
      deferredTimers.push(idleTimer);
      const playersSection=document.querySelector<HTMLElement>(".players-section");
      const stopSessionPolling=()=>{
        if(sessionPollTimer!==undefined){clearInterval(sessionPollTimer);sessionPollTimer=undefined}
        sessionRequest++;
        sessionRefreshQueued=false;
        sessionsLoading=false;
      };
      const startSessionPolling=()=>{
        if(disposed||!sessionSettingsInitialized||!sessionSectionVisible)return;
        void refreshSessions();
        if(sessionPollTimer===undefined)sessionPollTimer=setInterval(()=>void refreshSessions(),10_000);
      };
      if(playersSection&&typeof IntersectionObserver!=="undefined"){
        sessionObserver=new IntersectionObserver((entries)=>{
          const entry=entries.find(item=>item.target===playersSection);
          if(!entry)return;
          const visible=entry.isIntersecting&&entry.intersectionRatio>0;
          if(visible===sessionSectionVisible)return;
          sessionSectionVisible=visible;
          if(visible)startSessionPolling();else stopSessionPolling();
        });
        sessionObserver.observe(playersSection);
      }else sessionSectionVisible=true;
      void (async()=>{
        try{
          const connection=await connectSettingsStore(normalizeAppSettings);
          if(disposed){connection.disconnect();return}
          disconnectSettingsStore=connection.disconnect;
          applyExternalSettings(connection.settings);
          unsubscribeSettingsStore=settingsStore.subscribe((next)=>applyExternalSettings(next));
          applyAppFont(settings.fontId);
          setLocale(settings.language);
        }catch{}
        if(disposed)return;
        sessionSettingsInitialized=true;
        startSessionPolling();
      })();
    }
    return()=>{
      disposed=true;
      clearTimeout(clockTimer);
      studioDisposed=true;
      if(deferredFrame)cancelAnimationFrame(deferredFrame);
      deferredTimers.forEach(timer=>clearTimeout(timer));
      if(sessionPollTimer!==undefined){clearInterval(sessionPollTimer);sessionPollTimer=undefined}
      sessionObserver?.disconnect();
      sessionObserver=undefined;
      sessionRequest++;
      sessionRefreshQueued=false;
      if(closePromise)void closePromise.then(unlisten=>unlisten()).catch(()=>{});
      if(saveTimer)clearTimeout(saveTimer);
      void persist().catch(()=>{});
      unsubscribeSettingsStore?.();
      unsubscribeSettingsStore=undefined;
      disconnectSettingsStore?.();
      disconnectSettingsStore=undefined;
      unsubscribe();
      liveMediaDisconnect?.();
      liveMediaDisconnect=undefined;
    };
  });
  async function persist(){
    if(saveTimer){clearTimeout(saveTimer);saveTimer=undefined}
    if(!nativeRuntime)return;
    persistPending=true;
    if(persistInFlight){await persistInFlight;return}
    persistInFlight=(async()=>{
      while(persistPending){
        persistPending=false;
        const patch={...pendingSettingsPatch};
        pendingSettingsPatch={};
        if(!Object.keys(patch).length)continue;
        try{
          await settingsApi.updateSettings(patch);
          settingsSaveFailed=false;
        }catch(error){
          pendingSettingsPatch={...patch,...pendingSettingsPatch};
          persistPending=false;
          settingsSaveFailed=true;
          throw error;
        }
      }
    })().finally(()=>{persistInFlight=undefined});
    await persistInFlight;
  }
  function schedulePersist(){if(saveTimer)clearTimeout(saveTimer);saveTimer=setTimeout(()=>void persist().catch(()=>{}),300)}
  function commitPreference(patch:Partial<AppPreferences>){
    settings={...settings,...patch};
    pendingSettingsPatch={...pendingSettingsPatch,...patch};
    if(!nativeRuntime)return;
    schedulePersist();
  }
  function updatePreference(patch:Partial<AppPreferences>){commitPreference(patch)}
  function toggleCircularAlbum(){
    const enabled=!settings.floatingCircularAlbum;
    updatePreference(enabled?{floatingCircularAlbum:true,enableMvPlayback:false}:{floatingCircularAlbum:false});
  }
  function setAlwaysOnTop(value:boolean){updatePreference({alwaysOnTop:value})}
  function setAutoStart(value:boolean){updatePreference({autoStart:value})}
  // Publish each edit immediately; persist() serializes concurrent changes and
  // retains the newest pending values if saving fails.
  $effect(()=>{
    if(!appearanceDirty)return;
    const patch={...appearanceDraft};
    untrack(()=>{
      commitPreference(patch);
      void persist().catch(()=>{});
    });
  });
  function setLanguage(value:AppLanguage){setLocale(value);updatePreference({language:value})}
  function mergePlayerOrderIds(existing:string[],available:MediaSessionInfo[],selected:string[]|null){
    return [...new Set([...existing,...available.map(item=>item.id),...(selected??[])].filter(id=>id.trim().length>0))];
  }
  function currentPlayerOrderIds(){return mergePlayerOrderIds(settings.playerOrderIds,sessions,settings.selectedPlayerIds)}
  function sessionFor(id:string):MediaSessionInfo{
    const session=sessions.find(item=>item.id===id)??{id,displayName:playerNameFromId(id),source:"generic",isPlaying:false};
    return settings.playerSourceOverrides[id]==="netease"?{...session,displayName:"网易云音乐",source:"netease"}:session;
  }
  async function setNeteasePlayer(id:string,enabled:boolean){
    const overrides={...settings.playerSourceOverrides};
    if(enabled)overrides[id]="netease";else delete overrides[id];
    updatePreference({playerSourceOverrides:overrides});
    try{await persist();await refreshSessions();await mediaApi.getMediaInfo()}catch{}
  }
  async function refreshSessions(){
    if(!nativeRuntime)return;
    if(sessionInFlight){sessionRefreshQueued=true;return}
    sessionInFlight=true;
    const request=++sessionRequest;
    if(!sessionsLoaded)sessionsLoading=true;
    try{
      const nextSessions=await mediaApi.listMediaSessions();
      if(studioDisposed||request!==sessionRequest)return;
      sessions=nextSessions;
      const activeIds=new Set(nextSessions.map(item=>item.id));
      const selectedIds=new Set(settings.selectedPlayerIds??[]);
      const retainedOrder=settings.playerOrderIds.filter(id=>activeIds.has(id)||selectedIds.has(id));
      const nextOrder=mergePlayerOrderIds(retainedOrder,nextSessions,settings.selectedPlayerIds);
      if(nextOrder.join("\u0000")!==settings.playerOrderIds.join("\u0000"))updatePreference({playerOrderIds:nextOrder});
    }catch(error){console.error("[播放器] 刷新媒体会话失败:",error)}
    finally{
      if(request===sessionRequest){sessionsLoading=false;sessionsLoaded=true}
      sessionInFlight=false;
      if(sessionRefreshQueued&&sessionSettingsInitialized&&sessionSectionVisible&&!studioDisposed){
        sessionRefreshQueued=false;
        void refreshSessions();
      }
    }
  }
  function togglePlayer(id:string,checked:boolean){
    const order=currentPlayerOrderIds();
    const selected=settings.selectedPlayerIds===null?[...order]:[...settings.selectedPlayerIds];
    const nextSelected=checked?[...selected.filter(item=>item!==id),id]:selected.filter(item=>item!==id);
    const activeIds=new Set(sessions.map(item=>item.id));
    const nextOrder=[...order.filter(item=>nextSelected.includes(item)),...order.filter(item=>!nextSelected.includes(item)&&activeIds.has(item))];
    updatePreference({selectedPlayerIds:nextSelected,playerOrderIds:nextOrder});
  }
  function movePlayer(id:string,direction:-1|1){
    const order=currentPlayerOrderIds();
    const next=moveSelectedPlayer(order,settings.selectedPlayerIds,id,direction);
    if(next!==order)updatePreference({playerOrderIds:next});
  }
  let playerRows=$derived.by(()=>{
    const order=currentPlayerOrderIds();
    const selected=new Set(settings.selectedPlayerIds===null?order:settings.selectedPlayerIds);
    const activeIds=new Set(sessions.map(item=>item.id));
    return {selected:order.filter(id=>selected.has(id)).map(sessionFor),unselected:order.filter(id=>!selected.has(id)&&activeIds.has(id)).map(sessionFor)};
  });
  async function searchWeather(){weatherMessage=t("searching");try{weatherResults=await idleApi.searchLocations(weatherQuery,$locale==="zh-CN"?"zh":$locale);weatherMessage=weatherResults.length?"":t("noCity")}catch{weatherMessage=t("weatherFailed")}}
  function weatherLocationLabel(item:WeatherLocationCandidate){return [...new Set([item.name,item.admin2,item.admin1,item.country].filter(Boolean))].join(" · ")}
  function weatherCandidateDetail(item:WeatherLocationCandidate){const label=[...new Set([item.admin2,item.admin1,item.country].filter(Boolean))].join(" · ");const duplicates=weatherResults.filter(other=>weatherLocationLabel(other)===weatherLocationLabel(item)).length;return duplicates>1?`${label} · ${item.latitude.toFixed(2)}, ${item.longitude.toFixed(2)}`:label}
  function selectWeather(item:WeatherLocationCandidate){updatePreference({weatherLocation:{name:weatherLocationLabel(item),latitude:item.latitude,longitude:item.longitude}});weatherResults=[];weatherMessage=t("citySaved");void persist().then(()=>{if(!studioDisposed)void refreshWeatherSnapshot()}).catch(()=>{})}
  async function clearCache(){cacheMessage=t("clearing");try{await cacheApi.clearCache();cacheMessage=t("cacheCleared")}catch{cacheMessage=t("clearFailed")}}
</script>

<svelte:head><title>Isle Studio</title></svelte:head>
<main>
  <header class="studio-header">
    <div class="studio-brand">
      <span class="studio-wordmark">ISLE / STUDIO</span>
      <h1>Isle Studio</h1>
      <p>{t("studioIntro")}</p>
    </div>
    <div class="studio-status" class:browser={!nativeRuntime} aria-live="polite">
      <i aria-hidden="true"></i>
      <span><strong>{nativeRuntime&&followingLive?t("currentMusic"):nativeRuntime?t("previewReady"):t("browserDemo")}</strong><small>{t("previewLabel")}</small></span>
    </div>
  </header>
  {#if settingsSaveFailed}
    <div class="settings-save-error" role="alert"><span>{t("settingsSaveFailed")}</span><button type="button" onclick={()=>void persist().catch(()=>{})}>{t("retrySave")}</button></div>
  {/if}
  <div class="workspace">
    <section class="stage" class:attached-top-preview={appearanceDraft.islandStyle === "edge" && appearanceDraft.islandEdge === "top"} aria-label={t("previewLabel")} bind:clientWidth={stageWidth} bind:clientHeight={stageHeight}>
      <div class="stage-topline"><span><i aria-hidden="true"></i>{t("previewLabel")}</span><small>{nativeRuntime&&followingLive?t("currentMusic"):nativeRuntime?t("previewReady"):t("browserDemo")}</small></div>
      <div class="preview-host" style={`width:${previewHost.width}px;height:${previewHost.height}px;${previewPositionStyle}`}>
        {#if previewReady}
        <IslandSurface widgetsEnabled={settings.islandWidgetsEnabled} onWidgetsToggle={async enabled=>{if(nativeRuntime)await settingsApi.updateSettings({islandWidgetsEnabled:enabled});settings={...settings,islandWidgetsEnabled:enabled};}} islandWidgets={settings.islandWidgets} onLayoutPreview={(layout,editing)=>{previewLayout=layout;previewEditing=editing}} onWidgetsSave={savePreviewWidgets} weatherCity={nativeRuntime ? settings.weatherLocation?.name ?? "" : "Shanghai"} weatherUpdatedAt={weatherSnapshot?.weatherUpdatedAt ?? null} onRegionChange={({geometry}) => currentGeometry = geometry} onPageChange={(page)=>previewPage=page} media={sample} clockText={previewClock} clockTimeZone={settings.clockTimeZone} idle={nativeRuntime && followingLive && !sample.title} idleTime={previewClock} idleWeatherTemperature={weatherSnapshot?.weatherTemperature ?? null} idleWeatherCode={weatherSnapshot?.weatherCode ?? null} idleWeatherForecast={previewForecast} {mode} islandStyle={appearanceDraft.islandStyle} edge={appearanceDraft.islandEdge} position={sample.positionMs} expandedLength={previewExpandedLength} expandedRadius={previewExpandedRadius} collapsedEdgeShoulderRadius={appearanceDraft.collapsedEdgeShoulderRadius} expandedEdgeShoulderRadius={appearanceDraft.expandedEdgeShoulderRadius} compactLength={previewCompactLength} showSpectrum={appearanceDraft.showSpectrum} spectrumMode={appearanceDraft.spectrumMode} spectrumTopColor={previewSpectrumTopColor} spectrumBottomColor={previewSpectrumBottomColor} enableAnimations={settings.enableAnimations} reduceAnimations={settings.reduceAnimations} previewSpectrum={!nativeRuntime&&appearanceDraft.spectrumMode==="realtime"?previewBars:undefined} previewHighlight={previewTarget} previewEdgePosition={appearanceDraft.islandEdgePosition} interactive simulateHidden enabledTools={previewTools} showCustomFunctionPanel={settings.showCustomFunctionPanel} systemAudio={{volumePercent:42,muted:false,deviceId:"studio-speakers",deviceName:"Speakers"}} audioDevices={[{id:"studio-speakers",name:"Speakers (Realtek Audio)",isDefault:true},{id:"studio-headphones",name:"Headphones",isDefault:false}]} onSettingsToggle={()=>{if(nativeRuntime)void windowApi.showStudioWindow()}} onToggle={()=>mode=mode==="expanded"?"compact":"expanded"} onMediaAction={(action)=>{if(nativeRuntime&&followingLive){if(action==="cycle_netease_mode"){void mediaApi.cycleNeteasePlaybackMode();return}void mediaApi.controlMedia(action);return}if(action==="play_pause")scenario=scenario==="paused"?"playing":"paused"}} />
        {/if}
      </div>
      <div class="stage-caption" class:editing={previewTarget!==null} aria-live="polite">
        <strong>{previewTarget?t("previewEditing"):t("previewReady")}{previewTarget?`：${previewTargetLabel(previewTarget)}`:""}</strong>
        <span>{enumLabel(mode)} · {enumLabel(appearanceDraft.islandStyle)} · {enumLabel(appearanceDraft.islandEdge)} · {currentGeometry.width} × {currentGeometry.height} · r{currentGeometry.radius}</span>
      </div>
    </section>
    <aside aria-label={t("preferences")}>
      <nav class="settings-nav" aria-label={t("preferences")}>
        <a href="#studio-layout"><SlidersHorizontal size={16}/><span>{t("settingsNavAppearance")}</span></a>
        <a href="#studio-media"><Music2 size={16}/><span>{t("settingsNavMedia")}</span></a>
        <a href="#studio-desktop"><CloudSun size={16}/><span>{t("settingsNavDesktop")}</span></a>
        <a href="#studio-system"><Settings2 size={16}/><span>{t("settingsNavSystem")}</span></a>
      </nav>
      <section id="studio-layout" class="appearance-section">
        <div class="section-heading"><h2>{t("islandLayout")}</h2><span class="live-indicator"><i></i>{t("previewReady")}</span></div>
        <div class="preview-state"><div class="field-label"><span>{t("previewState")}</span><small>{t("previewReady")}</small></div><div class="segmented">{#each ["compact","hover","expanded","hidden"] as item}<button aria-pressed={mode===item} class:active={mode===item} onclick={()=>mode=item as IslandMode}>{enumLabel(item)}</button>{/each}</div></div>
        <div class="preview-setting" role="group" data-preview-target="shape" onmouseenter={()=>previewTarget="shape"} onfocusin={()=>previewTarget="shape"} onmouseleave={()=>previewTarget=null} onfocusout={()=>previewTarget=null}>
          <div class="field-label"><span>{t("shape")}</span><small>{t("shapeHint")}</small></div>
          <div class="segmented two"><button aria-pressed={appearanceDraft.islandStyle==="floating"} class:active={appearanceDraft.islandStyle==="floating"} onclick={()=>{appearanceDraft.islandStyle="floating";previewTarget="shape"}}>{t("floating")}</button><button aria-pressed={appearanceDraft.islandStyle==="edge"} class:active={appearanceDraft.islandStyle==="edge"} onclick={()=>{appearanceDraft.islandStyle="edge";previewTarget="shape"}}>{t("attached")}</button></div>
        </div>
        {#if appearanceDraft.islandStyle === "floating"}
        <fieldset class="shape-settings" data-shape="floating">
          <legend>{t("floating")}</legend>
          <StudioSlider label={t("compactLength")} min={80} max={300} unit="px" bind:value={appearanceDraft.floatingCompactLength} previewTarget={previewTarget === "compactLength" ? "compactLength" : null} onPreviewStart={()=>{appearanceDraft.islandStyle="floating";previewTarget="compactLength"}} onPreviewMove={()=>{appearanceDraft.islandStyle="floating";previewTarget="compactLength"}} onPreviewEnd={()=>previewTarget=null}/>
          <StudioSlider label={t("expandedLength")} min={480} max={900} unit="px" bind:value={appearanceDraft.floatingExpandedLength} onPreviewStart={()=>{mode="expanded"}} onPreviewMove={()=>{mode="expanded"}}/>
          <StudioSlider label={t("expandedRadius")} min={0} max={80} unit="px" bind:value={appearanceDraft.floatingExpandedCornerRadius} previewTarget={previewTarget === "radius" ? "radius" : null} onPreviewStart={()=>{appearanceDraft.islandStyle="floating";previewTarget="radius"}} onPreviewMove={()=>{appearanceDraft.islandStyle="floating";previewTarget="radius"}} onPreviewEnd={()=>previewTarget=null}/>
        </fieldset>
        {:else}
        <fieldset class="shape-settings" data-shape="edge">
          <legend>{t("attached")}</legend>
          <StudioSlider label={t("compactLength")} min={80} max={300} unit="px" bind:value={appearanceDraft.edgeCompactLength} previewTarget={previewTarget === "compactLength" ? "compactLength" : null} onPreviewStart={()=>{appearanceDraft.islandStyle="edge";previewTarget="compactLength"}} onPreviewMove={()=>{appearanceDraft.islandStyle="edge";previewTarget="compactLength"}} onPreviewEnd={()=>previewTarget=null}/>
          <StudioSlider label={t("collapsedShoulder")} min={0} max={16} unit="px" bind:value={appearanceDraft.collapsedEdgeShoulderRadius} previewTarget={previewTarget === "shoulder" ? "shoulder" : null} onPreviewStart={()=>{appearanceDraft.islandStyle="edge";previewTarget="shoulder"}} onPreviewMove={()=>{appearanceDraft.islandStyle="edge";previewTarget="shoulder"}} onPreviewEnd={()=>previewTarget=null}/>
          <StudioSlider label={t("expandedShoulder")} min={0} max={64} unit="px" bind:value={appearanceDraft.expandedEdgeShoulderRadius} previewTarget={previewTarget === "shoulder" ? "shoulder" : null} onPreviewStart={()=>{appearanceDraft.islandStyle="edge";previewTarget="shoulder"}} onPreviewMove={()=>{appearanceDraft.islandStyle="edge";previewTarget="shoulder"}} onPreviewEnd={()=>previewTarget=null}/>
          <StudioSlider label={t("expandedLength")} min={480} max={900} unit="px" bind:value={appearanceDraft.edgeExpandedLength} onPreviewStart={()=>{mode="expanded"}} onPreviewMove={()=>{mode="expanded"}}/>
          <StudioSlider label={t("expandedRadius")} min={0} max={80} unit="px" bind:value={appearanceDraft.edgeExpandedCornerRadius} previewTarget={previewTarget === "radius" ? "radius" : null} onPreviewStart={()=>{appearanceDraft.islandStyle="edge";previewTarget="radius"}} onPreviewMove={()=>{appearanceDraft.islandStyle="edge";previewTarget="radius"}} onPreviewEnd={()=>previewTarget=null}/>
        </fieldset>
        {/if}
      </section>
      <section class="preferences-section">
        <div class="section-title"><h2>{t("preferences")}</h2><span class="section-note">{t("studioIntro")}</span></div>
        <div class="select-grid">
          <div class="compact-select-row"><span>{t("language")}</span><StudioSelect label={t("language")} value={settings.language} options={[
            {value:"system",label:t("systemLanguage")},{value:"zh-CN",label:t("chinese")},
            {value:"en",label:t("english")},{value:"ja",label:t("japanese")}
          ]} onSelect={(value)=>setLanguage(value as AppLanguage)}/></div>
          <div class="compact-select-row"><span>{t("timeZone")}</span><StudioSelect label={t("timeZone")} value={settings.clockTimeZone} options={clockTimeZoneOptions} onSelect={(value)=>updatePreference({clockTimeZone:value})}/></div>
        </div>
      </section>
      <section id="studio-media" class="netease-progress-section">
        <h2>{t("neteaseConnection")}</h2>
        <div class="segmented two progress-options" role="group" aria-label={t("neteaseConnection")}>
          <button aria-pressed={settings.neteaseConnectionMode==="standard"} class:active={settings.neteaseConnectionMode==="standard"} disabled={neteaseSetupBusy} onclick={()=>updatePreference({neteaseConnectionMode:"standard"})}><span>{t("neteaseStandard")}</span><small>{t("neteaseStandardShort")}</small></button>
          <button aria-pressed={settings.neteaseConnectionMode==="enhanced"} class:active={settings.neteaseConnectionMode==="enhanced"} disabled={neteaseSetupBusy} onclick={()=>updatePreference({neteaseConnectionMode:"enhanced"})}><span>{t("neteaseEnhanced")}</span><small>{t("neteaseEnhancedShort")}</small></button>
        </div>
        <p class="hint">{t(settings.neteaseConnectionMode==="standard"?"neteaseStandardHint":"neteaseEnhancedHint")}</p>
        {#if settings.neteaseConnectionMode==="enhanced"}
        <div class="netease-auto-row">
          <label><input type="checkbox" checked={settings.neteaseAutoConnect} disabled={!nativeRuntime||neteaseSetupBusy} onchange={(event)=>setNeteaseAutoConnect(event.currentTarget.checked)}/><span>{t("neteaseAutoConnect")}</span></label>
        </div>
        <p class="hint">{t("neteaseAutoConnectHint")}</p>
        <div class="netease-actions">
          <button type="button" class="netease-primary" title={t("neteaseBindHint")} disabled={!nativeRuntime||neteaseSetupBusy} onclick={()=>setupNetease(true)}><RefreshCw size={15}/>{t(neteaseSetupBusy?"neteaseConnecting":"neteaseBindAction")}</button>
          <button type="button" disabled={!nativeRuntime||neteaseSetupBusy} onclick={()=>setupNetease(false)}>{t("neteaseCheckBind")}</button>
        </div>
        <details class="netease-details">
          <summary>{t("neteaseMoreOptions")}</summary>
          <p class="hint">{t("neteaseSetupHint")}</p>
          <p class="hint">{t("neteasePlayButtonHint")}</p>
          <button type="button" disabled={!nativeRuntime||neteaseSetupBusy} onclick={()=>setupNetease(false,true,true)}>{t(neteaseSetupBusy?"neteaseConnecting":"neteaseConfigureLaunchEntries")}</button>
        </details>
        {#if neteaseSetupMessage}<p class="message" role="status" aria-live="polite" aria-atomic="true">{neteaseSetupMessage}</p>{/if}
        {#if neteaseShortcut}<p class="hint netease-shortcut">{neteaseShortcut}</p>{/if}
        {/if}
        <h2>{t("neteaseProgress")}</h2>

        <div class="segmented two progress-options" role="group" aria-label={t("neteaseProgress")}>
          <button aria-pressed={settings.neteaseProgressMode==="real"} class:active={settings.neteaseProgressMode==="real"} onclick={()=>updatePreference({neteaseProgressMode:"real"})}><span>{t("neteaseProgressReal")}</span><small>{t("neteaseProgressRealHint")}</small></button>
          <button aria-pressed={settings.neteaseProgressMode==="estimate"} class:active={settings.neteaseProgressMode==="estimate"} onclick={()=>updatePreference({neteaseProgressMode:"estimate"})}><span>{t("neteaseProgressEstimate")}</span><small>{t("neteaseProgressEstimateHint")}</small></button>
        </div>

        {#if nativeRuntime && followingLive && liveMedia.source==="netease" && settings.neteaseConnectionMode==="enhanced" && liveMedia.neteaseProgressAvailable===false}<p class="netease-progress-status" role="status">{t("neteaseProgressUnavailable")}</p>{/if}
        {#if nativeRuntime && followingLive && liveMedia.source==="netease" && liveMedia.progressKnown===false && settings.neteaseProgressMode==="estimate"}<p class="netease-progress-status" role="status">{t("neteaseProgressPending")}</p>{/if}
      </section>
      <section class="spectrum-section"><h2>{t("spectrum")}</h2><div class="preview-setting" role="group" data-preview-target="spectrum" onmouseenter={()=>previewTarget="spectrum"} onfocusin={()=>previewTarget="spectrum"} onmouseleave={()=>previewTarget=null} onfocusout={()=>previewTarget=null}><div class="setting-grid single"><button type="button" class="setting-choice" class:active={appearanceDraft.showSpectrum} aria-pressed={appearanceDraft.showSpectrum} onclick={()=>{appearanceDraft.showSpectrum=!appearanceDraft.showSpectrum;previewTarget="spectrum"}}><span class="choice-mark" aria-hidden="true">{appearanceDraft.showSpectrum?"✓":""}</span><span class="choice-copy"><strong>{t("showSpectrum")}</strong><small>{t("showSpectrumHint")}</small></span></button></div>{#if appearanceDraft.showSpectrum}<div class="field-label"><span>{t("animationSource")}</span><small>{t("animationSourceHint")}</small></div><div class="segmented two"><button aria-pressed={appearanceDraft.spectrumMode==="realtime"} class:active={appearanceDraft.spectrumMode==="realtime"} onclick={()=>{appearanceDraft.spectrumMode="realtime";previewTarget="spectrum"}}>{t("realtime")}</button><button aria-pressed={appearanceDraft.spectrumMode==="random"} class:active={appearanceDraft.spectrumMode==="random"} onclick={()=>{appearanceDraft.spectrumMode="random";previewTarget="spectrum"}}>{t("random")}</button></div>{/if}</div></section>
      <section class="players-section"><div class="section-title"><h2>{t("players")}</h2><button class="icon-button" aria-label={t("refreshPlayers")} disabled={!nativeRuntime} onclick={refreshSessions}><RefreshCw size={15}/></button></div><p class="hint">{t("playersHint")}</p>
        {#each sessions.filter(player=>player.source==="generic"||settings.playerSourceOverrides[player.id]==="netease") as player (player.id)}
          <div class="player-identity">
            <p class="hint" title={player.id}>{settings.playerSourceOverrides[player.id]==="netease"?"网易云音乐":player.displayName}</p>
            <button type="button" disabled={!nativeRuntime} onclick={()=>setNeteasePlayer(player.id,settings.playerSourceOverrides[player.id]!=="netease")}>{t(settings.playerSourceOverrides[player.id]==="netease"?"playerIdentityReset":"playerIdentityNetease")}</button>
          </div>
        {/each}
        {#if sessionsLoading}<div class="ordered-list skeleton-list" aria-hidden="true">{#each [0,1,2] as row}<div class="ordered-row skeleton-row"><span><i></i><i></i></span></div>{/each}</div>{:else}<div class="ordered-list">{#each playerRows.selected as player,index (player.id)}<div class="ordered-row" class:offline={!sessions.some(item=>item.id===player.id)}><button type="button" class="player-selection" role="checkbox" aria-checked="true" aria-label={t("select",{name:player.displayName})} onclick={()=>togglePlayer(player.id,false)}><span class="choice-mark" aria-hidden="true">✓</span></button><span><strong>{player.displayName}</strong><small>{player.isPlaying?t("playingNow"):sessions.some(item=>item.id===player.id)?t("detected"):t("offline")}</small></span><button aria-label={t("moveUp")} disabled={index===0} onclick={()=>movePlayer(player.id,-1)}><ChevronUp size={14}/></button><button aria-label={t("moveDown")} disabled={index===playerRows.selected.length-1} onclick={()=>movePlayer(player.id,1)}><ChevronDown size={14}/></button></div>{/each}{#each playerRows.unselected as player (player.id)}<div class="ordered-row" class:offline={!sessions.some(item=>item.id===player.id)}><button type="button" class="player-selection" role="checkbox" aria-checked="false" aria-label={t("select",{name:player.displayName})} onclick={()=>togglePlayer(player.id,true)}><span class="choice-mark" aria-hidden="true"></span></button><span><strong>{player.displayName}</strong><small>{player.isPlaying?t("playingNow"):sessions.some(item=>item.id===player.id)?t("detected"):t("offline")}</small></span></div>{/each}{#if !playerRows.selected.length&&!playerRows.unselected.length}<p class="empty">{t("noSessions")}</p>{/if}</div>{/if}
      </section>
      <section id="studio-desktop" class="idle-section"><div class="section-title"><h2>{t("idleContent")}</h2><span class="section-note">{t("idleFixedLabel")}</span></div>
        <p class="hint idle-fixed-hint">{t("idleFixedHint")}</p>
        <div class="idle-layout-preview" aria-hidden="true"><span>{t("clock")}</span><span><CloudSun size={14}/>{t("weather")}</span></div>
        <div class="weather-box"><label for="weather-city">{t("weatherCity")}</label><div class="search-row"><input id="weather-city" placeholder={t("cityExample")} bind:value={weatherQuery} onkeydown={(e)=>{if(e.key==="Enter")void searchWeather()}}/><button aria-label={t("searchCity")} onclick={searchWeather}><Search size={15}/></button></div>{#if settings.weatherLocation}<p class="selected-city">{t("currentCity",{name:settings.weatherLocation.name})}</p>{/if}{#if weatherMessage}<p class="message">{weatherMessage}</p>{/if}{#if weatherResults.length}<div class="weather-results">{#each weatherResults as item}<button onclick={()=>selectWeather(item)}><strong>{item.name}</strong><small>{weatherCandidateDetail(item)}</small></button>{/each}</div>{/if}<p class="hint">{t("weatherSource")}</p></div>
      </section>
      <section class="media-features-section"><h2>{t("mediaFeatures")}</h2><div class="setting-grid">
        <button type="button" class="setting-choice" class:active={settings.enableHdCover} aria-pressed={settings.enableHdCover} onclick={()=>updatePreference({enableHdCover:!settings.enableHdCover})}><span class="choice-mark" aria-hidden="true">{settings.enableHdCover?"✓":""}</span><span class="choice-copy"><strong>{t("hdCover")}</strong><small>{t("hdCoverHint")}</small></span></button>
        <button type="button" class="setting-choice" class:active={settings.floatingCircularAlbum} aria-pressed={settings.floatingCircularAlbum} onclick={toggleCircularAlbum}><span class="choice-mark" aria-hidden="true">{settings.floatingCircularAlbum?"✓":""}</span><span class="choice-copy"><strong>{t("floatingCircularAlbum")}</strong><small>{t("floatingCircularAlbumHint")}</small></span></button>
        <button type="button" class="setting-choice" class:active={settings.enableMvPlayback&&!settings.floatingCircularAlbum} aria-pressed={settings.enableMvPlayback&&!settings.floatingCircularAlbum} disabled={settings.floatingCircularAlbum} title={settings.floatingCircularAlbum?t("mvUnavailableInCircular"):undefined} onclick={()=>updatePreference({enableMvPlayback:!settings.enableMvPlayback})}><span class="choice-mark" aria-hidden="true">{settings.enableMvPlayback&&!settings.floatingCircularAlbum?"✓":""}</span><span class="choice-copy"><strong>{t("mvPlayback")}</strong><small>{settings.floatingCircularAlbum?t("mvUnavailableInCircular"):t("mvPlaybackHint")}</small></span></button>
      </div></section>
      <section id="studio-system" class="app-behavior-section"><h2>{t("appBehavior")}</h2>
        <div class="volume-settings">
          <div class="setting-grid single"><button type="button" class="setting-choice" class:active={settings.wheelVolumeControl} aria-pressed={settings.wheelVolumeControl} onclick={()=>updatePreference({wheelVolumeControl:!settings.wheelVolumeControl})}><span class="choice-mark" aria-hidden="true">{settings.wheelVolumeControl?"✓":""}</span><span class="choice-copy"><strong>{t("wheelVolumeControl")}</strong><small>{t("wheelVolumeControlHint")}</small></span></button></div>
          {#if settings.wheelVolumeControl}
            <div class="volume-protection-options">
              <div class="setting-grid single"><button type="button" class="setting-choice" class:active={settings.volumeProtectionEnabled} aria-pressed={settings.volumeProtectionEnabled} onclick={()=>updatePreference({volumeProtectionEnabled:!settings.volumeProtectionEnabled})}><span class="choice-mark" aria-hidden="true">{settings.volumeProtectionEnabled?"✓":""}</span><span class="choice-copy"><strong>{t("volumeProtection")}</strong><small>{t("volumeProtectionHint")}</small></span></button></div>
              {#if settings.volumeProtectionEnabled}<div class="volume-protection-limit"><StudioSlider label={t("volumeProtectionLimit")} min={40} max={100} unit="%" value={settings.volumeProtectionLimit} onPreviewMove={(value)=>updatePreference({volumeProtectionLimit:value})}/></div>{/if}
            </div>
          {/if}
        </div>
        <div class="setting-grid"><button type="button" class="setting-choice" class:active={settings.alwaysOnTop} aria-pressed={settings.alwaysOnTop} disabled={!nativeRuntime} onclick={()=>setAlwaysOnTop(!settings.alwaysOnTop)}><span class="choice-mark" aria-hidden="true">{settings.alwaysOnTop?"✓":""}</span><span class="choice-copy"><strong>{t("alwaysOnTop")}</strong><small>{t("alwaysOnTopHint")}</small></span></button><button type="button" class="setting-choice" class:active={settings.autoStart} aria-pressed={settings.autoStart} disabled={!nativeRuntime} onclick={()=>setAutoStart(!settings.autoStart)}><span class="choice-mark" aria-hidden="true">{settings.autoStart?"✓":""}</span><span class="choice-copy"><strong>{t("autoStart")}</strong><small>{t("autoStartHint")}</small></span></button></div>
        <div class="select-row"><span><Monitor size={17}/>{t("monitor")}</span><StudioSelect label={t("monitor")} value={String(settings.monitorIndex)} disabled={!nativeRuntime} options={monitors.length?monitors.map((monitor)=>({value:String(monitor.index),label:monitor.name})):[{value:String(settings.monitorIndex),label:t("primaryMonitor") }]} onSelect={(value)=>updatePreference({monitorIndex:Number(value)})}/></div>
      </section>
      <section class="capture-section"><h2>{t("capture")}</h2><p class="hint">{t("captureHint")}</p><div class="setting-grid">
        <button type="button" class="setting-choice" class:active={settings.captureHideOnScreenshot} aria-pressed={settings.captureHideOnScreenshot} disabled={!nativeRuntime} onclick={()=>updatePreference({captureHideOnScreenshot:!settings.captureHideOnScreenshot})}><span class="choice-mark" aria-hidden="true">{settings.captureHideOnScreenshot?"✓":""}</span><span class="choice-copy"><strong>{t("hideScreenshot")}</strong><small>{t("hideScreenshotHint")}</small></span></button>
        <button type="button" class="setting-choice" class:active={settings.captureHideOnRecording} aria-pressed={settings.captureHideOnRecording} disabled={!nativeRuntime} onclick={()=>updatePreference({captureHideOnRecording:!settings.captureHideOnRecording})}><span class="choice-mark" aria-hidden="true">{settings.captureHideOnRecording?"✓":""}</span><span class="choice-copy"><strong>{t("hideRecording")}</strong><small>{t("hideRecordingHint")}</small></span></button>
        <button type="button" class="setting-choice" class:active={settings.captureHideOnFullscreen} aria-pressed={settings.captureHideOnFullscreen} disabled={!nativeRuntime} onclick={()=>updatePreference({captureHideOnFullscreen:!settings.captureHideOnFullscreen})}><span class="choice-mark" aria-hidden="true">{settings.captureHideOnFullscreen?"✓":""}</span><span class="choice-copy"><strong>{t("hideFullscreen")}</strong><small>{t("hideFullscreenHint")}</small></span></button>
        <div class="setting-choice unavailable" aria-disabled="true"><span class="choice-mark" aria-hidden="true">—</span><span class="choice-copy"><strong>{t("hideShare")}</strong><small>{t("screenShareUnavailable")}</small></span></div>
      </div></section>
      <section class="tools-section"><h2>{t("tools")}</h2><div class="tool-list"><button disabled={!nativeRuntime} onclick={()=>windowApi.openFloatingWindow()}><ExternalLink size={17}/>{t("openFloating")}</button><button disabled={!nativeRuntime} onclick={()=>windowApi.resetFloatingWindow()}><RotateCcw size={17}/>{t("resetFloating")}</button><button disabled={!nativeRuntime} onclick={clearCache}><Trash2 size={17}/>{t("clearCache")}</button></div>{#if cacheMessage}<p class="message">{cacheMessage}</p>{/if}</section>
    </aside>
  </div>
</main>

<style>
  .player-identity{display:flex;align-items:center;gap:12px;margin-bottom:10px}
  .player-identity .hint{min-width:0;flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .player-identity button{flex-shrink:0;padding:8px 12px;border:1px solid #ffffff20;border-radius:10px;background:#ffffff08;color:inherit;cursor:pointer}
  .progress-options button{display:flex;flex-direction:column;align-items:center;justify-content:center;gap:5px;min-width:0;padding:12px 8px}
  .progress-options button span{font-weight:600;line-height:1.3}
  .progress-options button small{font-size:11px;line-height:1.5;opacity:.72}
  .netease-progress-section h2:not(:first-child){margin-top:20px}
  .netease-auto-row{margin:14px 0}
  .netease-auto-row label{display:flex;align-items:center;gap:9px;font-size:12px;cursor:pointer}
  .netease-auto-row input{margin:0;accent-color:var(--studio-accent)}
  .netease-actions{display:grid;grid-template-columns:1fr 1fr;gap:8px}
  .netease-actions button,.netease-details button{display:flex;align-items:center;justify-content:center;gap:7px;min-height:36px;padding:8px 10px;border:1px solid #40434b;border-radius:10px;background:#23262c;color:#e9eaf0;font:500 12px/1.4 var(--app-font);cursor:pointer}
  .netease-actions .netease-primary{background:#f5f6f7;color:#111216;border-color:transparent}
  .netease-details{margin-top:12px;font-size:11px;color:#aeb3c0}
  .netease-details summary{cursor:pointer;width:fit-content}
  .netease-details button{width:100%;margin-top:10px}
  .netease-shortcut{overflow-wrap:anywhere;user-select:text}
  :global(html),:global(body),:global(#app){min-width:780px;min-height:600px;background:#fff;color:#111113}
  :global(body){overflow:auto}
  main{--studio-ink:#15161a;--studio-muted:#747680;--studio-line:#e3e4e8;--studio-panel:#f5f5f7;--studio-surface:#fff;--studio-accent:#3158c8;min-height:100vh;padding:0 32px 54px;box-sizing:border-box;background:#f7f7f9}
  .studio-header{max-width:1240px;margin:0 auto 24px;display:flex;align-items:flex-end;justify-content:space-between;gap:24px}
  .settings-save-error{max-width:1240px;margin:-10px auto 18px;padding:11px 14px;display:flex;align-items:center;justify-content:space-between;gap:12px;border:1px solid #a84f4f;border-radius:10px;background:#3b2022;color:#ffdada;font-size:12px}.settings-save-error button{flex:none;padding:6px 10px;border:1px solid #d48282;border-radius:7px;background:#532a2d;color:#fff;cursor:pointer}
  .studio-brand{min-width:0}.studio-wordmark{display:block;margin-bottom:8px;color:#92939b;font-size:11px;font-weight:700;letter-spacing:.16em}.studio-brand h1{margin:0;color:var(--studio-ink);font-size:26px;line-height:1.05;letter-spacing:-.04em}.studio-brand p{max-width:48ch;margin:8px 0 0;color:var(--studio-muted);font-size:12px;line-height:1.5}
  .studio-status{display:flex;align-items:center;gap:9px;flex:none;padding:9px 11px;border:1px solid var(--studio-line);border-radius:12px;background:rgba(255,255,255,.72);color:var(--studio-muted)}.studio-status i,.stage-topline i{width:7px;height:7px;border-radius:50%;background:var(--studio-accent);box-shadow:0 0 0 4px rgba(49,88,200,.11)}.studio-status span{display:flex;flex-direction:column;gap:2px;min-width:0}.studio-status strong{color:var(--studio-ink);font-size:12px;font-weight:600}.studio-status small{max-width:170px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--studio-muted);font-size:11px}.studio-status.browser i{background:#9c9da5;box-shadow:0 0 0 4px rgba(156,157,165,.12)}
  .workspace{max-width:1240px;margin:auto;display:grid;grid-template-columns:minmax(380px,.86fr) minmax(440px,1.14fr);gap:24px;align-items:start}
  .workspace aside{width:100%;min-width:0;display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:14px;align-content:start}
  .stage{position:sticky;top:24px;height:clamp(380px,42vw,468px);display:flex;align-items:center;justify-content:center;border:1px solid #e3e4e8;border-radius:24px;background:#fff;box-shadow:0 16px 38px rgba(20,20,26,.07);overflow:hidden;contain:layout paint;isolation:isolate}
  .stage::before{content:"";position:absolute;inset:16px;border:1px dashed #e9e9ed;border-radius:18px;pointer-events:none}
  .stage-topline{position:absolute;z-index:3;top:17px;left:20px;right:20px;display:flex;align-items:center;justify-content:space-between;gap:12px;color:#898a93;font-size:11px;pointer-events:none}.stage-topline span{display:flex;align-items:center;gap:8px;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.stage-topline i{width:5px;height:5px;box-shadow:none;background:#b1b2ba}.stage-topline small{flex:none;font-size:11px;color:#a0a1a8}
  .stage.attached-top-preview .stage-topline{top:auto;bottom:50px}
  .preview-host{position:absolute;z-index:1;top:0;left:0;transform-origin:top left;will-change:transform}
  .stage :global(.island-surface){z-index:1}
  .stage-caption{position:absolute;z-index:3;left:20px;right:20px;bottom:17px;display:flex;align-items:center;justify-content:center;gap:7px;min-width:0;color:#777780;font-size:11px;pointer-events:none}
  .stage-caption strong{max-width:42%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px;font-weight:600}
  .stage-caption span{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-variant-numeric:tabular-nums}
  .stage-caption.editing{color:var(--studio-accent)}.stage-caption.editing strong{color:#2449ac}
  aside>section{min-width:0;padding:17px;border:1px solid var(--studio-line);border-radius:16px;background:var(--studio-panel)}
  aside>.appearance-section,aside>.preferences-section,aside>.players-section,aside>.idle-section{grid-column:1/-1}
  h2{margin:0 0 10px;color:var(--studio-ink);font-size:14px;letter-spacing:.01em;line-height:1.2}
  .shape-settings{display:flex;flex-direction:column;gap:14px;min-width:0;margin:14px 0 0;padding:12px 13px;border:1px solid #e5e5e9;border-radius:14px}
  .appearance-section .shape-settings :global(.studio-slider){margin-top:0}
  .shape-settings{border-color:#34373d;background:linear-gradient(160deg,#1c1e22,#151619);border-radius:16px}.shape-settings legend{padding:0 8px;font-size:13px;font-weight:650;color:#e6e8ed}
  .section-heading{display:flex;align-items:flex-start;justify-content:space-between;gap:16px;margin-bottom:16px}
  .section-heading h2{margin:0;font-size:16px;letter-spacing:-.01em}
  .live-indicator{display:flex;align-items:center;gap:6px;color:#777780;font-size:11px;white-space:nowrap}.live-indicator i{width:6px;height:6px;border-radius:50%;background:var(--studio-accent);box-shadow:0 0 0 4px rgba(49,88,200,.11)}
  .section-title{display:flex;align-items:baseline;justify-content:space-between;gap:12px}.section-title h2{margin:0}.section-note{max-width:230px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:#92939b;font-size:11px}
  .preview-state{padding-bottom:16px;border-bottom:1px solid #e5e5e8}
  .segmented{display:grid;grid-template-columns:repeat(4,1fr);padding:3px;border-radius:11px;background:#e9e9ec}
  .segmented.two{grid-template-columns:repeat(2,1fr)}
  .segmented button{border:0;background:transparent;color:#66666e;font-size:12px;cursor:pointer;transition:color 140ms ease,background 140ms ease,transform 120ms ease}.segmented button:hover{color:#111113}
  .segmented button{min-height:34px;padding:7px 4px;border-radius:8px}
  .segmented button.active{color:#111;background:#fff;box-shadow:0 2px 8px rgba(0,0,0,.08)}
  .field-label{display:flex;align-items:baseline;justify-content:space-between;margin:15px 0 8px;font-size:12px}
  .field-label:first-of-type{margin-top:0}.field-label small{color:#777780;font-size:11px}
  .select-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px}.preferences-section{background:#fff;border-color:#e0e1e6}.preferences-section .section-title{margin-bottom:12px}
  .compact-select-row{display:flex;align-items:center;justify-content:space-between;gap:12px;min-width:0;padding:10px 12px;border:1px solid transparent;border-radius:11px;background:#f6f6f8;transition:border-color 140ms ease,background 140ms ease}.compact-select-row:focus-within{border-color:#cfd3df;background:#fff}
  .compact-select-row span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px}.compact-select-row :global(.studio-select){flex:0 1 150px;margin-left:auto}
  .select-row{display:flex;align-items:center;justify-content:space-between;padding:11px 0;border-top:1px solid #e5e5e8}.select-row span{display:flex;align-items:center;gap:8px;font-size:12px}.select-row :global(.studio-select){flex:0 1 160px;margin-left:auto}
  .icon-button,.ordered-row button,.search-row button{display:grid;place-items:center;border:0;border-radius:8px;background:#fff;color:#414148;cursor:pointer;transition:background 140ms ease,color 140ms ease,transform 120ms ease}.icon-button{width:30px;height:30px}.icon-button:hover,.ordered-row button:hover,.search-row button:hover{background:#ececf0;color:#111113}
  .hint,.empty{margin:7px 0;color:#777780;font-size:11px;line-height:1.45}.ordered-list{display:flex;flex-direction:column;gap:6px;margin-top:11px}.ordered-row{display:flex;align-items:center;gap:7px;min-height:42px;padding:7px;border:1px solid transparent;border-radius:10px;background:#fff;transition:border-color 140ms ease,background 140ms ease}.ordered-row:hover{border-color:#dedfe4;background:#fcfcfd}.ordered-row.offline{opacity:.62}.ordered-row>span{min-width:0;display:flex;flex:1;flex-direction:column;gap:2px}.ordered-row strong{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px}.ordered-row small{color:#777780;font-size:11px}.ordered-row button{width:27px;height:27px}
  .skeleton-list{pointer-events:none}.skeleton-row{min-height:48px}.skeleton-row>span{gap:6px}.skeleton-row i{display:block;height:7px;border-radius:99px;background:linear-gradient(90deg,#ececf0 25%,#f7f7f8 50%,#ececf0 75%);background-size:200% 100%;animation:skeleton-shimmer 1.25s ease-in-out infinite}.skeleton-row i:first-child{width:42%}.skeleton-row i:last-child{width:24%;opacity:.75}@keyframes skeleton-shimmer{to{background-position:-200% 0}}
  .idle-fixed-hint{max-width:58ch;margin-top:0}.idle-layout-preview{display:flex;align-items:center;justify-content:space-between;margin-top:13px;padding:10px 12px;border:1px solid #e1e1e5;border-radius:10px;background:#fff;color:#4c4d55;font-size:12px}.idle-layout-preview>span{display:flex;align-items:center;gap:6px}.idle-layout-preview :global(svg){color:#3158c8}.weather-box{margin-top:14px;padding-top:13px;border-top:1px solid #e2e2e5}.weather-box>label{font-size:12px;font-weight:600}.search-row{display:grid;grid-template-columns:1fr 34px;gap:6px;margin-top:7px}.search-row input{min-width:0;border:1px solid #dddde1;border-radius:9px;padding:8px}.selected-city{margin:7px 0 0;font-size:11px;color:#39724a}.weather-results{display:flex;flex-direction:column;gap:4px;margin-top:7px}.weather-results button{display:flex;align-items:center;justify-content:space-between;border:0;border-radius:8px;padding:8px;background:#fff;text-align:left;cursor:pointer}.weather-results small{color:#777780}
  .tool-list button{display:flex;align-items:center;gap:8px;border:1px solid transparent;border-radius:11px;color:#414148;background:#fff;cursor:pointer;transition:color 140ms ease,background 140ms ease,border-color 140ms ease,transform 120ms ease}.tool-list button:hover{border-color:#d9dbe1;color:#111113;background:#fcfcfd}.tool-list{display:flex;flex-direction:column;gap:7px}.tool-list button{padding:10px 11px;font-size:12px}.tool-list button:active,.icon-button:active,.ordered-row button:active{transform:scale(.97)}
  button:disabled,input:disabled{cursor:not-allowed;opacity:.45}.message{margin:10px 0 0;color:#39724a;font-size:12px}.segmented button:focus-visible,.tool-list button:focus-visible,.icon-button:focus-visible,.ordered-row button:focus-visible,.weather-results button:focus-visible,input:focus-visible{outline:2px solid #111;outline-offset:2px}
  .setting-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:7px}.setting-grid.single{grid-template-columns:1fr}.setting-choice{min-width:0;min-height:66px;display:flex;align-items:flex-start;gap:8px;padding:11px;border:1px solid #e1e1e5;border-radius:11px;color:#36363d;background:#fff;text-align:left;cursor:pointer;transition:transform 120ms ease,background 140ms ease,border-color 140ms ease,color 140ms ease}.setting-choice:hover{border-color:#cfd1d8;background:#fcfcfd}.setting-choice.active{border-color:#111113;color:#fff;background:#111113}.setting-choice.active:hover{border-color:#111113;background:#202126}.setting-choice:active{transform:scale(.98)}.setting-choice:disabled{cursor:not-allowed}.setting-choice.unavailable{cursor:not-allowed;opacity:.7}.choice-mark{display:grid;place-items:center;flex:none;width:17px;height:17px;margin-top:1px;border:1px solid #bdbdc4;border-radius:5px;color:transparent;font-size:12px;font-weight:800;line-height:1}.setting-choice.active .choice-mark,.player-selection .choice-mark{border-color:#fff;color:#111113;background:#fff}.choice-copy{min-width:0;display:flex;flex-direction:column;gap:4px}.choice-copy strong{font-size:12px;line-height:1.2}.choice-copy small{color:#777780;font-size:11px;line-height:1.35}.setting-choice.active .choice-copy small{color:rgba(255,255,255,.68)}.player-selection{width:27px!important;height:27px!important;padding:0!important;border:0!important;background:transparent!important}.player-selection .choice-mark{width:17px;height:17px;margin:0}.setting-choice:focus-visible,.player-selection:focus-visible{outline:2px solid #111;outline-offset:2px}.player-selection:focus-visible{outline-color:#111}.setting-choice.active:focus-visible{outline-color:#fff}
  /* Studio rhythm: controls within a group stay close; groups and panels breathe. */
  main{--studio-space-2:7px;--studio-space-3:9px;--studio-space-4:12px;--studio-space-5:15px}
  .workspace aside{gap:var(--studio-space-3)}
  aside>section{padding:var(--studio-space-4)}
  .section-title{margin-bottom:var(--studio-space-2)}
  .section-title+.hint{margin-top:0}
  .preview-state{padding-bottom:var(--studio-space-3);margin-bottom:var(--studio-space-3)}
  .preview-state .field-label,.preview-setting .field-label{margin:0 0 var(--studio-space-2)}
  .appearance-section>.preview-setting{margin-top:var(--studio-space-3)}
  .appearance-section :global(.studio-slider){margin-top:var(--studio-space-3)}
  .select-grid{gap:var(--studio-space-2)}
  .setting-grid{gap:7px}
  .volume-settings{display:flex;flex-direction:column;gap:8px;margin-bottom:14px}
  .volume-protection-options{display:flex;flex-direction:column;gap:10px;padding:9px;border:1px solid #35383e;border-radius:14px;background:linear-gradient(160deg,#1b1d21,#141518)}
  .volume-protection-limit{padding:2px 3px 0}
  .volume-protection-limit :global(.studio-slider){gap:7px}
  .volume-protection-limit :global(.range-label){font-size:12px}
  .setting-choice{min-height:56px;padding:10px}
  .compact-select-row{min-height:42px;padding:8px 10px}
  .ordered-list{margin-top:var(--studio-space-2);gap:6px}

  /* Each control has a readable rest, hover, selected, focus, and disabled state. */
  .segmented button:hover:not(.active){background:rgba(255,255,255,.55)}
  .segmented button.active{box-shadow:0 1px 4px rgba(20,20,26,.11)}
  .setting-choice:hover:not(:disabled):not(.active){border-color:#b9bdc8;background:#fff}
  .tool-list button:hover:not(:disabled){border-color:#b9bdc8}
  .icon-button:hover:not(:disabled),.ordered-row button:hover:not(:disabled),.search-row button:hover:not(:disabled){background:#e9eaee}
  .compact-select-row:hover:not(:focus-within){background:#f0f1f4}
  .compact-select-row:focus-within{border-color:var(--studio-accent);background:#fff}
  .search-row input:focus-visible{outline:2px solid var(--studio-accent);outline-offset:2px}
  .segmented button:focus-visible,.setting-choice:focus-visible,.tool-list button:focus-visible,.icon-button:focus-visible,.ordered-row button:focus-visible,.weather-results button:focus-visible,.search-row button:focus-visible{outline:2px solid var(--studio-accent);outline-offset:2px}
  .setting-choice.active:focus-visible{outline-offset:3px}
  button:disabled,input:disabled{cursor:not-allowed}
  .setting-choice:disabled{opacity:1;color:#858892;background:#eceef1;border-color:#eceef1}
  .setting-choice:disabled .choice-copy small{color:#858892}
  .setting-choice.active:disabled{color:#555963;background:#e1e3e8;border-color:#e1e3e8}
  .setting-choice:disabled .choice-mark{border-color:#a8acb5}
  .icon-button:disabled,.ordered-row button:disabled,.search-row button:disabled{background:#eff0f2;color:#a1a4ad}

  /* Match the island shell: deep black canvas, layered slate surfaces, quiet highlights. */
  :global(html),:global(body),:global(#app){background:#08090b;color:#f4f5f7}
  main{--studio-ink:#f4f5f7;--studio-muted:#aeb2bd;--studio-line:#35383e;--studio-panel:#17191d;--studio-surface:#22252a;--studio-accent:#91caff;--studio-control-ink:#f4f5f7;--studio-control-muted:#aeb2bd;--studio-range-accent:#91caff;background:#0b0c0e;color:var(--studio-ink)}
  .studio-wordmark{color:#a7aab4}
  .studio-status{background:#17191d;border-color:#34373e;color:var(--studio-muted);border-radius:14px}
  .studio-status.browser i{background:#9da1ab;box-shadow:0 0 0 4px rgba(157,161,171,.14)}
  .stage{border-color:#363940;box-shadow:0 20px 48px rgba(0,0,0,.34)}
  .stage-caption,.stage-caption strong{color:#777d88}
  .stage-caption.editing,.stage-caption.editing strong{color:#91caff}
  aside>section,.preferences-section{background:linear-gradient(160deg,#191b1f 0%,#141518 100%);border-color:var(--studio-line);box-shadow:inset 0 1px rgba(255,255,255,.025),0 10px 26px rgba(0,0,0,.12);border-radius:18px}
  .section-note,.live-indicator,.field-label small,.hint,.empty,.ordered-row small,.choice-copy small{color:var(--studio-muted)}
  .preview-state,.select-row,.weather-box{border-color:#34373d}
  .segmented{background:#0b0c0e;border:1px solid #2b2e34}
  .segmented button{color:#aeb2bd}
  .segmented button:hover:not(.active){color:#fff;background:#292c32}
  .segmented button.active{color:#f5f6f7;background:#30333a;box-shadow:inset 0 1px rgba(255,255,255,.08),0 2px 8px rgba(0,0,0,.24)}
  .setting-choice,.tool-list button,.weather-results button,.ordered-row,.idle-layout-preview{color:#e9ebef;background:linear-gradient(160deg,#24272c,#1d1f23);border-color:#383b42}.setting-choice{border-radius:14px}
  .setting-choice:hover:not(:disabled):not(.active),.tool-list button:hover:not(:disabled),.weather-results button:hover,.ordered-row:hover{color:#fff;background:#2a2d33;border-color:#555a64}
  .setting-choice.active{color:#f5f6f7;background:linear-gradient(160deg,#30333a,#23262b);border-color:#626873;box-shadow:inset 0 1px rgba(255,255,255,.09),0 5px 14px rgba(0,0,0,.16)}
  .setting-choice.active:hover:not(:disabled){background:linear-gradient(160deg,#383c44,#282b31);border-color:#7c8490}
  .netease-actions .netease-primary{color:#f4f5f7;background:linear-gradient(160deg,#343840,#24272d);border-color:#555a64}
  .setting-choice.active .choice-copy small{color:#b7bdc8}
  .choice-mark{border-color:#737983}
  .setting-choice.active .choice-mark,.player-selection .choice-mark{color:#111216;background:#91caff;border-color:#91caff}
  .player-selection[aria-checked="false"] .choice-mark{color:transparent;background:transparent;border-color:#81858e}
  .setting-choice:disabled{color:#858a95;background:#1b1d21;border-color:#2d3036}
  .setting-choice.active:disabled{color:#aeb2bd;background:#282b30;border-color:#3a3e46}
  .setting-choice:disabled .choice-copy small{color:#858995}
  .setting-choice:disabled .choice-mark{border-color:#777b85}
  .compact-select-row{background:#202227;border-color:#373a41;border-radius:12px}
  .compact-select-row:hover:not(:focus-within){background:#292c32}
  .compact-select-row:focus-within{background:#24272d;border-color:var(--studio-accent)}
  .search-row input{color:#f2f3f5;background:#1d1f23;border:1px solid #41444c;border-radius:11px}
  .preferences-section{position:relative;z-index:2}
  .search-row input::placeholder{color:#969aa5}
  .icon-button,.ordered-row button,.search-row button{color:#edf0f4;background:#22252a;border:1px solid #3b3e46}
  .icon-button:hover:not(:disabled),.ordered-row button:hover:not(:disabled),.search-row button:hover:not(:disabled){color:#fff;background:#30343b;border-color:#626873}
  .icon-button:disabled,.ordered-row button:disabled,.search-row button:disabled{color:#81858e;background:#1b1d21;border-color:#2d3036}
  .idle-layout-preview :global(svg){color:#84a8ff}
  .selected-city,.message{color:#9fd2ad}
  .skeleton-row i{background:linear-gradient(90deg,#2a2d33 25%,#393c43 50%,#2a2d33 75%);background-size:200% 100%}

  /* Settings take the island's black surface, translucent controls and compact spacing. */
  :global(html),:global(body),:global(#app){background:#111214;color:#fff}
  main{--studio-ink:#fff;--studio-muted:rgba(255,255,255,.68);--studio-line:rgba(255,255,255,.10);--studio-panel:#000;--studio-surface:rgba(255,255,255,.06);--studio-accent:#91caff;--studio-control-ink:#fff;--studio-control-muted:rgba(255,255,255,.68);--studio-range-accent:#fff;background:#111214;padding-top:24px;font-family:var(--app-font);font-synthesis:none}
  .studio-header{align-items:center;margin-bottom:24px}
  .studio-brand h1{font-size:24px;font-weight:700;letter-spacing:-.03em}
  .studio-wordmark{color:rgba(255,255,255,.50);font-weight:500;letter-spacing:.12em}
  .studio-brand p{color:rgba(255,255,255,.55)}
  .studio-status{background:rgba(255,255,255,.06);border-color:rgba(255,255,255,.10);border-radius:999px}
  .workspace{gap:20px}
  .stage{background:#202124;border-color:rgba(255,255,255,.10);border-radius:32px;box-shadow:none}
  .stage::before{display:none}
  .stage-topline,.stage-topline small,.stage-caption,.stage-caption strong{color:rgba(255,255,255,.68)}
  .stage-caption.editing,.stage-caption.editing strong{color:#91caff}
  .workspace aside{display:flex;flex-direction:column;gap:0;background:#000;border:1px solid rgba(255,255,255,.10);border-radius:32px;min-width:0}
  .settings-nav{position:sticky;top:0;z-index:10;display:flex;gap:4px;align-items:center;overflow-x:auto;overscroll-behavior-x:contain;padding:14px 20px;background:#000;border-radius:31px 31px 0 0;border-bottom:1px solid rgba(255,255,255,.10);scrollbar-width:thin}
  .settings-nav a{display:flex;flex:0 0 auto;align-items:center;justify-content:center;gap:7px;min-height:36px;padding:0 12px;border-radius:12px;color:rgba(255,255,255,.68);text-decoration:none;font-size:12px;font-weight:500;white-space:nowrap;transition:background 140ms ease,color 140ms ease}
  .settings-nav a:hover{color:#fff;background:rgba(255,255,255,.10)}
  .settings-nav a:focus-visible{outline:2px solid #91caff;outline-offset:-2px;color:#fff;background:rgba(255,255,255,.10)}
  aside>section,.preferences-section{scroll-margin-top:76px;width:100%;box-sizing:border-box;padding:24px;border:0;border-bottom:1px solid rgba(255,255,255,.10);border-radius:0;background:transparent;box-shadow:none}
  aside>section:last-child{border-bottom:0}
  aside h2,.section-heading h2{font-size:18px;font-weight:700;line-height:1.2;letter-spacing:0}
  .section-heading{align-items:center;margin-bottom:20px}
  .section-note,.live-indicator,.field-label small,.hint,.empty,.ordered-row small,.choice-copy small{color:rgba(255,255,255,.55)}
  .shape-settings{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:16px;margin-top:16px;padding:16px;border:1px solid rgba(255,255,255,.10);border-radius:20px;background:rgba(255,255,255,.035)}
  .shape-settings legend{color:rgba(255,255,255,.68);font-weight:500}
  .segmented{padding:4px;background:rgba(255,255,255,.06);border:0;border-radius:14px}
  .segmented button{min-height:36px;border-radius:10px;color:rgba(255,255,255,.68)}
  .segmented button:hover:not(.active){color:#fff;background:rgba(255,255,255,.10)}
  .segmented button.active{color:#fff;background:rgba(255,255,255,.16);box-shadow:none}
  .segmented button:active{transform:scale(.97)}
  .setting-grid{gap:6px}
  .setting-choice,.tool-list button,.weather-results button,.ordered-row,.idle-layout-preview{color:rgba(255,255,255,.95);background:transparent;border-color:rgba(255,255,255,.10);box-shadow:none}
  .setting-choice{min-height:56px;border-radius:12px}
  .setting-choice:hover:not(:disabled):not(.active),.tool-list button:hover:not(:disabled),.weather-results button:hover,.ordered-row:hover{color:#fff;background:rgba(255,255,255,.10);border-color:rgba(255,255,255,.18)}
  .setting-choice.active,.setting-choice.active:hover:not(:disabled){color:#fff;background:rgba(255,255,255,.08);border-color:rgba(255,255,255,.22);box-shadow:none}
  .setting-choice.active .choice-copy small{color:rgba(255,255,255,.68)}
  .setting-choice.active .choice-mark,.player-selection .choice-mark{color:#000;background:#fff;border-color:#fff}
  .setting-choice:disabled,.setting-choice.active:disabled{color:rgba(255,255,255,.45);background:rgba(255,255,255,.035);border-color:rgba(255,255,255,.07);box-shadow:none}
  .setting-choice:disabled .choice-copy small{color:rgba(255,255,255,.45)}
  .volume-protection-options{padding:0 0 0 12px;border:0;border-left:1px solid rgba(255,255,255,.19);border-radius:0;background:transparent}
  .compact-select-row,.compact-select-row:hover:not(:focus-within),.compact-select-row:focus-within{background:rgba(255,255,255,.06);border-color:transparent;border-radius:12px}
  .compact-select-row:focus-within{border-color:#91caff}
  .netease-actions button,.netease-details button,.netease-actions .netease-primary,.icon-button,.ordered-row button,.search-row button{color:#fff;background:rgba(255,255,255,.06);border-color:rgba(255,255,255,.10);border-radius:12px}
  .netease-actions button:hover:not(:disabled),.netease-details button:hover:not(:disabled),.icon-button:hover:not(:disabled),.ordered-row button:hover:not(:disabled),.search-row button:hover:not(:disabled){background:rgba(255,255,255,.10);border-color:rgba(255,255,255,.22)}
  .search-row input{color:#fff;background:rgba(255,255,255,.06);border-color:rgba(255,255,255,.19)}
  .idle-layout-preview :global(svg){color:#91caff}
  @media(max-width:1040px){main{padding-inline:24px}.workspace{grid-template-columns:minmax(340px,.8fr) minmax(400px,1.2fr);gap:18px}}
  @media(max-width:900px){:global(html),:global(body),:global(#app){min-width:0}main{padding:20px 18px 36px}.studio-header{align-items:flex-start;flex-direction:column;gap:14px;margin-bottom:18px}.studio-status{align-self:stretch}.workspace{grid-template-columns:1fr}.stage{position:relative;top:0;height:340px}.workspace aside{grid-template-columns:repeat(2,minmax(0,1fr))}}
  @media(max-width:620px){main{padding:16px 12px 28px}.shape-settings{grid-template-columns:1fr}.settings-nav{padding:10px 12px}.settings-nav a{padding:0 10px} .workspace aside{gap:0} .workspace aside>section{padding:20px 16px}.studio-brand h1{font-size:23px}.studio-brand p{font-size:12px}.stage{height:300px;border-radius:20px}.stage::before{inset:12px;border-radius:15px}.stage-topline{left:15px;right:15px;top:14px}.workspace aside{grid-template-columns:1fr;gap:0}aside>section{padding:16px}aside>.appearance-section,aside>.preferences-section,aside>.players-section,aside>.idle-section{grid-column:auto}.select-grid{grid-template-columns:1fr}}
  @media(prefers-reduced-motion:reduce){.settings-nav a{transition:none}.skeleton-row i{animation:none}.setting-choice,.segmented button,.tool-list button,.ordered-row,.icon-button,.ordered-row button,.search-row button{transition:none}}
  @media(prefers-reduced-transparency:reduce){}
</style>

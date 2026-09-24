<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { spring } from "svelte/motion";
  import { RotateCcw,Trash2,ExternalLink,Monitor,RefreshCw,ChevronUp,ChevronDown,CloudSun,Search } from "lucide-svelte";
  import IslandSurface from "$lib/IslandSurface.svelte";
  import StudioSlider from "$lib/StudioSlider.svelte";
  import StudioSelect from "$lib/StudioSelect.svelte";
  import { CUSTOM_PANEL_WIDTH, geometryFor, hostFor, type IslandMode } from "$lib/islandGeometry";
  import { DEMO_MEDIA, media, connectMedia } from "$lib/mediaStore";
  import { extractSpectrumColorsFromImage } from "$lib/spectrumColors";
  import { settingsApi } from "$lib/api/settings";
  import { windowApi } from "$lib/api/window";
  import { cacheApi } from "$lib/api/cache";
  import { mediaApi } from "$lib/api/media";
  import { playerNameFromId } from "$lib/playerNames";
  import { moveSelectedPlayer } from "$lib/playerOrder";
  import { idleApi } from "$lib/api/idle";
  import { applyAppFont } from "$lib/font";
  import { locale, setLocale, translate, type TranslationKey } from "$lib/i18n";
  import { DEFAULT_SETTINGS, type AppLanguage, type AppPreferences, type IdleSnapshot, type MediaSessionInfo, type MediaState, type MonitorInfo, type WeatherForecastDay, type WeatherLocationCandidate } from "$lib/api/types";
  import type { IslandTool } from "$lib/featureRail";
  import { formatClock } from "$lib/countdown";
  let previewNow = $state(Date.now());

  type Scenario="playing"|"paused"|"no-art"|"long-title";
  type PreviewTarget="shape"|"compactLength"|"shoulder"|"radius"|"background"|"spectrum"|null;
  type AppearanceDraft=Pick<AppPreferences,"islandStyle"|"islandEdge"|"islandEdgePosition"|"compactLength"|"collapsedEdgeShoulderRadius"|"expandedEdgeShoulderRadius"|"expandedCornerRadius"|"showSpectrum"|"spectrumMode"|"floatingUseAlbumColor"|"floatingFillColor">;

  function createAppearanceDraft(source:AppPreferences):AppearanceDraft{
    return {
      islandStyle:source.islandStyle,
      islandEdge:source.islandEdge,
      islandEdgePosition:source.islandEdgePosition,
      compactLength:source.compactLength,
      collapsedEdgeShoulderRadius:source.collapsedEdgeShoulderRadius,
      expandedEdgeShoulderRadius:source.expandedEdgeShoulderRadius,
      expandedCornerRadius:source.expandedCornerRadius,
      showSpectrum:source.showSpectrum,
      spectrumMode:source.spectrumMode,
      floatingUseAlbumColor:source.floatingUseAlbumColor,
      floatingFillColor:source.floatingFillColor,
    };
  }

  let mode=$state<IslandMode>("expanded"); let scenario=$state<Scenario>("playing"); let progress=$state(50); let settings=$state<AppPreferences>({...DEFAULT_SETTINGS}); let monitors=$state<MonitorInfo[]>([]); let cacheMessage=$state("");
  const previewClock = $derived(formatClock(settings.clockTimeZone, $locale, previewNow));
  let nativeRuntime=$state(false); let followingLive=$state(false); let liveMedia=$state<MediaState>(DEMO_MEDIA); let liveMediaDisconnect:undefined|(()=>void); let stageWidth=$state(0); let stageHeight=$state(0); let previewReady=$state(false);
  let idleSnapshot=$state<IdleSnapshot|null>(null);
  const demoForecast:WeatherForecastDay[]=[0,1,2,3].map((offset)=>({date:new Date(Date.now()+offset*86_400_000).toISOString().slice(0,10),weatherCode:[1,61,2,3][offset],temperatureMax:[33,34,34,32][offset],temperatureMin:[26,24,24,23][offset]}));
  let sessions=$state<MediaSessionInfo[]>([]); let sessionsLoading=$state(false); let sessionsLoaded=$state(false); let sessionRequest=0; let weatherQuery=$state(""); let weatherResults=$state<WeatherLocationCandidate[]>([]); let weatherMessage=$state("");
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let persistInFlight: Promise<void> | undefined;
  let persistPending=false;
  let pendingSettingsPatch:Partial<AppPreferences>={};
  let appearanceDraft=$state<AppearanceDraft>(createAppearanceDraft(DEFAULT_SETTINGS));
  let previewTarget=$state<PreviewTarget>(null);
  let applyingAppearance=$state(false);
  let appearanceDirty=$derived.by(()=>{
    return appearanceDraft.islandStyle!==settings.islandStyle||
      appearanceDraft.islandEdge!==settings.islandEdge||
      appearanceDraft.islandEdgePosition!==settings.islandEdgePosition||
      appearanceDraft.compactLength!==settings.compactLength||
      appearanceDraft.collapsedEdgeShoulderRadius!==settings.collapsedEdgeShoulderRadius||
      appearanceDraft.expandedEdgeShoulderRadius!==settings.expandedEdgeShoulderRadius||
      appearanceDraft.expandedCornerRadius!==settings.expandedCornerRadius||
      appearanceDraft.showSpectrum!==settings.showSpectrum||
      appearanceDraft.spectrumMode!==settings.spectrumMode||
      appearanceDraft.floatingUseAlbumColor!==settings.floatingUseAlbumColor||
      appearanceDraft.floatingFillColor!==settings.floatingFillColor;
  });
  let studioDisposed=false;
  const previewBars=[.45,.78,.58,.96,.7,.38];
  let scenarioMedia=$derived.by<MediaState>(()=>{const base={...DEMO_MEDIA,positionMs:244000*progress/100,lastUpdatedTimestamp:Date.now()};if(scenario==="paused")return{...base,isPlaying:false};if(scenario==="no-art")return{...base,albumArt:""};if(scenario==="long-title")return{...base,title:"宇宙尽头的浪漫主义与一场不会结束的午夜公路旅行",artist:"The Extremely Long Artist Name · 特别长的专辑名称"};return base});
  let sample=$derived(nativeRuntime&&followingLive?liveMedia:scenarioMedia);
  let previewForecast=$derived(nativeRuntime?(idleSnapshot?.weatherForecast??[]):demoForecast);
  let previewSpectrumTopColor=$state("#fff");
  let previewSpectrumBottomColor=$state("#888");
  $effect(()=>{
    const cover=sample.albumArt;
    if(!cover){previewSpectrumTopColor="#fff";previewSpectrumBottomColor="#888";return}
    let cancelled=false;
    void extractSpectrumColorsFromImage(cover).then(colors=>{
      if(cancelled||!colors)return;
      previewSpectrumTopColor=`rgb(${colors.top.join(",")})`;
      previewSpectrumBottomColor=`rgb(${colors.bottom.join(",")})`;
    }).catch(()=>{});
    return()=>{cancelled=true};
  });
  // Keep the island at the current preview scale while the surrounding stage changes width.
  const previewScaleFloor=.79;
  let previewTools=$derived.by<IslandTool[]>(()=>[
    ...(settings.showSettingsTool?["settings"]:[]),
    ...(settings.showFloatingTool?["floating"]:[]),
    ...(settings.showVolumeTool?["volume"]:[]),
    ...(settings.showTimerTool?["timer"]:[]),
    ...(settings.showHideTool?["hide"]:[]),
  ] as IslandTool[]);
  let previewPanelVisible=$derived(settings.showCustomFunctionPanel&&previewTools.length>0);
  let previewPanelExtraWidth=$derived(previewPanelVisible?CUSTOM_PANEL_WIDTH:0);
  let previewExpandedExtraWidth=$derived(previewPanelExtraWidth);
  let previewExpandedRadius=$derived(appearanceDraft.expandedCornerRadius);
  let previewHost=$derived(hostFor(appearanceDraft.islandStyle,appearanceDraft.islandEdge,appearanceDraft.compactLength,previewExpandedExtraWidth));
  let currentGeometry=$derived(geometryFor(mode,previewExpandedRadius,appearanceDraft.islandEdge,appearanceDraft.compactLength,previewExpandedExtraWidth));
  let previewScale=$derived(Math.min(previewScaleFloor, Math.max(0.1, (stageWidth - 24) / previewHost.width)));
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
  async function enableLivePreview(){
    if(!nativeRuntime)return;
    if(liveMediaDisconnect)return;
    try{
      const currentMedia=await mediaApi.getMediaInfo();
      if(studioDisposed)return;
      liveMedia=currentMedia;
      const disconnect=await connectMedia();
      if(studioDisposed)disconnect();
      else{liveMediaDisconnect=disconnect;followingLive=true}
    }catch{liveMediaDisconnect=undefined;followingLive=false}
  }
  async function refreshIdleSnapshot(){
    if(!nativeRuntime)return;
    try{const snapshot=await idleApi.getSnapshot();if(!studioDisposed)idleSnapshot=snapshot}catch{}
  }
  const t=(key:TranslationKey,values:Record<string,string|number>={})=>translate(key,values,$locale);
  const enumLabel=(value:string)=>t(value as TranslationKey);
  const previewTargetLabel=(target:PreviewTarget)=>target? t(({shape:"shape",compactLength:"compactLength",shoulder:"shoulder",radius:"expandedRadius",background:"floatingBackground",spectrum:"spectrum"} as const)[target]):t("previewReady");
  function syncAppearanceDrafts(){
    appearanceDraft=createAppearanceDraft(settings);
  }
  onMount(()=>{
    studioDisposed=false;
    const clockTimer=setInterval(()=>previewNow=Date.now(),1000);
    nativeRuntime=Boolean((window as any).__TAURI_INTERNALS__);
    const unsubscribe=media.subscribe(value=>liveMedia=value);
    let disposed=false;
    let deferredFrame=0;
    let deferredTimers:ReturnType<typeof setTimeout>[]=[];
    let closePromise:Promise<()=>void>|undefined;
    if(nativeRuntime){
      const appWindow=getCurrentWindow();
      closePromise=appWindow.onCloseRequested(async(event)=>{
        event.preventDefault();
        await appWindow.hide();
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
      void refreshIdleSnapshot();
      const idleTimer=setInterval(()=>void refreshIdleSnapshot(),30_000);
      deferredTimers.push(idleTimer);
      void (async()=>{
        try{
          settings={...DEFAULT_SETTINGS,...await settingsApi.getPreferences()};
          if(disposed)return;
          syncAppearanceDrafts();
          applyAppFont(settings.fontId);
          setLocale(settings.language);
        }catch{}
        if(disposed)return;
        void refreshSessions();
        deferredTimers.push(setInterval(()=>void refreshSessions(),10_000));
      })();
    }
    return()=>{
      disposed=true;
      clearInterval(clockTimer);
      studioDisposed=true;
      if(deferredFrame)cancelAnimationFrame(deferredFrame);
      deferredTimers.forEach(timer=>clearTimeout(timer));
      if(closePromise)void closePromise.then(unlisten=>unlisten()).catch(()=>{});
      if(saveTimer)clearTimeout(saveTimer);
      void persist();
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
        if(Object.keys(patch).length) await settingsApi.updateSettings(patch).catch(()=>{});
      }
    })().finally(()=>{persistInFlight=undefined});
    await persistInFlight;
  }
  function schedulePersist(){if(saveTimer)clearTimeout(saveTimer);saveTimer=setTimeout(()=>void persist(),300)}
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
  async function setAlwaysOnTop(value:boolean){const previous=settings.alwaysOnTop;settings={...settings,alwaysOnTop:value};if(!nativeRuntime)return;try{await settingsApi.setAlwaysOnTop(value)}catch(error){settings={...settings,alwaysOnTop:previous};console.error("[置顶] 应用设置失败:",error)}}
  async function setAutoStart(value:boolean){const previous=settings.autoStart;settings={...settings,autoStart:value};if(!nativeRuntime)return;try{await settingsApi.setAutoStart(value)}catch{settings={...settings,autoStart:previous}}}
  function resetAppearanceDraft(){appearanceDraft=createAppearanceDraft(settings);previewTarget=null}
  async function applyAppearance(){
    if(!appearanceDirty||applyingAppearance)return;
    applyingAppearance=true;
    try{
      commitPreference({...appearanceDraft});
      await persist();
      syncAppearanceDrafts();
      previewTarget=null;
    }finally{applyingAppearance=false}
  }
  function setLanguage(value:AppLanguage){setLocale(value);updatePreference({language:value})}
  function mergePlayerOrderIds(existing:string[],available:MediaSessionInfo[],selected:string[]|null){
    return [...new Set([...existing,...available.map(item=>item.id),...(selected??[])].filter(id=>id.trim().length>0))];
  }
  function currentPlayerOrderIds(){return mergePlayerOrderIds(settings.playerOrderIds,sessions,settings.selectedPlayerIds)}
  function sessionFor(id:string):MediaSessionInfo{return sessions.find(item=>item.id===id)??{id,displayName:playerNameFromId(id),source:"generic",isPlaying:false}}
  async function refreshSessions(){
    if(!nativeRuntime)return;
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
    finally{if(request===sessionRequest){sessionsLoading=false;sessionsLoaded=true}}
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
  function selectWeather(item:WeatherLocationCandidate){updatePreference({weatherLocation:{name:weatherLocationLabel(item),latitude:item.latitude,longitude:item.longitude}});weatherResults=[];weatherMessage=t("citySaved");setTimeout(()=>void refreshIdleSnapshot(),600)}
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
  <div class="workspace">
    <section class="stage" class:attached-top-preview={appearanceDraft.islandStyle === "edge" && appearanceDraft.islandEdge === "top"} aria-label={t("previewLabel")} bind:clientWidth={stageWidth} bind:clientHeight={stageHeight}>
      <div class="stage-topline"><span><i aria-hidden="true"></i>{t("previewLabel")}</span><small>{nativeRuntime&&followingLive?t("currentMusic"):nativeRuntime?t("previewReady"):t("browserDemo")}</small></div>
      <div class="preview-host" style={`width:${previewHost.width}px;height:${previewHost.height}px;${previewPositionStyle}`}>
        {#if previewReady}
        <IslandSurface media={sample} clockText={previewClock} clockTimeZone={settings.clockTimeZone} idle={nativeRuntime && followingLive && !sample.title} idleTime={previewClock} idleWeatherTemperature={idleSnapshot?.weatherTemperature ?? null} idleWeatherCode={idleSnapshot?.weatherCode ?? null} idleWeatherForecast={previewForecast} {mode} islandStyle={appearanceDraft.islandStyle} edge={appearanceDraft.islandEdge} position={sample.positionMs} expandedRadius={appearanceDraft.expandedCornerRadius} collapsedEdgeShoulderRadius={appearanceDraft.collapsedEdgeShoulderRadius} expandedEdgeShoulderRadius={appearanceDraft.expandedEdgeShoulderRadius} compactLength={appearanceDraft.compactLength} showSpectrum={appearanceDraft.showSpectrum} spectrumMode={appearanceDraft.spectrumMode} spectrumTopColor={previewSpectrumTopColor} spectrumBottomColor={previewSpectrumBottomColor} enableAnimations={settings.enableAnimations} reduceAnimations={settings.reduceAnimations} previewSpectrum={!nativeRuntime&&appearanceDraft.spectrumMode==="realtime"?previewBars:undefined} previewHighlight={previewTarget === "background" ? null : previewTarget} previewEdgePosition={appearanceDraft.islandEdgePosition} interactive simulateHidden enabledTools={previewTools} showCustomFunctionPanel={settings.showCustomFunctionPanel} systemAudio={{volumePercent:42,muted:false,deviceId:"studio-speakers",deviceName:"Speakers"}} audioDevices={[{id:"studio-speakers",name:"Speakers (Realtek Audio)",isDefault:true},{id:"studio-headphones",name:"Headphones",isDefault:false}]} onSettingsToggle={()=>{if(nativeRuntime)void windowApi.showStudioWindow()}} onToggle={()=>mode=mode==="expanded"?"compact":"expanded"} onMediaAction={(action)=>{if(nativeRuntime&&followingLive){void mediaApi.controlMedia(action);return}if(action==="play_pause")scenario=scenario==="paused"?"playing":"paused"}} />
        {/if}
      </div>
      <div class="stage-caption" class:editing={previewTarget!==null} aria-live="polite">
        <strong>{previewTarget?t("previewEditing"):t("previewReady")}{previewTarget?`：${previewTargetLabel(previewTarget)}`:""}</strong>
        <span>{enumLabel(mode)} · {enumLabel(appearanceDraft.islandStyle)} · {enumLabel(appearanceDraft.islandEdge)} · {currentGeometry.width} × {currentGeometry.height} · r{currentGeometry.radius}</span>
      </div>
    </section>
    <aside>
      <section class="appearance-section">
        <div class="section-heading"><h2>{t("islandLayout")}</h2><span class="live-indicator"><i></i>{t("previewReady")}</span></div>
        <div class="preview-state"><div class="field-label"><span>{t("previewState")}</span><small>{t("previewReady")}</small></div><div class="segmented">{#each ["compact","hover","expanded","hidden"] as item}<button aria-pressed={mode===item} class:active={mode===item} onclick={()=>mode=item as IslandMode}>{enumLabel(item)}</button>{/each}</div></div>
        <div class="preview-setting" role="group" data-preview-target="shape" onmouseenter={()=>previewTarget="shape"} onfocusin={()=>previewTarget="shape"} onmouseleave={()=>previewTarget=null} onfocusout={()=>previewTarget=null}>
          <div class="field-label"><span>{t("shape")}</span><small>{t("shapeHint")}</small></div>
          <div class="segmented two"><button aria-pressed={appearanceDraft.islandStyle==="floating"} class:active={appearanceDraft.islandStyle==="floating"} onclick={()=>{appearanceDraft.islandStyle="floating";previewTarget="shape"}}>{t("floating")}</button><button aria-pressed={appearanceDraft.islandStyle==="edge"} class:active={appearanceDraft.islandStyle==="edge"} onclick={()=>{appearanceDraft.islandStyle="edge";previewTarget="shape"}}>{t("attached")}</button></div>
        </div>
        <StudioSlider label={t("compactLength")} hint={t("shapeHint")} min={80} max={300} unit="px" bind:value={appearanceDraft.compactLength} previewTarget={previewTarget === "compactLength" ? "compactLength" : null} onPreviewStart={()=>previewTarget="compactLength"} onPreviewMove={()=>previewTarget="compactLength"} onPreviewEnd={()=>previewTarget=null}/>
        <StudioSlider label={t("collapsedShoulder")} min={0} max={16} unit="px" bind:value={appearanceDraft.collapsedEdgeShoulderRadius} previewTarget={previewTarget === "shoulder" ? "shoulder" : null} onPreviewStart={()=>previewTarget="shoulder"} onPreviewMove={()=>previewTarget="shoulder"} onPreviewEnd={()=>previewTarget=null}/>
        <StudioSlider label={t("expandedShoulder")} min={0} max={64} unit="px" bind:value={appearanceDraft.expandedEdgeShoulderRadius} previewTarget={previewTarget === "shoulder" ? "shoulder" : null} onPreviewStart={()=>previewTarget="shoulder"} onPreviewMove={()=>previewTarget="shoulder"} onPreviewEnd={()=>previewTarget=null}/>
        <StudioSlider label={t("expandedRadius")} min={0} max={80} unit="px" bind:value={appearanceDraft.expandedCornerRadius} previewTarget={previewTarget === "radius" ? "radius" : null} onPreviewStart={()=>previewTarget="radius"} onPreviewMove={()=>previewTarget="radius"} onPreviewEnd={()=>previewTarget=null}/>
        <div class="preview-setting" role="group" data-preview-target="background" onmouseenter={()=>previewTarget="background"} onfocusin={()=>previewTarget="background"} onmouseleave={()=>previewTarget=null} onfocusout={()=>previewTarget=null}>
          <div class="setting-grid single"><button type="button" class="setting-choice" class:active={appearanceDraft.floatingUseAlbumColor} aria-pressed={appearanceDraft.floatingUseAlbumColor} onclick={()=>{appearanceDraft.floatingUseAlbumColor=!appearanceDraft.floatingUseAlbumColor;previewTarget="background"}}><span class="choice-mark" aria-hidden="true">{appearanceDraft.floatingUseAlbumColor?"✓":""}</span><span class="choice-copy"><strong>{t("useAlbumColor")}</strong><small>{t("useAlbumColorHint")}</small></span></button></div>
          <label class="select-row color-row"><span>{t("floatingBackground")}</span><input aria-label={t("floatingBackground")} type="color" bind:value={appearanceDraft.floatingFillColor} oninput={()=>previewTarget="background"}/></label>
          <p class="hint">{t("floatingBackgroundHint")}</p>
        </div>
        <div class="appearance-apply-bar" class:dirty={appearanceDirty}>
          <div class="apply-state"><i aria-hidden="true"></i><span>{appearanceDirty?t("appearancePreviewing"):t("appearanceApplied")}</span></div>
          <div class="apply-actions"><button type="button" class="reset-button" disabled={!appearanceDirty} onclick={resetAppearanceDraft}>{t("resetAppearance")}</button><button type="button" class="apply-button" disabled={!appearanceDirty||applyingAppearance} onclick={applyAppearance}>{applyingAppearance?t("applyingAppearance"):t("applyAppearance")}</button></div>
        </div>
      </section>
      <section class="preferences-section">
        <div class="section-title"><h2>{t("preferences")}</h2><span class="section-note">{t("studioIntro")}</span></div>
        <div class="select-grid">
          <div class="compact-select-row"><span>{t("language")}</span><StudioSelect label={t("language")} value={settings.language} options={[
            {value:"system",label:t("systemLanguage")},{value:"zh-CN",label:t("chinese")},
            {value:"en",label:t("english")},{value:"ja",label:t("japanese")}
          ]} onSelect={(value)=>setLanguage(value as AppLanguage)}/></div>
          <div class="compact-select-row"><span>{t("clock")}</span><StudioSelect label={t("clock")} value={settings.clockTimeZone} options={[
            {value:"system",label:t("systemLanguage")},{value:"Asia/Taipei",label:"Asia/Taipei"},
            {value:"Asia/Tokyo",label:"Asia/Tokyo"},{value:"America/New_York",label:"America/New_York"},
            {value:"Europe/London",label:"Europe/London"},{value:"UTC",label:"UTC"}
          ]} onSelect={(value)=>updatePreference({clockTimeZone:value})}/></div>
        </div>
      </section>
      <section class="spectrum-section"><h2>{t("spectrum")}</h2><div class="preview-setting" role="group" data-preview-target="spectrum" onmouseenter={()=>previewTarget="spectrum"} onfocusin={()=>previewTarget="spectrum"} onmouseleave={()=>previewTarget=null} onfocusout={()=>previewTarget=null}><div class="setting-grid single"><button type="button" class="setting-choice" class:active={appearanceDraft.showSpectrum} aria-pressed={appearanceDraft.showSpectrum} onclick={()=>{appearanceDraft.showSpectrum=!appearanceDraft.showSpectrum;previewTarget="spectrum"}}><span class="choice-mark" aria-hidden="true">{appearanceDraft.showSpectrum?"✓":""}</span><span class="choice-copy"><strong>{t("showSpectrum")}</strong><small>{t("showSpectrumHint")}</small></span></button></div>{#if appearanceDraft.showSpectrum}<div class="field-label"><span>{t("animationSource")}</span><small>{t("animationSourceHint")}</small></div><div class="segmented two"><button aria-pressed={appearanceDraft.spectrumMode==="realtime"} class:active={appearanceDraft.spectrumMode==="realtime"} onclick={()=>{appearanceDraft.spectrumMode="realtime";previewTarget="spectrum"}}>{t("realtime")}</button><button aria-pressed={appearanceDraft.spectrumMode==="random"} class:active={appearanceDraft.spectrumMode==="random"} onclick={()=>{appearanceDraft.spectrumMode="random";previewTarget="spectrum"}}>{t("random")}</button></div>{/if}</div></section>
      <section class="feature-tools-section">
        <h2>{t("featureTools")}</h2>
        <p class="hint">{t("featureToolsHint")}</p>
        <button type="button" class="setting-choice panel-toggle-choice" class:active={settings.showCustomFunctionPanel} aria-pressed={settings.showCustomFunctionPanel} onclick={()=>updatePreference({showCustomFunctionPanel:!settings.showCustomFunctionPanel})}>
          <span class="choice-mark" aria-hidden="true">{settings.showCustomFunctionPanel?"✓":""}</span>
          <span class="choice-copy"><strong>{t("customFunctionPanel")}</strong><small>{t("customFunctionPanelHint")}</small></span>
        </button>
        <div class="setting-grid" class:tools-disabled={!settings.showCustomFunctionPanel}>
          <button type="button" class="setting-choice" class:active={settings.showSettingsTool} aria-pressed={settings.showSettingsTool} disabled={!settings.showCustomFunctionPanel} onclick={()=>updatePreference({showSettingsTool:!settings.showSettingsTool})}><span class="choice-mark" aria-hidden="true">{settings.showSettingsTool?"✓":""}</span><span class="choice-copy"><strong>{t("settingsTool")}</strong></span></button>
          <button type="button" class="setting-choice" class:active={settings.showFloatingTool} aria-pressed={settings.showFloatingTool} disabled={!settings.showCustomFunctionPanel} onclick={()=>updatePreference({showFloatingTool:!settings.showFloatingTool})}><span class="choice-mark" aria-hidden="true">{settings.showFloatingTool?"✓":""}</span><span class="choice-copy"><strong>{t("floatingTool")}</strong></span></button>
          <button type="button" class="setting-choice" class:active={settings.showVolumeTool} aria-pressed={settings.showVolumeTool} disabled={!settings.showCustomFunctionPanel} onclick={()=>updatePreference({showVolumeTool:!settings.showVolumeTool})}><span class="choice-mark" aria-hidden="true">{settings.showVolumeTool?"✓":""}</span><span class="choice-copy"><strong>{t("volumeTool")}</strong></span></button>
          <button type="button" class="setting-choice" class:active={settings.showTimerTool} aria-pressed={settings.showTimerTool} disabled={!settings.showCustomFunctionPanel} onclick={()=>updatePreference({showTimerTool:!settings.showTimerTool})}><span class="choice-mark" aria-hidden="true">{settings.showTimerTool?"✓":""}</span><span class="choice-copy"><strong>{t("timerTool")}</strong></span></button>
          <button type="button" class="setting-choice" class:active={settings.showHideTool} aria-pressed={settings.showHideTool} disabled={!settings.showCustomFunctionPanel} onclick={()=>updatePreference({showHideTool:!settings.showHideTool})}><span class="choice-mark" aria-hidden="true">{settings.showHideTool?"✓":""}</span><span class="choice-copy"><strong>{t("hideTool")}</strong></span></button>
        </div>
      </section>
      <section class="players-section"><div class="section-title"><h2>{t("players")}</h2><button class="icon-button" aria-label={t("refreshPlayers")} disabled={!nativeRuntime} onclick={refreshSessions}><RefreshCw size={15}/></button></div><p class="hint">{t("playersHint")}</p>
        {#if sessionsLoading}<div class="ordered-list skeleton-list" aria-hidden="true">{#each [0,1,2] as row}<div class="ordered-row skeleton-row"><span><i></i><i></i></span></div>{/each}</div>{:else}<div class="ordered-list">{#each playerRows.selected as player,index (player.id)}<div class="ordered-row" class:offline={!sessions.some(item=>item.id===player.id)}><button type="button" class="player-selection" role="checkbox" aria-checked="true" aria-label={t("select",{name:player.displayName})} onclick={()=>togglePlayer(player.id,false)}><span class="choice-mark" aria-hidden="true">✓</span></button><span><strong>{player.displayName}</strong><small>{player.isPlaying?t("playingNow"):sessions.some(item=>item.id===player.id)?t("detected"):t("offline")}</small></span><button aria-label={t("moveUp")} disabled={index===0} onclick={()=>movePlayer(player.id,-1)}><ChevronUp size={14}/></button><button aria-label={t("moveDown")} disabled={index===playerRows.selected.length-1} onclick={()=>movePlayer(player.id,1)}><ChevronDown size={14}/></button></div>{/each}{#each playerRows.unselected as player (player.id)}<div class="ordered-row" class:offline={!sessions.some(item=>item.id===player.id)}><button type="button" class="player-selection" role="checkbox" aria-checked="false" aria-label={t("select",{name:player.displayName})} onclick={()=>togglePlayer(player.id,true)}><span class="choice-mark" aria-hidden="true"></span></button><span><strong>{player.displayName}</strong><small>{player.isPlaying?t("playingNow"):sessions.some(item=>item.id===player.id)?t("detected"):t("offline")}</small></span></div>{/each}{#if !playerRows.selected.length&&!playerRows.unselected.length}<p class="empty">{t("noSessions")}</p>{/if}</div>{/if}
      </section>
      <section class="idle-section"><div class="section-title"><h2>{t("idleContent")}</h2><span class="section-note">{t("idleFixedLabel")}</span></div>
        <p class="hint idle-fixed-hint">{t("idleFixedHint")}</p>
        <div class="idle-layout-preview" aria-hidden="true"><span>{t("clock")}</span><span><CloudSun size={14}/>{t("weather")}</span></div>
        <div class="weather-box"><label for="weather-city">{t("weatherCity")}</label><div class="search-row"><input id="weather-city" placeholder={t("cityExample")} bind:value={weatherQuery} onkeydown={(e)=>{if(e.key==="Enter")void searchWeather()}}/><button aria-label={t("searchCity")} onclick={searchWeather}><Search size={15}/></button></div>{#if settings.weatherLocation}<p class="selected-city">{t("currentCity",{name:settings.weatherLocation.name})}</p>{/if}{#if weatherMessage}<p class="message">{weatherMessage}</p>{/if}{#if weatherResults.length}<div class="weather-results">{#each weatherResults as item}<button onclick={()=>selectWeather(item)}><strong>{item.name}</strong><small>{weatherCandidateDetail(item)}</small></button>{/each}</div>{/if}<p class="hint">{t("weatherSource")}</p></div>
      </section>
      <section class="media-features-section"><h2>{t("mediaFeatures")}</h2><div class="setting-grid">
        <button type="button" class="setting-choice" class:active={settings.enableHdCover} aria-pressed={settings.enableHdCover} onclick={()=>updatePreference({enableHdCover:!settings.enableHdCover})}><span class="choice-mark" aria-hidden="true">{settings.enableHdCover?"✓":""}</span><span class="choice-copy"><strong>{t("hdCover")}</strong><small>{t("hdCoverHint")}</small></span></button>
        <button type="button" class="setting-choice" class:active={settings.floatingCircularAlbum} aria-pressed={settings.floatingCircularAlbum} onclick={toggleCircularAlbum}><span class="choice-mark" aria-hidden="true">{settings.floatingCircularAlbum?"✓":""}</span><span class="choice-copy"><strong>{t("floatingCircularAlbum")}</strong><small>{t("floatingCircularAlbumHint")}</small></span></button>
        <button type="button" class="setting-choice" class:active={settings.enableMvPlayback&&!settings.floatingCircularAlbum} aria-pressed={settings.enableMvPlayback&&!settings.floatingCircularAlbum} disabled={settings.floatingCircularAlbum} title={settings.floatingCircularAlbum?t("mvUnavailableInCircular"):undefined} onclick={()=>updatePreference({enableMvPlayback:!settings.enableMvPlayback})}><span class="choice-mark" aria-hidden="true">{settings.enableMvPlayback&&!settings.floatingCircularAlbum?"✓":""}</span><span class="choice-copy"><strong>{t("mvPlayback")}</strong><small>{settings.floatingCircularAlbum?t("mvUnavailableInCircular"):t("mvPlaybackHint")}</small></span></button>
      </div></section>
      <section class="app-behavior-section"><h2>{t("appBehavior")}</h2>
        <div class="setting-grid"><button type="button" class="setting-choice" class:active={settings.alwaysOnTop} aria-pressed={settings.alwaysOnTop} disabled={!nativeRuntime} onclick={()=>setAlwaysOnTop(!settings.alwaysOnTop)}><span class="choice-mark" aria-hidden="true">{settings.alwaysOnTop?"✓":""}</span><span class="choice-copy"><strong>{t("alwaysOnTop")}</strong><small>{t("alwaysOnTopHint")}</small></span></button><button type="button" class="setting-choice" class:active={settings.autoStart} aria-pressed={settings.autoStart} disabled={!nativeRuntime} onclick={()=>setAutoStart(!settings.autoStart)}><span class="choice-mark" aria-hidden="true">{settings.autoStart?"✓":""}</span><span class="choice-copy"><strong>{t("autoStart")}</strong><small>{t("autoStartHint")}</small></span></button></div>
        <div class="select-row"><span><Monitor size={17}/>{t("monitor")}</span><StudioSelect label={t("monitor")} value={String(settings.monitorIndex)} disabled={!nativeRuntime} options={monitors.length?monitors.map((monitor)=>({value:String(monitor.index),label:monitor.name})):[{value:String(settings.monitorIndex),label:t("primaryMonitor") }]} onSelect={(value)=>updatePreference({monitorIndex:Number(value)})}/></div>
      </section>
      <section class="capture-section"><h2>{t("capture")}</h2><p class="hint">{t("captureHint")}</p><div class="setting-grid">
        <button type="button" class="setting-choice" class:active={settings.captureHideOnScreenshot} aria-pressed={settings.captureHideOnScreenshot} disabled={!nativeRuntime} onclick={()=>updatePreference({captureHideOnScreenshot:!settings.captureHideOnScreenshot})}><span class="choice-mark" aria-hidden="true">{settings.captureHideOnScreenshot?"✓":""}</span><span class="choice-copy"><strong>{t("hideScreenshot")}</strong><small>{t("hideScreenshotHint")}</small></span></button>
        <button type="button" class="setting-choice" class:active={settings.captureHideOnRecording} aria-pressed={settings.captureHideOnRecording} disabled={!nativeRuntime} onclick={()=>updatePreference({captureHideOnRecording:!settings.captureHideOnRecording})}><span class="choice-mark" aria-hidden="true">{settings.captureHideOnRecording?"✓":""}</span><span class="choice-copy"><strong>{t("hideRecording")}</strong><small>{t("hideRecordingHint")}</small></span></button>
        <button type="button" class="setting-choice" class:active={settings.captureHideOnFullscreen} aria-pressed={settings.captureHideOnFullscreen} disabled={!nativeRuntime} onclick={()=>updatePreference({captureHideOnFullscreen:!settings.captureHideOnFullscreen})}><span class="choice-mark" aria-hidden="true">{settings.captureHideOnFullscreen?"✓":""}</span><span class="choice-copy"><strong>{t("hideFullscreen")}</strong><small>{t("hideFullscreenHint")}</small></span></button>
        <button type="button" class="setting-choice" class:active={settings.captureHideOnScreenShare} aria-pressed={settings.captureHideOnScreenShare} disabled={!nativeRuntime} onclick={()=>updatePreference({captureHideOnScreenShare:!settings.captureHideOnScreenShare})}><span class="choice-mark" aria-hidden="true">{settings.captureHideOnScreenShare?"✓":""}</span><span class="choice-copy"><strong>{t("hideShare")}</strong><small>{t("hideShareHint")}</small></span></button>
      </div></section>
      <section class="tools-section"><h2>{t("tools")}</h2><div class="tool-list"><button disabled={!nativeRuntime} onclick={()=>windowApi.openFloatingWindow()}><ExternalLink size={17}/>{t("openFloating")}</button><button disabled={!nativeRuntime} onclick={()=>windowApi.resetFloatingWindow()}><RotateCcw size={17}/>{t("resetFloating")}</button><button disabled={!nativeRuntime} onclick={clearCache}><Trash2 size={17}/>{t("clearCache")}</button></div>{#if cacheMessage}<p class="message">{cacheMessage}</p>{/if}</section>
    </aside>
  </div>
</main>

<style>
  :global(html),:global(body),:global(#app){min-width:780px;min-height:600px;background:#fff;color:#111113}
  :global(body){overflow:auto}
  main{--studio-ink:#15161a;--studio-muted:#747680;--studio-line:#e3e4e8;--studio-panel:#f5f5f7;--studio-surface:#fff;--studio-accent:#3158c8;min-height:100vh;padding:28px 32px 54px;box-sizing:border-box;background:#f7f7f9}
  .studio-header{max-width:1240px;margin:0 auto 24px;display:flex;align-items:flex-end;justify-content:space-between;gap:24px}
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
  h2{margin:0 0 13px;color:var(--studio-ink);font-size:14px;letter-spacing:.01em;line-height:1.2}
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
  .select-row{display:flex;align-items:center;justify-content:space-between;padding:11px 0;border-top:1px solid #e5e5e8}.select-row span{display:flex;align-items:center;gap:8px;font-size:12px}.select-row :global(.studio-select){flex:0 1 160px;margin-left:auto}.color-row input[type=color]{width:44px;height:28px;padding:3px;border:1px solid #d7d7dc;border-radius:8px;background:#fff;cursor:pointer}
  .icon-button,.ordered-row button,.search-row button{display:grid;place-items:center;border:0;border-radius:8px;background:#fff;color:#414148;cursor:pointer;transition:background 140ms ease,color 140ms ease,transform 120ms ease}.icon-button{width:30px;height:30px}.icon-button:hover,.ordered-row button:hover,.search-row button:hover{background:#ececf0;color:#111113}
  .hint,.empty{margin:7px 0;color:#777780;font-size:11px;line-height:1.45}.ordered-list{display:flex;flex-direction:column;gap:6px;margin-top:11px}.ordered-row{display:flex;align-items:center;gap:7px;min-height:42px;padding:7px;border:1px solid transparent;border-radius:10px;background:#fff;transition:border-color 140ms ease,background 140ms ease}.ordered-row:hover{border-color:#dedfe4;background:#fcfcfd}.ordered-row.offline{opacity:.62}.ordered-row>span{min-width:0;display:flex;flex:1;flex-direction:column;gap:2px}.ordered-row strong{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px}.ordered-row small{color:#777780;font-size:11px}.ordered-row button{width:27px;height:27px}
  .skeleton-list{pointer-events:none}.skeleton-row{min-height:48px}.skeleton-row>span{gap:6px}.skeleton-row i{display:block;height:7px;border-radius:99px;background:linear-gradient(90deg,#ececf0 25%,#f7f7f8 50%,#ececf0 75%);background-size:200% 100%;animation:skeleton-shimmer 1.25s ease-in-out infinite}.skeleton-row i:first-child{width:42%}.skeleton-row i:last-child{width:24%;opacity:.75}@keyframes skeleton-shimmer{to{background-position:-200% 0}}
  .idle-fixed-hint{max-width:58ch;margin-top:0}.idle-layout-preview{display:flex;align-items:center;justify-content:space-between;margin-top:13px;padding:10px 12px;border:1px solid #e1e1e5;border-radius:10px;background:#fff;color:#4c4d55;font-size:12px}.idle-layout-preview>span{display:flex;align-items:center;gap:6px}.idle-layout-preview :global(svg){color:#3158c8}.weather-box{margin-top:14px;padding-top:13px;border-top:1px solid #e2e2e5}.weather-box>label{font-size:12px;font-weight:600}.search-row{display:grid;grid-template-columns:1fr 34px;gap:6px;margin-top:7px}.search-row input{min-width:0;border:1px solid #dddde1;border-radius:9px;padding:8px}.selected-city{margin:7px 0 0;font-size:11px;color:#39724a}.weather-results{display:flex;flex-direction:column;gap:4px;margin-top:7px}.weather-results button{display:flex;align-items:center;justify-content:space-between;border:0;border-radius:8px;padding:8px;background:#fff;text-align:left;cursor:pointer}.weather-results small{color:#777780}
  .tool-list button{display:flex;align-items:center;gap:8px;border:1px solid transparent;border-radius:11px;color:#414148;background:#fff;cursor:pointer;transition:color 140ms ease,background 140ms ease,border-color 140ms ease,transform 120ms ease}.tool-list button:hover{border-color:#d9dbe1;color:#111113;background:#fcfcfd}.tool-list{display:flex;flex-direction:column;gap:7px}.tool-list button{padding:10px 11px;font-size:12px}.tool-list button:active,.icon-button:active,.ordered-row button:active{transform:scale(.97)}
  button:disabled,input:disabled{cursor:not-allowed;opacity:.45}.message{margin:10px 0 0;color:#39724a;font-size:12px}.segmented button:focus-visible,.tool-list button:focus-visible,.icon-button:focus-visible,.ordered-row button:focus-visible,.weather-results button:focus-visible,input:focus-visible{outline:2px solid #111;outline-offset:2px}
  .setting-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:7px}.setting-grid.single{grid-template-columns:1fr}.setting-choice{min-width:0;min-height:66px;display:flex;align-items:flex-start;gap:8px;padding:11px;border:1px solid #e1e1e5;border-radius:11px;color:#36363d;background:#fff;text-align:left;cursor:pointer;transition:transform 120ms ease,background 140ms ease,border-color 140ms ease,color 140ms ease}.setting-choice:hover{border-color:#cfd1d8;background:#fcfcfd}.setting-choice.active{border-color:#111113;color:#fff;background:#111113}.setting-choice.active:hover{border-color:#111113;background:#202126}.setting-choice:active{transform:scale(.98)}.setting-choice:disabled{cursor:not-allowed}.panel-toggle-choice{width:100%;margin-bottom:8px}.tools-disabled{opacity:.45}.choice-mark{display:grid;place-items:center;flex:none;width:17px;height:17px;margin-top:1px;border:1px solid #bdbdc4;border-radius:5px;color:transparent;font-size:12px;font-weight:800;line-height:1}.setting-choice.active .choice-mark,.player-selection .choice-mark{border-color:#fff;color:#111113;background:#fff}.choice-copy{min-width:0;display:flex;flex-direction:column;gap:4px}.choice-copy strong{font-size:12px;line-height:1.2}.choice-copy small{color:#777780;font-size:11px;line-height:1.35}.setting-choice.active .choice-copy small{color:rgba(255,255,255,.68)}.player-selection{width:27px!important;height:27px!important;padding:0!important;border:0!important;background:transparent!important}.player-selection .choice-mark{width:17px;height:17px;margin:0}.setting-choice:focus-visible,.player-selection:focus-visible{outline:2px solid #111;outline-offset:2px}.player-selection:focus-visible{outline-color:#111}.setting-choice.active:focus-visible{outline-color:#fff}
  .appearance-apply-bar{position:sticky;bottom:12px;z-index:20;display:flex;align-items:center;justify-content:space-between;gap:10px;margin-top:18px;padding:10px;border:1px solid #dedee3;border-radius:12px;background:rgba(255,255,255,.96);box-shadow:0 10px 28px rgba(20,20,26,.09);backdrop-filter:blur(10px)}.apply-state{display:flex;align-items:center;gap:6px;min-width:0;color:#777780;font-size:11px}.apply-state i{width:6px;height:6px;flex:none;border-radius:50%;background:#b7b7be}.appearance-apply-bar.dirty .apply-state{color:var(--studio-accent)}.appearance-apply-bar.dirty .apply-state i{background:var(--studio-accent);box-shadow:0 0 0 4px rgba(49,88,200,.13)}.apply-actions{display:flex;gap:6px}.reset-button,.apply-button{height:30px;padding:0 12px;border:0;border-radius:8px;font:600 11px/1 var(--app-font);cursor:pointer;transition:transform 120ms ease,background 140ms ease}.reset-button{color:#55555d;background:#f0f0f2}.reset-button:hover:not(:disabled){background:#e7e7eb}.apply-button{color:#fff;background:#111113}.apply-button:hover:not(:disabled){background:#2a2b31}.reset-button:disabled,.apply-button:disabled{opacity:.38;cursor:default}.appearance-apply-bar button:focus-visible{outline:2px solid #111;outline-offset:2px}
  /* Studio rhythm: controls within a group stay close; groups and panels breathe. */
  main{--studio-space-2:8px;--studio-space-3:12px;--studio-space-4:16px;--studio-space-5:20px}
  .workspace aside{gap:var(--studio-space-4)}
  aside>section{padding:var(--studio-space-5)}
  .section-title{margin-bottom:var(--studio-space-3)}
  .section-title+.hint{margin-top:0}
  .preview-state{padding-bottom:var(--studio-space-4);margin-bottom:var(--studio-space-4)}
  .preview-state .field-label,.preview-setting .field-label{margin:0 0 var(--studio-space-2)}
  .appearance-section>.preview-setting{margin-top:var(--studio-space-4)}
  .appearance-section :global(.studio-slider){margin-top:var(--studio-space-4)}
  .appearance-apply-bar{margin-top:var(--studio-space-5)}
  .select-grid{gap:var(--studio-space-3)}
  .setting-grid{gap:var(--studio-space-2)}
  .setting-choice{min-height:70px;padding:var(--studio-space-3)}
  .compact-select-row{min-height:48px}
  .ordered-list{margin-top:var(--studio-space-3);gap:var(--studio-space-2)}

  /* Each control has a readable rest, hover, selected, focus, and disabled state. */
  .segmented button:hover:not(.active){background:rgba(255,255,255,.55)}
  .segmented button.active{box-shadow:0 1px 4px rgba(20,20,26,.11)}
  .setting-choice:hover:not(:disabled):not(.active){border-color:#b9bdc8;background:#fff}
  .tool-list button:hover:not(:disabled){border-color:#b9bdc8}
  .icon-button:hover:not(:disabled),.ordered-row button:hover:not(:disabled),.search-row button:hover:not(:disabled){background:#e9eaee}
  .compact-select-row:hover:not(:focus-within){background:#f0f1f4}
  .compact-select-row:focus-within{border-color:var(--studio-accent);background:#fff}
  .search-row input:focus-visible{outline:2px solid var(--studio-accent);outline-offset:2px}
  .segmented button:focus-visible,.setting-choice:focus-visible,.tool-list button:focus-visible,.icon-button:focus-visible,.ordered-row button:focus-visible,.weather-results button:focus-visible,.search-row button:focus-visible,.appearance-apply-bar button:focus-visible{outline:2px solid var(--studio-accent);outline-offset:2px}
  .setting-choice.active:focus-visible{outline-offset:3px}
  button:disabled,input:disabled{cursor:not-allowed}
  .setting-choice:disabled{opacity:1;color:#858892;background:#eceef1;border-color:#eceef1}
  .setting-choice:disabled .choice-copy small{color:#858892}
  .setting-choice.active:disabled{color:#555963;background:#e1e3e8;border-color:#e1e3e8}
  .setting-choice:disabled .choice-mark{border-color:#a8acb5}
  .icon-button:disabled,.ordered-row button:disabled,.search-row button:disabled{background:#eff0f2;color:#a1a4ad}
  .appearance-apply-bar button:disabled{cursor:not-allowed}

  /* Studio borrows the island's black shell and white active surfaces. */
  :global(html),:global(body),:global(#app){background:#0b0c0e;color:#f5f6f7}
  main{--studio-ink:#f5f6f7;--studio-muted:#afb2bc;--studio-line:#35373d;--studio-panel:#181a1e;--studio-surface:#22252a;--studio-accent:#84a8ff;--studio-control-ink:#f5f6f7;--studio-control-muted:#aeb1ba;--studio-range-accent:#f5f6f7;background:#0e0f12;color:var(--studio-ink)}
  .studio-wordmark{color:#a7aab4}
  .studio-status{background:#1b1d22;border-color:#373a43;color:var(--studio-muted)}
  .studio-status.browser i{background:#9da1ab;box-shadow:0 0 0 4px rgba(157,161,171,.14)}
  .stage{border-color:#383b43;box-shadow:0 20px 48px rgba(0,0,0,.23)}
  .stage-caption,.stage-caption strong{color:#5d6069}
  .stage-caption.editing,.stage-caption.editing strong{color:#3158c8}
  aside>section,.preferences-section{background:var(--studio-panel);border-color:var(--studio-line)}
  .section-note,.live-indicator,.field-label small,.hint,.empty,.ordered-row small,.choice-copy small{color:var(--studio-muted)}
  .preview-state,.select-row,.weather-box{border-color:#35373d}
  .segmented{background:#0f1013}
  .segmented button{color:#b8bbc4}
  .segmented button:hover:not(.active){color:#fff;background:#303238}
  .segmented button.active{color:#111216;background:#f5f6f7;box-shadow:0 2px 6px rgba(0,0,0,.23)}
  .setting-choice,.tool-list button,.weather-results button,.ordered-row,.idle-layout-preview{color:#f1f2f4;background:#22252a;border-color:#3a3d45}
  .setting-choice:hover:not(:disabled):not(.active),.tool-list button:hover:not(:disabled),.weather-results button:hover,.ordered-row:hover{color:#fff;background:#2b2e34;border-color:#666b76}
  .setting-choice.active{color:#111216;background:#f5f6f7;border-color:#f5f6f7}
  .setting-choice.active:hover:not(:disabled){background:#fff;border-color:#fff}
  .setting-choice.active .choice-copy small{color:#51545d}
  .choice-mark{border-color:#81858e}
  .setting-choice.active .choice-mark,.player-selection .choice-mark{color:#fff;background:#141519;border-color:#141519}
  .player-selection[aria-checked="false"] .choice-mark{color:transparent;background:transparent;border-color:#81858e}
  .setting-choice:disabled{color:#888c97;background:#202126;border-color:#303239}
  .setting-choice.active:disabled{color:#c5c7ce;background:#303239;border-color:#41434a}
  .setting-choice:disabled .choice-copy small{color:#858995}
  .setting-choice:disabled .choice-mark{border-color:#777b85}
  .compact-select-row{background:#22252a;border-color:#3a3d45}
  .compact-select-row:hover:not(:focus-within){background:#2b2e34}
  .compact-select-row:focus-within{background:#292c32;border-color:var(--studio-accent)}
  .search-row input,.color-row input[type=color]{color:#f2f3f5;background:#22252a;border:1px solid #484b53}
  .preferences-section{position:relative;z-index:2}
  .search-row input::placeholder{color:#969aa5}
  .icon-button,.ordered-row button,.search-row button{color:#edf0f4;background:#282b31;border:1px solid #42454e}
  .icon-button:hover:not(:disabled),.ordered-row button:hover:not(:disabled),.search-row button:hover:not(:disabled){color:#fff;background:#343740;border-color:#6b707a}
  .icon-button:disabled,.ordered-row button:disabled,.search-row button:disabled{color:#81858e;background:#202126;border-color:#303239}
  .idle-layout-preview :global(svg){color:#84a8ff}
  .selected-city,.message{color:#9fd2ad}
  .appearance-apply-bar{background:#1c1e23;border-color:#40434b;box-shadow:0 14px 30px rgba(0,0,0,.26)}
  .apply-state{color:#b0b3bc}
  .reset-button{color:#f0f1f3;background:#30333a}
  .reset-button:hover:not(:disabled){background:#41454e}
  .apply-button{color:#111216;background:#f5f6f7}
  .apply-button:hover:not(:disabled){background:#fff}
  .appearance-apply-bar button:disabled{opacity:.44}
  .skeleton-row i{background:linear-gradient(90deg,#2a2d33 25%,#393c43 50%,#2a2d33 75%);background-size:200% 100%}

  @media(max-width:1040px){main{padding-inline:24px}.workspace{grid-template-columns:minmax(340px,.8fr) minmax(400px,1.2fr);gap:18px}}
  @media(max-width:900px){:global(html),:global(body),:global(#app){min-width:0}main{padding:22px 18px 36px}.studio-header{align-items:flex-start;flex-direction:column;gap:14px;margin-bottom:18px}.studio-status{align-self:stretch}.workspace{grid-template-columns:1fr}.stage{position:relative;top:0;height:340px}.workspace aside{grid-template-columns:repeat(2,minmax(0,1fr))}.appearance-apply-bar{position:static}}
  @media(max-width:620px){main{padding:16px 12px 28px}.studio-brand h1{font-size:23px}.studio-brand p{font-size:12px}.stage{height:300px;border-radius:20px}.stage::before{inset:12px;border-radius:15px}.stage-topline{left:15px;right:15px;top:14px}.workspace aside{grid-template-columns:1fr;gap:12px}aside>section{padding:16px}aside>.appearance-section,aside>.preferences-section,aside>.players-section,aside>.idle-section{grid-column:auto}.select-grid{grid-template-columns:1fr}.appearance-apply-bar{align-items:stretch;flex-direction:column}.apply-actions{justify-content:flex-end}}
  @media(prefers-reduced-motion:reduce){.skeleton-row i{animation:none}.setting-choice,.segmented button,.tool-list button,.ordered-row,.icon-button,.ordered-row button,.search-row button,.apply-button,.reset-button{transition:none}}
  @media(prefers-reduced-transparency:reduce){.appearance-apply-bar{background:#fff;backdrop-filter:none}}
</style>

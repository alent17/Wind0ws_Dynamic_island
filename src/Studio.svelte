<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { spring } from "svelte/motion";
  import { Play,Pause,ImageOff,Type,RotateCcw,Trash2,ExternalLink,Monitor,ArrowUp,ArrowRight,ArrowDown,ArrowLeft,RefreshCw,ChevronUp,ChevronDown,Plus,X,Search } from "lucide-svelte";
  import IslandSurface from "$lib/IslandSurface.svelte";
  import { geometryFor, hostFor, type IslandEdge, type IslandMode, type IslandStyle } from "$lib/islandGeometry";
  import { DEMO_MEDIA, media, connectMedia } from "$lib/mediaStore";
  import { settingsApi } from "$lib/api/settings";
  import { windowApi } from "$lib/api/window";
  import { cacheApi } from "$lib/api/cache";
  import { mediaApi } from "$lib/api/media";
  import { idleApi } from "$lib/api/idle";
  import { applyAppFont, FONT_OPTIONS } from "$lib/font";
  import { locale, setLocale, translate, type TranslationKey } from "$lib/i18n";
  import { DEFAULT_SETTINGS, type AppLanguage, type AppPreferences, type IdleContentItem, type MediaSessionInfo, type MediaState, type MonitorInfo, type WeatherLocationCandidate } from "$lib/api/types";

  type Scenario="playing"|"paused"|"no-art"|"long-title";
  let mode=$state<IslandMode>("expanded"); let scenario=$state<Scenario>("playing"); let progress=$state(50); let settings=$state<AppPreferences>({...DEFAULT_SETTINGS}); let monitors=$state<MonitorInfo[]>([]); let cacheMessage=$state("");
  let nativeRuntime=$state(false); let followingLive=$state(true); let liveMedia=$state<MediaState>(DEMO_MEDIA); let stageWidth=$state(0); let stageHeight=$state(0);
  let sessions=$state<MediaSessionInfo[]>([]); let weatherQuery=$state(""); let weatherResults=$state<WeatherLocationCandidate[]>([]); let weatherMessage=$state("");
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let textCommitTimer: ReturnType<typeof setTimeout> | undefined;
  let persistInFlight: Promise<void> | undefined;
  let persistPending=false;
  let idlePreferenceRaf=0;
  let queuedIdlePreference:Partial<AppPreferences>={};
  let draftEdgePosition=$state(DEFAULT_SETTINGS.islandEdgePosition);
  let draftCompactLength=$state(DEFAULT_SETTINGS.compactLength);
  let draftShoulderRadius=$state(DEFAULT_SETTINGS.edgeShoulderRadius);
  let draftExpandedRadius=$state(DEFAULT_SETTINGS.expandedCornerRadius);
  let appBehaviorSection:HTMLElement|null=null;
  let autoStartLoaded=false;
  let studioDisposed=false;
  const previewBars=[.45,.78,.58,.96,.7,.38];
  let scenarioMedia=$derived.by<MediaState>(()=>{const base={...DEMO_MEDIA,positionMs:244000*progress/100,lastUpdatedTimestamp:Date.now()};if(scenario==="paused")return{...base,isPlaying:false};if(scenario==="no-art")return{...base,albumArt:""};if(scenario==="long-title")return{...base,title:"宇宙尽头的浪漫主义与一场不会结束的午夜公路旅行",artist:"The Extremely Long Artist Name · 特别长的专辑名称"};return base});
  let sample=$derived(nativeRuntime&&followingLive?liveMedia:scenarioMedia);
  let currentGeometry=$derived(geometryFor(mode,draftExpandedRadius,settings.islandEdge,draftCompactLength));
  let previewHost=$derived(hostFor(settings.islandStyle,settings.islandEdge,draftCompactLength));
  const previewPosition=spring(50,{stiffness:.1,damping:.7,precision:.1});
  $effect(()=>{previewPosition.set(draftEdgePosition,{hard:!settings.enableAnimations||settings.reduceAnimations})});
  let previewPositionStyle=$derived.by(()=>{const p=$previewPosition/100;const horizontal=settings.islandEdge==="top"||settings.islandEdge==="bottom";const x=horizontal?Math.max(0,stageWidth-previewHost.width)*p:settings.islandEdge==="right"?Math.max(0,stageWidth-previewHost.width):0;const y=horizontal?(settings.islandEdge==="bottom"?Math.max(0,stageHeight-previewHost.height):0):Math.max(0,stageHeight-previewHost.height)*p;return `transform:translate3d(${x}px,${y}px,0)`});
  function selectScenario(next:Scenario){scenario=next;followingLive=false}
  const t=(key:TranslationKey,values:Record<string,string|number>={})=>translate(key,values,$locale);
  const enumLabel=(value:string)=>t(value as TranslationKey);
  function syncAppearanceDrafts(){
    draftEdgePosition=settings.islandEdgePosition;
    draftCompactLength=settings.compactLength;
    draftShoulderRadius=settings.edgeShoulderRadius;
    draftExpandedRadius=settings.expandedCornerRadius;
  }
  function loadAutoStart(){
    if(studioDisposed||!nativeRuntime||autoStartLoaded)return;
    autoStartLoaded=true;
    void settingsApi.getAutoStart().then(autoStart=>{if(!studioDisposed)settings={...settings,autoStart}}).catch(()=>{if(!studioDisposed)autoStartLoaded=false});
  }
  onMount(()=>{
    studioDisposed=false;
    nativeRuntime=Boolean((window as any).__TAURI_INTERNALS__);
    const unsubscribe=media.subscribe(value=>liveMedia=value);
    let disconnect:undefined|(()=>void);
    let disposed=false;
    let deferredFrame=0;
    let deferredTimers:ReturnType<typeof setTimeout>[]=[];
    let autoStartObserver:IntersectionObserver|undefined;
    let closePromise:Promise<()=>void>|undefined;
    if(nativeRuntime){
      const appWindow=getCurrentWindow();
      closePromise=appWindow.onCloseRequested(async(event)=>{
        event.preventDefault();
        await appWindow.hide();
      });
      void (async()=>{
        try{
          settings={...DEFAULT_SETTINGS,...await settingsApi.getPreferences()};
          syncAppearanceDrafts();
          applyAppFont(settings.fontId);
          setLocale(settings.language);
        }catch{}
        if(disposed)return;
        if(appBehaviorSection){
          autoStartObserver=new IntersectionObserver(entries=>{
            if(entries.some(entry=>entry.isIntersecting)){
              loadAutoStart();
              autoStartObserver?.disconnect();
              autoStartObserver=undefined;
            }
          },{threshold:0});
          autoStartObserver.observe(appBehaviorSection);
        }
        deferredFrame=requestAnimationFrame(()=>{
          deferredTimers=[
            setTimeout(()=>{
              if(disposed)return;
              void connectMedia().then(stop=>{if(disposed)stop();else disconnect=stop}).catch(()=>{});
            },100),
            setTimeout(()=>{
              if(disposed)return;
              void windowApi.getMonitors().then(value=>{if(!disposed)monitors=value}).catch(()=>{});
              void refreshSessions();
            },250),
          ];
        });
      })();
    }
    return()=>{
      disposed=true;
      studioDisposed=true;
      if(deferredFrame)cancelAnimationFrame(deferredFrame);
      deferredTimers.forEach(timer=>clearTimeout(timer));
      autoStartObserver?.disconnect();
      if(closePromise)void closePromise.then(unlisten=>unlisten()).catch(()=>{});
      if(textCommitTimer){clearTimeout(textCommitTimer);textCommitTimer=undefined;commitPreference({idleItems:settings.idleItems})}
      if(idlePreferenceRaf){cancelAnimationFrame(idlePreferenceRaf);idlePreferenceRaf=0}
      queuedIdlePreference={};
      if(saveTimer)clearTimeout(saveTimer);
      void persist();
      unsubscribe();
      disconnect?.();
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
        const snapshot={...settings};
        await settingsApi.savePreferences(snapshot).catch(()=>{});
      }
    })().finally(()=>{persistInFlight=undefined});
    await persistInFlight;
  }
  function schedulePersist(){if(saveTimer)clearTimeout(saveTimer);saveTimer=setTimeout(()=>void persist(),300)}
  function flushQueuedIdlePreference(){
    const patch=queuedIdlePreference;
    queuedIdlePreference={};
    if(!Object.keys(patch).length)return;
    settings={...settings,...patch};
  }
  function queueIdlePreferenceUpdate(patch:Partial<AppPreferences>){
    queuedIdlePreference={...queuedIdlePreference,...patch};
    if(idlePreferenceRaf)return;
    idlePreferenceRaf=requestAnimationFrame(()=>{idlePreferenceRaf=0;flushQueuedIdlePreference()});
  }
  function commitPreference(patch:Partial<AppPreferences>){
    if(idlePreferenceRaf){cancelAnimationFrame(idlePreferenceRaf);idlePreferenceRaf=0}
    flushQueuedIdlePreference();
    settings={...settings,...patch};
    if(!nativeRuntime)return;
    schedulePersist();
  }
  function updatePreference(patch:Partial<AppPreferences>){commitPreference(patch)}
  async function setAlwaysOnTop(value:boolean){settings={...settings,alwaysOnTop:value};if(!nativeRuntime)return;await settingsApi.setAlwaysOnTop(value).catch(()=>{})}
  async function setAutoStart(value:boolean){settings={...settings,autoStart:value};if(nativeRuntime)await settingsApi.setAutoStart(value).catch(()=>{})}
  function setIslandStyle(value:IslandStyle){updatePreference({islandStyle:value})}
  function setIslandEdge(value:IslandEdge){updatePreference({islandEdge:value})}
  function commitEdgePosition(){commitPreference({islandEdgePosition:draftEdgePosition})}
  function commitCompactLength(){commitPreference({compactLength:draftCompactLength})}
  function commitShoulderRadius(){commitPreference({edgeShoulderRadius:draftShoulderRadius})}
  function commitRadius(){commitPreference({expandedCornerRadius:draftExpandedRadius})}
  function setFont(value:AppPreferences["fontId"]){applyAppFont(value);updatePreference({fontId:value})}
  function setLanguage(value:AppLanguage){setLocale(value);updatePreference({language:value})}
  function mergePlayerOrderIds(existing:string[],available:MediaSessionInfo[],selected:string[]|null){
    return [...new Set([...existing,...available.map(item=>item.id),...(selected??[])].filter(id=>id.trim().length>0))];
  }
  function currentPlayerOrderIds(){return mergePlayerOrderIds(settings.playerOrderIds,sessions,settings.selectedPlayerIds)}
  function sessionFor(id:string):MediaSessionInfo{return sessions.find(item=>item.id===id)??{id,displayName:id,source:"generic",isPlaying:false}}
  async function refreshSessions(){
    if(!nativeRuntime)return;
    try{
      const nextSessions=await mediaApi.listMediaSessions();
      sessions=nextSessions;
      const nextOrder=mergePlayerOrderIds(settings.playerOrderIds,nextSessions,settings.selectedPlayerIds);
      if(nextOrder.join("\u0000")!==settings.playerOrderIds.join("\u0000"))updatePreference({playerOrderIds:nextOrder});
    }catch{sessions=[]}
  }
  function togglePlayer(id:string,checked:boolean){
    const order=currentPlayerOrderIds();
    const selected=settings.selectedPlayerIds===null?[...order]:[...settings.selectedPlayerIds];
    const nextSelected=checked?[...selected.filter(item=>item!==id),id]:selected.filter(item=>item!==id);
    const nextOrder=[...nextSelected,...order.filter(item=>!nextSelected.includes(item))];
    updatePreference({selectedPlayerIds:nextSelected,playerOrderIds:nextOrder});
  }
  function movePlayer(id:string,direction:-1|1){
    const order=currentPlayerOrderIds();
    const selected=settings.selectedPlayerIds===null?[...order]:settings.selectedPlayerIds.filter(item=>order.includes(item));
    const from=selected.indexOf(id);const to=from+direction;
    if(from<0||to<0||to>=selected.length)return;
    [selected[from],selected[to]]=[selected[to],selected[from]];
    updatePreference({playerOrderIds:[...selected,...order.filter(item=>!selected.includes(item))]});
  }
  let playerRows=$derived.by(()=>{
    const order=currentPlayerOrderIds();
    const selected=new Set(settings.selectedPlayerIds===null?order:settings.selectedPlayerIds);
    return {selected:order.filter(id=>selected.has(id)).map(sessionFor),unselected:order.filter(id=>!selected.has(id)).map(sessionFor)};
  });
  const idleLabel=(kind:string)=>t((({clock:"clock",date:"date",weather:"weather",network:"network",cpu:"cpu",memory:"memory",battery:"battery",custom:"customText"} as const)[kind]??"customText") as TranslationKey);
  function patchIdle(id:string,patch:Partial<IdleContentItem>){updatePreference({idleItems:settings.idleItems.map(item=>item.id===id?{...item,...patch}:item)})}
  function updateIdleText(id:string,text:string){
    queueIdlePreferenceUpdate({idleItems:settings.idleItems.map(item=>item.id===id?{...item,text}:item)});
    if(textCommitTimer)clearTimeout(textCommitTimer);
    textCommitTimer=setTimeout(()=>{textCommitTimer=undefined;commitPreference({idleItems:settings.idleItems})},300);
  }
  function commitIdleText(){if(!textCommitTimer)return;clearTimeout(textCommitTimer);textCommitTimer=undefined;commitPreference({idleItems:settings.idleItems})}
  function moveIdle(id:string,direction:-1|1){const items=[...settings.idleItems];const from=items.findIndex(item=>item.id===id);const to=from+direction;if(from<0||to<0||to>=items.length)return;[items[from],items[to]]=[items[to],items[from]];updatePreference({idleItems:items})}
  function addCustom(){const id=`custom-${Date.now()}`;updatePreference({idleItems:[...settings.idleItems,{id,kind:"custom",enabled:true,text:"(｡･ω･｡)ﾉ♡"}]})}
  function removeIdle(id:string){updatePreference({idleItems:settings.idleItems.filter(item=>item.id!==id)})}
  async function searchWeather(){weatherMessage=t("searching");try{weatherResults=await idleApi.searchLocations(weatherQuery,$locale==="zh-CN"?"zh":$locale);weatherMessage=weatherResults.length?"":t("noCity")}catch{weatherMessage=t("weatherFailed")}}
  function weatherLocationLabel(item:WeatherLocationCandidate){return [...new Set([item.name,item.admin2,item.admin1,item.country].filter(Boolean))].join(" · ")}
  function weatherCandidateDetail(item:WeatherLocationCandidate){const label=[...new Set([item.admin2,item.admin1,item.country].filter(Boolean))].join(" · ");const duplicates=weatherResults.filter(other=>weatherLocationLabel(other)===weatherLocationLabel(item)).length;return duplicates>1?`${label} · ${item.latitude.toFixed(2)}, ${item.longitude.toFixed(2)}`:label}
  function selectWeather(item:WeatherLocationCandidate){updatePreference({weatherLocation:{name:weatherLocationLabel(item),latitude:item.latitude,longitude:item.longitude}});weatherResults=[];weatherMessage=t("citySaved")}
  async function clearCache(){cacheMessage=t("clearing");try{await cacheApi.clearCache();cacheMessage=t("cacheCleared")}catch{cacheMessage=t("clearFailed")}}
</script>

<svelte:head><title>Isle Studio</title></svelte:head>
<main>
  <header><div><h1>Isle Studio</h1><p>{t("studioIntro")}</p></div><span class="status">{nativeRuntime?(followingLive?t("currentMusic"):t("simulated")):t("browserDemo")}</span></header>
  <div class="workspace">
    <section class="stage" aria-label={t("previewLabel")} bind:clientWidth={stageWidth} bind:clientHeight={stageHeight}>
      <div class="ruler top"><span>0%</span><span>50%</span><span>100%</span></div>
      <div class="preview-host" style={`width:${previewHost.width}px;height:${previewHost.height}px;${previewPositionStyle}`}>
        <IslandSurface media={sample} {mode} islandStyle={settings.islandStyle} edge={settings.islandEdge} position={sample.positionMs} expandedRadius={draftExpandedRadius} edgeShoulderRadius={draftShoulderRadius} compactLength={draftCompactLength} showSpectrum={settings.showSpectrum} spectrumMode={settings.spectrumMode} enableAnimations={settings.enableAnimations} reduceAnimations={settings.reduceAnimations} previewSpectrum={settings.spectrumMode==="realtime"?previewBars:undefined} interactive simulateHidden onToggle={()=>mode=mode==="expanded"?"compact":"expanded"} onMediaAction={(action)=>{followingLive=false;if(action==="play_pause")scenario=scenario==="paused"?"playing":"paused"}} />
      </div>
      <div class="caption"><strong>{enumLabel(mode)}</strong><span>{enumLabel(settings.islandStyle)} · {enumLabel(settings.islandEdge)} {draftEdgePosition}% · {currentGeometry.width} × {currentGeometry.height} · r{currentGeometry.radius} · {t("shoulderShort")} {draftShoulderRadius}px</span></div>
    </section>
    <aside>
      <section><h2>{t("previewState")}</h2><div class="segmented">{#each ["compact","hover","expanded","hidden"] as item}<button class:active={mode===item} onclick={()=>mode=item as IslandMode}>{enumLabel(item)}</button>{/each}</div></section>
      <section><h2>{t("islandLayout")}</h2>
        <div class="field-label"><span>{t("shape")}</span><small>{t("shapeHint")}</small></div>
        <div class="segmented two"><button aria-pressed={settings.islandStyle==="floating"} class:active={settings.islandStyle==="floating"} onclick={()=>setIslandStyle("floating")}>{t("floating")}</button><button aria-pressed={settings.islandStyle==="edge"} class:active={settings.islandStyle==="edge"} onclick={()=>setIslandStyle("edge")}>{t("attached")}</button></div>
        <div class="field-label"><span>{t("screenEdge")}</span><small>{t("screenEdgeHint")}</small></div>
        <div class="edge-grid">
          <button aria-label={t("screenTop")} aria-pressed={settings.islandEdge==="top"} class:active={settings.islandEdge==="top"} onclick={()=>setIslandEdge("top")}><ArrowUp size={16}/>{t("top")}</button>
          <button aria-label={t("screenRight")} aria-pressed={settings.islandEdge==="right"} class:active={settings.islandEdge==="right"} onclick={()=>setIslandEdge("right")}><ArrowRight size={16}/>{t("right")}</button>
          <button aria-label={t("screenBottom")} aria-pressed={settings.islandEdge==="bottom"} class:active={settings.islandEdge==="bottom"} onclick={()=>setIslandEdge("bottom")}><ArrowDown size={16}/>{t("bottom")}</button>
          <button aria-label={t("screenLeft")} aria-pressed={settings.islandEdge==="left"} class:active={settings.islandEdge==="left"} onclick={()=>setIslandEdge("left")}><ArrowLeft size={16}/>{t("left")}</button>
        </div>
        <label class="range-label"><span>{t("edgePosition")}</span><output>{draftEdgePosition}%</output></label>
        <input aria-label={t("edgePosition")} type="range" min="0" max="100" step="1" bind:value={draftEdgePosition} onchange={commitEdgePosition}/>
        <button class="center-button" onclick={()=>{draftEdgePosition=50;commitPreference({islandEdgePosition:50})}}>{t("center")}</button>
        <label class="range-label"><span>{t("compactLength")}</span><output>{draftCompactLength}px</output></label>
        <input aria-label={t("compactLength")} type="range" min="80" max="300" step="1" bind:value={draftCompactLength} onchange={commitCompactLength}/>
        <label class="range-label"><span>{t("shoulder")}</span><output>{draftShoulderRadius}px</output></label>
        <input aria-label={t("shoulder")} type="range" min="0" max="16" step="1" bind:value={draftShoulderRadius} onchange={commitShoulderRadius}/>
         <label class="range-label"><span>{t("expandedRadius")}</span><output>{draftExpandedRadius}px</output></label>
         <input aria-label={t("expandedRadius")} type="range" min="0" max="80" step="1" bind:value={draftExpandedRadius} onchange={commitRadius}/>
         <div class="setting-grid single"><button type="button" class="setting-choice" class:active={settings.floatingUseAlbumColor} aria-pressed={settings.floatingUseAlbumColor} onclick={()=>updatePreference({floatingUseAlbumColor:!settings.floatingUseAlbumColor})}><span class="choice-mark" aria-hidden="true">{settings.floatingUseAlbumColor?"✓":""}</span><span class="choice-copy"><strong>{t("useAlbumColor")}</strong><small>{t("useAlbumColorHint")}</small></span></button></div>
         <label class="select-row color-row"><span>{t("floatingBackground")}</span><input aria-label={t("floatingBackground")} type="color" value={settings.floatingFillColor} onchange={(e)=>updatePreference({floatingFillColor:e.currentTarget.value})}/></label>
         <p class="hint">{t("floatingBackgroundHint")}</p>
       </section>
      <section><h2>{t("language")}</h2><label class="select-row"><span>{t("language")}</span><select value={settings.language} onchange={(e)=>setLanguage(e.currentTarget.value as AppLanguage)}><option value="system">{t("systemLanguage")}</option><option value="zh-CN">{t("chinese")}</option><option value="en">{t("english")}</option><option value="ja">{t("japanese")}</option></select></label></section>
      <section><h2>{t("clock")}</h2><label class="select-row"><span>{t("clock")}</span><select value={settings.clockTimeZone} onchange={(e)=>updatePreference({clockTimeZone:e.currentTarget.value})}><option value="system">{t("systemLanguage")}</option><option value="Asia/Taipei">Asia/Taipei</option><option value="Asia/Tokyo">Asia/Tokyo</option><option value="America/New_York">America/New_York</option><option value="Europe/London">Europe/London</option><option value="UTC">UTC</option></select></label></section>
      <section><h2>{t("font")}</h2><label class="select-row"><span><Type size={17}/>{t("appFont")}</span><select value={settings.fontId} onchange={(e)=>setFont(e.currentTarget.value as AppPreferences["fontId"])}>{#each FONT_OPTIONS as font}<option value={font.id}>{font.id==="system"?t("systemDefault"):font.label}</option>{/each}</select></label><p class="hint">{t("fontHint")}</p></section>
      <section><h2>{t("mediaScene")}</h2><div class="scenario-grid">
        <button class:active={!followingLive&&scenario==="playing"} onclick={()=>selectScenario("playing")}><Play size={17}/>{t("play")}</button><button class:active={!followingLive&&scenario==="paused"} onclick={()=>selectScenario("paused")}><Pause size={17}/>{t("pause")}</button><button class:active={!followingLive&&scenario==="no-art"} onclick={()=>selectScenario("no-art")}><ImageOff size={17}/>{t("noCover")}</button><button class:active={!followingLive&&scenario==="long-title"} onclick={()=>selectScenario("long-title")}><Type size={17}/>{t("longTitle")}</button>
      </div>{#if nativeRuntime}<button class="follow-live" class:active={followingLive} onclick={()=>followingLive=true}>{t("followMusic")}</button>{/if}<label class="progress-label"><span>{t("playbackProgress",{percent:progress})}</span><output>{progress}%</output></label><input type="range" min="0" max="100" value={progress} oninput={(e)=>{progress=Number(e.currentTarget.value);followingLive=false}}/><div class="endpoints"><button onclick={()=>{progress=0;followingLive=false}}>0%</button><button onclick={()=>{progress=50;followingLive=false}}>50%</button><button onclick={()=>{progress=100;followingLive=false}}>100%</button></div></section>
      <section><h2>{t("spectrum")}</h2><div class="setting-grid single"><button type="button" class="setting-choice" class:active={settings.showSpectrum} aria-pressed={settings.showSpectrum} onclick={()=>updatePreference({showSpectrum:!settings.showSpectrum})}><span class="choice-mark" aria-hidden="true">{settings.showSpectrum?"✓":""}</span><span class="choice-copy"><strong>{t("showSpectrum")}</strong><small>{t("showSpectrumHint")}</small></span></button></div>{#if settings.showSpectrum}<div class="field-label"><span>{t("animationSource")}</span><small>{t("animationSourceHint")}</small></div><div class="segmented two"><button aria-pressed={settings.spectrumMode==="realtime"} class:active={settings.spectrumMode==="realtime"} onclick={()=>updatePreference({spectrumMode:"realtime"})}>{t("realtime")}</button><button aria-pressed={settings.spectrumMode==="random"} class:active={settings.spectrumMode==="random"} onclick={()=>updatePreference({spectrumMode:"random"})}>{t("random")}</button></div>{/if}</section>
      <section><div class="section-title"><h2>{t("players")}</h2><button class="icon-button" aria-label={t("refreshPlayers")} disabled={!nativeRuntime} onclick={refreshSessions}><RefreshCw size={15}/></button></div><p class="hint">{t("playersHint")}</p>
        <div class="ordered-list">{#each playerRows.selected as player,index (player.id)}<div class="ordered-row" class:offline={!sessions.some(item=>item.id===player.id)}><button type="button" class="player-selection" role="checkbox" aria-checked="true" aria-label={t("select",{name:player.displayName})} onclick={()=>togglePlayer(player.id,false)}><span class="choice-mark" aria-hidden="true">✓</span></button><span><strong>{player.displayName}</strong><small>{player.isPlaying?t("playingNow"):sessions.some(item=>item.id===player.id)?t("detected"):t("offline")}</small></span><button aria-label={t("moveUp")} disabled={index===0} onclick={()=>movePlayer(player.id,-1)}><ChevronUp size={14}/></button><button aria-label={t("moveDown")} disabled={index===playerRows.selected.length-1} onclick={()=>movePlayer(player.id,1)}><ChevronDown size={14}/></button></div>{/each}{#each playerRows.unselected as player (player.id)}<div class="ordered-row" class:offline={!sessions.some(item=>item.id===player.id)}><button type="button" class="player-selection" role="checkbox" aria-checked="false" aria-label={t("select",{name:player.displayName})} onclick={()=>togglePlayer(player.id,true)}><span class="choice-mark" aria-hidden="true"></span></button><span><strong>{player.displayName}</strong><small>{player.isPlaying?t("playingNow"):sessions.some(item=>item.id===player.id)?t("detected"):t("offline")}</small></span></div>{/each}{#if !playerRows.selected.length&&!playerRows.unselected.length}<p class="empty">{t("noSessions")}</p>{/if}</div>
      </section>
      <section><h2>{t("idleContent")}</h2>
        <div class="setting-grid single"><button type="button" class="setting-choice" class:active={settings.idleContentEnabled} aria-pressed={settings.idleContentEnabled} onclick={()=>updatePreference({idleContentEnabled:!settings.idleContentEnabled})}><span class="choice-mark" aria-hidden="true">{settings.idleContentEnabled?"✓":""}</span><span class="choice-copy"><strong>{t("enableIdle")}</strong><small>{t("enableIdleHint")}</small></span></button></div>
        {#if settings.idleContentEnabled}
          <label class="range-label"><span>{t("interval")}</span><output>{t("seconds",{value:settings.idleRotationSeconds})}</output></label><input aria-label={t("idleInterval")} type="range" min="2" max="60" value={settings.idleRotationSeconds} oninput={(e)=>queueIdlePreferenceUpdate({idleRotationSeconds:Number(e.currentTarget.value)})} onchange={(e)=>commitPreference({idleRotationSeconds:Number(e.currentTarget.value)})}/>
          <div class="ordered-list idle-list">{#each settings.idleItems as item,index (item.id)}<div class="ordered-row idle-row"><button type="button" class="mini-choice" class:active={item.enabled} role="checkbox" aria-checked={item.enabled} aria-label={t("enableItem",{name:idleLabel(item.kind)})} onclick={()=>patchIdle(item.id,{enabled:!item.enabled})}><span class="choice-mark" aria-hidden="true">{item.enabled?"✓":""}</span></button><span><strong>{idleLabel(item.kind)}</strong>{#if item.kind==="custom"}<input aria-label={t("customText")} maxlength="120" value={item.text} oninput={(e)=>updateIdleText(item.id,e.currentTarget.value)} onchange={commitIdleText} onblur={commitIdleText}/>{/if}</span><button aria-label={t("moveUp")} disabled={index===0} onclick={()=>moveIdle(item.id,-1)}><ChevronUp size={14}/></button><button aria-label={t("moveDown")} disabled={index===settings.idleItems.length-1} onclick={()=>moveIdle(item.id,1)}><ChevronDown size={14}/></button>{#if item.kind==="custom"}<button aria-label={t("deleteCustom")} onclick={()=>removeIdle(item.id)}><X size={14}/></button>{/if}</div>{/each}</div>
          <button class="center-button add-button" onclick={addCustom}><Plus size={15}/>{t("addCustom")}</button>
          <div class="weather-box"><label for="weather-city">{t("weatherCity")}</label><div class="search-row"><input id="weather-city" placeholder={t("cityExample")} bind:value={weatherQuery} onkeydown={(e)=>{if(e.key==="Enter")void searchWeather()}}/><button aria-label={t("searchCity")} onclick={searchWeather}><Search size={15}/></button></div>{#if settings.weatherLocation}<p class="selected-city">{t("currentCity",{name:settings.weatherLocation.name})}</p>{/if}{#if weatherMessage}<p class="message">{weatherMessage}</p>{/if}{#if weatherResults.length}<div class="weather-results">{#each weatherResults as item}<button onclick={()=>selectWeather(item)}><strong>{item.name}</strong><small>{weatherCandidateDetail(item)}</small></button>{/each}</div>{/if}<p class="hint">{t("weatherSource")}</p></div>
        {/if}
      </section>
      <section><h2>{t("mediaFeatures")}</h2><div class="setting-grid">
        <button type="button" class="setting-choice" class:active={settings.enableHdCover} aria-pressed={settings.enableHdCover} onclick={()=>updatePreference({enableHdCover:!settings.enableHdCover})}><span class="choice-mark" aria-hidden="true">{settings.enableHdCover?"✓":""}</span><span class="choice-copy"><strong>{t("hdCover")}</strong><small>{t("hdCoverHint")}</small></span></button>
        <button type="button" class="setting-choice" class:active={settings.enableMvPlayback} aria-pressed={settings.enableMvPlayback} onclick={()=>updatePreference({enableMvPlayback:!settings.enableMvPlayback})}><span class="choice-mark" aria-hidden="true">{settings.enableMvPlayback?"✓":""}</span><span class="choice-copy"><strong>{t("mvPlayback")}</strong><small>{t("mvPlaybackHint")}</small></span></button>
      </div></section>
      <section bind:this={appBehaviorSection}><h2>{t("appBehavior")}</h2>
        <div class="setting-grid"><button type="button" class="setting-choice" class:active={settings.alwaysOnTop} aria-pressed={settings.alwaysOnTop} disabled={!nativeRuntime} onclick={()=>setAlwaysOnTop(!settings.alwaysOnTop)}><span class="choice-mark" aria-hidden="true">{settings.alwaysOnTop?"✓":""}</span><span class="choice-copy"><strong>{t("alwaysOnTop")}</strong><small>{t("alwaysOnTopHint")}</small></span></button><button type="button" class="setting-choice" class:active={settings.autoStart} aria-pressed={settings.autoStart} disabled={!nativeRuntime} onclick={()=>setAutoStart(!settings.autoStart)}><span class="choice-mark" aria-hidden="true">{settings.autoStart?"✓":""}</span><span class="choice-copy"><strong>{t("autoStart")}</strong><small>{t("autoStartHint")}</small></span></button></div>
        <label class="select-row"><span><Monitor size={17}/>{t("monitor")}</span><select value={settings.monitorIndex} disabled={!nativeRuntime} onchange={(e)=>updatePreference({monitorIndex:Number(e.currentTarget.value)})}>{#each monitors as monitor}<option value={monitor.index}>{monitor.name}</option>{/each}{#if !monitors.length}<option>{t("primaryMonitor")}</option>{/if}</select></label>
      </section>
      <section><h2>{t("capture")}</h2><p class="hint">{t("captureHint")}</p><div class="setting-grid">
        <button type="button" class="setting-choice" class:active={settings.captureHideOnScreenshot} aria-pressed={settings.captureHideOnScreenshot} disabled={!nativeRuntime} onclick={()=>updatePreference({captureHideOnScreenshot:!settings.captureHideOnScreenshot})}><span class="choice-mark" aria-hidden="true">{settings.captureHideOnScreenshot?"✓":""}</span><span class="choice-copy"><strong>{t("hideScreenshot")}</strong><small>{t("hideScreenshotHint")}</small></span></button>
        <button type="button" class="setting-choice" class:active={settings.captureHideOnRecording} aria-pressed={settings.captureHideOnRecording} disabled={!nativeRuntime} onclick={()=>updatePreference({captureHideOnRecording:!settings.captureHideOnRecording})}><span class="choice-mark" aria-hidden="true">{settings.captureHideOnRecording?"✓":""}</span><span class="choice-copy"><strong>{t("hideRecording")}</strong><small>{t("hideRecordingHint")}</small></span></button>
        <button type="button" class="setting-choice" class:active={settings.captureHideOnFullscreen} aria-pressed={settings.captureHideOnFullscreen} disabled={!nativeRuntime} onclick={()=>updatePreference({captureHideOnFullscreen:!settings.captureHideOnFullscreen})}><span class="choice-mark" aria-hidden="true">{settings.captureHideOnFullscreen?"✓":""}</span><span class="choice-copy"><strong>{t("hideFullscreen")}</strong><small>{t("hideFullscreenHint")}</small></span></button>
        <button type="button" class="setting-choice" class:active={settings.captureHideOnScreenShare} aria-pressed={settings.captureHideOnScreenShare} disabled={!nativeRuntime} onclick={()=>updatePreference({captureHideOnScreenShare:!settings.captureHideOnScreenShare})}><span class="choice-mark" aria-hidden="true">{settings.captureHideOnScreenShare?"✓":""}</span><span class="choice-copy"><strong>{t("hideShare")}</strong><small>{t("hideShareHint")}</small></span></button>
      </div></section>
      <section><h2>{t("tools")}</h2><div class="tool-list"><button disabled={!nativeRuntime} onclick={()=>windowApi.openFloatingWindow()}><ExternalLink size={17}/>{t("openFloating")}</button><button disabled={!nativeRuntime} onclick={()=>windowApi.resetFloatingWindow()}><RotateCcw size={17}/>{t("resetFloating")}</button><button disabled={!nativeRuntime} onclick={clearCache}><Trash2 size={17}/>{t("clearCache")}</button></div>{#if cacheMessage}<p class="message">{cacheMessage}</p>{/if}</section>
    </aside>
  </div>
</main>

<style>
  :global(html),:global(body),:global(#app){min-width:780px;min-height:600px;background:#fff;color:#111113} :global(body){overflow:auto}
  main{min-height:100vh;padding:38px 44px 44px;box-sizing:border-box;background:#fff}header{max-width:1180px;margin:0 auto 28px;display:flex;justify-content:space-between;align-items:flex-end;border-bottom:1px solid #e5e5e8;padding-bottom:20px}h1{margin:0;font-size:32px;line-height:1;letter-spacing:-.035em}header p{margin:9px 0 0;color:#6b6b72;font-size:14px}.status{padding:7px 10px;border-radius:999px;background:#eefaf0;color:#25713a;font-size:10px;font-weight:700;letter-spacing:.08em}
  .workspace{max-width:1180px;margin:auto;display:grid;grid-template-columns:minmax(420px,1fr) 320px;gap:20px;align-items:start}.workspace aside{width:100%}.stage{position:sticky;top:26px;min-height:560px;display:flex;align-items:center;justify-content:center;border:1px solid #e8e8eb;border-radius:24px;background:#fff;box-shadow:0 8px 24px rgba(20,20,26,.05);overflow:hidden;contain:layout paint;isolation:isolate}.stage:before,.stage:after{content:"";position:absolute;background:#f0f0f2}.stage:before{width:1px;height:100%;left:50%}.stage:after{height:1px;width:100%;top:50%}.preview-host{position:absolute;z-index:1;top:0;left:0;will-change:transform}.stage :global(.island-surface){z-index:1}.caption{position:absolute;z-index:2;left:22px;bottom:20px;display:flex;gap:8px;align-items:baseline}.caption strong{font-size:12px;text-transform:capitalize}.caption span,.ruler{color:#96969d;font-size:10px;font-variant-numeric:tabular-nums}.ruler{position:absolute;display:flex;justify-content:space-between}.ruler.top{z-index:2;top:12px;left:18px;right:18px}
  aside{display:flex;flex-direction:column;gap:12px}aside section{padding:18px;border-radius:16px;background:#f5f5f7}h2{margin:0 0 13px;font-size:12px;letter-spacing:.01em}.segmented{display:grid;grid-template-columns:repeat(4,1fr);padding:3px;border-radius:11px;background:#e9e9ec}.segmented.two{grid-template-columns:repeat(2,1fr)}.segmented button,.endpoints button{border:0;background:transparent;color:#66666e;font-size:10px}.segmented button{min-height:32px;padding:7px 4px;border-radius:8px}.segmented button.active{color:#111;background:#fff;box-shadow:0 2px 8px rgba(0,0,0,.08)}.field-label,.range-label{display:flex;align-items:baseline;justify-content:space-between;margin:14px 0 8px;font-size:11px}.field-label:first-of-type{margin-top:0}.field-label small{color:#777780;font-size:9px}.range-label output{color:#66666e;font-variant-numeric:tabular-nums}.edge-grid{display:grid;grid-template-columns:repeat(4,1fr);gap:6px}.edge-grid button{min-height:44px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:3px;border:1px solid transparent;border-radius:10px;color:#66666e;background:#fff;font-size:10px;cursor:pointer}.edge-grid button.active{color:#fff;background:#111113}.center-button{width:100%;min-height:36px;margin-top:7px;border:0;border-radius:10px;color:#414148;background:#fff;cursor:pointer}
  input[type="range"]{width:100%;accent-color:#111113}
   .select-row{display:flex;align-items:center;justify-content:space-between;padding:11px 0;border-top:1px solid #e5e5e8}.select-row span{display:flex;align-items:center;gap:8px;font-size:12px}.select-row select{max-width:160px;border:0;border-radius:8px;padding:6px 8px;background:#fff}.color-row input[type=color]{width:44px;height:28px;padding:3px;border:1px solid #d7d7dc;border-radius:8px;background:#fff;cursor:pointer}
  .section-title{display:flex;align-items:center;justify-content:space-between}.section-title h2{margin:0}.icon-button,.ordered-row button,.search-row button{display:grid;place-items:center;border:0;border-radius:8px;background:#fff;color:#414148;cursor:pointer}.icon-button{width:30px;height:30px}.hint,.empty{margin:7px 0;color:#777780;font-size:10px;line-height:1.45}.ordered-list{display:flex;flex-direction:column;gap:6px;margin-top:11px}.ordered-row{display:flex;align-items:center;gap:7px;min-height:42px;padding:7px;border-radius:10px;background:#fff}.ordered-row.offline{opacity:.62}.ordered-row>span{min-width:0;display:flex;flex:1;flex-direction:column;gap:2px}.ordered-row strong{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:11px}.ordered-row small{color:#777780;font-size:9px}.ordered-row button{width:27px;height:27px}.idle-list{margin-top:14px}.idle-row>span>input{width:100%;min-width:0;border:1px solid #dddde1;border-radius:7px;padding:5px 7px;font-size:10px}.add-button{display:flex;align-items:center;justify-content:center;gap:6px}.weather-box{margin-top:14px;padding-top:13px;border-top:1px solid #e2e2e5}.weather-box>label{font-size:11px;font-weight:600}.search-row{display:grid;grid-template-columns:1fr 34px;gap:6px;margin-top:7px}.search-row input{min-width:0;border:1px solid #dddde1;border-radius:9px;padding:8px}.selected-city{margin:7px 0 0;font-size:10px;color:#39724a}.weather-results{display:flex;flex-direction:column;gap:4px;margin-top:7px}.weather-results button{display:flex;align-items:center;justify-content:space-between;border:0;border-radius:8px;padding:8px;background:#fff;text-align:left;cursor:pointer}.weather-results small{color:#777780}.scenario-grid{display:grid;grid-template-columns:1fr 1fr;gap:7px}.scenario-grid button,.tool-list button,.follow-live{display:flex;align-items:center;gap:8px;border:0;border-radius:11px;color:#414148;background:#fff;cursor:pointer}.scenario-grid button{padding:10px}.scenario-grid button.active{color:#fff;background:#111113}.follow-live{width:100%;justify-content:center;margin-top:8px;padding:9px;font-size:11px}.follow-live.active{color:#25713a;background:#eefaf0}.progress-label{display:flex;justify-content:space-between;margin-top:16px;font-size:11px}.progress-label output{font-variant-numeric:tabular-nums;color:#6b6b72}.endpoints{display:flex;justify-content:space-between}.tool-list{display:flex;flex-direction:column;gap:7px}.tool-list button{padding:10px 11px}.tool-list button:active,.scenario-grid button:active,.edge-grid button:active,.center-button:active{transform:scale(.98)}button:disabled,select:disabled,input:disabled{cursor:not-allowed;opacity:.45}.message{margin:10px 0 0;color:#39724a;font-size:11px}.segmented button:focus-visible,.scenario-grid button:focus-visible,.edge-grid button:focus-visible,.center-button:focus-visible,.tool-list button:focus-visible,.icon-button:focus-visible,.ordered-row button:focus-visible,.weather-results button:focus-visible,select:focus-visible,input:focus-visible{outline:2px solid #111;outline-offset:2px}@media(max-width:848px){:global(html),:global(body),:global(#app){min-width:0}main{padding:24px}.workspace{grid-template-columns:1fr}.stage{position:relative;top:0;min-height:420px}}
   .setting-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:7px}.setting-grid.single{grid-template-columns:1fr}.setting-choice{min-width:0;min-height:66px;display:flex;align-items:flex-start;gap:8px;padding:11px;border:1px solid #e1e1e5;border-radius:11px;color:#36363d;background:#fff;text-align:left;cursor:pointer;transition:transform 120ms ease,background 140ms ease,border-color 140ms ease,color 140ms ease}.setting-choice.active{border-color:#111113;color:#fff;background:#111113}.setting-choice:active{transform:scale(.98)}.setting-choice:disabled{cursor:not-allowed}.choice-mark{display:grid;place-items:center;flex:none;width:17px;height:17px;margin-top:1px;border:1px solid #bdbdc4;border-radius:5px;color:transparent;font-size:11px;font-weight:800;line-height:1}.setting-choice.active .choice-mark,.mini-choice.active .choice-mark,.player-selection .choice-mark{border-color:#fff;color:#111113;background:#fff}.choice-copy{min-width:0;display:flex;flex-direction:column;gap:4px}.choice-copy strong{font-size:11px;line-height:1.2}.choice-copy small{color:#777780;font-size:9px;line-height:1.35}.setting-choice.active .choice-copy small{color:rgba(255,255,255,.68)}.player-selection{width:27px!important;height:27px!important;padding:0!important;border:0!important;background:transparent!important}.player-selection .choice-mark{width:17px;height:17px;margin:0}.mini-choice{width:27px!important;height:27px!important;padding:0!important;border:0!important;background:transparent!important}.mini-choice.active .choice-mark{border-color:#111113;color:#fff;background:#111113}.setting-choice:focus-visible,.player-selection:focus-visible,.mini-choice:focus-visible{outline:2px solid #111;outline-offset:2px}.player-selection:focus-visible{outline-color:#111}.setting-choice.active:focus-visible{outline-color:#fff}
</style>

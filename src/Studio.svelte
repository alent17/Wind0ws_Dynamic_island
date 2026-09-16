<script lang="ts">
  import { onMount } from "svelte";
  import { spring } from "svelte/motion";
  import { emit } from "@tauri-apps/api/event";
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
  let appearanceDragging=$state(false);
  let previewRaf=0;
  let queuedPreview:Partial<AppPreferences>={};
  const previewBars=[.45,.78,.58,.96,.7,.38];
  let scenarioMedia=$derived.by<MediaState>(()=>{const base={...DEMO_MEDIA,positionMs:244000*progress/100,lastUpdatedTimestamp:Date.now()};if(scenario==="paused")return{...base,isPlaying:false};if(scenario==="no-art")return{...base,albumArt:""};if(scenario==="long-title")return{...base,title:"宇宙尽头的浪漫主义与一场不会结束的午夜公路旅行",artist:"The Extremely Long Artist Name · 特别长的专辑名称"};return base});
  let sample=$derived(nativeRuntime&&followingLive?liveMedia:scenarioMedia);
  let currentGeometry=$derived(geometryFor(mode,settings.expandedCornerRadius??45,settings.islandEdge,settings.compactLength));
  let previewHost=$derived(hostFor(settings.islandStyle,settings.islandEdge,settings.compactLength));
  const previewPosition=spring(50,{stiffness:.1,damping:.7,precision:.1});
  $effect(()=>{previewPosition.set(settings.islandEdgePosition,{hard:!settings.enableAnimations||settings.reduceAnimations})});
  let previewPositionStyle=$derived.by(()=>{const p=$previewPosition/100;const horizontal=settings.islandEdge==="top"||settings.islandEdge==="bottom";const x=horizontal?Math.max(0,stageWidth-previewHost.width)*p:settings.islandEdge==="right"?Math.max(0,stageWidth-previewHost.width):0;const y=horizontal?(settings.islandEdge==="bottom"?Math.max(0,stageHeight-previewHost.height):0):Math.max(0,stageHeight-previewHost.height)*p;return `transform:translate3d(${x}px,${y}px,0)`});
  function selectScenario(next:Scenario){scenario=next;followingLive=false}
  const t=(key:TranslationKey,values:Record<string,string|number>={})=>translate(key,values,$locale);
  const enumLabel=(value:string)=>t(value as TranslationKey);
  onMount(()=>{
    nativeRuntime=Boolean((window as any).__TAURI_INTERNALS__);
    const unsubscribe=media.subscribe(value=>liveMedia=value);
    let disconnect:undefined|(()=>void);
    let disposed=false;
    let deferredFrame=0;
    let deferredTimer:ReturnType<typeof setTimeout>|undefined;
    if(nativeRuntime){
      void (async()=>{
        try{
          settings={...DEFAULT_SETTINGS,...await settingsApi.getPreferences()};
          applyAppFont(settings.fontId);
          setLocale(settings.language);
        }catch{}
        if(disposed)return;
        deferredFrame=requestAnimationFrame(()=>{
          deferredTimer=setTimeout(()=>{
            if(disposed)return;
            void connectMedia().then(stop=>{if(disposed)stop();else disconnect=stop}).catch(()=>{});
            void settingsApi.getAutoStart().then(autoStart=>{if(!disposed)settings={...settings,autoStart}}).catch(()=>{});
            void windowApi.getMonitors().then(value=>{if(!disposed)monitors=value}).catch(()=>{});
            void refreshSessions();
          },0);
        });
      })();
    }
    return()=>{
      disposed=true;
      if(deferredFrame)cancelAnimationFrame(deferredFrame);
      if(deferredTimer)clearTimeout(deferredTimer);
      if(textCommitTimer){clearTimeout(textCommitTimer);textCommitTimer=undefined;commitPreference({idleItems:settings.idleItems})}
      if(previewRaf){cancelAnimationFrame(previewRaf);previewRaf=0}
      queuedPreview={};
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
  function flushPreview(){
    const patch=queuedPreview;
    queuedPreview={};
    if(!Object.keys(patch).length)return;
    settings={...settings,...patch};
    if(nativeRuntime) void emit("settings-preview",patch);
  }
  function previewPreference(patch:Partial<AppPreferences>){
    queuedPreview={...queuedPreview,...patch};
    if(previewRaf)return;
    previewRaf=requestAnimationFrame(()=>{previewRaf=0;flushPreview()});
  }
  function commitPreference(patch:Partial<AppPreferences>){
    queuedPreview={...queuedPreview,...patch};
    if(previewRaf){cancelAnimationFrame(previewRaf);previewRaf=0}
    flushPreview();
    if(!nativeRuntime)return;
    schedulePersist();
  }
  function updatePreference(patch:Partial<AppPreferences>){commitPreference(patch)}
  async function setAlwaysOnTop(value:boolean){settings={...settings,alwaysOnTop:value};if(!nativeRuntime)return;await settingsApi.setAlwaysOnTop(value).catch(()=>{})}
  async function setAutoStart(value:boolean){settings={...settings,autoStart:value};if(nativeRuntime)await settingsApi.setAutoStart(value).catch(()=>{})}
  function setIslandStyle(value:IslandStyle){updatePreference({islandStyle:value})}
  function setIslandEdge(value:IslandEdge){updatePreference({islandEdge:value})}
  function setAlongEdge(value:number,commit=false){const patch={islandEdgePosition:Math.min(100,Math.max(0,value))};commit?commitPreference(patch):previewPreference(patch)}
  function setRadius(value:number,commit=false){const patch={expandedCornerRadius:Math.min(80,Math.max(0,value))};commit?commitPreference(patch):previewPreference(patch)}
  function setShoulderRadius(value:number,commit=false){const patch={edgeShoulderRadius:Math.min(16,Math.max(0,value))};commit?commitPreference(patch):previewPreference(patch)}
  function setCompactLength(value:number,commit=false){const patch={compactLength:Math.min(300,Math.max(80,value))};commit?commitPreference(patch):previewPreference(patch)}
  function setFont(value:AppPreferences["fontId"]){applyAppFont(value);updatePreference({fontId:value})}
  function setLanguage(value:AppLanguage){setLocale(value);updatePreference({language:value})}
  async function refreshSessions(){if(!nativeRuntime)return;try{sessions=await mediaApi.listMediaSessions()}catch{sessions=[]}}
  function playerChecked(id:string){return settings.selectedPlayerIds===null?true:settings.selectedPlayerIds.includes(id)}
  function togglePlayer(id:string,checked:boolean){let selected=settings.selectedPlayerIds===null?sessions.map(item=>item.id):[...settings.selectedPlayerIds];selected=checked?[...selected.filter(item=>item!==id),id]:selected.filter(item=>item!==id);updatePreference({selectedPlayerIds:selected})}
  function movePlayer(id:string,direction:-1|1){const selected=[...(settings.selectedPlayerIds??sessions.map(item=>item.id))];const from=selected.indexOf(id);const to=from+direction;if(from<0||to<0||to>=selected.length)return;[selected[from],selected[to]]=[selected[to],selected[from]];updatePreference({selectedPlayerIds:selected})}
  let playerRows=$derived.by(()=>{const selected=settings.selectedPlayerIds??sessions.map(item=>item.id);const byId=new Map(sessions.map(item=>[item.id,item]));return [...selected.map(id=>byId.get(id)??{id,displayName:id,source:"generic",isPlaying:false}),...sessions.filter(item=>!selected.includes(item.id))]});
  const idleLabel=(kind:string)=>t((({clock:"clock",date:"date",weather:"weather",network:"network",cpu:"cpu",memory:"memory",battery:"battery",custom:"customText"} as const)[kind]??"customText") as TranslationKey);
  function patchIdle(id:string,patch:Partial<IdleContentItem>){updatePreference({idleItems:settings.idleItems.map(item=>item.id===id?{...item,...patch}:item)})}
  function previewIdleText(id:string,text:string){
    previewPreference({idleItems:settings.idleItems.map(item=>item.id===id?{...item,text}:item)});
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
        <IslandSurface media={sample} {mode} islandStyle={settings.islandStyle} edge={settings.islandEdge} position={sample.positionMs} expandedRadius={settings.expandedCornerRadius??45} edgeShoulderRadius={settings.edgeShoulderRadius??8} compactLength={settings.compactLength} showSpectrum={settings.showSpectrum} spectrumMode={settings.spectrumMode} enableAnimations={settings.enableAnimations} reduceAnimations={settings.reduceAnimations || appearanceDragging} previewSpectrum={settings.spectrumMode==="realtime"?previewBars:undefined} interactive simulateHidden onToggle={()=>mode=mode==="expanded"?"compact":"expanded"} onMediaAction={(action)=>{followingLive=false;if(action==="play_pause")scenario=scenario==="paused"?"playing":"paused"}} />
      </div>
      <div class="caption"><strong>{enumLabel(mode)}</strong><span>{enumLabel(settings.islandStyle)} · {enumLabel(settings.islandEdge)} {settings.islandEdgePosition}% · {currentGeometry.width} × {currentGeometry.height} · r{currentGeometry.radius} · {t("shoulderShort")} {settings.edgeShoulderRadius}px</span></div>
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
        <label class="range-label"><span>{t("edgePosition")}</span><output>{settings.islandEdgePosition}%</output></label>
        <input aria-label={t("edgePosition")} type="range" min="0" max="100" step="1" value={settings.islandEdgePosition} onpointerdown={()=>appearanceDragging=true} onpointerup={()=>appearanceDragging=false} onpointercancel={()=>appearanceDragging=false} oninput={(e)=>setAlongEdge(Number(e.currentTarget.value))} onchange={(e)=>{appearanceDragging=false;setAlongEdge(Number(e.currentTarget.value),true)}}/>
        <button class="center-button" onclick={()=>setAlongEdge(50,true)}>{t("center")}</button>
        <label class="range-label"><span>{t("compactLength")}</span><output>{settings.compactLength}px</output></label>
        <input aria-label={t("compactLength")} type="range" min="80" max="300" step="1" value={settings.compactLength} onpointerdown={()=>appearanceDragging=true} onpointerup={()=>appearanceDragging=false} onpointercancel={()=>appearanceDragging=false} oninput={(e)=>setCompactLength(Number(e.currentTarget.value))} onchange={(e)=>{appearanceDragging=false;setCompactLength(Number(e.currentTarget.value),true)}}/>
        <label class="range-label"><span>{t("shoulder")}</span><output>{settings.edgeShoulderRadius}px</output></label>
        <input aria-label={t("shoulder")} type="range" min="0" max="16" step="1" value={settings.edgeShoulderRadius} onpointerdown={()=>appearanceDragging=true} onpointerup={()=>appearanceDragging=false} onpointercancel={()=>appearanceDragging=false} oninput={(e)=>setShoulderRadius(Number(e.currentTarget.value))} onchange={(e)=>{appearanceDragging=false;setShoulderRadius(Number(e.currentTarget.value),true)}}/>
        <label class="range-label"><span>{t("expandedRadius")}</span><output>{settings.expandedCornerRadius}px</output></label>
        <input aria-label={t("expandedRadius")} type="range" min="0" max="80" step="1" value={settings.expandedCornerRadius} onpointerdown={()=>appearanceDragging=true} onpointerup={()=>appearanceDragging=false} onpointercancel={()=>appearanceDragging=false} oninput={(e)=>setRadius(Number(e.currentTarget.value))} onchange={(e)=>{appearanceDragging=false;setRadius(Number(e.currentTarget.value),true)}}/>
      </section>
      <section><h2>{t("language")}</h2><label class="select-row"><span>{t("language")}</span><select value={settings.language} onchange={(e)=>setLanguage(e.currentTarget.value as AppLanguage)}><option value="system">{t("systemLanguage")}</option><option value="zh-CN">{t("chinese")}</option><option value="en">{t("english")}</option><option value="ja">{t("japanese")}</option></select></label></section>
      <section><h2>{t("font")}</h2><label class="select-row"><span><Type size={17}/>{t("appFont")}</span><select value={settings.fontId} onchange={(e)=>setFont(e.currentTarget.value as AppPreferences["fontId"])}>{#each FONT_OPTIONS as font}<option value={font.id}>{font.id==="system"?t("systemDefault"):font.label}</option>{/each}</select></label><p class="hint">{t("fontHint")}</p></section>
      <section><h2>{t("mediaScene")}</h2><div class="scenario-grid">
        <button class:active={!followingLive&&scenario==="playing"} onclick={()=>selectScenario("playing")}><Play size={17}/>{t("play")}</button><button class:active={!followingLive&&scenario==="paused"} onclick={()=>selectScenario("paused")}><Pause size={17}/>{t("pause")}</button><button class:active={!followingLive&&scenario==="no-art"} onclick={()=>selectScenario("no-art")}><ImageOff size={17}/>{t("noCover")}</button><button class:active={!followingLive&&scenario==="long-title"} onclick={()=>selectScenario("long-title")}><Type size={17}/>{t("longTitle")}</button>
      </div>{#if nativeRuntime}<button class="follow-live" class:active={followingLive} onclick={()=>followingLive=true}>{t("followMusic")}</button>{/if}<label class="progress-label"><span>{t("playbackProgress",{percent:progress})}</span><output>{progress}%</output></label><input type="range" min="0" max="100" value={progress} oninput={(e)=>{progress=Number(e.currentTarget.value);followingLive=false}}/><div class="endpoints"><button onclick={()=>{progress=0;followingLive=false}}>0%</button><button onclick={()=>{progress=50;followingLive=false}}>50%</button><button onclick={()=>{progress=100;followingLive=false}}>100%</button></div></section>
      <section><h2>{t("spectrum")}</h2><label class="toggle-row"><span><strong>{t("showSpectrum")}</strong><small>{t("showSpectrumHint")}</small></span><input type="checkbox" checked={settings.showSpectrum} onchange={(e)=>updatePreference({showSpectrum:e.currentTarget.checked})}/></label>{#if settings.showSpectrum}<div class="field-label"><span>{t("animationSource")}</span><small>{t("animationSourceHint")}</small></div><div class="segmented two"><button aria-pressed={settings.spectrumMode==="realtime"} class:active={settings.spectrumMode==="realtime"} onclick={()=>updatePreference({spectrumMode:"realtime"})}>{t("realtime")}</button><button aria-pressed={settings.spectrumMode==="random"} class:active={settings.spectrumMode==="random"} onclick={()=>updatePreference({spectrumMode:"random"})}>{t("random")}</button></div>{/if}</section>
      <section><div class="section-title"><h2>{t("players")}</h2><button class="icon-button" aria-label={t("refreshPlayers")} disabled={!nativeRuntime} onclick={refreshSessions}><RefreshCw size={15}/></button></div><p class="hint">{t("playersHint")}</p>
        <div class="ordered-list">{#each playerRows as player,index (player.id)}<div class="ordered-row" class:offline={!sessions.some(item=>item.id===player.id)}><input aria-label={t("select",{name:player.displayName})} type="checkbox" checked={playerChecked(player.id)} onchange={(e)=>togglePlayer(player.id,e.currentTarget.checked)}/><span><strong>{player.displayName}</strong><small>{player.isPlaying?t("playingNow"):sessions.some(item=>item.id===player.id)?t("detected"):t("offline")}</small></span>{#if playerChecked(player.id)}<button aria-label={t("moveUp")} disabled={index===0} onclick={()=>movePlayer(player.id,-1)}><ChevronUp size={14}/></button><button aria-label={t("moveDown")} disabled={index===(settings.selectedPlayerIds??sessions.map(item=>item.id)).length-1} onclick={()=>movePlayer(player.id,1)}><ChevronDown size={14}/></button>{/if}</div>{/each}{#if !playerRows.length}<p class="empty">{t("noSessions")}</p>{/if}</div>
      </section>
      <section><h2>{t("idleContent")}</h2>
        <label class="toggle-row"><span><strong>{t("enableIdle")}</strong><small>{t("enableIdleHint")}</small></span><input type="checkbox" checked={settings.idleContentEnabled} onchange={(e)=>updatePreference({idleContentEnabled:e.currentTarget.checked})}/></label>
        {#if settings.idleContentEnabled}
          <label class="range-label"><span>{t("interval")}</span><output>{t("seconds",{value:settings.idleRotationSeconds})}</output></label><input aria-label={t("idleInterval")} type="range" min="2" max="60" value={settings.idleRotationSeconds} oninput={(e)=>previewPreference({idleRotationSeconds:Number(e.currentTarget.value)})} onchange={(e)=>commitPreference({idleRotationSeconds:Number(e.currentTarget.value)})}/>
          <div class="ordered-list idle-list">{#each settings.idleItems as item,index (item.id)}<div class="ordered-row idle-row"><input aria-label={t("enableItem",{name:idleLabel(item.kind)})} type="checkbox" checked={item.enabled} onchange={(e)=>patchIdle(item.id,{enabled:e.currentTarget.checked})}/><span><strong>{idleLabel(item.kind)}</strong>{#if item.kind==="custom"}<input aria-label={t("customText")} maxlength="120" value={item.text} oninput={(e)=>previewIdleText(item.id,e.currentTarget.value)} onchange={commitIdleText} onblur={commitIdleText}/>{/if}</span><button aria-label={t("moveUp")} disabled={index===0} onclick={()=>moveIdle(item.id,-1)}><ChevronUp size={14}/></button><button aria-label={t("moveDown")} disabled={index===settings.idleItems.length-1} onclick={()=>moveIdle(item.id,1)}><ChevronDown size={14}/></button>{#if item.kind==="custom"}<button aria-label={t("deleteCustom")} onclick={()=>removeIdle(item.id)}><X size={14}/></button>{/if}</div>{/each}</div>
          <button class="center-button add-button" onclick={addCustom}><Plus size={15}/>{t("addCustom")}</button>
          <div class="weather-box"><label for="weather-city">{t("weatherCity")}</label><div class="search-row"><input id="weather-city" placeholder={t("cityExample")} bind:value={weatherQuery} onkeydown={(e)=>{if(e.key==="Enter")void searchWeather()}}/><button aria-label={t("searchCity")} onclick={searchWeather}><Search size={15}/></button></div>{#if settings.weatherLocation}<p class="selected-city">{t("currentCity",{name:settings.weatherLocation.name})}</p>{/if}{#if weatherMessage}<p class="message">{weatherMessage}</p>{/if}{#if weatherResults.length}<div class="weather-results">{#each weatherResults as item}<button onclick={()=>selectWeather(item)}><strong>{item.name}</strong><small>{weatherCandidateDetail(item)}</small></button>{/each}</div>{/if}<p class="hint">{t("weatherSource")}</p></div>
        {/if}
      </section>
      <section><h2>{t("mediaFeatures")}</h2>
        <label class="toggle-row"><span><strong>{t("hdCover")}</strong><small>{t("hdCoverHint")}</small></span><input type="checkbox" checked={settings.enableHdCover} onchange={(e)=>updatePreference({enableHdCover:e.currentTarget.checked})}/></label>
        <label class="toggle-row"><span><strong>{t("mvPlayback")}</strong><small>{t("mvPlaybackHint")}</small></span><input type="checkbox" checked={settings.enableMvPlayback} onchange={(e)=>updatePreference({enableMvPlayback:e.currentTarget.checked})}/></label>
      </section>
      <section><h2>{t("appBehavior")}</h2>
        <label class="toggle-row"><span><strong>{t("alwaysOnTop")}</strong><small>{t("alwaysOnTopHint")}</small></span><input type="checkbox" checked={settings.alwaysOnTop} disabled={!nativeRuntime} onchange={(e)=>setAlwaysOnTop(e.currentTarget.checked)}/></label>
        <label class="toggle-row"><span><strong>{t("autoStart")}</strong><small>{t("autoStartHint")}</small></span><input type="checkbox" checked={settings.autoStart} disabled={!nativeRuntime} onchange={(e)=>setAutoStart(e.currentTarget.checked)}/></label>
        <label class="select-row"><span><Monitor size={17}/>{t("monitor")}</span><select value={settings.monitorIndex} disabled={!nativeRuntime} onchange={(e)=>updatePreference({monitorIndex:Number(e.currentTarget.value)})}>{#each monitors as monitor}<option value={monitor.index}>{monitor.name}</option>{/each}{#if !monitors.length}<option>{t("primaryMonitor")}</option>{/if}</select></label>
      </section>
      <section><h2>{t("capture")}</h2><p class="hint">{t("captureHint")}</p>
        <label class="toggle-row"><span><strong>{t("hideScreenshot")}</strong><small>{t("hideScreenshotHint")}</small></span><input type="checkbox" checked={settings.captureHideOnScreenshot} disabled={!nativeRuntime} onchange={(e)=>updatePreference({captureHideOnScreenshot:e.currentTarget.checked})}/></label>
        <label class="toggle-row"><span><strong>{t("hideRecording")}</strong><small>{t("hideRecordingHint")}</small></span><input type="checkbox" checked={settings.captureHideOnRecording} disabled={!nativeRuntime} onchange={(e)=>updatePreference({captureHideOnRecording:e.currentTarget.checked})}/></label>
        <label class="toggle-row"><span><strong>{t("hideFullscreen")}</strong><small>{t("hideFullscreenHint")}</small></span><input type="checkbox" checked={settings.captureHideOnFullscreen} disabled={!nativeRuntime} onchange={(e)=>updatePreference({captureHideOnFullscreen:e.currentTarget.checked})}/></label>
        <label class="toggle-row"><span><strong>{t("hideShare")}</strong><small>{t("hideShareHint")}</small></span><input type="checkbox" checked={settings.captureHideOnScreenShare} disabled={!nativeRuntime} onchange={(e)=>updatePreference({captureHideOnScreenShare:e.currentTarget.checked})}/></label>
      </section>
      <section><h2>{t("tools")}</h2><div class="tool-list"><button disabled={!nativeRuntime} onclick={()=>windowApi.openFloatingWindow()}><ExternalLink size={17}/>{t("openFloating")}</button><button disabled={!nativeRuntime} onclick={()=>windowApi.resetFloatingWindow()}><RotateCcw size={17}/>{t("resetFloating")}</button><button disabled={!nativeRuntime} onclick={clearCache}><Trash2 size={17}/>{t("clearCache")}</button></div>{#if cacheMessage}<p class="message">{cacheMessage}</p>{/if}</section>
    </aside>
  </div>
</main>

<style>
  :global(html),:global(body),:global(#app){min-width:780px;min-height:600px;background:#fff;color:#111113} :global(body){overflow:auto}
  main{min-height:100vh;padding:38px 44px 44px;box-sizing:border-box;background:#fff}header{max-width:1180px;margin:0 auto 28px;display:flex;justify-content:space-between;align-items:flex-end;border-bottom:1px solid #e5e5e8;padding-bottom:20px}h1{margin:0;font-size:32px;line-height:1;letter-spacing:-.035em}header p{margin:9px 0 0;color:#6b6b72;font-size:14px}.status{padding:7px 10px;border-radius:999px;background:#eefaf0;color:#25713a;font-size:10px;font-weight:700;letter-spacing:.08em}
  .workspace{max-width:1180px;margin:auto;display:grid;grid-template-columns:minmax(420px,1fr) 320px;gap:20px;align-items:start}.stage{position:sticky;top:26px;min-height:560px;display:flex;align-items:center;justify-content:center;border:1px solid #e8e8eb;border-radius:24px;background:#fff;box-shadow:0 8px 24px rgba(20,20,26,.05);overflow:hidden;contain:layout paint;isolation:isolate}.stage:before,.stage:after{content:"";position:absolute;background:#f0f0f2}.stage:before{width:1px;height:100%;left:50%}.stage:after{height:1px;width:100%;top:50%}.preview-host{position:absolute;z-index:1;top:0;left:0;will-change:transform}.stage :global(.island-surface){z-index:1}.caption{position:absolute;z-index:2;left:22px;bottom:20px;display:flex;gap:8px;align-items:baseline}.caption strong{font-size:12px;text-transform:capitalize}.caption span,.ruler{color:#96969d;font-size:10px;font-variant-numeric:tabular-nums}.ruler{position:absolute;display:flex;justify-content:space-between}.ruler.top{z-index:2;top:12px;left:18px;right:18px}
  aside{display:flex;flex-direction:column;gap:12px}aside section{padding:18px;border-radius:16px;background:#f5f5f7;content-visibility:auto;contain-intrinsic-size:auto 180px}h2{margin:0 0 13px;font-size:12px;letter-spacing:.01em}.segmented{display:grid;grid-template-columns:repeat(4,1fr);padding:3px;border-radius:11px;background:#e9e9ec}.segmented.two{grid-template-columns:repeat(2,1fr)}.segmented button,.endpoints button{border:0;background:transparent;color:#66666e;font-size:10px}.segmented button{min-height:32px;padding:7px 4px;border-radius:8px}.segmented button.active{color:#111;background:#fff;box-shadow:0 2px 8px rgba(0,0,0,.08)}.field-label,.range-label{display:flex;align-items:baseline;justify-content:space-between;margin:14px 0 8px;font-size:11px}.field-label:first-of-type{margin-top:0}.field-label small{color:#777780;font-size:9px}.range-label output{color:#66666e;font-variant-numeric:tabular-nums}.edge-grid{display:grid;grid-template-columns:repeat(4,1fr);gap:6px}.edge-grid button{min-height:44px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:3px;border:1px solid transparent;border-radius:10px;color:#66666e;background:#fff;font-size:10px;cursor:pointer}.edge-grid button.active{color:#fff;background:#111113}.center-button{width:100%;min-height:36px;margin-top:7px;border:0;border-radius:10px;color:#414148;background:#fff;cursor:pointer}
  .scenario-grid{display:grid;grid-template-columns:1fr 1fr;gap:7px}.scenario-grid button,.tool-list button,.follow-live{display:flex;align-items:center;gap:8px;border:0;border-radius:11px;color:#414148;background:#fff;cursor:pointer}.scenario-grid button{padding:10px}.scenario-grid button.active{color:#fff;background:#111113}.follow-live{width:100%;justify-content:center;margin-top:8px;padding:9px;font-size:11px}.follow-live.active{color:#25713a;background:#eefaf0}.progress-label{display:flex;justify-content:space-between;margin-top:16px;font-size:11px}.progress-label output{font-variant-numeric:tabular-nums;color:#6b6b72}input[type="range"]{width:100%;accent-color:#111113}.endpoints{display:flex;justify-content:space-between}
  .toggle-row,.select-row{display:flex;align-items:center;justify-content:space-between;padding:11px 0;border-top:1px solid #e5e5e8}.toggle-row:first-of-type{border-top:0}.toggle-row span{display:flex;flex-direction:column;gap:3px}.toggle-row strong{font-size:12px}.toggle-row small{color:#777780;font-size:10px}.toggle-row input{width:38px;height:22px;accent-color:#111}.select-row span{display:flex;align-items:center;gap:8px;font-size:12px}.select-row select{max-width:160px;border:0;border-radius:8px;padding:6px 8px;background:#fff}
  .section-title{display:flex;align-items:center;justify-content:space-between}.section-title h2{margin:0}.icon-button,.ordered-row button,.search-row button{display:grid;place-items:center;border:0;border-radius:8px;background:#fff;color:#414148;cursor:pointer}.icon-button{width:30px;height:30px}.hint,.empty{margin:7px 0;color:#777780;font-size:10px;line-height:1.45}.ordered-list{display:flex;flex-direction:column;gap:6px;margin-top:11px}.ordered-row{display:flex;align-items:center;gap:7px;min-height:42px;padding:7px;border-radius:10px;background:#fff}.ordered-row.offline{opacity:.62}.ordered-row>span{min-width:0;display:flex;flex:1;flex-direction:column;gap:2px}.ordered-row strong{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:11px}.ordered-row small{color:#777780;font-size:9px}.ordered-row button{width:27px;height:27px}.idle-list{margin-top:14px}.idle-row>span>input{width:100%;min-width:0;border:1px solid #dddde1;border-radius:7px;padding:5px 7px;font-size:10px}.add-button{display:flex;align-items:center;justify-content:center;gap:6px}.weather-box{margin-top:14px;padding-top:13px;border-top:1px solid #e2e2e5}.weather-box>label{font-size:11px;font-weight:600}.search-row{display:grid;grid-template-columns:1fr 34px;gap:6px;margin-top:7px}.search-row input{min-width:0;border:1px solid #dddde1;border-radius:9px;padding:8px}.selected-city{margin:7px 0 0;font-size:10px;color:#39724a}.weather-results{display:flex;flex-direction:column;gap:4px;margin-top:7px}.weather-results button{display:flex;align-items:center;justify-content:space-between;border:0;border-radius:8px;padding:8px;background:#fff;text-align:left;cursor:pointer}.weather-results small{color:#777780}.tool-list{display:flex;flex-direction:column;gap:7px}.tool-list button{padding:10px 11px}.tool-list button:active,.scenario-grid button:active,.edge-grid button:active,.center-button:active{transform:scale(.98)}button:disabled,select:disabled,input:disabled{cursor:not-allowed;opacity:.45}.message{margin:10px 0 0;color:#39724a;font-size:11px}.segmented button:focus-visible,.scenario-grid button:focus-visible,.edge-grid button:focus-visible,.center-button:focus-visible,.tool-list button:focus-visible,.icon-button:focus-visible,.ordered-row button:focus-visible,.weather-results button:focus-visible,select:focus-visible,input:focus-visible{outline:2px solid #111;outline-offset:2px}@media(max-width:848px){:global(html),:global(body),:global(#app){min-width:0}main{padding:24px}.workspace{grid-template-columns:1fr}.stage{position:relative;top:0;min-height:420px}}
</style>

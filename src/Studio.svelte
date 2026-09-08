<script lang="ts">
  import { onMount } from "svelte";
  import { Play,Pause,ImageOff,Type,RotateCcw,Trash2,ExternalLink,Monitor,Power } from "lucide-svelte";
  import DynamicIsland, { type IslandMode } from "$lib/DynamicIsland.svelte";
  import { DEMO_MEDIA } from "$lib/mediaStore";
  import { settingsApi } from "$lib/api/settings";
  import { windowApi } from "$lib/api/window";
  import { cacheApi } from "$lib/api/cache";
  import type { AppPreferences, MediaState, MonitorInfo } from "$lib/api/types";

  type Scenario="playing"|"paused"|"no-art"|"long-title";
  let mode=$state<IslandMode>("expanded"); let scenario=$state<Scenario>("playing"); let progress=$state(50); let settings=$state<AppPreferences|null>(null); let monitors=$state<MonitorInfo[]>([]); let cacheMessage=$state("");
  let sample=$derived.by<MediaState>(()=>{const base={...DEMO_MEDIA,positionMs:244000*progress/100,lastUpdatedTimestamp:Date.now()};if(scenario==="paused")return{...base,isPlaying:false};if(scenario==="no-art")return{...base,albumArt:""};if(scenario==="long-title")return{...base,title:"宇宙尽头的浪漫主义与一场不会结束的午夜公路旅行",artist:"The Extremely Long Artist Name · 特别长的专辑名称"};return base});
  onMount(async()=>{try{settings=await settingsApi.getPreferences()}catch{}try{monitors=await windowApi.getMonitors()}catch{}});
  async function setPreference(key:"autoHide"|"alwaysOnTop",value:boolean){if(!settings)return;settings={...settings,[key]:value};if(key==="alwaysOnTop")await settingsApi.setAlwaysOnTop(value).catch(()=>{});else await settingsApi.savePreferences(settings).catch(()=>{})}
  async function setAutoStart(value:boolean){if(!settings)return;settings={...settings,autoStart:value};await settingsApi.setAutoStart(value).catch(()=>{})}
  async function clearCache(){cacheMessage="正在清理…";try{await cacheApi.clearCache();cacheMessage="缓存已清理"}catch{cacheMessage="清理失败，请稍后重试"}}
</script>

<svelte:head><title>Isle Studio</title></svelte:head>
<main>
  <header><div><h1>Isle Studio</h1><p>在真实组件上检查形变、边界与媒体状态。</p></div><span class="status">LIVE COMPONENT</span></header>
  <div class="workspace">
    <section class="stage" aria-label="灵动岛白底预览">
      <div class="ruler top"><span>0</span><span>371</span></div>
      <DynamicIsland media={sample} {mode} position={sample.positionMs} interactive={false} liveControls={false} onPreviewPlayPause={() => scenario = scenario === "paused" ? "playing" : "paused"}/>
      <div class="caption"><strong>{mode}</strong><span>{mode==="expanded"?"371 × 156 · r44":mode==="hover"?"136 × 39":"126 × 37"}</span></div>
    </section>
    <aside>
      <section><h2>预览状态</h2><div class="segmented">{#each ["compact","hover","expanded","hidden"] as item}<button class:active={mode===item} onclick={()=>mode=item as IslandMode}>{item}</button>{/each}</div></section>
      <section><h2>媒体场景</h2><div class="scenario-grid">
        <button class:active={scenario==="playing"} onclick={()=>scenario="playing"}><Play size={17}/>播放</button><button class:active={scenario==="paused"} onclick={()=>scenario="paused"}><Pause size={17}/>暂停</button><button class:active={scenario==="no-art"} onclick={()=>scenario="no-art"}><ImageOff size={17}/>无封面</button><button class:active={scenario==="long-title"} onclick={()=>scenario="long-title"}><Type size={17}/>长标题</button>
      </div><label class="progress-label"><span>播放进度</span><output>{progress}%</output></label><input type="range" min="0" max="100" bind:value={progress}/><div class="endpoints"><button onclick={()=>progress=0}>0%</button><button onclick={()=>progress=50}>50%</button><button onclick={()=>progress=100}>100%</button></div></section>
      <section><h2>应用行为</h2>
        <label class="toggle-row"><span><strong>自动收起</strong><small>指针移开后延迟 2.5 秒</small></span><input type="checkbox" checked={settings?.autoHide??true} onchange={(e)=>setPreference("autoHide",e.currentTarget.checked)}/></label>
        <label class="toggle-row"><span><strong>始终置顶</strong><small>让岛体保持在工作流上方</small></span><input type="checkbox" checked={settings?.alwaysOnTop??true} onchange={(e)=>setPreference("alwaysOnTop",e.currentTarget.checked)}/></label>
        <label class="toggle-row"><span><strong>开机启动</strong><small>登录 Windows 后自动运行</small></span><input type="checkbox" checked={settings?.autoStart??false} onchange={(e)=>setAutoStart(e.currentTarget.checked)}/></label>
        <label class="select-row"><span><Monitor size={17}/>显示器</span><select value={settings?.monitorIndex??0} onchange={(e)=>windowApi.moveToMonitor(Number(e.currentTarget.value))}>{#each monitors as monitor}<option value={monitor.index}>{monitor.name}</option>{/each}{#if !monitors.length}<option>主显示器</option>{/if}</select></label>
      </section>
      <section><h2>工具</h2><div class="tool-list"><button onclick={()=>windowApi.openFloatingWindow()}><ExternalLink size={17}/>打开悬浮播放器</button><button onclick={()=>windowApi.resetFloatingWindow()}><RotateCcw size={17}/>复位悬浮窗</button><button onclick={clearCache}><Trash2 size={17}/>清理媒体缓存</button></div>{#if cacheMessage}<p class="message">{cacheMessage}</p>{/if}</section>
    </aside>
  </div>
</main>

<style>
  :global(html),:global(body),:global(#app){min-width:780px;min-height:600px;background:#fff;color:#111113} :global(body){overflow:auto}
  main{min-height:100vh;padding:38px 44px 44px;box-sizing:border-box;background:#fff}header{max-width:1180px;margin:0 auto 28px;display:flex;justify-content:space-between;align-items:flex-end;border-bottom:1px solid #e5e5e8;padding-bottom:20px}h1{margin:0;font-size:32px;line-height:1;letter-spacing:-.035em}header p{margin:9px 0 0;color:#6b6b72;font-size:14px}.status{padding:7px 10px;border-radius:999px;background:#eefaf0;color:#25713a;font-size:10px;font-weight:700;letter-spacing:.08em}
  .workspace{max-width:1180px;margin:auto;display:grid;grid-template-columns:minmax(420px,1fr) 320px;gap:20px;align-items:start}.stage{position:sticky;top:26px;min-height:560px;display:flex;align-items:center;justify-content:center;border:1px solid #e8e8eb;border-radius:24px;background:#fff;box-shadow:0 20px 55px rgba(20,20,26,.07);overflow:hidden}.stage:before,.stage:after{content:"";position:absolute;background:#f0f0f2}.stage:before{width:1px;height:100%;left:50%}.stage:after{height:1px;width:100%;top:50%}.stage :global(.island){z-index:1}.caption{position:absolute;z-index:2;left:22px;bottom:20px;display:flex;gap:8px;align-items:baseline}.caption strong{font-size:12px;text-transform:capitalize}.caption span,.ruler{color:#96969d;font-size:10px;font-variant-numeric:tabular-nums}.ruler{position:absolute;display:flex;justify-content:space-between}.ruler.top{top:12px;left:18px;right:18px}
  aside{display:flex;flex-direction:column;gap:12px}aside section{padding:18px;border-radius:16px;background:#f5f5f7}h2{margin:0 0 13px;font-size:12px;letter-spacing:.01em}.segmented{display:grid;grid-template-columns:repeat(4,1fr);padding:3px;border-radius:11px;background:#e9e9ec}.segmented button,.endpoints button{border:0;background:transparent;color:#66666e;font-size:10px}.segmented button{padding:7px 4px;border-radius:8px}.segmented button.active{color:#111;background:#fff;box-shadow:0 2px 8px rgba(0,0,0,.08)}
  .scenario-grid{display:grid;grid-template-columns:1fr 1fr;gap:7px}.scenario-grid button,.tool-list button{display:flex;align-items:center;gap:8px;border:0;border-radius:11px;color:#414148;background:#fff;cursor:pointer}.scenario-grid button{padding:10px}.scenario-grid button.active{color:#fff;background:#111113}.progress-label{display:flex;justify-content:space-between;margin-top:16px;font-size:11px}.progress-label output{font-variant-numeric:tabular-nums;color:#6b6b72}input[type="range"]{width:100%;accent-color:#111113}.endpoints{display:flex;justify-content:space-between}
  .toggle-row,.select-row{display:flex;align-items:center;justify-content:space-between;padding:11px 0;border-top:1px solid #e5e5e8}.toggle-row:first-of-type{border-top:0}.toggle-row span{display:flex;flex-direction:column;gap:3px}.toggle-row strong{font-size:12px}.toggle-row small{color:#777780;font-size:10px}.toggle-row input{width:38px;height:22px;accent-color:#111}.select-row span{display:flex;align-items:center;gap:8px;font-size:12px}.select-row select{max-width:160px;border:0;border-radius:8px;padding:6px 8px;background:#fff}
  .tool-list{display:flex;flex-direction:column;gap:7px}.tool-list button{padding:10px 11px}.tool-list button:active,.scenario-grid button:active{transform:scale(.98)}.message{margin:10px 0 0;color:#39724a;font-size:11px}.segmented button:focus-visible,.scenario-grid button:focus-visible,.tool-list button:focus-visible,select:focus-visible,input:focus-visible{outline:2px solid #111;outline-offset:2px}@media(max-width:800px){:global(html),:global(body),:global(#app){min-width:0}main{padding:24px}.workspace{grid-template-columns:1fr}.stage{position:relative;top:0;min-height:420px}}
</style>

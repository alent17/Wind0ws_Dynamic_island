<script lang="ts">
  import { tick, onDestroy, onMount, type Snippet } from "svelte";
  import { fade, slide } from "svelte/transition";
  import { widgetEditMotion, islandMorphEasing } from "$lib/islandMotion";
  import { X, Check, Minus, ChevronLeft, ChevronRight } from "lucide-svelte";
  import SolidIcon from "$lib/components/media/SolidIcon.svelte";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import { WIDGET_KINDS, normalizeWidgets, widgetPanelOrder, widgetsFromPanelOrder, moveLayoutPanel, widgetPanelWidth, type IslandWidgets, type IslandWidgetKind, type IslandPanelKind } from "$lib/widgetLayout";

  let { layout, editing, enabled = true, reveal = 1, animate = true, styleMorphing = false, busy = false, error = false, musicWidth, children, widget, onChange, onSave, onCancel } = $props<{
    animate?: boolean; enabled?: boolean; reveal?: number; styleMorphing?: boolean;
    layout: IslandWidgets; editing: boolean; busy?: boolean; error?: boolean; musicWidth: number;
    children: Snippet; widget: Snippet<[IslandWidgetKind]>;
    onChange: (layout: IslandWidgets) => void; onSave: () => void; onCancel: () => void;
  }>();
  const t = (key: TranslationKey | IslandPanelKind) => translate(key === "system" ? "systemMonitor" : key === "music" ? "widgetMusic" : key, {}, $locale);
  const order = $derived(widgetPanelOrder(layout));
  const visibleOrder = $derived(editing || enabled || reveal > 0 ? order : ["music"] as IslandPanelKind[]);
  const count = $derived(order.length - 1);
  let root: HTMLDivElement;
  let panels: HTMLDivElement;
  let drag = $state<{kind:IslandPanelKind;pointer:number;fromCatalog:boolean;snapshot:IslandWidgets} | null>(null);
  let selected = $state<IslandWidgetKind | null>(null);
  let moved = $state(false);
  let valid = false;
  let startX = 0, startY = 0;
  let source: HTMLElement | null = null;
  let sourceRect: DOMRect;
  let ghost: HTMLDivElement | null = null;
  let previousCursor = "";
  let revision = 0;
  const swaps = new Map<HTMLElement,Animation>();
  function revealCard(node: HTMLElement, {card}: {card:boolean}) {
    const easing = widgetEditMotion(!editing || !animate || !card);
    if (!card) return {...easing};
    const width = parseFloat(node.style.width) || parseFloat(getComputedStyle(node).width);
    const revealFromRight = node.classList.contains("side-left");
    return {...easing,css:(t:number)=>{
      const hidden = `${(1-t)*100}%`;
      const clip = revealFromRight ? `inset(0 ${hidden} 0 0)` : `inset(0 0 0 ${hidden})`;
      return `width:${width*t}px;clip-path:${clip};opacity:${t};overflow:clip;`;
    }};
  }
  async function updateOrder(next: IslandPanelKind[]) {
    if(next.join() === order.join()) return;
    const token = ++revision;
    const previous = new Map([...panels.querySelectorAll<HTMLElement>("[data-panel-id]")].map(node=>[node.dataset.panelId,node.getBoundingClientRect()]));
    onChange(widgetsFromPanelOrder(next));
    await tick();
    if(token !== revision || !editing || !root.isConnected || !animate) return;
    for(const node of panels.querySelectorAll<HTMLElement>("[data-panel-id]")) {
      const old = previous.get(node.dataset.panelId);
      swaps.get(node)?.cancel();swaps.delete(node);
      if(!old || node.dataset.panelId === drag?.kind) continue;
      const rect = node.getBoundingClientRect();
      const scale = rect.width / node.offsetWidth || 1;
      const dx = (old.x-rect.x)/scale,dy=(old.y-rect.y)/scale;
      if(Math.abs(dx)+Math.abs(dy)<.5) continue;
      const animation=node.animate([{transform:`translate(${dx}px,${dy}px)`},{transform:"translate(0,0)"}],{duration:180,easing:"cubic-bezier(.25,.1,.25,1)"});
      swaps.set(node,animation);
      animation.onfinish=()=>{if(swaps.get(node)===animation)swaps.delete(node);};
    }
  }
  function makeGhost() {
    if(!source) return;
    ghost=document.createElement("div");
    ghost.className=root.className+" drag-preview";
    ghost.setAttribute("aria-hidden","true");ghost.inert=true;
    const computed=getComputedStyle(root);
    for(const key of computed) if(key.startsWith("--"))ghost.style.setProperty(key,computed.getPropertyValue(key));
    Object.assign(ghost.style,{position:"fixed",display:"block",width:`${sourceRect.width}px`,height:`${sourceRect.height}px`,margin:"0",padding:"0",zIndex:"2147483647",pointerEvents:"none",overflow:"hidden",borderRadius:"18px",background:"#080808",boxShadow:"0 14px 38px #0009,0 0 0 1px #ffffff50",opacity:".95"});
    const clone=source.cloneNode(true) as HTMLElement;
    clone.classList.remove("drag-source");
    Object.assign(clone.style,{position:"absolute",left:"0",top:"0",width:`${source.offsetWidth}px`,height:`${source.offsetHeight}px`,margin:"0",opacity:"1",transformOrigin:"0 0",transform:`scale(${sourceRect.width/source.offsetWidth},${sourceRect.height/source.offsetHeight})`});
    ghost.append(clone);document.body.append(ghost);
    previousCursor=document.body.style.cursor;document.body.style.cursor="grabbing";
  }
  function start(event: PointerEvent, kind: IslandPanelKind, fromCatalog=false) {
    if(!editing || busy || event.button!==0 || !event.isPrimary) return;
    if(fromCatalog && !order.includes(kind) && count>=2) return;
    event.preventDefault();event.stopPropagation();
    if(kind!=="music")selected=kind;
    source=(event.currentTarget as HTMLElement).closest<HTMLElement>("[data-panel-id]") ?? event.currentTarget as HTMLElement;
    sourceRect=source.getBoundingClientRect();
    startX=event.clientX;startY=event.clientY;moved=false;valid=false;
    drag={kind,pointer:event.pointerId,fromCatalog,snapshot:normalizeWidgets(layout)};
    root.focus({preventScroll:true});root.setPointerCapture(event.pointerId);
  }
  function move(event: PointerEvent) {
    if(!drag || event.pointerId!==drag.pointer) return;
    if(!moved && Math.hypot(event.clientX-startX,event.clientY-startY)<6) return;
    if(!moved){makeGhost();moved=true;}
    if(ghost){ghost.style.left=`${event.clientX-(startX-sourceRect.x)}px`;ghost.style.top=`${event.clientY-(startY-sourceRect.y)}px`;}
    const area=panels.getBoundingClientRect();
    valid=event.clientX>=area.left-24 && event.clientX<=area.right+24 && event.clientY>=area.top && event.clientY<=area.bottom;
    if(ghost)ghost.style.opacity=valid?".95":".55";
    if(!valid) return;
    const rest=order.filter(kind=>kind!==drag!.kind);
    let index=rest.length;
    for(let i=0;i<rest.length;i++) {
      const node=panels.querySelector<HTMLElement>(`[data-panel-id="${rest[i]}"]`);
      if(!node)continue;
      const rect=node.getBoundingClientRect();
      const transform=new DOMMatrixReadOnly(getComputedStyle(node).transform);
      const center=rect.left+rect.width/2-transform.m41*(rect.width/node.offsetWidth || 1);
      if(event.clientX<center){index=i;break;}
    }
    void updateOrder(moveLayoutPanel(order,drag.kind,index));
  }
  function cleanup() {
    if(ghost){ghost.remove();ghost=null;document.body.style.cursor=previousCursor;}
    if(drag && root?.hasPointerCapture(drag.pointer))root.releasePointerCapture(drag.pointer);
    drag=null;moved=false;source=null;valid=false;
  }
  function cancelDrag(restore=true) {
    if(drag && moved && restore)onChange(drag.snapshot);
    revision++;cleanup();
  }
  function finish(event: PointerEvent) {
    if(!drag || event.pointerId!==drag.pointer)return;
    const item=drag,add=item.fromCatalog && !moved && item.kind!=="music";
    if(event.type!=="pointerup" || (moved && !valid)) {cancelDrag();return;}
    cleanup();
    if(add)toggleFromCatalog(item.kind as IslandWidgetKind);
  }
  function toggleFromCatalog(kind:IslandWidgetKind) {
    if(busy)return;
    if(order.includes(kind)) {selected=null;void updateOrder(order.filter(item=>item!==kind));}
    else if(count<2) {selected=kind;void updateOrder([...order,kind]);}
  }
  function nudge(kind:IslandPanelKind,direction:-1|1) {
    const index=order.indexOf(kind);
    if(index+direction<0 || index+direction>=order.length)return;
    void updateOrder(moveLayoutPanel(order,kind,index+direction));
  }
  function remove(kind:IslandWidgetKind) {
    onChange(widgetsFromPanelOrder(order.filter(item=>item!==kind)));
  }
  $effect(()=>{if(!editing || busy){cancelDrag(false);selected=null;}});
  onMount(()=>{
    const escape=(event:KeyboardEvent)=>{if(drag && event.key==="Escape"){event.preventDefault();event.stopImmediatePropagation();cancelDrag();}};
    document.addEventListener("keydown",escape,true);
    return ()=>document.removeEventListener("keydown",escape,true);
  });
  onDestroy(()=>{cancelDrag(false);for(const animation of swaps.values())animation.cancel();swaps.clear();});
</script>

<svelte:window onpointermove={move} onpointerup={finish} onpointercancel={finish} onblur={()=>cancelDrag()} />

{#snippet dots()}
  <span class="drag-dots" aria-hidden="true"><i></i><i></i><i></i></span>
{/snippet}
{#snippet tools(kind:IslandPanelKind,index:number)}
  <div class="card-tools" class:music-tools={kind==="music"} in:fade={{duration:animate?180:0}} out:fade={{duration:animate?120:0}}>
    <button data-layout-edit-control type="button" aria-label={`${t("widgetMoveLeft")} ${t(kind)}`} disabled={busy || index===0} onclick={()=>nudge(kind,-1)}><ChevronLeft size={24}/></button>
    <button data-layout-edit-control type="button" class="grip" aria-label={`${t("widgetDrag")} ${t(kind)}`} disabled={busy} onpointerdown={event=>start(event,kind)} onkeydown={event=>{if(event.key==="ArrowLeft" || event.key==="ArrowRight"){event.preventDefault();nudge(kind,event.key==="ArrowLeft"?-1:1);}}}>{@render dots()}{t(kind)}</button>
    <button data-layout-edit-control type="button" aria-label={`${t("widgetMoveRight")} ${t(kind)}`} disabled={busy || index===order.length-1} onclick={()=>nudge(kind,1)}><ChevronRight size={24}/></button>
  </div>
{/snippet}

<div class="widget-layout" class:editing class:single-panel={count===0} class:no-motion={!animate} class:style-morphing={styleMorphing} class:dragging={moved} bind:this={root} role="group" tabindex="-1" aria-label={t("widgetEdit")} style={`--widget-width:${widgetPanelWidth(musicWidth)}px`}>
  <div class="panels" bind:this={panels}>
    {#each visibleOrder as kind,index (kind)}
      {@const music=kind==="music"}
      {@const side=index<visibleOrder.indexOf("music")?"left":"right"}
      <div role="group" aria-label={t(kind)} onpointerdown={event=>{if(editing && !(event.target as HTMLElement).closest("button"))start(event,kind);}} class:music-slot={music} class:widget-cell={!music} class:side={!music} class:side-left={!music && side==="left"} class:side-right={!music && side==="right"} class:outlined={editing && music} class:revealing={!editing && reveal<1} class:drag-source={moved && drag?.kind===kind} data-panel-id={kind} data-widget-side={music?undefined:side} style:width={`${music?musicWidth:(widgetPanelWidth(musicWidth)+12)*(editing?1:reveal)}px`} style:opacity={music || editing?1:reveal} inert={!music && !editing && (!enabled || reveal<.98)} in:revealCard={{card:!music}} out:revealCard={{card:!music}}>
        {#if music}
          {#if editing}{@render tools(kind,index)}{/if}
          <div class="music-content" inert={editing}>{@render children()}</div>
        {:else}
          <section class="widget-card" class:outlined={editing} data-widget-kind={kind}>
            {#if editing}
              <button data-layout-edit-control type="button" class="remove" aria-label={`${t("widgetRemove")} ${t(kind)}`} disabled={busy} onclick={()=>remove(kind)}><Minus size={22} strokeWidth={2.5}/></button>
              {@render tools(kind,index)}
            {/if}
            <div class="widget-content" inert={editing}>{@render widget(kind)}</div>
          </section>
        {/if}
      </div>
    {/each}
  </div>
  {#if editing}
    <footer class="editor-bar" aria-label={t("widgetEdit")} transition:slide={{duration:animate?420:0,easing:islandMorphEasing}}>
      <div class="catalog" onwheel={event=>{const el=event.currentTarget;if(el.scrollWidth>el.clientWidth){event.preventDefault();el.scrollLeft+=Math.abs(event.deltaX)>Math.abs(event.deltaY)?event.deltaX:event.deltaY;}}}>
        {#each WIDGET_KINDS as kind}
          {@const added=order.includes(kind)}
          <button data-layout-edit-control type="button" class="catalog-item" class:selected={selected===kind} class:added disabled={busy || (!added && count>=2)} aria-label={`${t("widgetChoose")} ${t(kind)}`} aria-pressed={added} onpointerdown={event=>start(event,kind,true)} onclick={event=>{if(event.detail===0)toggleFromCatalog(kind);}}>
            <span class="app-icon" class:timer={kind==="timer"} class:weather={kind==="weather"} class:clock={kind==="clock"} class:calendar={kind==="calendar"}><SolidIcon {kind} size={32}/>{#if added}<i><Check size={12}/></i>{/if}</span>
          </button>
        {/each}
      </div>
      {#if error}<p class="hint" role="status">{t("settingsSaveFailed")}</p>{/if}
      <div class="edit-actions">
        <button class="cancel" data-layout-edit-control type="button" aria-label={t("widgetCancel")} disabled={busy} onclick={onCancel}><X size={28}/></button>
        <button data-layout-edit-control type="button" class="done" aria-label={t("widgetDone")} disabled={busy} onclick={onSave}><Check size={30}/></button>
      </div>
    </footer>
  {/if}
</div>

<style>
  button,.app-icon{box-sizing:border-box}
  .side{flex:none}.side.revealing{overflow:hidden}.side-left{justify-content:flex-end}
  .widget-cell{position:relative;display:flex;align-items:stretch;flex:none;width:calc(var(--widget-width) + 12px)}.widget-cell.side-left>.widget-card{margin-right:12px}.widget-cell.side-right>.widget-card{margin-left:12px}
  .widget-layout{height:100%;min-height:0;display:flex;flex-direction:column;color:#fff;background:#000;--gap:12px}
  .panels{display:flex;flex:1;min-height:0;align-items:stretch;justify-content:center;position:relative;padding-left:var(--widget-edge-inset,0px);padding-right:var(--widget-edge-inset,0px);padding-top:var(--widget-vertical-inset,0px);padding-bottom:var(--widget-vertical-inset,0px);box-sizing:border-box;transition:padding 360ms cubic-bezier(.25,.1,.25,1)}
  .music-slot{flex:none;position:relative;min-width:0;transition:margin 360ms cubic-bezier(.25,.1,.25,1)}.music-content{height:100%;box-sizing:border-box;transition:padding-top 360ms cubic-bezier(.25,.1,.25,1)}
  .side{display:flex;align-items:stretch;position:relative;gap:0}
  .widget-card{position:relative;box-sizing:border-box;width:var(--widget-width);flex:none;padding:30px 24px 24px;min-height:0;display:flex;flex-direction:column;justify-content:center;transition:margin 360ms cubic-bezier(.25,.1,.25,1),padding 360ms cubic-bezier(.25,.1,.25,1)}
  .widget-content{min-width:0;min-height:0;flex:1;height:100%;display:flex;align-items:center}.widget-content :global(>*){width:100%}
  .editing .panels{padding-top:calc(14px + var(--widget-vertical-inset,0px));box-sizing:border-box}.editing .music-slot{margin:8px 0 4px}.editing .widget-card{margin:8px 0 4px;padding-top:66px}
  .editing .music-content{padding-top:66px;box-sizing:border-box}.card-tools.music-tools{right:16px}
  .drag-dots{display:flex;align-items:center;justify-content:center;gap:3px;pointer-events:none}.drag-dots i{width:3px;height:3px;flex:none;border-radius:50%;background:currentColor;opacity:.6}
  .drag-source{opacity:.25!important}.dragging .card-tools .grip{cursor:grabbing}
  .editing .widget-cell,.editing .music-slot{cursor:grab}.dragging .widget-cell,.dragging .music-slot{cursor:grabbing}
  .outlined{outline:2px solid #ffffff48;outline-offset:-8px;border-radius:24px;transition:outline-color 200ms ease}.outlined:hover,.outlined:focus-within{outline-color:#ffffff80}.drag-source{opacity:.45}
  button{font:inherit;color:inherit;border:0;border-radius:12px;background:transparent;cursor:pointer;touch-action:none;transition:background 150ms ease,box-shadow 150ms ease,color 150ms ease}button:disabled{opacity:.35;cursor:default}button:focus-visible{outline:0;background:rgba(255,255,255,.1);box-shadow:none}
  .remove{position:absolute;z-index:4;right:4px;top:0;background:#ff453a;border:0;color:#fff;border-radius:50%;width:32px;height:32px;display:grid;place-items:center;padding:0;box-shadow:0 2px 8px rgba(255,69,58,.28)}
  .card-tools{position:absolute;top:13px;left:16px;right:52px;display:flex;align-items:center;justify-content:space-between;color:#dedee3}.card-tools button{display:flex;align-items:center;background:transparent;justify-content:center;min-width:44px;min-height:44px;padding:6px;font-size:18px;border-radius:12px}.card-tools .grip{cursor:grab;gap:8px;padding:6px 12px;border:0;font-weight:500}.card-tools button:disabled{opacity:1;color:#606068}
  .editor-bar{position:relative;display:flex;flex:none;height:132px;box-sizing:border-box;align-items:center;justify-content:space-between;gap:12px;background:#000;padding:12px calc(20px + var(--editor-edge-inset,0px)) 26px 20px;border-top:1px solid #ffffff24;overflow:hidden}
  .no-motion .panels,.no-motion .music-slot,.no-motion .music-content,.no-motion .widget-card{transition:none}
  .style-morphing .panels{transition:none}
  .catalog{display:flex;flex:1;justify-content:center;gap:8px;align-items:center}.catalog-item{position:relative;display:flex;flex-direction:column;align-items:center;gap:8px;background:transparent;font-size:14px;color:#bcbcc5;padding:4px;min-width:60px}.app-icon{position:relative;width:52px;height:52px;border-radius:12px;display:grid;place-items:center;background:transparent;border:0;color:#e1e1e7}.app-icon.timer,.app-icon.weather,.app-icon.clock{color:#e1e1e7;background:transparent}.app-icon i{position:absolute;bottom:-3px;right:-3px;border-radius:50%;background:#e9e9ee;color:#080808;display:grid;place-items:center;width:18px;height:18px;border:2px solid #000}.selected .app-icon{background:rgba(255,255,255,.1);color:#fff;box-shadow:none}.selected{color:#fff}.edit-actions{position:static;display:flex;flex:none;gap:14px}.edit-actions button{display:grid;place-items:center;width:52px;height:52px;padding:0;border-radius:12px;background:transparent;border:0}.edit-actions .done{background:transparent;color:#fff}.edit-actions .cancel{background:#ff453a;color:#fff;box-shadow:0 4px 12px rgba(255,69,58,.28)}.hint{position:absolute;bottom:6px;left:20px;right:20px;text-align:center;font-size:14px;color:#aaa;margin:0;pointer-events:none}
  .single-panel .catalog{gap:4px}.single-panel .catalog-item{flex:0 0 52px;width:52px;min-width:0;gap:5px;font-size:12px}.single-panel .app-icon{width:44px;height:44px}.single-panel .edit-actions{gap:8px}.single-panel .edit-actions button{width:44px;height:44px}
  .widget-content :global(.timer-panel:not(.active) .timer-readout strong),.widget-content :global(.volume-readout strong){font-size:60px}
  .widget-content :global(.timer-ruler){height:96px;flex-basis:96px}
  .widget-content :global(.volume-ruler){height:72px;flex-basis:72px}
  .widget-content :global(.ruler-track){height:92px}
  .widget-content :global(.ruler-track button),.widget-content :global(.ruler-mark){height:92px}
  .widget-content :global(.volume-ruler .volume-track){top:0;right:0;bottom:0;left:0;height:auto}
  .widget-content :global(.tick-label){height:30px;font-size:20px}
  .widget-content :global(.tick){height:50px}
  .widget-content :global(.timer-main-button){height:56px;font-size:18px}
  .widget-content :global(.timer-readout small),.widget-content :global(.volume-label){font-size:16px}
  .widget-content :global(.device-trigger){min-height:40px;font-size:16px}
  .widget-content :global(.timer-ruler .ruler-track){left:50%;transform:translateX(calc(-5px - var(--active-index) * 10px))}
  .widget-content :global(.current-weather span){font-size:60px}
  .widget-content :global(.current-weather strong){font-size:20px}
  .widget-content :global(.forecast>div),.widget-content :global(.clock-face .caption){font-size:16px}
  .widget-content :global(.forecast small),.widget-content :global(.updated){font-size:14px}
  .widget-content :global(.timer-footer),.widget-content :global(.volume-footer){margin-top:auto}
  button:hover:not(:disabled){background:rgba(255,255,255,.1);box-shadow:none;color:#fff}
  .remove:hover:not(:disabled),.remove:focus-visible,.edit-actions .cancel:hover:not(:disabled),.edit-actions .cancel:focus-visible{color:#ff453a}
  .card-tools button:hover:not(:disabled){background:rgba(255,255,255,.1);color:#fff}
  .widget-content :global(.clock-face .digits){font-size:64px}
  .widget-content :global(.timer-active-controls .timer-main-button){width:auto;min-width:78px;height:44px;font-size:20px}
  .widget-content :global(.timer-active-controls .timer-reset){width:auto;min-width:76px;height:44px;font-size:20px}
  .catalog{min-width:0;overflow-x:auto;scrollbar-width:thin;scrollbar-color:#ffffff33 transparent;padding:5px}.catalog-item{flex:none}
</style>

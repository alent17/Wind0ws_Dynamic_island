<script lang="ts">
  import type { Snippet } from "svelte";
  import { Clock, CloudSun, Timer, Volume2, X, Check, Minus, GripVertical, ChevronLeft, ChevronRight } from "lucide-svelte";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import { WIDGET_KINDS, placeWidget, widgetPanelWidth, type IslandWidgets, type IslandWidgetKind, type WidgetSide } from "$lib/widgetLayout";

  let { layout, editing, busy = false, error = false, musicWidth, children, widget, onChange, onSave, onCancel } = $props<{
    layout: IslandWidgets; editing: boolean; busy?: boolean; error?: boolean; musicWidth: number;
    children: Snippet; widget: Snippet<[IslandWidgetKind]>;
    onChange: (layout: IslandWidgets) => void; onSave: () => void; onCancel: () => void;
  }>();
  const t = (key: TranslationKey) => translate(key, {}, $locale);
  const icons = { timer: Timer, weather: CloudSun, clock: Clock, volume: Volume2 };
  const count = $derived(layout.left.length + layout.right.length);
  let root: HTMLDivElement;
  let drag = $state<{kind: IslandWidgetKind; pointer: number} | null>(null);
  let target = $state<{side: WidgetSide; index: number} | null>(null);
  let selected = $state<IslandWidgetKind | null>(null);
  let moved = false;
  let startX = 0, startY = 0;

  function start(event: PointerEvent, kind: IslandWidgetKind) {
    if (!editing || busy || event.button !== 0) return;
    event.preventDefault();
    selected = kind;
    drag = {kind, pointer: event.pointerId};
    startX = event.clientX; startY = event.clientY; moved = false;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }
  function move(event: PointerEvent) {
    if (!drag || event.pointerId !== drag.pointer) return;
    if (Math.hypot(event.clientX - startX, event.clientY - startY) < 5 && !moved) return;
    moved = true;
    const elements = document.elementsFromPoint(event.clientX, event.clientY);
    const zone = elements.find(el => el instanceof HTMLElement && el.dataset.widgetSide && root.contains(el)) as HTMLElement | undefined;
    target = zone ? {side: zone.dataset.widgetSide as WidgetSide, index: Number(zone.dataset.widgetIndex)} : null;
  }
  function finish(event: PointerEvent) {
    if (!drag || event.pointerId !== drag.pointer) return;
    if (event.type === "pointerup" && moved && target) onChange(placeWidget(layout, drag.kind, target.side, target.index));
    drag = null; target = null;
  }
  function remove(kind: IslandWidgetKind) {
    onChange({left: layout.left.filter((item: IslandWidgetKind) => item !== kind), right: layout.right.filter((item: IslandWidgetKind) => item !== kind)});
  }
  function nudge(kind: IslandWidgetKind, side: WidgetSide, index: number, direction: -1 | 1) {
    if ((side === "left" && index === 0 && direction === -1) || (side === "right" && index === layout.right.length - 1 && direction === 1)) return;
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= layout[side].length) {
      onChange(placeWidget(layout, kind, side === "left" ? "right" : "left", side === "left" ? 0 : layout.left.length));
    } else {
      onChange(placeWidget(layout, kind, side, direction === 1 ? index + 2 : index - 1));
    }
  }
  $effect(() => { if (!editing || busy) { drag = null; target = null; selected = null; } });
</script>

<svelte:window onpointermove={move} onpointerup={finish} onpointercancel={finish} onblur={() => {drag = null; target = null;}} />

<div class="widget-layout" class:editing class:dragging={!!drag} bind:this={root} style={`--widget-width:${widgetPanelWidth(musicWidth)}px`}>
  <div class="panels">
    {#each ["left", "music", "right"] as side}
      {#if side === "music"}
        <div class="music-slot" class:outlined={editing} style:width={`${musicWidth}px`}>
          <div class="music-content" inert={editing}>{@render children()}</div>
        </div>
      {:else}
        {@const widgetSide = side as WidgetSide}
        <div class="side" class:empty={layout[widgetSide].length === 0} data-widget-side={side} data-widget-index={layout[widgetSide].length}>
          {#each layout[widgetSide] as kind, index (kind)}
            {#if editing}<button data-layout-edit-control type="button" class="drop-slot" class:highlight={target?.side === side && target?.index === index} data-widget-side={side} data-widget-index={index} aria-label={t(side === "left" ? "widgetAddLeft" : "widgetAddRight")} disabled={!selected || busy} onclick={() => selected && onChange(placeWidget(layout, selected, widgetSide, index))}>+</button>{/if}
            <section class="widget-card" class:outlined={editing} class:drag-source={drag?.kind === kind} data-widget-kind={kind}>
              {#if editing}
                <button data-layout-edit-control type="button" class="remove" aria-label={`${t("widgetRemove")} ${t(kind)}`} disabled={busy} onclick={() => remove(kind)}><Minus size={22} strokeWidth={2.5}/></button>
                <div class="card-tools">
                  <button data-layout-edit-control type="button" aria-label={`${t("widgetMoveLeft")} ${t(kind)}`} disabled={busy || (side === "left" && index === 0)} onclick={() => nudge(kind, widgetSide, index, -1)}><ChevronLeft size={24}/></button>
                  <button data-layout-edit-control type="button" class="grip" aria-label={`${t("widgetDrag")} ${t(kind)}`} disabled={busy} onpointerdown={event => start(event, kind)}><GripVertical size={22}/>{t(kind)}</button>
                  <button data-layout-edit-control type="button" aria-label={`${t("widgetMoveRight")} ${t(kind)}`} disabled={busy || (side === "right" && index === layout.right.length - 1)} onclick={() => nudge(kind, widgetSide, index, 1)}><ChevronRight size={24}/></button>
                </div>
              {/if}
              <div class="widget-content" inert={editing}>{@render widget(kind)}</div>
            </section>
          {/each}
          {#if editing}<button data-layout-edit-control type="button" class="drop-slot last" class:highlight={target?.side === side && target?.index === layout[widgetSide].length} data-widget-side={side} data-widget-index={layout[widgetSide].length} aria-label={t(side === "left" ? "widgetAddLeft" : "widgetAddRight")} disabled={!selected || busy} onclick={() => selected && onChange(placeWidget(layout, selected, widgetSide))}>+</button>{/if}
        </div>
      {/if}
    {/each}
  </div>
  {#if editing}
    <footer class="editor-bar" aria-label={t("widgetEdit")}>
      <div class="catalog">
        {#each WIDGET_KINDS as kind}
          {@const Icon = icons[kind]}
          {@const added = layout.left.includes(kind) || layout.right.includes(kind)}
          <button data-layout-edit-control type="button" class="catalog-item" class:selected={selected === kind} class:added disabled={busy || (!added && count >= 2)} aria-label={`${t("widgetChoose")} ${t(kind)}`} aria-pressed={selected === kind} onpointerdown={event => start(event, kind)} onclick={() => selected = kind}>
            <span class="app-icon" class:timer={kind === "timer"} class:weather={kind === "weather"} class:clock={kind === "clock"}><Icon size={32}/>{#if added}<i><Check size={12}/></i>{/if}</span>
            <span>{t(kind)}</span>
          </button>
        {/each}
      </div>
      <p class="hint" role="status">{error ? t("settingsSaveFailed") : count >= 2 ? t("widgetLimit") : t("widgetHint")}</p>
      <div class="edit-actions">
        <button data-layout-edit-control type="button" aria-label={t("widgetCancel")} disabled={busy} onclick={onCancel}><X size={28}/></button>
        <button data-layout-edit-control type="button" class="done" aria-label={t("widgetDone")} disabled={busy} onclick={onSave}><Check size={30}/></button>
      </div>
    </footer>
  {/if}
</div>

<style>
  .widget-layout{height:100%;min-height:0;display:flex;flex-direction:column;color:#fff;background:#000;--gap:12px}
  .panels{display:flex;flex:1;min-height:0;align-items:stretch;justify-content:center;position:relative;padding-left:var(--widget-edge-inset,0px);padding-right:var(--widget-edge-inset,0px);padding-top:var(--widget-vertical-inset,0px);padding-bottom:var(--widget-vertical-inset,0px);box-sizing:border-box}
  .music-slot{flex:none;position:relative;min-width:0}.music-content{height:100%}
  .side{display:flex;align-items:stretch;position:relative;gap:12px}.side.empty{gap:0}.side:first-child:not(.empty){margin-right:12px}.side:last-child:not(.empty){margin-left:12px}
  .widget-card{position:relative;box-sizing:border-box;width:var(--widget-width);flex:none;padding:30px 24px 24px;min-height:0;display:flex;flex-direction:column;justify-content:center}
  .widget-content{min-width:0;min-height:0;flex:1;height:100%;display:flex;align-items:center}.widget-content :global(>*){width:100%}
  .editing .panels{padding-top:calc(14px + var(--widget-vertical-inset,0px));box-sizing:border-box}.editing .music-slot{margin:8px 0 4px}.editing .widget-card{margin:8px 0 4px;padding-top:66px}
  .outlined{outline:1px dashed #ffffff29;outline-offset:-9px;border-radius:24px}.drag-source{opacity:.45}
  button{font:inherit;color:inherit;border:0;cursor:pointer;touch-action:none}button:disabled{opacity:.35;cursor:default}button:focus-visible{outline:2px solid #9fcaff;outline-offset:2px}
  .remove{position:absolute;z-index:4;right:4px;top:0;background:#c53b3b;border:1px solid #f46a65;color:#fff;border-radius:50%;width:36px;height:36px;display:grid;place-items:center;padding:0}
  .card-tools{position:absolute;top:13px;left:16px;right:52px;display:flex;align-items:center;justify-content:space-between;color:#aaa}.card-tools button{display:flex;align-items:center;background:transparent;justify-content:center;min-width:44px;min-height:44px;padding:6px;font-size:14px;border-radius:9px}.card-tools .grip{cursor:grab;gap:6px}
  .drop-slot{position:absolute;z-index:5;top:40%;width:36px;height:60px;margin-left:-18px;background:transparent;border:1px dashed #ffffff29;border-radius:8px;color:#ccc;font-size:26px}.drop-slot.last{right:-18px;margin:0}.side:first-child.empty .drop-slot{left:calc(14px + var(--editor-edge-inset,0px));right:auto}.side:last-child.empty .drop-slot{left:auto;right:calc(14px + var(--editor-edge-inset,0px))}.side:first-child:not(.empty)>.drop-slot:first-child{margin-left:2px}.side:last-child:not(.empty)>.drop-slot.last{right:2px}.drop-slot.highlight{background:#ffffff26;border-color:#ddd;color:#fff}.dragging .drop-slot{opacity:1}
  .editor-bar{position:relative;display:flex;flex:none;height:132px;box-sizing:border-box;align-items:center;justify-content:center;gap:14px;background:#000;padding:12px 144px 26px 24px;border-top:1px solid #ffffff08}
  .catalog{display:flex;gap:8px;align-items:center}.catalog-item{position:relative;display:flex;flex-direction:column;align-items:center;gap:8px;background:transparent;font-size:12px;color:#a9a9b3;padding:4px;min-width:60px}.app-icon{position:relative;width:52px;height:52px;border-radius:14px;display:grid;place-items:center;background:#292b38;box-shadow:0 0 0 3px #ffffff06;color:#8bbcff}.app-icon.timer{color:#eea12b;background:#30220e}.app-icon.weather{color:#ffd363;background:#193a52}.app-icon.clock{background:#ddd;color:#272735}.app-icon i{position:absolute;bottom:-3px;right:-3px;border-radius:50%;background:#2386f4;display:grid;place-items:center;width:18px;height:18px;color:white;border:2px solid #000}.selected .app-icon{outline:2px solid #bbb;outline-offset:3px}.edit-actions{position:absolute;right:calc(22px + var(--editor-edge-inset,0px));display:flex;gap:14px}.edit-actions button{display:grid;place-items:center;width:52px;height:52px;padding:0;border-radius:50%;background:#ffffff08;border:1px solid #ffffff20}.edit-actions .done{background:#1684ed;border-color:#419df4}.hint{position:absolute;bottom:6px;left:20px;right:20px;text-align:center;font-size:12px;color:#999;margin:0;pointer-events:none}
  .widget-content :global(.timer-readout strong),.widget-content :global(.volume-readout strong){font-size:60px}
  .widget-content :global(.timer-ruler),.widget-content :global(.volume-ruler){height:96px;flex-basis:96px}
  .widget-content :global(.ruler-track){height:92px}
  .widget-content :global(.ruler-track button),.widget-content :global(.ruler-mark){height:92px}
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
  .remove:hover:not(:disabled){background:#e24b45;border-color:#ff938c}
  .card-tools button:hover:not(:disabled){background:#ffffff12;color:#fff}
  .widget-content :global(.clock-face .digits){font-size:64px}
  .widget-content :global(.timer-actions.timer-active .timer-main-button),.widget-content :global(.timer-reset){width:56px;height:56px}
  .widget-content :global(.timer-actions svg){width:26px;height:26px}
</style>

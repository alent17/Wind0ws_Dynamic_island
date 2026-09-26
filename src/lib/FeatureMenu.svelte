<script lang="ts">
  import { onMount, type ComponentType } from "svelte";
  import { locale, translate } from "$lib/i18n";
  const t = (key: "featureTools") => translate(key, {}, $locale);
  let { items, toolbar = false, activeId = "", scrollPosition = 0, onScroll, onSelect } = $props<{
    items: { id: string; label: string; shortLabel?: string; icon: ComponentType }[];
    toolbar?: boolean;
    activeId?: string;
    scrollPosition?: number;
    onScroll?: (value: number) => void;
    onSelect: (id: string) => void;
  }>();
  let rail: HTMLDivElement;
  let pointer: number | null = null;
  let startX = 0;
  let startScroll = 0;
  let dragged = false;
  onMount(() => { rail.scrollLeft = scrollPosition; });
  function down(event: PointerEvent) {
    if (event.button !== 0) return;
    pointer = event.pointerId; startX = event.clientX; startScroll = rail.scrollLeft; dragged = false;
  }
  function move(event: PointerEvent) {
    if (pointer !== event.pointerId || rail.scrollWidth <= rail.clientWidth) return;
    if (Math.abs(event.clientX - startX) > 5) {
      dragged = true; rail.setPointerCapture(event.pointerId);
      rail.scrollLeft = startScroll + startX - event.clientX;
    }
  }
  function up(event: PointerEvent) {
    if (pointer !== event.pointerId) return;
    if (event.type === "pointercancel") dragged = true;
    pointer = null;
    if (rail.hasPointerCapture(event.pointerId)) rail.releasePointerCapture(event.pointerId);
  }
  function key(event: KeyboardEvent) {
    const buttons = [...rail.querySelectorAll<HTMLButtonElement>("button")];
    const index = buttons.indexOf(document.activeElement as HTMLButtonElement);
    if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const next = event.key === "Home" ? 0 : event.key === "End" ? buttons.length - 1
      : Math.max(0, Math.min(buttons.length - 1, index + (event.key === "ArrowRight" ? 1 : -1)));
    buttons[next]?.focus({ preventScroll: true });
    buttons[next]?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }
</script>

<div class="feature-menu" class:toolbar bind:this={rail} role="toolbar" tabindex="-1" aria-label={t("featureTools")}
  onclick={(event) => { if (dragged && event.detail > 0) event.stopPropagation(); }}
  onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={up} onlostpointercapture={up}
  onkeydown={key} onscroll={() => onScroll?.(rail.scrollLeft)}
  onwheel={(event) => {
    if (rail.scrollWidth <= rail.clientWidth) return;
    event.preventDefault(); rail.scrollLeft += Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY;
  }}>
  {#each items as item}
    <button class="function-icon" class:active={activeId === item.id} aria-pressed={activeId === item.id} type="button" aria-label={item.label} title={item.label}
      onclick={(event) => { event.stopPropagation(); if (!dragged || event.detail === 0) onSelect(item.id); }}>
      <span class="function-glyph"><item.icon size={16} strokeWidth={1.8} /></span>
    </button>
  {/each}
</div>

<style>
  .feature-menu{display:flex;gap:4px;width:100%;max-width:156px;margin:auto;overflow-x:auto;overflow-y:hidden;scrollbar-width:none;touch-action:pan-y;user-select:none;overscroll-behavior-x:contain}
  .feature-menu::-webkit-scrollbar{display:none}
  .function-icon{display:flex;flex:0 0 28px;min-width:0;flex-direction:column;align-items:center;gap:0;padding:0;border:0;background:transparent;color:#d9eaff;cursor:pointer;font:500 9px/1.2 var(--app-font,system-ui)}
  .function-glyph{display:grid;place-items:center;width:28px;height:28px;border-radius:12px;background:transparent;transition:background 140ms,transform 140ms}
  .function-icon:hover .function-glyph{background:rgba(255,255,255,.1)}
  .function-icon:active .function-glyph{transform:scale(.95)}
  .function-icon:focus-visible{outline:none}.function-icon:focus-visible .function-glyph{background:rgba(255,255,255,.1);outline:2px solid #91caff;outline-offset:-2px}
  @media(prefers-reduced-motion:reduce){.function-glyph{transition:none}}
  .function-icon.active{color:#91caff}.toolbar{gap:4px}.toolbar .function-icon{flex-basis:calc((100% - 16px)/5)}.toolbar .function-glyph{width:100%}
</style>

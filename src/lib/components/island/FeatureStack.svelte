<script lang="ts">
  import { untrack, type Snippet } from "svelte";
  import { ArrowLeft } from "lucide-svelte";
  import { WIDGET_KINDS, type IslandWidgetKind } from "$lib/widgetLayout";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import SolidIcon from "$lib/components/media/SolidIcon.svelte";

  let { widget, onBack, initial, onSelect, reduceMotion = false } = $props<{widget:Snippet<[IslandWidgetKind]>;onBack:()=>void;initial?:IslandWidgetKind;onSelect?:(kind:IslandWidgetKind)=>void;reduceMotion?:boolean}>();
  let selected = $state<IslandWidgetKind>(untrack(() => initial ?? "timer"));
  function select(kind:IslandWidgetKind) { selected = kind; onSelect?.(kind); }
  const t = (key:TranslationKey) => translate(key, {}, $locale);
  const label = (kind:IslandWidgetKind) => t(kind === "system" ? "systemMonitor" : kind);
</script>
<section class="feature-stack" class:reduce-motion={reduceMotion} aria-label={t("featureStackTitle")} data-stop-toggle>
  <header><button class="back" type="button" aria-label={t("back")} onclick={onBack}><ArrowLeft size={24}/></button><strong>{t("featureStackTitle")}</strong></header>
  <div class="stack-stage" aria-label={t("featureStackTitle")}>
    {#each WIDGET_KINDS as kind,index (kind)}
      <div class="stack-card back-card" class:selected={selected === kind} style={`--depth:${index}`}>
        <button class="card-handle" type="button" aria-label={label(kind)} aria-pressed={selected === kind} onpointermove={(event) => {if(event.pointerType === "mouse" && (event.movementX || event.movementY)) select(kind);}} onfocus={() => select(kind)} onclick={() => select(kind)}>
          <SolidIcon {kind} size={21}/><span>{label(kind)}</span>
        </button>
      </div>
    {/each}
    <section class="stack-card front-card" aria-label={label(selected)}>
      <div class="front-heading"><SolidIcon kind={selected} size={21}/><span>{label(selected)}</span></div>
      {#key selected}<div class="stack-content" data-stack-kind={selected}>{@render widget(selected)}</div>{/key}
    </section>
  </div>
</section>
<style>
  .feature-stack{display:flex;flex-direction:column;width:100%;height:100%;box-sizing:border-box;padding:calc(12px + var(--extension-block-inset,0px)) calc(24px + var(--extension-inline-inset,0px));color:#fff;font-family:var(--app-font,system-ui)}
  header{display:flex;align-items:center;gap:10px;flex:none;height:45px}header strong{font-size:19px}.back{display:grid;place-items:center;width:42px;height:42px;border:0;border-radius:12px;background:transparent;color:#fff;cursor:pointer}
  .stack-stage{position:relative;flex:1;min-height:0}.stack-card{position:absolute;left:0;right:0;top:calc(var(--depth, 5) * 36px);height:249px;box-sizing:border-box;border:1px solid #ffffff30;border-radius:32px;background:#101010;overflow:hidden;animation:stack-arrive 420ms cubic-bezier(.23,1,.32,1) both;animation-delay:calc(var(--depth, 5) * 20ms)}
  .back-card{z-index:calc(var(--depth) + 1);background:#181818}.back-card:nth-child(even){background:#202020}.back-card.selected{border-color:#ffffff75}.card-handle{position:relative;z-index:1;display:flex;align-items:center;gap:8px;width:100%;height:36px;padding:0 23px;border:0;background:transparent;color:#ffffffcf;font:600 15px var(--app-font,system-ui);text-align:left;cursor:pointer}.card-handle:focus-visible{outline:2px solid #fff;outline-offset:-3px}
  .front-card{--depth:6;top:216px;z-index:7;display:flex;flex-direction:column;background:#090909;box-shadow:0 -2px 12px #0006}.front-heading{display:flex;align-items:center;gap:8px;height:34px;flex:none;padding:0 23px;color:#fff;font-size:15px;font-weight:600}.stack-content{display:flex;align-items:center;flex:1;min-height:0;width:100%;box-sizing:border-box;padding:8px 24px 20px;animation:content-arrive 180ms ease both}.stack-content :global(>*){width:100%}
  .stack-content :global(.timer-panel:not(.active) .timer-readout strong),.stack-content :global(.volume-readout strong){font-size:60px}.stack-content :global(.timer-ruler){height:70px;flex-basis:70px}.stack-content :global(.timer-footer),.stack-content :global(.volume-footer){margin-top:auto}.stack-content :global(.timer-ruler .ruler-track){left:50%;transform:translateX(calc(-5px - var(--active-index) * 10px))}.stack-content :global(.timer-main-button){font-size:18px}.stack-content :global(.timer-readout small),.stack-content :global(.volume-label){font-size:16px}
  @keyframes stack-arrive{from{opacity:0;transform:translateY(-24px) scale(.98)}to{opacity:1;transform:translateY(0) scale(1)}}@keyframes content-arrive{from{opacity:0;transform:translateY(5px)}to{opacity:1;transform:translateY(0)}}.reduce-motion .stack-card,.reduce-motion .stack-content{animation:none}@media(prefers-reduced-motion:reduce){.stack-card,.stack-content{animation:none}}
</style>

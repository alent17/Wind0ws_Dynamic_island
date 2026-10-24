<script lang="ts">
  import { ListMusic, Repeat, Repeat1, Shuffle } from "lucide-svelte";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import type { MediaState } from "$lib/api/types";

  let { mode, disabled = false, error = "", onCycle } = $props<{
    mode: NonNullable<MediaState["neteasePlaybackMode"]>;
    disabled?: boolean;
    error?: string;
    onCycle?: () => void;
  }>();

  const t = (key: TranslationKey, values: Record<string, string | number> = {}) => translate(key, values, $locale);
  type NeteasePlaybackMode = NonNullable<MediaState["neteasePlaybackMode"]>;
  const MODE_KEYS: Record<NeteasePlaybackMode, TranslationKey> = {
    sequential: "modeSequential",
    repeat_list: "modeRepeatList",
    repeat_one: "modeRepeatOne",
    shuffle: "modeShuffle",
  };
  const modeKey = (value: NeteasePlaybackMode): TranslationKey => MODE_KEYS[value];
</script>

<span class="mode-control-wrap">
  <button
    type="button"
    class="mode-control"
    aria-label={t("cycleNeteaseMode", { mode: t(modeKey(mode)) })}
    aria-describedby={error ? "netease-mode-error" : undefined}
    aria-busy={disabled}
    title={t("cycleNeteaseMode", { mode: t(modeKey(mode)) })}
    {disabled}
    onclick={(event) => { event.stopPropagation(); onCycle?.(); }}
  >
    {#if mode === "sequential"}<ListMusic size={34} />
    {:else if mode === "repeat_one"}<Repeat1 size={34} />
    {:else if mode === "shuffle"}<Shuffle size={34} />
    {:else}<Repeat size={34} />{/if}
  </button>
  {#if error}<span id="netease-mode-error" class="mode-error" role="alert">{error}</span>{/if}
</span>

<style>
  .mode-control-wrap{position:relative;display:inline-flex;align-items:center;justify-content:center;width:48px;height:48px;flex:none}
  .mode-control{position:relative;isolation:isolate;width:48px;height:48px;display:grid;place-items:center;padding:0;border:0;border-radius:9px;color:rgba(255,255,255,.72);background:transparent;cursor:pointer;transition:transform 140ms cubic-bezier(.23,1,.32,1),color 140ms ease,background 150ms ease}
  .mode-control::before{content:"";position:absolute;inset:-2.5px;z-index:-1;border-radius:inherit;background:rgba(255,255,255,.1);opacity:0;pointer-events:none;transition:opacity 150ms ease}
  .mode-control:hover:not(:disabled){color:#fff}
  .mode-control:hover:not(:disabled)::before{opacity:1}
  .mode-control:active:not(:disabled){transform:scale(.92)}
  .mode-control:focus-visible{outline:2px solid #fff;outline-offset:2px}
  .mode-control:disabled{opacity:.42;cursor:default}
  .mode-error{position:absolute;z-index:5;top:calc(100% + 4px);right:0;width:max-content;max-width:min(260px,60vw);padding:5px 8px;border:1px solid rgba(255,180,171,.34);border-radius:7px;background:rgba(32,16,18,.96);color:#ffb4ab;font-size:11px;line-height:1.35;white-space:normal;overflow-wrap:anywhere;pointer-events:none;box-shadow:0 4px 14px rgba(0,0,0,.22)}
  @media(prefers-reduced-motion:reduce){.mode-control{transition:none}}
</style>

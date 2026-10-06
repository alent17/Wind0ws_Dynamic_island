<script lang="ts">
  import { ListMusic, Repeat, Repeat1, Shuffle } from "lucide-svelte";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import type { MediaState } from "$lib/api/types";

  let { mode, disabled = false, onCycle } = $props<{
    mode: NonNullable<MediaState["neteasePlaybackMode"]>;
    disabled?: boolean;
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

<button
  type="button"
  class="mode-control"
  aria-label={t("cycleNeteaseMode", { mode: t(modeKey(mode)) })}
  title={t("cycleNeteaseMode", { mode: t(modeKey(mode)) })}
  {disabled}
  onclick={(event) => { event.stopPropagation(); onCycle?.(); }}
>
  {#if mode === "sequential"}<ListMusic size={21} />
  {:else if mode === "repeat_one"}<Repeat1 size={21} />
  {:else if mode === "shuffle"}<Shuffle size={21} />
  {:else}<Repeat size={21} />{/if}
</button>

<style>
  .mode-control{width:40px;height:40px;display:grid;place-items:center;padding:0;border:0;border-radius:10px;color:rgba(255,255,255,.72);background:transparent;cursor:pointer;transition:transform 140ms cubic-bezier(.23,1,.32,1),color 140ms ease,background 150ms ease}
  .mode-control:hover:not(:disabled){color:#fff;background:rgba(255,255,255,.1)}
  .mode-control:active:not(:disabled){transform:scale(.92)}
  .mode-control:focus-visible{outline:2px solid #fff;outline-offset:2px}
  .mode-control:disabled{opacity:.42;cursor:default}
  @media(prefers-reduced-motion:reduce){.mode-control{transition:none}}
</style>

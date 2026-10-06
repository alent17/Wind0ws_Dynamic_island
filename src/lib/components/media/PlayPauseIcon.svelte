<script lang="ts">
  import { Pause, Play } from "lucide-svelte";

  let { playing = false, size = 32, reduceMotion = false } = $props<{
    playing?: boolean;
    size?: number;
    reduceMotion?: boolean;
  }>();
</script>

<span
  class="play-pause-icon"
  class:playing
  class:reduce-motion={reduceMotion}
  style={`--play-pause-size:${size}px`}
  aria-hidden="true"
>
  <Play class="play-glyph" {size} fill="currentColor" />
  <Pause class="pause-glyph" {size} fill="currentColor" />
</span>

<style>
  .play-pause-icon{position:relative;display:inline-grid;flex:none;width:var(--play-pause-size);height:var(--play-pause-size);place-items:center}
  .play-pause-icon :global(svg){position:absolute;inset:0;transition:opacity 150ms ease,transform 220ms cubic-bezier(.22,1,.36,1);transform-origin:center}
  .play-pause-icon :global(.play-glyph){opacity:1;transform:scale(1) rotate(0deg)}
  .play-pause-icon :global(.pause-glyph){opacity:0;transform:scale(.72) rotate(-32deg)}
  .play-pause-icon.playing :global(.play-glyph){opacity:0;transform:scale(.72) rotate(32deg)}
  .play-pause-icon.playing :global(.pause-glyph){opacity:1;transform:scale(1) rotate(0deg)}
  .play-pause-icon.reduce-motion :global(svg){transition:none}
  @media (prefers-reduced-motion:reduce){.play-pause-icon :global(svg){transition:none}}
</style>

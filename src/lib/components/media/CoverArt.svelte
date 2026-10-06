<script lang="ts">
  import { onDestroy, untrack } from "svelte";

  let { src = "", alt = "", className = "" } = $props<{
    src?: string;
    alt?: string;
    className?: string;
  }>();

  let sourceA = $state("");
  let sourceB = $state("");
  let activeSlot = $state<"a" | "b">("a");
  let generation = 0;
  let destroyed = false;
  let cleanupTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    const requestedSrc = src;
    const currentGeneration = ++generation;
    if (!requestedSrc) {
      if (cleanupTimer) clearTimeout(cleanupTimer);
      sourceA = "";
      sourceB = "";
      return;
    }
    if (requestedSrc === untrack(() => activeSlot === "a" ? sourceA : sourceB)) return;

    const image = new Image();
    image.onload = () => {
      if (currentGeneration !== generation) return;
      if (cleanupTimer) clearTimeout(cleanupTimer);
      const outgoingSlot = activeSlot;
      const incomingSlot = outgoingSlot === "a" ? "b" : "a";
      if (incomingSlot === "a") sourceA = requestedSrc;
      else sourceB = requestedSrc;
      activeSlot = incomingSlot;
      cleanupTimer = setTimeout(() => {
        if (currentGeneration !== generation) return;
        if (outgoingSlot === "a") sourceA = "";
        else sourceB = "";
      }, 220);
    };
    image.onerror = () => {
      // Keep the last successfully rendered cover if a new URL is unavailable.
    };
    image.src = requestedSrc;

    return () => {
      image.onload = null;
      image.onerror = null;
      if (cleanupTimer) clearTimeout(cleanupTimer);
      if (!destroyed && currentGeneration === generation) {
        if (untrack(() => activeSlot) === "a") sourceB = "";
        else sourceA = "";
      }
    };
  });

  onDestroy(() => {
    destroyed = true;
    generation++;
    if (cleanupTimer) clearTimeout(cleanupTimer);
  });
</script>

<span class="cover-art" aria-hidden="true">
  {#if sourceA}<img class="cover-image {className}" class:active={activeSlot === "a"} src={sourceA} {alt} draggable="false" />{/if}
  {#if sourceB}<img class="cover-image {className}" class:active={activeSlot === "b"} src={sourceB} {alt} draggable="false" />{/if}
</span>

<style>
  .cover-art{position:absolute;inset:0;display:block;overflow:hidden;pointer-events:none}
  :global(.cover-image){position:absolute;inset:0;width:100%;height:100%;display:block;object-fit:cover;opacity:0;transition:opacity 220ms ease;-webkit-user-drag:none;user-select:none}
  :global(.cover-image.active){opacity:1}
  @media (prefers-reduced-motion:reduce){:global(.cover-image){transition:none}}
</style>

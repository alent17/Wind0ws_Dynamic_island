<script lang="ts">
  import SolidIcon from "./SolidIcon.svelte";
  let { code = null, size = 16, strokeWidth = 1.8 }: { code?: number | null; size?: number; strokeWidth?: number } = $props();
  let kind = $derived(code === null ? "cloud" : code === 0 ? "sun" : code <= 3 ? "partly" : code <= 48 ? "fog" : code <= 67 ? "rain" : code <= 77 ? "snow" : code <= 86 ? "rain" : "storm");
</script>
<SolidIcon {kind} {size} colored/>

<script lang="ts">
  import CoverArt from "../src/lib/components/media/CoverArt.svelte";
  let src = $state("");
  (window as any).setCover = (value: string) => { src = value; };
</script>
<div style="position:relative;width:84px;height:84px"><CoverArt {src} /></div>

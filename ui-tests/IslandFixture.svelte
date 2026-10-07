<script lang="ts">
  import IslandSurface from "../src/lib/components/island/IslandSurface.svelte";
  import { DEMO_MEDIA } from "../src/lib/mediaStore";
  import type { IslandMode, IslandEdge, IslandStyle } from "../src/lib/islandGeometry";
  let media = $state({ ...DEMO_MEDIA });
  let animate = $state(false);
  let mode = $state<IslandMode>("compact");
  let edge = $state<IslandEdge>("top");
  let style = $state<IslandStyle>("floating");
  let length = $state(80);
  let scale = $state(1);
  let toolbar = $state(false);
  let offset = $state(0);
  let hostScale = $state(1);
  let settled = $state(false);
  (window as any).configureIsland = (config: any) => {
    if (config.playing !== undefined) media.isPlaying = config.playing;
    if (config.animate !== undefined) animate = config.animate;
    if (config.mode) mode = config.mode;
    if (config.edge) edge = config.edge;
    if (config.style) style = config.style;
    if (config.length) length = config.length;
    if (config.scale) scale = config.scale;
    if (config.toolbar !== undefined) toolbar = config.toolbar;
    if (config.offset !== undefined) offset = config.offset;
    if (config.hostScale !== undefined) hostScale = config.hostScale;
  };
</script>
<div class="stage" style:transform={`translateX(${offset}px) scale(${hostScale})`} data-settled={settled}>
  <IslandSurface {media} {mode} {edge} islandStyle={style} compactLength={length} expandedScale={scale} showCustomFunctionPanel={toolbar} showSpectrum={animate} spectrumMode={animate ? "random" : "realtime"} collapsedEdgeShoulderRadius={12} expandedEdgeShoulderRadius={32} onToggle={() => mode = mode === "expanded" ? "compact" : "expanded"} onRegionChange={(change) => settled = change.settled} />
</div>
<style>.stage{position:absolute;left:150px;top:50px;width:850px;height:600px}</style>

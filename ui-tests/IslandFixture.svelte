<script lang="ts">
  import IslandSurface from "../src/lib/components/island/IslandSurface.svelte";
  import { DEMO_MEDIA } from "../src/lib/mediaStore";
  import type { IslandMode, IslandEdge, IslandStyle } from "../src/lib/islandGeometry";
  import { normalizeWidgets, type IslandWidgets } from "../src/lib/widgetLayout";
  let widgets = $state<IslandWidgets>({left:[],right:[]});
  let saveFails = $state(false);
  let media = $state({ ...DEMO_MEDIA });
  let animate = $state(false);
  let mode = $state<IslandMode>("compact");
  let idle = $state(false);
  let visible = $state(true);
  let rememberedTitle = $state("");
  let edge = $state<IslandEdge>("top");
  let style = $state<IslandStyle>("floating");
  let length = $state(80);
  let expandedLength = $state(600);
  let shoulder = $state(32);
  let scale = $state(1);
  let toolbar = $state(false);
  let offset = $state(0);
  let hostScale = $state(1);
  let settled = $state(false);
  (window as any).configureIsland = (config: any) => {
    if (config.expandedLength !== undefined) expandedLength = config.expandedLength;
    if (config.shoulder !== undefined) shoulder = config.shoulder;
    if (config.source !== undefined) media.source = config.source;
    if (config.neteasePlaybackMode !== undefined) media.neteasePlaybackMode = config.neteasePlaybackMode;
    if (config.capabilities !== undefined) media.capabilities = {...media.capabilities,...config.capabilities};
    if (config.widgets !== undefined) widgets = normalizeWidgets(config.widgets);
    if (config.saveFails !== undefined) saveFails = config.saveFails;
    if (config.playing !== undefined) media.isPlaying = config.playing;
    if (config.title !== undefined) media.title = config.title;
    if (config.idle !== undefined) idle = config.idle;
    if (config.visible !== undefined) visible = config.visible;
    if (config.rememberedTitle !== undefined) rememberedTitle = config.rememberedTitle;
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
  <IslandSurface islandWidgets={widgets} onWidgetsSave={async layout => {if(saveFails) throw new Error("save failed"); widgets=layout;}} {media} {mode} {edge} {idle} {visible} lastPlayedMedia={rememberedTitle ? {title:rememberedTitle,artist:"Fixture",albumArt:"",sourceDisplay:"Fixture"} : null} islandStyle={style} compactLength={length} expandedScale={scale} showCustomFunctionPanel={toolbar} showSpectrum={animate} spectrumMode={animate ? "random" : "realtime"} collapsedEdgeShoulderRadius={12} expandedEdgeShoulderRadius={shoulder} {expandedLength} onToggle={() => mode = mode === "expanded" ? "compact" : "expanded"} onHoverChange={(hovering) => { if (mode !== "expanded") mode = hovering ? "hover" : "compact"; }} onRegionChange={(change) => settled = change.settled} />
</div>
<style>.stage{position:absolute;left:150px;top:50px;width:1050px;height:600px}</style>

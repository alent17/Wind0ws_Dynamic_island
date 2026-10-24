<script lang="ts">
  import { tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import IslandSurface from "../src/lib/components/island/IslandSurface.svelte";
  import { DEMO_MEDIA } from "../src/lib/mediaStore";
  import type { IslandRegionChange, IslandEdge, IslandStyle } from "../src/lib/islandGeometry";
  const query = new URLSearchParams(location.search);
  const edge: IslandEdge = ["top","right","bottom","left"].includes(query.get("edge") ?? "")
    ? query.get("edge") as IslandEdge : "top";
  const style: IslandStyle = query.get("style") === "edge" ? "edge" : "floating";
  const media = { ...DEMO_MEDIA, isPlaying: false, albumArt: "" };
  const events: object[] = [];
  let claimed = false;
  let ready = false;
  function record(kind: string, event: PointerEvent | MouseEvent) {
    if (!ready) return;
    events.push({kind,x:event.clientX,y:event.clientY,trusted:event.isTrusted,time:performance.now()});
    document.title = "Compact native events " + JSON.stringify(events);
  }
  async function accept(change: IslandRegionChange) {
    if (!change.settled || claimed) return;
    claimed = true;
    await tick();
    // One DOM read after settling: use actual component placement, no copied offset formula.
    const rect = document.querySelector(".island-surface")!.getBoundingClientRect();
    try {
      const receipt = await invoke("apply_compact", {region:{mode:"compact",settled:true,
        x:rect.x,y:rect.y,width:change.geometry.width,height:change.geometry.height,
        radii:change.radii,polygon:change.polygon}});
      ready = true;
      document.title = "Compact native ready " + JSON.stringify({edge,style,receipt});
    } catch (error) { document.title = "Compact native failed " + String(error); }
  }
</script>
<svelte:window onpointerdown={(e)=>record("pointerdown",e)} onpointerup={(e)=>record("pointerup",e)} onclick={(e)=>record("click",e)} />
<div class="stage">
  <IslandSurface {media} mode="compact" {edge} islandStyle={style} compactLength={80}
    expandedScale={1} showSpectrum={false} enableAnimations={false} reduceAnimations={true}
    collapsedEdgeShoulderRadius={8} boxShadow="none" onRegionChange={(c)=>void accept(c)} />
</div>
<style>:global(html),:global(body){margin:0;width:100%;height:100%;background:transparent} .stage{position:absolute;left:0;top:0;width:640px;height:360px}</style>

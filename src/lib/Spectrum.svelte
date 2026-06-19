<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  let { topColor = "#ffffff", bottomColor = "#888888" }: { topColor?: string; bottomColor?: string } = $props();

  const NUM_BARS = 6;
  const BAR_WIDTH = 4;
  const BAR_GAP = 3;
  const MAX_HEIGHT = 32;
  const MIN_HEIGHT = 3;
  const CORNER_RADIUS = 2;
  const CANVAS_HEIGHT = 36;

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let bars = $state(new Float32Array(NUM_BARS));
  let animationId: number;
  let unlisten: (() => void) | undefined;

  const CANVAS_WIDTH = NUM_BARS * (BAR_WIDTH + BAR_GAP) - BAR_GAP;

  onMount(async () => {
    ctx = canvas.getContext("2d")!;

    unlisten = await listen<number[]>("spectrum-data", (event) => {
      bars = new Float32Array(event.payload);
    });

    await invoke("start_spectrum");
    renderLoop();
  });

  onDestroy(async () => {
    cancelAnimationFrame(animationId);
    unlisten?.();
    await invoke("stop_spectrum");
  });

  function renderLoop() {
    animationId = requestAnimationFrame(renderLoop);
    draw();
  }

  function draw() {
    ctx.clearRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);

    for (let i = 0; i < NUM_BARS; i++) {
      const value = bars[i] ?? 0;
      const barH = MIN_HEIGHT + value * (MAX_HEIGHT - MIN_HEIGHT);
      const x = i * (BAR_WIDTH + BAR_GAP);
      const y = (CANVAS_HEIGHT - barH) / 2;

      const gradient = ctx.createLinearGradient(x, y + barH, x, y);
      gradient.addColorStop(0, bottomColor);
      gradient.addColorStop(1, topColor);

      const alpha = 0.5 + value * 0.5;
      ctx.globalAlpha = alpha;
      ctx.fillStyle = gradient;
      ctx.beginPath();
      ctx.roundRect(x, y, BAR_WIDTH, barH, CORNER_RADIUS);
      ctx.fill();
      ctx.globalAlpha = 1;

      if (value > 0.7) {
        ctx.fillStyle = "rgba(255, 255, 255, 0.9)";
        ctx.beginPath();
        ctx.arc(x + BAR_WIDTH / 2, y - 2, 1.2, 0, Math.PI * 2);
        ctx.fill();
      }
    }
  }
</script>

<canvas
  bind:this={canvas}
  width={CANVAS_WIDTH}
  height={CANVAS_HEIGHT}
  style="display:block;"
/>

<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';

  const NUM_BARS    = 6;
  const BAR_WIDTH   = 4;
  const BAR_GAP     = 3;
  const MAX_HEIGHT  = 30;
  const MIN_HEIGHT  = 2;
  const CORNER_R    = 2;
  const CANVAS_H    = 38;

  const CANVAS_W = NUM_BARS * (BAR_WIDTH + BAR_GAP) - BAR_GAP;

  let canvasEl: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let animId: number;
  let unlisten: UnlistenFn | undefined;

  let bars = $state<Float32Array>(new Float32Array(NUM_BARS));

  let latestBars = new Float32Array(NUM_BARS);

  onMount(async () => {
    const dpr = window.devicePixelRatio || 1;
    canvasEl.width  = CANVAS_W * dpr;
    canvasEl.height = CANVAS_H * dpr;
    canvasEl.style.width  = `${CANVAS_W}px`;
    canvasEl.style.height = `${CANVAS_H}px`;

    ctx = canvasEl.getContext('2d')!;
    ctx.scale(dpr, dpr);

    unlisten = await listen<number[]>('spectrum-data', (event) => {
      const payload = event.payload;
      for (let i = 0; i < NUM_BARS; i++) {
        latestBars[i] = payload[i] ?? 0;
      }
      bars = new Float32Array(latestBars);
    });

    try {
      await invoke('start_spectrum');
    } catch (e) {
      console.error('[Spectrum] 启动失败:', e);
    }

    startRenderLoop();
  });

  onDestroy(async () => {
    cancelAnimationFrame(animId);
    unlisten?.();
    try {
      await invoke('stop_spectrum');
    } catch (_) {}
  });

  function startRenderLoop() {
    const render = () => {
      animId = requestAnimationFrame(render);
      draw();
    };
    render();
  }

  function draw() {
    ctx.clearRect(0, 0, CANVAS_W, CANVAS_H);

    for (let i = 0; i < NUM_BARS; i++) {
      const value = latestBars[i] ?? 0;

      const barH = MIN_HEIGHT + value * (MAX_HEIGHT - MIN_HEIGHT);
      const x    = i * (BAR_WIDTH + BAR_GAP);
      const y    = (CANVAS_H - barH) / 2;

      const alpha = 0.45 + value * 0.55;
      ctx.fillStyle = `rgba(255, 255, 255, ${alpha.toFixed(2)})`;

      ctx.beginPath();
      ctx.roundRect(x, y, BAR_WIDTH, barH, CORNER_R);
      ctx.fill();

      if (value > 0.65) {
        const dotOpacity = (value - 0.65) / 0.35;
        ctx.fillStyle = `rgba(255, 255, 255, ${(dotOpacity * 0.9).toFixed(2)})`;
        ctx.beginPath();
        ctx.arc(
          x + BAR_WIDTH / 2,
          y - 2.5,
          1.5,
          0,
          Math.PI * 2
        );
        ctx.fill();
      }
    }
  }
</script>

<canvas bind:this={canvasEl} style="display:block;" />

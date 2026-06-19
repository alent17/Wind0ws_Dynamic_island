<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';

  let {
    topColor = '#ffffff',
    bottomColor = '#888888',
    scale = 1,
  }: { topColor?: string; bottomColor?: string; scale?: number } = $props();

  const NUM_BARS    = 6;
  const BASE_BAR_W  = 2;
  const BASE_BAR_GAP = 1.5;
  const BASE_MAX_H  = 14;
  const MIN_HEIGHT  = 2;
  const BASE_CORNER_R = 1;
  const BASE_CANVAS_H = 18;

  const BAR_WIDTH   = BASE_BAR_W * scale;
  const BAR_GAP     = BASE_BAR_GAP * (1 + (scale - 1) * 0.4);
  const MAX_HEIGHT  = BASE_MAX_H * scale;
  const CORNER_R    = BASE_CORNER_R * scale;
  const CANVAS_H    = BASE_CANVAS_H * scale;
  const CANVAS_W    = NUM_BARS * (BAR_WIDTH + BAR_GAP) - BAR_GAP;

  let canvasEl: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D;
  let animId: number;
  let unlisten: UnlistenFn | undefined;

  let bars = $state<Float32Array>(new Float32Array(NUM_BARS));
  let latestBars = new Float32Array(NUM_BARS);

  function parseColor(color: string): [number, number, number] {
    const rgbMatch = color.match(/rgb\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)/);
    if (rgbMatch) {
      return [+rgbMatch[1], +rgbMatch[2], +rgbMatch[3]];
    }
    const h = color.replace('#', '');
    return [
      parseInt(h.substring(0, 2), 16),
      parseInt(h.substring(2, 4), 16),
      parseInt(h.substring(4, 6), 16),
    ];
  }

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

    const [tr, tg, tb] = parseColor(topColor);
    const [br, bg, bb] = parseColor(bottomColor);

    for (let i = 0; i < NUM_BARS; i++) {
      const value = latestBars[i] ?? 0;
      const barH = MIN_HEIGHT + value * (MAX_HEIGHT - MIN_HEIGHT);
      const x = i * (BAR_WIDTH + BAR_GAP);
      const y = (CANVAS_H - barH) / 2;

      const gradient = ctx.createLinearGradient(x, y + barH, x, y);
      gradient.addColorStop(0, `rgba(${br}, ${bg}, ${bb}, 0.9)`);
      gradient.addColorStop(1, `rgba(${tr}, ${tg}, ${tb}, 1.0)`);

      const alpha = 0.6 + value * 0.4;
      ctx.globalAlpha = alpha;
      ctx.fillStyle = gradient;
      ctx.beginPath();
      ctx.roundRect(x, y, BAR_WIDTH, barH, CORNER_R);
      ctx.fill();
    }
    ctx.globalAlpha = 1;
  }
</script>

<canvas bind:this={canvasEl} style="display:block;" />

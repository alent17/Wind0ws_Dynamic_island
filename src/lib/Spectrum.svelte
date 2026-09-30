<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { retainSpectrum, spectrumValues } from "$lib/spectrumStore";
  import { shouldAnimateSpectrum, smoothSpectrumValue } from "$lib/spectrumRender";
  import { createSpectrumPalette } from "$lib/spectrumColors";
  import type { SpectrumMode } from "$lib/api/types";
  import { locale, translate } from "$lib/i18n";

  let {
    topColor = "#ffffff",
    bottomColor = "#888888",
    scale = 1,
    active = true,
    playing = true,
    mode = "realtime",
    reduceMotion = false,
    values,
  }: {
    topColor?: string;
    bottomColor?: string;
    scale?: number;
    active?: boolean;
    playing?: boolean;
    mode?: SpectrumMode;
    reduceMotion?: boolean;
    values?: number[];
  } = $props();

  const NUM_BARS = 6;
  const MIN_HEIGHT = 2;
  const MIN_FRAME_MS = 1000 / 30;
  let canvasEl: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let animId = 0;
  let renderTimer: ReturnType<typeof setTimeout> | undefined;
  let lastFrameTime = 0;
  let mounted = $state(false);
  let latestBars = new Float32Array(NUM_BARS);
  let randomBars = new Float32Array(NUM_BARS);
  let visibleBars = new Float32Array(NUM_BARS);
  const heights = new Float32Array(NUM_BARS);
  const paintedHeights = new Float32Array(NUM_BARS).fill(Number.NaN);
  let barColors: string[] = ["#ffffff", "#ffffff"];

  const barWidth = $derived(2 * scale);
  const barGap = $derived(1.5 * (1 + (scale - 1) * 0.4));
  const maxHeight = $derived(16 * scale);
  const cornerRadius = $derived(scale);
  const canvasHeight = $derived(18 * scale);
  const canvasWidth = $derived(NUM_BARS * (barWidth + barGap) - barGap);

  function resizeCanvas() {
    if (!canvasEl || !ctx) return;
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    canvasEl.width = Math.round(canvasWidth * dpr);
    canvasEl.height = Math.round(canvasHeight * dpr);
    canvasEl.style.width = `${canvasWidth}px`;
    canvasEl.style.height = `${canvasHeight}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    rebuildPalette();
  }

  function rebuildPalette() {
    if (!ctx) return;
    // Use only two solid colors, each shared by one half of the six bars.
    barColors = createSpectrumPalette(topColor, bottomColor, 2);
    paintedHeights.fill(Number.NaN);
  }

  function targetAt(index: number) {
    const sourceBars = mode === "random" ? randomBars : latestBars;
    const rawTarget = !playing ? 0 : reduceMotion ? 0.18 : values?.[index] ?? sourceBars[index] ?? 0;
    return rawTarget > 0.02 && !values && mode === "realtime"
      ? Math.min(1, 0.025 + rawTarget * 1.12)
      : rawTarget;
  }

  function barsMoving() {
    return visibleBars.some((value, index) => Math.abs(targetAt(index) - value) > 0.002);
  }

  function draw(frameMs = 1000 / 60) {
    if (!ctx) return;
    let changed = false;
    for (let index = 0; index < NUM_BARS; index += 1) {
      const target = targetAt(index);
      if (values || reduceMotion) visibleBars[index] = target;
      else {
        visibleBars[index] = smoothSpectrumValue(visibleBars[index], target, frameMs, index);
      }
      const value = visibleBars[index];
      // Quarter-pixel steps keep motion smooth without invalidating the native
      // transparent surface for changes too small to see.
      heights[index] = Math.round((MIN_HEIGHT + value * (maxHeight - MIN_HEIGHT)) * 4) / 4;
      changed ||= heights[index] !== paintedHeights[index];
    }
    if (!changed) return;
    ctx.clearRect(0, 0, canvasWidth, canvasHeight);
    ctx.beginPath();
    ctx.fillStyle = barColors[0] ?? "#ffffff";
    for (let index = 0; index < NUM_BARS; index += 1) {
      if (index === NUM_BARS / 2) {
        ctx.fill();
        ctx.beginPath();
        ctx.fillStyle = barColors[1] ?? "#ffffff";
      }
      const height = heights[index];
      const x = index * (barWidth + barGap);
      const y = (canvasHeight - height) / 2;
      ctx.roundRect(x, y, barWidth, height, cornerRadius);
    }
    ctx.fill();
    paintedHeights.set(heights);
    ctx.globalAlpha = 1;
  }

  function render(timestamp: number) {
    animId = 0;
    if (!mounted || !active) { lastFrameTime = 0; return; }
    draw(lastFrameTime ? Math.min(50, timestamp - lastFrameTime) : 1000 / 60);
    lastFrameTime = timestamp;
    if (values || reduceMotion || !barsMoving()) {
      lastFrameTime = 0;
      return;
    }
    renderTimer = window.setTimeout(() => {
      renderTimer = undefined;
      if (mounted && active) render(performance.now());
      else lastFrameTime = 0;
    // Account for draw time without a second callback for every frame.
    }, Math.max(0, timestamp + MIN_FRAME_MS - performance.now()));
  }

  function cancelRenderLoop() {
    if (animId) { cancelAnimationFrame(animId); animId = 0; }
    if (renderTimer !== undefined) { clearTimeout(renderTimer); renderTimer = undefined; }
  }

  function ensureRenderLoop() {
    if (values || reduceMotion) {
      cancelRenderLoop();
      if (active && mounted) draw();
      lastFrameTime = 0;
      return;
    }
    const shouldAnimate = shouldAnimateSpectrum(active, mounted, false, barsMoving());
    if (shouldAnimate && !animId && renderTimer === undefined) animId = requestAnimationFrame(render);
    if (!shouldAnimate) { cancelRenderLoop(); lastFrameTime = 0; }
  }

  $effect(() => { active; values; mode; playing; reduceMotion; ensureRenderLoop(); });
  $effect(() => { scale; resizeCanvas(); if (mounted && active) draw(); });
  $effect(() => { topColor; bottomColor; rebuildPalette(); if (mounted && active) draw(); });

  $effect(() => {
    if (!mounted || !active || !playing || values || mode !== "realtime" || reduceMotion) return;
    const stopListening = spectrumValues.subscribe((next) => { latestBars = next; ensureRenderLoop(); });
    const release = retainSpectrum();
    return () => {
      stopListening();
      release();
    };
  });

  $effect(() => {
    if (!mounted || values || mode !== "random" || !active || !playing) {
      randomBars = new Float32Array(NUM_BARS);
      return;
    }
    if (reduceMotion) {
      randomBars = Float32Array.from({ length: NUM_BARS }, () => 0.18);
      return () => {
        randomBars = new Float32Array(NUM_BARS);
      };
    }
    const retarget = () => {
      const now = Date.now();
      for (let index = 0; index < NUM_BARS; index += 1) {
        const beat = Math.max(0, Math.sin(now / 175 + index * 0.78));
        const ripple = (Math.sin(now / 255 + index * 1.6) + 1) * 0.5;
        const accent = Math.random() < 0.12 ? Math.random() * 0.2 : 0;
        randomBars[index] = Math.min(1, 0.1 + beat * 0.34 + ripple * 0.16 + Math.random() * 0.32 + accent);
      }
      ensureRenderLoop();
    };
    retarget();
    const timer = setInterval(retarget, 125);
    return () => {
      clearInterval(timer);
      randomBars = new Float32Array(NUM_BARS);
    };
  });

  onMount(() => {
    mounted = true;
    ctx = canvasEl.getContext("2d");
    resizeCanvas();
    draw();
    ensureRenderLoop();
  });

  onDestroy(() => {
    mounted = false;
    cancelRenderLoop();
  });
</script>

<canvas bind:this={canvasEl} aria-label={translate("audioSpectrum",{},$locale)}></canvas>

<style>canvas{display:block}</style>

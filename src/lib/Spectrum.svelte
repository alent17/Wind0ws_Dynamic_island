<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { retainSpectrum, spectrumValues } from "$lib/spectrumStore";
  import { shouldAnimateSpectrum } from "$lib/spectrumRender";
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
  let canvasEl: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let animId = 0;
  let mounted = $state(false);
  let latestBars = new Float32Array(NUM_BARS);
  let randomBars = new Float32Array(NUM_BARS);
  let visibleBars = new Float32Array(NUM_BARS);
  let barGradient: CanvasGradient | string = "#ffffff";

  const barWidth = $derived(2 * scale);
  const barGap = $derived(1.5 * (1 + (scale - 1) * 0.4));
  const maxHeight = $derived(14 * scale);
  const cornerRadius = $derived(scale);
  const canvasHeight = $derived(18 * scale);
  const canvasWidth = $derived(NUM_BARS * (barWidth + barGap) - barGap);

  function parseColor(color: string): [number, number, number] {
    const rgb = color.match(/rgb\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)/);
    if (rgb) return [+rgb[1], +rgb[2], +rgb[3]];
    const hex = color.replace("#", "");
    if (hex.length === 3) return hex.split("").map((value) => parseInt(value + value, 16)) as [number, number, number];
    return [parseInt(hex.slice(0, 2), 16) || 255, parseInt(hex.slice(2, 4), 16) || 255, parseInt(hex.slice(4, 6), 16) || 255];
  }

  const topRgb = $derived(parseColor(topColor));
  const bottomRgb = $derived(parseColor(bottomColor));

  function resizeCanvas() {
    if (!canvasEl || !ctx) return;
    const dpr = window.devicePixelRatio || 1;
    canvasEl.width = Math.round(canvasWidth * dpr);
    canvasEl.height = Math.round(canvasHeight * dpr);
    canvasEl.style.width = `${canvasWidth}px`;
    canvasEl.style.height = `${canvasHeight}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    rebuildGradient();
  }

  function rebuildGradient() {
    if (!ctx) return;
    const [tr, tg, tb] = topRgb;
    const [br, bg, bb] = bottomRgb;
    const gradient = ctx.createLinearGradient(0, canvasHeight, 0, 0);
    gradient.addColorStop(0, `rgba(${br},${bg},${bb},.9)`);
    gradient.addColorStop(1, `rgb(${tr},${tg},${tb})`);
    barGradient = gradient;
  }

  function draw() {
    if (!ctx) return;
    ctx.clearRect(0, 0, canvasWidth, canvasHeight);
    for (let index = 0; index < NUM_BARS; index += 1) {
      const sourceBars = mode === "random" ? randomBars : latestBars;
      const target = !playing ? 0 : reduceMotion ? 0.18 : values?.[index] ?? sourceBars[index] ?? 0;
      if (values) visibleBars[index] = target;
      else visibleBars[index] += (target - visibleBars[index]) * (target > visibleBars[index] ? 0.34 : 0.12);
      const value = visibleBars[index];
      const height = MIN_HEIGHT + value * (maxHeight - MIN_HEIGHT);
      const x = index * (barWidth + barGap);
      const y = (canvasHeight - height) / 2;
      ctx.globalAlpha = 0.6 + value * 0.4;
      ctx.fillStyle = barGradient;
      ctx.beginPath();
      ctx.roundRect(x, y, barWidth, height, cornerRadius);
      ctx.fill();
    }
    ctx.globalAlpha = 1;
  }

  function render() {
    if (!mounted || !active) { animId = 0; return; }
    draw();
    if (values) { animId = 0; return; }
    if (!playing && visibleBars.every((value) => value < 0.01)) {
      visibleBars.fill(0);
      draw();
      animId = 0;
      return;
    }
    animId = requestAnimationFrame(render);
  }

  function ensureRenderLoop() {
    if (values) {
      if (animId) { cancelAnimationFrame(animId); animId = 0; }
      if (active && mounted) draw();
      return;
    }
    if (shouldAnimateSpectrum(active, mounted, playing, false, visibleBars.some((value) => value >= 0.01)) && !animId) animId = requestAnimationFrame(render);
    if (!active && animId) { cancelAnimationFrame(animId); animId = 0; }
  }

  $effect(() => { active; values; mode; playing; reduceMotion; ensureRenderLoop(); });
  $effect(() => { scale; resizeCanvas(); });
  $effect(() => { topRgb; bottomRgb; rebuildGradient(); if (values && mounted && active) draw(); });

  $effect(() => {
    if (!mounted || !active || !playing || values || mode !== "realtime" || reduceMotion) return;
    const stopListening = spectrumValues.subscribe((next) => latestBars = next);
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
      randomBars = Float32Array.from({ length: NUM_BARS }, (_, index) => {
        if (!playing) return 0;
        const wave = (Math.sin(Date.now() / 230 + index * 1.35) + 1) * 0.16;
        return Math.min(1, 0.16 + wave + Math.random() * 0.48);
      });
    };
    retarget();
    const timer = setInterval(retarget, 170);
    return () => {
      clearInterval(timer);
      randomBars = new Float32Array(NUM_BARS);
    };
  });

  onMount(() => {
    mounted = true;
    ctx = canvasEl.getContext("2d");
    resizeCanvas();
    ensureRenderLoop();
  });

  onDestroy(() => {
    mounted = false;
    if (animId) cancelAnimationFrame(animId);
  });
</script>

<canvas bind:this={canvasEl} aria-label={translate("audioSpectrum",{},$locale)}></canvas>

<style>canvas{display:block}</style>

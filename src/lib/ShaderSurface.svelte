<script lang="ts">
  import { onMount } from "svelte";
  import type { IslandTheme, SpectrumMode } from "$lib/api/types";

  let {
    theme,
    width,
    height,
    playing = false,
    accentColor = "#5a7cff",
    secondaryColor = "#30d5c8",
    spectrumMode = "realtime",
    previewSpectrum,
    expandProgress = 0,
    frameRate = 30,
    enableAnimations = true,
    reduceMotion = false,
    hardwareAcceleration = true,
    visible = true,
  } = $props<{
    theme: IslandTheme;
    width: number;
    height: number;
    playing?: boolean;
    accentColor?: string;
    secondaryColor?: string;
    spectrumMode?: SpectrumMode;
    previewSpectrum?: number[];
    expandProgress?: number;
    frameRate?: number;
    enableAnimations?: boolean;
    reduceMotion?: boolean;
    hardwareAcceleration?: boolean;
    visible?: boolean;
  }>();

  let canvas: HTMLCanvasElement;
  let mounted = false;
  let webglReady = $state(false);
  let raf = 0;
  let lastFrame = 0;
  let gl: WebGLRenderingContext | null = null;
  let program: WebGLProgram | null = null;
  let buffer: WebGLBuffer | null = null;

  let uResolution: WebGLUniformLocation | null = null;
  let uTime: WebGLUniformLocation | null = null;
  let uExpand: WebGLUniformLocation | null = null;
  let uEnergy: WebGLUniformLocation | null = null;
  let uTheme: WebGLUniformLocation | null = null;
  let uAccent: WebGLUniformLocation | null = null;
  let uSecondary: WebGLUniformLocation | null = null;

  const vertexSource = `
    attribute vec2 a_position;
    void main() {
      gl_Position = vec4(a_position, 0.0, 1.0);
    }
  `;

  const fragmentSource = `
    precision highp float;

    uniform vec2 u_resolution;
    uniform float u_time;
    uniform float u_expand;
    uniform float u_energy;
    uniform float u_theme;
    uniform vec3 u_accent;
    uniform vec3 u_secondary;

    float hash(vec2 p) {
      p = fract(p * vec2(123.34, 456.21));
      p += dot(p, p + 45.32);
      return fract(p.x * p.y);
    }

    float noise(vec2 p) {
      vec2 i = floor(p);
      vec2 f = fract(p);
      f = f * f * (3.0 - 2.0 * f);
      float a = hash(i);
      float b = hash(i + vec2(1.0, 0.0));
      float c = hash(i + vec2(0.0, 1.0));
      float d = hash(i + vec2(1.0, 1.0));
      return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
    }

    float blob(vec2 p, vec2 center, vec2 scale) {
      vec2 q = (p - center) / scale;
      return exp(-dot(q, q) * 2.65);
    }

    vec3 dialPalette(float t) {
      vec3 cyan = vec3(0.06, 0.58, 1.0);
      vec3 violet = vec3(0.48, 0.18, 1.0);
      vec3 magenta = vec3(1.0, 0.10, 0.55);
      vec3 orange = vec3(1.0, 0.46, 0.08);
      float x = fract(t);
      if (x < 0.25) return mix(cyan, violet, x * 4.0);
      if (x < 0.50) return mix(violet, magenta, (x - 0.25) * 4.0);
      if (x < 0.75) return mix(magenta, orange, (x - 0.50) * 4.0);
      return mix(orange, cyan, (x - 0.75) * 4.0);
    }

    void main() {
      vec2 resolution = max(u_resolution, vec2(1.0));
      vec2 uv = gl_FragCoord.xy / resolution;
      vec2 p = uv - 0.5;
      float aspect = resolution.x / resolution.y;
      p.x *= aspect;

      float t = u_time;
      float energy = clamp(u_energy, 0.0, 1.0);
      float expand = clamp(u_expand, 0.0, 1.0);

      float n1 = noise(vec2(p.x * 0.75 + t * 0.055, p.y * 1.8 - t * 0.04));
      float n2 = noise(vec2(p.y * 2.2 + t * 0.03, p.x * 0.45 + 8.0));
      vec2 warped = p + vec2((n1 - 0.5) * 0.22, (n2 - 0.5) * 0.12);

      float angle = atan(warped.y, warped.x);
      float radius = length(warped * vec2(0.58, 1.0));
      float phase = angle * 0.46 - radius * 3.8 - t * (0.34 + energy * 0.22);
      float dial = 0.5 + 0.5 * sin(phase * 3.0 + n1 * 2.1);
      float ribbon = pow(clamp(dial, 0.0, 1.0), 4.2);

      vec2 c1 = vec2(-aspect * 0.34 + sin(t * 0.31) * 0.16, 0.16 + cos(t * 0.23) * 0.08);
      vec2 c2 = vec2( aspect * 0.20 + cos(t * 0.27) * 0.19, -0.20 + sin(t * 0.20) * 0.09);
      vec2 c3 = vec2( sin(t * 0.19) * aspect * 0.18, 0.05 + cos(t * 0.17) * 0.16);

      float b1 = blob(p, c1, vec2(max(0.72, aspect * 0.31), 0.46));
      float b2 = blob(p, c2, vec2(max(0.64, aspect * 0.28), 0.40));
      float b3 = blob(p, c3, vec2(max(0.58, aspect * 0.24), 0.34));

      vec3 colorA;
      vec3 colorB;
      vec3 colorC;
      if (u_theme < 0.5) {
        colorA = dialPalette(0.02 + t * 0.018);
        colorB = dialPalette(0.35 + t * 0.015);
        colorC = dialPalette(0.67 + t * 0.012);
      } else {
        colorA = u_accent;
        colorB = u_secondary;
        colorC = mix(u_accent, vec3(1.0, 0.30, 0.62), 0.36);
      }

      vec3 color = colorA * b1 + colorB * b2 + colorC * b3;
      float body = clamp(max(max(b1, b2), b3), 0.0, 1.0);

      float spectrumBoost = 0.72 + energy * 0.62;
      color *= spectrumBoost;
      color += dialPalette(0.13 + angle / 6.28318 + t * 0.02) * ribbon * (0.16 + energy * 0.16);

      float edgeDistance = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
      float rim = 1.0 - smoothstep(0.0, 0.16, edgeDistance);
      float rimGlow = pow(rim, 1.8);
      vec3 rimColor = u_theme < 0.5
        ? dialPalette(angle / 6.28318 + t * 0.022)
        : mix(u_secondary, u_accent, 0.5 + 0.5 * sin(angle + t * 0.22));
      color += rimColor * rimGlow * (0.10 + 0.08 * expand);

      float diagonal = uv.x * 0.70 + uv.y * 0.30;
      float sheenCenter = 0.54 + sin(t * 0.16) * 0.11;
      float sheen = exp(-pow((diagonal - sheenCenter) * 10.0, 2.0));
      sheen *= smoothstep(0.08, 0.60, body + ribbon);
      color += vec3(0.90, 0.96, 1.0) * sheen * (0.13 + energy * 0.08);

      float micro = noise(gl_FragCoord.xy * 0.28 + vec2(t * 2.0, -t));
      color += (micro - 0.5) * 0.022;

      float luminance = max(max(color.r, color.g), color.b);
      float alpha = body * (0.34 + expand * 0.12)
        + ribbon * (0.12 + energy * 0.08)
        + rimGlow * 0.08
        + sheen * 0.08;
      alpha *= smoothstep(0.02, 0.18, luminance);
      alpha = clamp(alpha, 0.0, 0.72);

      gl_FragColor = vec4(max(color, 0.0), alpha);
    }
  `;

  function compileShader(context: WebGLRenderingContext, type: number, source: string) {
    const shader = context.createShader(type);
    if (!shader) return null;
    context.shaderSource(shader, source);
    context.compileShader(shader);
    if (!context.getShaderParameter(shader, context.COMPILE_STATUS)) {
      console.warn("[ShaderSurface] shader compilation failed:", context.getShaderInfoLog(shader));
      context.deleteShader(shader);
      return null;
    }
    return shader;
  }

  function destroyWebgl() {
    if (!gl) return;
    if (buffer) gl.deleteBuffer(buffer);
    if (program) gl.deleteProgram(program);
    buffer = null;
    program = null;
    gl = null;
    webglReady = false;
  }

  function initWebgl() {
    if (!mounted || !canvas || webglReady || !hardwareAcceleration) return;
    const context = canvas.getContext("webgl", {
      alpha: true,
      antialias: false,
      depth: false,
      stencil: false,
      premultipliedAlpha: true,
      powerPreference: "low-power",
      preserveDrawingBuffer: false,
    }) as WebGLRenderingContext | null;
    if (!context) return;

    const vertex = compileShader(context, context.VERTEX_SHADER, vertexSource);
    const fragment = compileShader(context, context.FRAGMENT_SHADER, fragmentSource);
    if (!vertex || !fragment) {
      if (vertex) context.deleteShader(vertex);
      if (fragment) context.deleteShader(fragment);
      return;
    }

    const nextProgram = context.createProgram();
    if (!nextProgram) return;
    context.attachShader(nextProgram, vertex);
    context.attachShader(nextProgram, fragment);
    context.linkProgram(nextProgram);
    context.deleteShader(vertex);
    context.deleteShader(fragment);
    if (!context.getProgramParameter(nextProgram, context.LINK_STATUS)) {
      console.warn("[ShaderSurface] program linking failed:", context.getProgramInfoLog(nextProgram));
      context.deleteProgram(nextProgram);
      return;
    }

    const nextBuffer = context.createBuffer();
    if (!nextBuffer) {
      context.deleteProgram(nextProgram);
      return;
    }
    context.bindBuffer(context.ARRAY_BUFFER, nextBuffer);
    context.bufferData(
      context.ARRAY_BUFFER,
      new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
      context.STATIC_DRAW,
    );

    const position = context.getAttribLocation(nextProgram, "a_position");
    context.useProgram(nextProgram);
    context.enableVertexAttribArray(position);
    context.vertexAttribPointer(position, 2, context.FLOAT, false, 0, 0);
    context.enable(context.BLEND);
    context.blendFunc(context.ONE, context.ONE_MINUS_SRC_ALPHA);

    gl = context;
    program = nextProgram;
    buffer = nextBuffer;
    uResolution = context.getUniformLocation(nextProgram, "u_resolution");
    uTime = context.getUniformLocation(nextProgram, "u_time");
    uExpand = context.getUniformLocation(nextProgram, "u_expand");
    uEnergy = context.getUniformLocation(nextProgram, "u_energy");
    uTheme = context.getUniformLocation(nextProgram, "u_theme");
    uAccent = context.getUniformLocation(nextProgram, "u_accent");
    uSecondary = context.getUniformLocation(nextProgram, "u_secondary");
    webglReady = true;
    resizeCanvas();
    requestRender();
  }

  function parseColor(value: string, fallback: [number, number, number]): [number, number, number] {
    const text = value.trim();
    const shortHex = /^#([0-9a-f]{3})$/i.exec(text);
    if (shortHex) {
      const [r, g, b] = shortHex[1].split("").map((part) => parseInt(part + part, 16) / 255);
      return [r, g, b];
    }
    const hex = /^#([0-9a-f]{6})$/i.exec(text);
    if (hex) {
      const raw = hex[1];
      return [
        parseInt(raw.slice(0, 2), 16) / 255,
        parseInt(raw.slice(2, 4), 16) / 255,
        parseInt(raw.slice(4, 6), 16) / 255,
      ];
    }
    const rgb = /^rgba?\(\s*([\d.]+)\s*[, ]\s*([\d.]+)\s*[, ]\s*([\d.]+)/i.exec(text);
    if (rgb) {
      return [
        Math.min(255, Number(rgb[1])) / 255,
        Math.min(255, Number(rgb[2])) / 255,
        Math.min(255, Number(rgb[3])) / 255,
      ];
    }
    return fallback;
  }

  function spectrumEnergy() {
    if (previewSpectrum?.length) {
      return Math.min(1, Math.max(0, previewSpectrum.reduce((sum, value) => sum + Math.max(0, value), 0) / previewSpectrum.length));
    }
    if (!playing) return 0.12;
    return spectrumMode === "random" ? 0.44 : 0.36;
  }

  function resizeCanvas() {
    if (!canvas) return;
    const dpr = Math.min(1.75, Math.max(1, window.devicePixelRatio || 1));
    const nextWidth = Math.max(1, Math.round(width * dpr));
    const nextHeight = Math.max(1, Math.round(height * dpr));
    if (canvas.width !== nextWidth || canvas.height !== nextHeight) {
      canvas.width = nextWidth;
      canvas.height = nextHeight;
    }
    if (gl) gl.viewport(0, 0, canvas.width, canvas.height);
  }

  function requestRender() {
    if (!mounted || raf || !visible) return;
    raf = requestAnimationFrame(render);
  }

  function render(now: number) {
    raf = 0;
    if (!mounted || !visible) return;
    if (!hardwareAcceleration) {
      if (webglReady) destroyWebgl();
      return;
    }
    if (!webglReady) initWebgl();
    if (!gl || !program || !webglReady) return;

    const fps = Math.max(12, Math.min(60, frameRate || 30));
    const interval = 1000 / fps;
    if (lastFrame && now - lastFrame < interval) {
      requestRender();
      return;
    }
    lastFrame = now;
    resizeCanvas();

    const animated = enableAnimations && !reduceMotion;
    const animationTime = animated ? now / 1000 : 1.35;
    const speed = playing ? 1.0 : 0.42;
    const accent = parseColor(accentColor, [0.35, 0.49, 1]);
    const secondary = parseColor(secondaryColor, [0.18, 0.84, 0.78]);

    gl.useProgram(program);
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    if (uResolution) gl.uniform2f(uResolution, canvas.width, canvas.height);
    if (uTime) gl.uniform1f(uTime, animationTime * speed);
    if (uExpand) gl.uniform1f(uExpand, expandProgress);
    if (uEnergy) gl.uniform1f(uEnergy, spectrumEnergy());
    if (uTheme) gl.uniform1f(uTheme, theme === "album-reactive" ? 1 : 0);
    if (uAccent) gl.uniform3f(uAccent, accent[0], accent[1], accent[2]);
    if (uSecondary) gl.uniform3f(uSecondary, secondary[0], secondary[1], secondary[2]);
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 6);

    if (animated && visible) requestRender();
  }

  function handleContextLost(event: Event) {
    event.preventDefault();
    if (raf) cancelAnimationFrame(raf);
    raf = 0;
    gl = null;
    program = null;
    buffer = null;
    webglReady = false;
  }

  function handleContextRestored() {
    webglReady = false;
    initWebgl();
  }

  onMount(() => {
    mounted = true;
    canvas.addEventListener("webglcontextlost", handleContextLost);
    canvas.addEventListener("webglcontextrestored", handleContextRestored);
    if (hardwareAcceleration) initWebgl();
    requestRender();
    return () => {
      mounted = false;
      if (raf) cancelAnimationFrame(raf);
      raf = 0;
      canvas.removeEventListener("webglcontextlost", handleContextLost);
      canvas.removeEventListener("webglcontextrestored", handleContextRestored);
      destroyWebgl();
    };
  });

  $effect(() => {
    width;
    height;
    theme;
    playing;
    accentColor;
    secondaryColor;
    spectrumMode;
    previewSpectrum;
    expandProgress;
    frameRate;
    enableAnimations;
    reduceMotion;
    visible;
    if (!mounted) return;
    if (hardwareAcceleration && !webglReady) initWebgl();
    if (!hardwareAcceleration && webglReady) destroyWebgl();
    resizeCanvas();
    requestRender();
  });
</script>

<div
  class="shader-surface"
  class:visible
  class:album-reactive={theme === "album-reactive"}
  style={`--shader-accent:${accentColor};--shader-secondary:${secondaryColor}`}
  aria-hidden="true"
>
  <canvas bind:this={canvas} class:active={webglReady && hardwareAcceleration}></canvas>
  <div class="shader-fallback" class:active={!webglReady || !hardwareAcceleration}></div>
  <div class="shader-gloss"></div>
</div>

<style>
  .shader-surface{position:absolute;inset:0;z-index:0;overflow:hidden;pointer-events:none;opacity:0;transition:opacity 180ms cubic-bezier(.23,1,.32,1);contain:strict}.shader-surface.visible{opacity:1}.shader-surface canvas,.shader-fallback,.shader-gloss{position:absolute;inset:0;width:100%;height:100%;pointer-events:none}.shader-surface canvas{display:block;opacity:0;transition:opacity 140ms ease}.shader-surface canvas.active{opacity:1}.shader-fallback{opacity:0;background:radial-gradient(80% 130% at 12% 12%,color-mix(in srgb,var(--shader-accent) 62%,transparent),transparent 67%),radial-gradient(78% 125% at 88% 82%,color-mix(in srgb,var(--shader-secondary) 58%,transparent),transparent 66%),conic-gradient(from 210deg at 50% 50%,rgba(38,146,255,.20),rgba(127,49,255,.16),rgba(255,45,142,.18),rgba(255,133,35,.15),rgba(38,146,255,.20));filter:saturate(1.12);transform:scale(1.08);transition:opacity 140ms ease}.shader-fallback.active{opacity:.62}.album-reactive .shader-fallback{background:radial-gradient(82% 135% at 12% 8%,color-mix(in srgb,var(--shader-accent) 72%,transparent),transparent 68%),radial-gradient(86% 138% at 92% 88%,color-mix(in srgb,var(--shader-secondary) 68%,transparent),transparent 70%),linear-gradient(115deg,color-mix(in srgb,var(--shader-accent) 16%,transparent),transparent 42%,color-mix(in srgb,var(--shader-secondary) 13%,transparent))}.shader-gloss{background:linear-gradient(112deg,rgba(255,255,255,.08),rgba(255,255,255,0) 24%,rgba(255,255,255,.025) 51%,rgba(255,255,255,0) 72%);mix-blend-mode:screen;opacity:.52}
  @media (prefers-reduced-motion: reduce){.shader-surface,.shader-surface canvas,.shader-fallback{transition:none}}
</style>

<script lang="ts">
  import IslandSurface from "../src/lib/IslandSurface.svelte";
  import { CUSTOM_PANEL_WIDTH, hostFor } from "../src/lib/islandGeometry";
  import { DEMO_MEDIA } from "../src/lib/mediaStore";
  import type { IslandMode } from "../src/lib/islandGeometry";
  import type { CountdownStatus } from "../src/lib/countdown";

  const LONG_TITLE = "宇宙尽头的浪漫主义与一场不会结束的午夜公路旅行";
  const LONG_ARTIST = "The Extremely Long Artist Name · 特别长的专辑名称";
  let longTrack = $state(false);
  let mode = $state<IslandMode>("expanded");
  let timerStatus = $state<CountdownStatus>("idle");
  let timerDurationMs = $state(0);
  let timerRemainingMs = $state(0);
  let timerFinished = $state(false);
  let mediaPlaying = $state(DEMO_MEDIA.isPlaying);
  let title = $derived(longTrack ? LONG_TITLE : "Dogs");
  let artist = $derived(longTrack ? LONG_ARTIST : "Artist");
  let media = $derived({ ...DEMO_MEDIA, title, artist, isPlaying: mediaPlaying });
  const extraWidth = CUSTOM_PANEL_WIDTH;
  let host = $derived(hostFor("edge", "top", 80, extraWidth));

  function startTimer(durationMs: number) {
    timerDurationMs = durationMs;
    timerRemainingMs = durationMs;
    timerStatus = "running";
  }

  function pauseTimer() { timerStatus = "paused"; }
  function resumeTimer() { timerStatus = "running"; }
  function resetTimer() {
    timerStatus = "idle";
    timerDurationMs = 0;
    timerRemainingMs = 0;
  }
</script>

<main>
  <nav aria-label="Island UI test controls">
    <button data-testid="short-track" onclick={() => longTrack = false}>Short track</button>
    <button data-testid="long-track" onclick={() => longTrack = true}>Long track</button>
    <button data-testid="compact-mode" onclick={() => mode = "compact"}>Collapsed</button>
    <button data-testid="expanded-mode" onclick={() => mode = "expanded"}>Expanded</button>
    <button data-testid="timer-half" onclick={() => { timerStatus = "running"; timerDurationMs = 60_000; timerRemainingMs = 30_000; mode = "compact"; }}>Timer at 50%</button>
    <button data-testid="timer-long" onclick={() => { timerStatus = "paused"; timerDurationMs = 99 * 3_600_000 + 59 * 60_000; timerRemainingMs = timerDurationMs; mode = "expanded"; }}>Long timer</button>
    <button data-testid="timer-complete" onclick={() => { timerFinished = true; mode = "expanded"; }}>Timer complete</button>
  </nav>

  <section class="island-stage" aria-label="Rendered attached island">
    <div class="island-host" style={`width:${host.width}px;height:${host.height}px`}>
      <IslandSurface
        {media}
        {mode}
        islandStyle="edge"
        edge="top"
        expandedRadius={45}
        collapsedEdgeShoulderRadius={8}
        expandedEdgeShoulderRadius={32}
        compactLength={80}
        showCustomFunctionPanel
        enabledTools={["settings", "floating", "volume", "timer", "hide"]}
        enableAnimations={false}
        reduceAnimations
        interactive={false}
        onMediaAction={(action) => { if (action === "play_pause") mediaPlaying = !mediaPlaying; }}
        clockText="12:34"
        idleWeatherForecast={[
          { date: "2026-09-24", weatherCode: 1, temperatureMax: 25, temperatureMin: 18 },
          { date: "2026-09-25", weatherCode: 61, temperatureMax: 24, temperatureMin: 17 },
          { date: "2026-09-26", weatherCode: 2, temperatureMax: 23, temperatureMin: 16 },
          { date: "2026-09-27", weatherCode: 0, temperatureMax: 26, temperatureMin: 18 },
        ]}
        systemAudio={{ volumePercent: 42, muted: false, deviceId: "speakers", deviceName: "Speakers" }}
        audioDevices={[]}
        timerStatus={timerStatus}
        timerDurationMs={timerDurationMs}
        timerRemainingMs={timerRemainingMs}
        {timerFinished}
        onTimerStart={startTimer}
        onTimerPause={pauseTimer}
        onTimerResume={resumeTimer}
        onTimerReset={resetTimer}
        onTimerFinishedDismiss={() => timerFinished = false}
      />
    </div>
  </section>
</main>

<style>
  :global(*){box-sizing:border-box}
  :global(html),:global(body),:global(#app){min-width:100%;min-height:100%;margin:0}
  :global(body){background:#1d222a;color:#f4f6f8;font:14px/1.4 system-ui,"Segoe UI",sans-serif}
  main{display:grid;justify-items:center;gap:20px;padding:28px}
  nav{display:flex;flex-wrap:wrap;gap:8px}
  nav button{height:34px;padding:0 12px;border:1px solid #505762;border-radius:8px;background:#2b313a;color:#f4f6f8;font:inherit;cursor:pointer}
  nav button:hover{background:#353c47}
  .island-stage{position:relative;width:960px;height:236px;overflow:visible;border:1px solid #353b44;border-radius:14px;background:linear-gradient(135deg,#29323e,#171b21)}
  .island-host{position:absolute;top:0;left:50%;transform:translateX(-50%)}
</style>

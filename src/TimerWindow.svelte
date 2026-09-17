<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { emit } from "@tauri-apps/api/event";
  import { Pause, Play, Plus, RotateCcw, X } from "lucide-svelte";
  import { eventManager } from "./utils/eventManager";
  import { Events } from "./utils/eventConstants";
  import { formatCountdown, type CountdownStatus } from "$lib/countdown";
  import { locale, translate, type TranslationKey } from "$lib/i18n";

  type TimerSnapshot = {
    status: CountdownStatus;
    durationMs: number;
    remainingMs: number;
    label: string;
  };

  const t = (key: TranslationKey, values: Record<string, string | number> = {}) => translate(key, values, $locale);
  const segmentCount = 5;
  let snapshot = $state<TimerSnapshot>({ status: "idle", durationMs: 0, remainingMs: 0, label: "倒计时" });
  let syncedAt = $state(Date.now());
  let now = $state(Date.now());
  let dispose: (() => void) | undefined;

  let remainingMs = $derived(snapshot.status === "running"
    ? Math.max(0, snapshot.remainingMs - (now - syncedAt))
    : snapshot.remainingMs);
  let active = $derived((snapshot.status === "running" || snapshot.status === "paused") && remainingMs > 0);
  let progress = $derived(snapshot.durationMs > 0 ? Math.min(1, Math.max(0, 1 - remainingMs / snapshot.durationMs)) : 0);
  let statusLabel = $derived(snapshot.status === "running" ? "进行中" : snapshot.status === "paused" ? "已暂停" : "准备开始");

  function send(action: string, durationMs?: number) {
    void emit(Events.TIMER_ACTION, { action, ...(durationMs === undefined ? {} : { durationMs }) });
  }

  function startPreset(minutes: number) {
    send("start", minutes * 60_000);
  }

  onMount(() => {
    let disposed = false;
    void (async () => {
      const unlisten = await eventManager.on(Events.TIMER_STATE_CHANGED, (value: TimerSnapshot) => {
        if (!value) return;
        snapshot = { ...snapshot, ...value };
        syncedAt = Date.now();
      });
      if (disposed) {
        unlisten();
        return;
      }
      dispose = unlisten;
      await emit(Events.TIMER_REQUEST_STATE);
    })();
    const interval = window.setInterval(() => now = Date.now(), 250);
    return () => {
      disposed = true;
      window.clearInterval(interval);
      dispose?.();
    };
  });

  function close() {
    void getCurrentWindow().close();
  }

  function startDragging(event: MouseEvent) {
    if ((event.target as HTMLElement).closest("button")) return;
    void getCurrentWindow().startDragging();
  }
</script>

<svelte:head><title>{t("timer")}</title></svelte:head>

<main class="timer-window">
  <div class="window-bar" role="toolbar" tabindex="-1" aria-label={t("timer")} onmousedown={startDragging}>
    <span class="window-label">{snapshot.label}</span>
    <button type="button" class="close-button" aria-label={t("close")} onclick={close}><X size={15} /></button>
  </div>

  <section class="timer-card" aria-label={t("timer")}>
    <div class="timer-heading">
      <span class="eyebrow">{active ? t("remainingTime") : t("startTimer")}</span>
      <span class="status-chip" class:paused={snapshot.status === "paused"}>{statusLabel}</span>
    </div>
    <strong class="readout">{formatCountdown(remainingMs)}</strong>
    <p class="timer-copy">{active ? "保持专注，时间结束会自动收起。" : "选择一个时长，开始一段专注时间。"}</p>

    <div class="segments" aria-hidden="true">
      {#each Array(segmentCount) as _, index}
        <span class:filled={progress >= (index + 1) / segmentCount} class:current={progress > index / segmentCount && progress < (index + 1) / segmentCount}></span>
      {/each}
    </div>

    {#if active}
      <div class="active-actions">
        <button type="button" class="action-button" onclick={() => send("adjust", 60_000)}><Plus size={15} />1 分钟</button>
        <button type="button" class="action-button" onclick={() => send("adjust", 300_000)}><Plus size={15} />5 分钟</button>
        <button type="button" class="primary-action" onclick={() => send(snapshot.status === "running" ? "pause" : "resume")}>
          {#if snapshot.status === "running"}<Pause size={16} />{t("pauseTimer")}{:else}<Play size={16} />{t("resumeTimer")}{/if}
        </button>
        <button type="button" class="icon-action" aria-label={t("cancelTimer")} onclick={() => send("reset")}><RotateCcw size={16} /></button>
      </div>
    {:else}
      <div class="preset-row">
        {#each [5, 10, 25, 60] as minutes}
          <button type="button" onclick={() => startPreset(minutes)}>{minutes}<small>min</small></button>
        {/each}
      </div>
    {/if}
  </section>
</main>

<style>
  :global(html),:global(body),:global(#app){width:100%;height:100%;margin:0;background:transparent;overflow:hidden}
  :global(body){font-family:var(--app-font,system-ui,sans-serif)}
  .timer-window{width:100%;height:100%;box-sizing:border-box;padding:10px;color:#fff;background:transparent}
  .window-bar{height:28px;display:flex;align-items:center;justify-content:space-between;padding:0 5px;color:rgba(255,255,255,.56);font-size:11px;cursor:grab;user-select:none}.window-bar:active{cursor:grabbing}.window-label{letter-spacing:.04em}.close-button{display:grid;place-items:center;width:26px;height:26px;border:0;border-radius:50%;color:rgba(255,255,255,.58);background:transparent;cursor:pointer;transition:background 140ms ease,color 140ms ease,transform 140ms ease}.close-button:hover{color:#fff;background:rgba(255,255,255,.1)}.close-button:active{transform:scale(.92)}
  .timer-card{min-height:calc(100% - 28px);box-sizing:border-box;padding:24px 24px 22px;border:1px solid rgba(255,255,255,.08);border-radius:28px;background:#111;box-shadow:0 18px 44px rgba(0,0,0,.28);font-variant-numeric:tabular-nums}.timer-heading{display:flex;align-items:center;justify-content:space-between;gap:10px}.eyebrow{color:rgba(255,255,255,.54);font-size:12px;letter-spacing:.06em}.status-chip{padding:7px 10px;border-radius:999px;color:#fff;background:#147ef5;font-size:10px;font-weight:700}.status-chip.paused{background:#3c5875}.readout{display:block;margin-top:25px;font-size:64px;line-height:.95;letter-spacing:-.07em;font-weight:700}.timer-copy{margin:12px 0 28px;color:rgba(255,255,255,.54);font-size:13px}.segments{display:grid;grid-template-columns:repeat(5,1fr);gap:5px;height:48px;padding:4px;border-radius:14px;background:#0c1d2e}.segments span{position:relative;border-radius:10px;background:#183b60;overflow:hidden}.segments span.current{background:repeating-linear-gradient(135deg,rgba(40,146,255,.9) 0 3px,rgba(40,146,255,.28) 3px 6px)}.segments span.filled{background:#1d78cb}.active-actions{display:grid;grid-template-columns:auto auto 1fr auto;gap:7px;margin-top:22px}.action-button,.primary-action,.icon-action,.preset-row button{display:inline-flex;align-items:center;justify-content:center;gap:5px;border:1px solid rgba(255,255,255,.1);border-radius:12px;color:rgba(255,255,255,.82);background:rgba(255,255,255,.06);font:inherit;font-size:11px;cursor:pointer;transition:background 140ms ease,color 140ms ease,transform 140ms ease}.action-button{padding:9px 10px}.primary-action{padding:9px 12px;color:#fff;background:#147ef5;border-color:transparent}.icon-action{width:34px}.action-button:hover,.icon-action:hover{color:#fff;background:rgba(255,255,255,.13)}.primary-action:hover{background:#278bff}.action-button:active,.primary-action:active,.icon-action:active,.preset-row button:active{transform:scale(.96)}.preset-row{display:grid;grid-template-columns:repeat(4,1fr);gap:8px;margin-top:24px}.preset-row button{flex-direction:column;gap:2px;min-height:56px;background:#173b61;font-size:17px;font-weight:700}.preset-row button:hover{color:#fff;background:#1e5989}.preset-row small{color:rgba(255,255,255,.55);font-size:9px;font-weight:500}
  button:focus-visible{outline:2px solid #fff;outline-offset:2px}@media(max-width:420px){.readout{font-size:52px}.active-actions{grid-template-columns:1fr 1fr}.primary-action{grid-column:1 / -1}.icon-action{width:auto;padding:9px}}
  @media(prefers-reduced-motion:reduce){.close-button,.action-button,.primary-action,.icon-action,.preset-row button{transition:none}}
</style>

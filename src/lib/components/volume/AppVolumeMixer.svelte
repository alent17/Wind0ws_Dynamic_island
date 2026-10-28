<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { ArrowLeft, ChevronLeft, ChevronRight, RefreshCw, Volume2, VolumeX } from "lucide-svelte";
  import { audioApi } from "$lib/api/audio";
  import type { AppAudioSession } from "$lib/api/types";
  import { locale, translate, type TranslationKey } from "$lib/i18n";
  import { LatestValueWriteQueue } from "$lib/latestValueWriteQueue";

  let { onBack, onMetrics } = $props<{ onBack: () => void; onMetrics?: (rows:number, total:number) => void }>();
  let sessions = $state<AppAudioSession[]>([]);
  const pageSize = 5;
  let page = $state(0);
  const pageCount = $derived(Math.max(1, Math.ceil(sessions.length / pageSize)));
  const visibleSessions = $derived(sessions.slice(page * pageSize, (page + 1) * pageSize));
  let loading = $state(true), failed = $state(false);
  let failedIcons = $state<string[]>([]);
  let disposed = false, loadingNow = false;
  type Write = {session:AppAudioSession;value:number;muted:boolean;volumeRevision:number};
  type Writer = {queue:LatestValueWriteQueue<Write>;volumeRevision:number;writtenVolumeRevision:number};
  const queues = new Map<string, Writer>();
  const key = (session: AppAudioSession) => `${session.deviceId}:${session.processId}:${session.systemSounds}`;
  const t = (text: TranslationKey) => translate(text, {}, $locale);
  $effect(() => {
    if (loading) return;
    const rows = visibleSessions.length, total = sessions.length;
    untrack(() => onMetrics?.(rows, total));
  });
  async function refresh() {
    if (loadingNow) return;
    loadingNow = true;
    try {
      const next = await audioApi.listAppSessions();
      if (disposed) return;
      sessions = next.map(item => {
        const writer = queues.get(key(item));
        if (writer?.queue.busy) return sessions.find(old => key(old) === key(item)) ?? item;
        writer?.queue.invalidate();
        return item;
      });
      page = Math.min(page, Math.max(0, Math.ceil(sessions.length / pageSize) - 1));
      failed = false;
    } catch { if (!disposed) failed = true; }
    finally { loadingNow = false; if (!disposed) loading = false; }
  }
  function updateAudio(session: AppAudioSession, value: number, muted: boolean, volumeChanged = false) {
    const id = key(session);
    sessions = sessions.map(item => key(item) === id ? {...item, volumePercent:value, muted} : item);
    let writer = queues.get(id);
    if (!writer) {
      const next = {volumeRevision:0,writtenVolumeRevision:0} as Writer;
      next.queue = new LatestValueWriteQueue(async ({session,value,muted,volumeRevision}) => {
        const updateVolume = volumeRevision > next.writtenVolumeRevision;
        await audioApi.setAppVolume(session, updateVolume ? value : null, muted);
        if (updateVolume) next.writtenVolumeRevision = volumeRevision;
      }, () => { if (!disposed) failed = true; });
      writer = next;
      queues.set(id, next);
    }
    if (volumeChanged) writer.volumeRevision++;
    void writer.queue.submit({session,value,muted,volumeRevision:writer.volumeRevision}, `${writer.volumeRevision}:${value}:${muted}`);
  }
  onMount(() => {
    void refresh();
    const timer = setInterval(() => void refresh(), 3000);
    return () => { disposed = true; clearInterval(timer); for (const writer of queues.values()) writer.queue.cancelPending(); };
  });
</script>

<section class="app-mixer" aria-label={t("appVolumeTitle")}>
  <header>
    <button type="button" class="icon-button" aria-label={t("back")} onclick={onBack}><ArrowLeft size={24}/></button>
    <div><span>{t("volume")}</span><strong>{t("appVolumeTitle")}</strong></div>
    <button type="button" class="icon-button refresh" aria-label={t("appVolumeRefresh")} onclick={() => void refresh()}><RefreshCw size={21}/></button>
  </header>
  {#if failed}<p class="message error" role="status">{t("appVolumeFailed")}</p>{/if}
  <div class="app-list" onwheel={event => event.stopPropagation()}>
    {#if loading}<p class="message">{t("appVolumeLoading")}</p>
    {:else if sessions.length === 0 && !failed}<p class="message">{t("appVolumeEmpty")}</p>
    {:else}
      {#each visibleSessions as session (key(session))}
        {@const name = session.systemSounds ? t("appVolumeSystem") : session.name}
        {@const value = session.muted ? 0 : session.volumePercent}
        <div class="app-row" data-stop-toggle title={name}>
          <button type="button" class="app-icon" class:muted={session.muted} aria-label={translate(session.muted ? "appUnmute" : "appMute", {name}, $locale)} aria-pressed={session.muted} title={translate(session.muted ? "appUnmute" : "appMute", {name}, $locale)} onclick={() => updateAudio(session, session.volumePercent, !session.muted)}>{#if session.icon && !failedIcons.includes(key(session))}<img src={session.icon} alt="" onerror={() => failedIcons = [...failedIcons, key(session)]} />{:else}<Volume2 size={24}/>{/if}{#if session.muted}<span class="mute-badge" aria-hidden="true"><VolumeX size={13}/></span>{/if}</button>
          <div class="app-ruler">
            <span class="fill" style:width={`${value}%`}></span>
            {#each Array.from({length:21}, (_, index) => index * 5) as mark}<i class:passed={mark <= value} style:left={`${mark}%`}></i>{/each}
            <input type="range" min="0" max="100" step="1" aria-label={`${name} · ${t("volume")}`} value={value} oninput={event => updateAudio(session, Number(event.currentTarget.value), false, true)}/>
          </div>
          <span class="app-value">{value}</span>
        </div>
      {/each}
    {/if}
  </div>
  {#if pageCount > 1}
    <nav class="pagination" data-stop-toggle aria-label={t("appVolumePages")}>
      <button type="button" aria-label={t("appVolumePreviousPage")} disabled={page === 0} onclick={() => page = Math.max(0, page - 1)}><ChevronLeft size={22}/></button>
      <span aria-live="polite">{page + 1} / {pageCount}</span>
      <button type="button" aria-label={t("appVolumeNextPage")} disabled={page >= pageCount - 1} onclick={() => page = Math.min(pageCount - 1, page + 1)}><ChevronRight size={22}/></button>
    </nav>
  {/if}
</section>

<style>
  .app-mixer{display:flex;flex-direction:column;box-sizing:border-box;width:100%;height:100%;padding:calc(30px + var(--extension-block-inset,0px)) calc(24px + var(--extension-inline-inset,0px)) calc(24px + var(--extension-block-inset,0px));color:#fff;font-family:var(--app-font,system-ui)}
  header{display:flex;align-items:center;gap:14px;flex:none;margin-bottom:18px}header>div{display:flex;flex:1;min-width:0;flex-direction:column;gap:3px}header span{font-size:14px;color:#ffffff80}header strong{font-size:24px;line-height:1.1}.icon-button{display:grid;place-items:center;width:42px;height:42px;border:0;border-radius:12px;background:transparent;color:#fff;cursor:pointer}.icon-button:hover,.icon-button:focus-visible{background:#ffffff1a;outline:none}.refresh{color:#ffffffb8}
  .app-list{flex:1;min-height:0;overflow-y:auto;overscroll-behavior:contain;scrollbar-width:thin;scrollbar-color:#ffffff40 transparent}.app-list::-webkit-scrollbar{width:5px}.app-list::-webkit-scrollbar-thumb{background:#ffffff40;border-radius:8px}
  .app-row{display:flex;align-items:center;gap:14px;padding:14px 4px;border-bottom:1px solid #ffffff1a}.app-icon{position:relative;display:grid;place-items:center;flex:none;width:40px;height:40px;padding:2px;border:0;border-radius:10px;background:transparent;color:#ffffffa6;cursor:pointer}.app-icon:hover,.app-icon:focus-visible{outline:none;background:#ffffff1a}.app-icon.muted img{opacity:.48}.app-icon img{display:block;width:36px;height:36px;object-fit:contain}.mute-badge{position:absolute;right:-3px;bottom:-3px;display:grid;place-items:center;width:18px;height:18px;border-radius:50%;background:#ff453a;color:#fff}.app-value{width:48px;flex:none;text-align:right;font-size:20px;font-weight:700;font-variant-numeric:tabular-nums}
  .app-ruler{height:34px;position:relative;flex:1;min-width:0;overflow:hidden}.fill{position:absolute;left:0;top:2px;bottom:2px;background:#193a8e;border-radius:2px}.app-ruler i{position:absolute;top:2px;bottom:2px;width:3px;background:#fff;transform:translateX(-50%);border-radius:1px}.app-ruler i.passed{opacity:0}.app-ruler input{position:absolute;inset:0;width:100%;height:100%;margin:0;opacity:0;cursor:pointer}.app-ruler:focus-within{outline:2px solid #91caff;outline-offset:3px;border-radius:4px}
  .message{font-size:16px;color:#ffffff99}.error{color:#ffb4ab;margin:0 0 8px}
  .pagination{display:flex;flex:none;align-items:center;justify-content:center;gap:12px;padding-top:8px;color:#ffffff99;font-size:15px;font-variant-numeric:tabular-nums}.pagination button{display:grid;place-items:center;width:36px;height:36px;padding:0;border:0;border-radius:10px;color:#fff;background:transparent;cursor:pointer}.pagination button:hover:not(:disabled),.pagination button:focus-visible{outline:none;background:#ffffff1a}.pagination button:disabled{opacity:.3;cursor:default}.pagination span{min-width:42px;text-align:center}
</style>

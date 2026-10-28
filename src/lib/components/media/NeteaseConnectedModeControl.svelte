<script lang="ts">
  import { onMount } from "svelte";
  import { listen, emit } from "@tauri-apps/api/event";
  import { mediaApi, type NeteasePlaybackMode } from "$lib/api/media";
  import type { MediaState } from "$lib/api/types";
  import { locale, translate } from "$lib/i18n";
  import NeteaseModeControl from "./NeteaseModeControl.svelte";
  import { neteaseModeState } from "$lib/neteaseModeState";

  let { media, compact = false, live = true } = $props<{ media: MediaState; compact?: boolean; live?: boolean }>();
  const entry = $derived(neteaseModeState(`${media.nativeEpoch}:${media.sourceDisplay}`));
  const mode = $derived($entry ?? (!live ? media.neteasePlaybackMode : undefined));
  const status = $derived(entry.status);
  let busy = $state(false);
  let error = $state("");
  let disposed = false;
  let revision = 0;
  const basis = () => `${media.nativeEpoch}:${media.nativeTrackGeneration}:${media.sourceDisplay}:${media.neteaseFeatures?.mode.source}`;
  onMount(() => {
    let stop: (() => void) | undefined;
    if (live) {
      void listen<{ session: string; epoch: string | undefined; mode: NeteasePlaybackMode }>("netease-mode-changed", event => {
        if (event.payload.session === media.sourceDisplay && event.payload.epoch === media.nativeEpoch) { revision++; entry.accept(event.payload.mode); }
      }).then(unlisten => { if (disposed) unlisten(); else stop = unlisten; }).catch(() => {});
    }
    return () => { disposed = true; revision++; stop?.(); };
  });
  // Follow session changes as well as mount/unmount. Confirmed modes remain
  // cached across tracks; an unknown mode gets a fresh chance on a new track.
  $effect(() => { if (live) return entry.activate(); });
  $effect(() => { if (live) { void media.nativeTrackGeneration; void entry.load(); } });
  async function cycle() {
    if (!live || busy || !mode) return;
    const started = basis();
    revision++;
    busy = true;
    error = "";
    try {
      const observed = await mediaApi.cycleNeteasePlaybackMode(media);
      if (disposed || started !== basis()) return;
      entry.accept(observed);
      void emit("netease-mode-changed", { session: media.sourceDisplay, epoch: media.nativeEpoch, mode: observed }).catch(() => {});
    } catch {
      if (!disposed && started === basis()) error = translate("neteaseModeFailed", {}, $locale);
    } finally { if (!disposed) busy = false; }
  }
</script>

{#if mode}
  <NeteaseModeControl {mode} {compact} disabled={!live || busy} {error} onCycle={cycle}/>
{:else}
  <button type="button" class="unknown-mode" disabled={!live || $status.loading} aria-busy={$status.loading}
    title={translate($status.loading ? "neteaseModeLoading" : "neteaseModeRetry", {}, $locale)}
    aria-label={translate($status.loading ? "neteaseModeLoading" : "neteaseModeRetry", {}, $locale)}
    onclick={(event) => { event.stopPropagation(); if(live)void entry.load(true); }}>—</button>
{/if}

<style>.unknown-mode{display:inline-grid;place-items:center;width:48px;height:48px;padding:0;border:0;border-radius:9px;background:transparent;color:inherit;opacity:.6;cursor:pointer}.unknown-mode:disabled{opacity:.4;cursor:default}.unknown-mode:focus-visible{outline:2px solid currentColor;outline-offset:2px}</style>

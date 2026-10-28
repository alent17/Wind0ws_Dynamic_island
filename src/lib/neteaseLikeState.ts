import { writable } from "svelte/store";
import { emit } from "@tauri-apps/api/event";
import { mediaApi, type NeteaseLikeState } from "./api/media";

// Owned by the application runtime, rather than by an expanded panel instance.
// Track generations distinguish a new play instance from a reopened panel.
const entries = new Map<string, ReturnType<typeof createEntry>>();
function createEntry(title: string, artist: string) {
  let value: NeteaseLikeState | null = null;
  let pending: Promise<void> | null = null;
  let retryAt = 0;
  let revision = 0;
  let busy = false;
  let attempts = 0;
  let retryTimer: ReturnType<typeof setTimeout> | undefined;
  const store = writable<{ value: NeteaseLikeState | null; busy: boolean; failed: boolean; loading: boolean; error: string }>({ value, busy: false, failed: false, loading: false, error: "" });
  const publish = (failed = false, error = "", loading = false) => store.set({ value, busy, failed, loading, error });
  const errorMessage = (error: unknown) => typeof error === "string" ? error : error && typeof error === "object" && "message" in error ? String(error.message) : "";
  const entry = {
    subscribe: store.subscribe,
    suspend() {
      revision++;
      clearTimeout(retryTimer);
      retryTimer = undefined;
      value = null;
      pending = null;
      busy = false;
      attempts = 0;
      retryAt = 0;
      publish();
    },
    acceptConfirmed(result: NeteaseLikeState) {
      if (!value || value.trackId !== result.trackId || pending || busy) return;
      value = { ...value, liked: result.liked };
      publish();
    },
    load() {
      if (value || pending || busy || Date.now() < retryAt) return pending ?? Promise.resolve();
      const started = revision;
      publish(false, "", true);
      pending = mediaApi.neteaseSongLike(title, artist).then(result => {
        if (started !== revision) return;
        value = result;
        attempts = 0;
        publish();
      }).catch((error: unknown) => {
        if (started !== revision) return;
        publish(true, errorMessage(error));
        // Only retry an unsuccessful initial read, at most twice. Reopening
        // the panel neither starts another read nor resets this retry budget.
        attempts++;
        if (attempts < 3) {
          const delay = attempts === 1 ? 500 : 1500;
          retryAt = Date.now() + delay;
          retryTimer = setTimeout(() => { retryTimer = undefined; retryAt = 0; void entry.load(); }, delay);
        } else {
          retryAt = Number.POSITIVE_INFINITY;
        }
      }).finally(() => { if (started === revision) pending = null; });
      return pending;
    },
    async toggle() {
      if (busy) return;
      clearTimeout(retryTimer);
      retryTimer = undefined;
      const started = revision;
      busy = true;
      publish();
      let failed = false;
      let failure = "";
      try {
        // A cached playId may expire even when SMTC still reports the same
        // title. Refresh only on an explicit click, never on panel reopening.
        await pending;
        if (started !== revision) return;
        const current = await mediaApi.neteaseSongLike(title, artist);
        if (started !== revision) return;
        value = current;
        const result = await mediaApi.neteaseSongLike(title, artist, current, !current.liked);
        if (started !== revision) return;
        value = result;
        acceptNeteaseLikeState(result);
        if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
          void emit("netease-like-changed", result).catch(() => {});
        }
      } catch (error: unknown) {
        failed = true;
        failure = errorMessage(error);
      } finally {
        if (started === revision) { busy = false; publish(failed, failure); }
      }
    },
  };
  return entry;
}

export function acceptNeteaseLikeState(result: NeteaseLikeState) {
  for (const entry of entries.values()) entry.acceptConfirmed(result);
}

export function neteaseLikeState(key: string, title: string, artist: string) {
  const identity = JSON.stringify([key, title, artist]);
  let entry = entries.get(identity);
  if (!entry) {
    entry = createEntry(title, artist);
    entries.set(identity, entry);
    if (entries.size > 32) entries.delete(entries.keys().next().value!);
  }
  return entry;
}

import { writable } from "svelte/store";
import { mediaApi, type NeteasePlaybackMode } from "./api/media";

const entries = new Map<string, ReturnType<typeof createEntry>>();
function createEntry() {
  let value: NeteasePlaybackMode | undefined;
  let pending: Promise<void> | undefined;
  let revision = 0;
  let users = 0;
  let attempts = 0;
  let retry: ReturnType<typeof setTimeout> | undefined;
  const delays = [500, 1500, 3000];
  const store = writable<NeteasePlaybackMode | undefined>(undefined);
  const status = writable({ loading: false, failed: false });
  function cancelRetry() { if (retry !== undefined) clearTimeout(retry); retry = undefined; }
  const entry = {
    subscribe: store.subscribe,
    status,
    get inUse() { return users > 0; },
    accept(mode: NeteasePlaybackMode) {
      revision++; cancelRetry(); pending = undefined; attempts = 0;
      value = mode; store.set(mode); status.set({ loading: false, failed: false });
    },
    clear() {
      revision++; cancelRetry(); value = undefined; pending = undefined; attempts = 0;
      store.set(undefined); status.set({ loading: false, failed: false });
    },
    activate() {
      users++; void entry.load();
      return () => { if (--users === 0) { cancelRetry(); attempts = 0; status.set({ loading: !!pending, failed: false }); } };
    },
    load(force = false): Promise<void> {
      if (value || pending) return pending ?? Promise.resolve();
      if (force) { cancelRetry(); attempts = 0; }
      if (retry !== undefined) return Promise.resolve();
      const started = revision;
      const attempt = attempts++;
      status.set({ loading: true, failed: false });
      pending = mediaApi.getNeteasePlaybackMode().then(mode => {
        if (started === revision) { value = mode ?? undefined; store.set(value); }
      }).catch(() => {}).finally(() => {
        if (started !== revision) return;
        pending = undefined;
        const delay = delays[attempt];
        if (!value && users > 0 && delay !== undefined) {
          retry = setTimeout(() => { retry = undefined; void entry.load(); }, delay);
          status.set({ loading: true, failed: false });
        } else {
          attempts = 0;
          status.set({ loading: false, failed: !value });
        }
      });
      return pending;
    },
  };
  return entry;
}
export function neteaseModeState(key: string) {
  let entry = entries.get(key);
  if (!entry) {
    entry = createEntry(); entries.set(key, entry);
    if (entries.size > 16) {
      const oldest = [...entries].find(([savedKey, saved]) => savedKey !== key && !saved.inUse);
      if (oldest) { oldest[1].clear(); entries.delete(oldest[0]); }
    }
  }
  return entry;
}
export function clearNeteaseModes() { for (const entry of entries.values()) entry.clear(); }

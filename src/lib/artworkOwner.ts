import { invoke } from "@tauri-apps/api/core";
import { artworkCacheKey } from "./artworkBudget";

type Slot = "cover" | "color" | "pixel";
let lastRevision = 0;
function revision() { return lastRevision = Math.max(Date.now() * 1000, lastRevision + 1); }
const aborted = () => new DOMException("Artwork superseded", "AbortError");

/** One current consumer per window/operation; backend sharing spans windows. */
export function createArtworkOwner(slot: Slot) {
  let current: { revision: number; cancelled: boolean } | undefined;
  function cancel() {
    if (!current) return;
    const previous = current; current = undefined; previous.cancelled = true;
    void invoke("cancel_artwork", { slot, revision: previous.revision }).catch(() => undefined);
  }
  return {
    cancel,
    async request<T>(args: Record<string, unknown>, versionSource?: string, signal?: AbortSignal): Promise<T> {
      cancel();
      const owner = { revision: revision(), cancelled: false }; current = owner;
      const stop = () => { if (current === owner) cancel(); };
      signal?.addEventListener("abort", stop, { once: true });
      try {
        if (signal?.aborted) { stop(); throw aborted(); }
        const coverVersion = versionSource === undefined ? undefined : await artworkCacheKey(versionSource, false);
        if (owner.cancelled) throw aborted();
        const result = await invoke<T>("request_artwork", { ...args, slot, revision: owner.revision, ...(coverVersion ? { coverVersion } : {}) });
        if (owner.cancelled) throw aborted();
        return result;
      } finally {
        signal?.removeEventListener("abort", stop);
        if (current === owner) current = undefined;
      }
    },
  };
}

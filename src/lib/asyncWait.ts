import { createAsyncCleanup } from "./asyncCleanup";

/** Disposal settles waits as false, so callers can exit without a dangling promise. */
export function waitForOwner(owner: ReturnType<typeof createAsyncCleanup>, milliseconds?: number): Promise<boolean> {
  if (owner.disposed) return Promise.resolve(false);
  return new Promise(resolve => {
    const frame = milliseconds === undefined;
    const id = frame ? requestAnimationFrame(() => finish(true)) : setTimeout(() => finish(true), milliseconds);
    const remove = owner.add(() => {
      if (frame) cancelAnimationFrame(id as number); else clearTimeout(id);
      finish(false);
    });
    function finish(ready: boolean) { remove(); resolve(ready && !owner.disposed); }
  });
}

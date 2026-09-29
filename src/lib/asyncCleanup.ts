/** Owns cleanup callbacks, including callbacks returned after disposal. */
export function createAsyncCleanup() {
  let disposed = false;
  const cleanups: Array<() => void> = [];

  return {
    get disposed() { return disposed; },
    add(cleanup: () => void) {
      if (disposed) cleanup();
      else cleanups.push(cleanup);
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      for (const cleanup of cleanups.splice(0)) cleanup();
    },
  };
}

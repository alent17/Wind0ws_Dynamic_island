/** Owns cleanup callbacks, including callbacks returned after disposal. */
export function createAsyncCleanup() {
  let disposed = false;
  const cleanups = new Set<() => void>();

  return {
    get disposed() { return disposed; },
    add(cleanup: () => void) {
      if (disposed) cleanup();
      else cleanups.add(cleanup);
      return () => cleanups.delete(cleanup);
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      const pending = [...cleanups];
      cleanups.clear();
      for (const cleanup of pending) cleanup();
    },
  };
}

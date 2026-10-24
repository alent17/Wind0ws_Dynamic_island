import { isTransportDiagnosticsEnabled, recordTransportCounter } from "./transportDiagnostics";

type Candidate<T> = { value: T; signature: string };

type IslandRegionSenderOptions<T> = {
  send: (value: T) => Promise<unknown> | unknown;
  onError?: (error: unknown) => void;
  retryBaseDelayMs?: number;
  retryMaxDelayMs?: number;
};

/** Serialize region IPC, retaining only the latest unsent geometry. */
export function createIslandRegionSender<T>({
  send,
  onError,
  retryBaseDelayMs = 250,
  retryMaxDelayMs = 2_000,
}: IslandRegionSenderOptions<T>) {
  let inFlight: Candidate<T> | undefined;
  let pending: Candidate<T> | undefined;
  let acknowledgedSignature: string | undefined;
  let retryTimer: ReturnType<typeof setTimeout> | undefined;
  let failureCount = 0;
  let reportedFailure = false;
  let disposed = false;

  function pump() {
    if (disposed || inFlight || retryTimer || !pending) return;
    const candidate = pending;
    pending = undefined;
    if (candidate.signature === acknowledgedSignature) {
      if (isTransportDiagnosticsEnabled()) recordTransportCounter("region", "deduped");
      return;
    }

    inFlight = candidate;
    let result: Promise<unknown> | unknown;
    try {
      result = send(candidate.value);
    } catch (error) {
      completeFailure(candidate, error);
      return;
    }
    void Promise.resolve(result).then(
      () => completeSuccess(candidate),
      (error: unknown) => completeFailure(candidate, error),
    );
  }

  function completeSuccess(candidate: Candidate<T>) {
    if (disposed || inFlight !== candidate) return;
    inFlight = undefined;
    acknowledgedSignature = candidate.signature;
    failureCount = 0;
    reportedFailure = false;
    // A same-signature settled/latest emission arrived while this exact shape
    // was in flight. Its state is now applied; a newer different shape stays queued.
    if (pending?.signature === candidate.signature) pending = undefined;
    pump();
  }

  function completeFailure(candidate: Candidate<T>, error: unknown) {
    if (disposed || inFlight !== candidate) return;
    inFlight = undefined;
    // A failed IPC may have failed before or after native application. Force a
    // retry of the latest state rather than trusting an earlier acknowledgement.
    acknowledgedSignature = undefined;
    if (!pending) pending = candidate;
    if (!reportedFailure) {
      reportedFailure = true;
      try { onError?.(error); } catch { /* diagnostics must not break retries */ }
    }

    const base = Math.max(1, retryBaseDelayMs);
    const max = Math.max(base, retryMaxDelayMs);
    const delay = Math.min(max, base * 2 ** failureCount);
    failureCount += 1;
    if (isTransportDiagnosticsEnabled()) recordTransportCounter("region", "retries");
    retryTimer = setTimeout(() => {
      retryTimer = undefined;
      pump();
    }, delay);
  }

  return {
    offer(value: T, signature: string) {
      if (disposed) return;
      if (isTransportDiagnosticsEnabled()) {
        recordTransportCounter("region", "offers");
        if (pending) recordTransportCounter("region", "replaced");
      }
      // Replacing this slot is intentional: only the latest unsent shape matters.
      pending = { value, signature };
      pump();
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      pending = undefined;
      if (retryTimer !== undefined) {
        clearTimeout(retryTimer);
        retryTimer = undefined;
      }
    },
  };
}

import { isTransportDiagnosticsEnabled, recordTransportCounter } from "./transportDiagnostics";

type PendingWrite<T> = { value: T };

/** Serializes writes and keeps only the newest value while one write is active. */
export class LatestValueWriteQueue<T> {
  private pending: PendingWrite<T> | null = null;
  private running: Promise<void> | null = null;
  private latestKey: string | null = null;

  constructor(
    private readonly write: (value: T) => Promise<void>,
    private readonly onError: (error: unknown, pendingWillRetry: boolean) => void,
  ) {}

  get busy(): boolean { return this.running !== null || this.pending !== null; }

  submit(value: T, key: string): Promise<void> {
    if (this.latestKey === key) {
      if (isTransportDiagnosticsEnabled()) recordTransportCounter("volume", "deduped");
      return this.running ?? Promise.resolve();
    }

    this.latestKey = key;
    if (isTransportDiagnosticsEnabled() && this.pending) recordTransportCounter("volume", "replaced");
    this.pending = { value };
    if (!this.running) {
      this.running = Promise.resolve().then(() => this.drain());
    }
    return this.running;
  }

  /** Call when the native value may have changed outside this queue. */
  invalidate(): void {
    this.latestKey = null;
  }

  cancelPending(): void {
    this.pending = null;
    this.latestKey = null;
  }

  private async drain(): Promise<void> {
    try {
      while (this.pending) {
        const next = this.pending;
        this.pending = null;
        try {
          await this.write(next.value);
        } catch (error) {
          // A failed write must be retryable even if it was superseded while active.
          // Keep draining the latest pending value; only clear dedupe when there is
          // no newer value to send, so the failed value can be explicitly retried.
          const pendingWillRetry = this.pending !== null;
          if (!pendingWillRetry) this.latestKey = null;
          try { this.onError(error, pendingWillRetry); } catch { /* error reporting must not stop the latest pending write */ }
        }
      }
    } finally {
      this.running = null;
    }
  }
}

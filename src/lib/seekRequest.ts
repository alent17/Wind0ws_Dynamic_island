/** Snapshot refreshes do not invalidate a user request; only explicit lifecycle changes do. */
export function createSeekRequestGate() {
  let generation = 0;
  return {
    begin() { return ++generation; },
    invalidate() { return ++generation; },
    isCurrent(requestId: number) { return requestId === generation; },
  };
}

/** Rollback is safe only if no newer native state arrived after this seek began. */
export function seekSnapshotAllowsRollback(requestRevision: number, currentRevision: number): boolean {
  return requestRevision === currentRevision;
}

export interface SeekHandlerOptions<Context, Result> {
  gate: ReturnType<typeof createSeekRequestGate>;
  createContext: (positionMs: number, requestId: number) => Context;
  isCurrentTrack: (context: Context) => boolean;
  isDisposed: () => boolean;
  request: (context: Context) => Promise<Result>;
  onBegin: (context: Context) => void;
  onConfirmed: (result: Result, context: Context) => void;
  onFailed: (error: unknown, context: Context) => void | Promise<void>;
  onError: (error: unknown, context: Context) => void;
}

/** Production request path shared by App and FloatingWindow. */
export function createSeekHandler<Context, Result>(options: SeekHandlerOptions<Context, Result>) {
  return async (positionMs: number): Promise<void> => {
    const requestId = options.gate.begin();
    const context = options.createContext(positionMs, requestId);
    options.onBegin(context);
    await executeSeekRequest(
      () => options.request(context),
      () => options.isCurrentTrack(context),
      () => options.gate.isCurrent(requestId) && !options.isDisposed(),
      (result) => options.onConfirmed(result, context),
      (error) => options.onFailed(error, context),
    ).catch((error) => {
      if (options.gate.isCurrent(requestId) && !options.isDisposed() && options.isCurrentTrack(context)) {
        options.onError(error, context);
      }
    });
  };
}

export async function executeSeekRequest<T>(
  request: () => Promise<T>,
  isCurrentTrack: () => boolean,
  isCurrentRequest: () => boolean,
  onConfirmed: (result: T) => void,
  onFailed: (error: unknown) => void | Promise<void>,
): Promise<void> {
  try {
    const result = await request();
    if (isCurrentTrack() && isCurrentRequest()) onConfirmed(result);
  } catch (error) {
    if (isCurrentTrack() && isCurrentRequest()) await onFailed(error);
    throw error;
  }
}

/** Keep command failures useful in a compact UI without exposing stacks or payloads. */
export function formatUserFacingError(error: unknown, fallback: string): string {
  let message = fallback;
  if (typeof error === "string") {
    message = error;
  } else if (error instanceof Error) {
    message = error.message;
  } else if (error && typeof error === "object") {
    const value = error as { code?: unknown; message?: unknown };
    const detail = typeof value.message === "string" ? value.message : "";
    const code = typeof value.code === "number" || typeof value.code === "string" ? String(value.code) : "";
    message = detail || (code ? `错误代码 ${code}` : message);
    if (detail && code) message = `[${code}] ${detail}`;
  }

  const firstLine = message.split(/[\r\n]/, 1)[0].trim();
  const safe = firstLine
    .replace(/https?:\/\/\S+/gi, "[链接]")
    .replace(/(?:[A-Za-z]:[\\/]|\\\\)[^\s"'<>]*/g, "[路径]")
    .slice(0, 180);
  return safe || fallback;
}

export function formatSeekError(error: unknown): string {
  return formatUserFacingError(error, "播放进度跳转失败");
}

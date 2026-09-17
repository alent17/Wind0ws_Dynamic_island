export type CaptureReason = "screenshot" | "recording" | "fullscreen" | "screenShare";

export interface CaptureSnapshot {
  screenshot: boolean;
  recording: boolean;
  fullscreen: boolean;
  screenShare: boolean;
}

export interface CapturePreferences {
  captureHideOnScreenshot: boolean;
  captureHideOnRecording: boolean;
  captureHideOnFullscreen: boolean;
  captureHideOnScreenShare: boolean;
}

export const EMPTY_CAPTURE_SNAPSHOT: CaptureSnapshot = {
  screenshot: false,
  recording: false,
  fullscreen: false,
  screenShare: false,
};

const preferenceFor: Record<CaptureReason, keyof CapturePreferences> = {
  screenshot: "captureHideOnScreenshot",
  recording: "captureHideOnRecording",
  fullscreen: "captureHideOnFullscreen",
  screenShare: "captureHideOnScreenShare",
};

export function activeCaptureReasons(
  snapshot: CaptureSnapshot,
  preferences: CapturePreferences,
): CaptureReason[] {
  return (Object.keys(preferenceFor) as CaptureReason[]).filter(
    (reason) => snapshot[reason] && preferences[preferenceFor[reason]],
  );
}

export function shouldHideForCapture(
  snapshot: CaptureSnapshot,
  preferences: CapturePreferences,
): boolean {
  return activeCaptureReasons(snapshot, preferences).length > 0;
}

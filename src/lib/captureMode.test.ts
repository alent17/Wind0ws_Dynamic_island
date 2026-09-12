import { describe, expect, it } from "vitest";
import {
  activeCaptureReasons,
  EMPTY_CAPTURE_SNAPSHOT,
  shouldHideForCapture,
  shouldProtectCapturedContent,
  type CapturePreferences,
} from "./captureMode";

const preferences: CapturePreferences = {
  captureHideOnScreenshot: true,
  captureHideOnRecording: false,
  captureHideOnFullscreen: true,
  captureHideOnScreenShare: false,
};

describe("capture mode", () => {
  it("only activates enabled capture reasons", () => {
    const snapshot = { ...EMPTY_CAPTURE_SNAPSHOT, screenshot: true, recording: true, fullscreen: true };
    expect(activeCaptureReasons(snapshot, preferences)).toEqual(["screenshot", "fullscreen"]);
    expect(shouldHideForCapture(snapshot, preferences)).toBe(true);
  });

  it("stays visible when only disabled reasons are active", () => {
    const snapshot = { ...EMPTY_CAPTURE_SNAPSHOT, recording: true, screenShare: true };
    expect(shouldHideForCapture(snapshot, preferences)).toBe(false);
  });

  it("protects captured output independently from fullscreen hiding", () => {
    expect(shouldProtectCapturedContent(preferences)).toBe(true);
    expect(shouldProtectCapturedContent({ ...preferences, captureHideOnScreenshot: false })).toBe(false);
  });
});

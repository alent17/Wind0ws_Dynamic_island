import { mount } from "svelte";
import SeekMainIslandFixture from "./SeekMainIslandFixture.svelte";

declare global {
  interface Window {
    __seekMediaInvokes: Array<{ command: "seek_media"; positionMs: number }>;
    __setSeekError: (message: string) => void;
    __pushNativeSnapshot: (positionMs: number) => void;
    __setDisplayedPosition: (positionMs: number) => void;
    __resolveSeek: (result: { trackId: string; positionMs: number; durationMs: number }) => void;
    __rejectSeek: (error: unknown) => void;
    __seekState: () => { positionMs: number; error: string; nativeRevision: number };
  }
}

window.__seekMediaInvokes = [];
mount(SeekMainIslandFixture, { target: document.getElementById("app")! });

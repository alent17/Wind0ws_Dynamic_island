import { describe, expect, it } from "vitest";
import { playerNameFromId } from "./playerNames";

describe("saved player names", () => {
  it("shows a useful name for offline packaged apps and browser profiles", () => {
    expect(playerNameFromId("AppleInc.AppleMusicWin_nzyj5cx40ttqa!App")).toBe("Apple Music");
    expect(playerNameFromId("Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic")).toBe("Windows 媒体播放器");
    expect(playerNameFromId("Chrome.UserData.Profile2")).toBe("Chrome · 个人资料 2");
    expect(playerNameFromId("OpenAI.Codex_2p2nqsd0c76g0!App")).toBe("OpenAI Codex");
  });
});

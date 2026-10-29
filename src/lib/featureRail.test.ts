import { describe, expect, it } from "vitest";
import { DEFAULT_SETTINGS } from "$lib/api/types";
import { BUILTIN_TOOLS, enabledBuiltinTools } from "./featureRail";

describe("built-in feature rail", () => {
  it("has unique IDs and follows visibility settings without hiding built-in pages", () => {
    expect(new Set(BUILTIN_TOOLS.map(tool => tool.id)).size).toBe(BUILTIN_TOOLS.length);
    const enabled = enabledBuiltinTools({ ...DEFAULT_SETTINGS, showTimerTool: false, showWeatherTool: false });
    expect(enabled).not.toContain("timer");
    expect(enabled).not.toContain("weather");
    expect(enabled).toContain("files");
    expect(enabled).toContain("system");
  });
});

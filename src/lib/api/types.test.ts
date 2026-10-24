import { describe, expect, it } from "vitest";
import { DEFAULT_SETTINGS, normalizeAppSettings } from "./types";

describe("normalizeAppSettings", () => {
  it("keeps the old real progress behavior when a persisted setting is missing", () => {
    expect(normalizeAppSettings({}).neteaseProgressMode).toBe("real");
  });

  it("preserves estimate and safely defaults an invalid runtime value", () => {
    expect(normalizeAppSettings({ neteaseProgressMode: "estimate" }).neteaseProgressMode).toBe("estimate");
    expect(normalizeAppSettings({ neteaseProgressMode: "unknown" as never }).neteaseProgressMode)
      .toBe(DEFAULT_SETTINGS.neteaseProgressMode);
  });

  it("prefers explicit mode values and seeds missing values from legacy shared settings", () => {
    const legacy = normalizeAppSettings({
      compactLength: 164,
      expandedCornerRadius: 62,
      floatingCompactLength: 220,
      floatingExpandedCornerRadius: 70,
    } as never);
    expect(legacy.floatingCompactLength).toBe(220);
    expect(legacy.edgeCompactLength).toBe(164);
    expect(legacy.floatingExpandedCornerRadius).toBe(70);
    expect(legacy.edgeExpandedCornerRadius).toBe(62);
  });
});

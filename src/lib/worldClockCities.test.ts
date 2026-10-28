import { describe, expect, it } from "vitest";
import { WORLD_CLOCK_CITIES, resolveWorldClockZone } from "./worldClockCities";

describe("world clock cities", () => {
  it("has unique city identities, localized names, and supported time zones", () => {
    expect(new Set(WORLD_CLOCK_CITIES.map(city => city.id)).size).toBe(WORLD_CLOCK_CITIES.length);
    expect(WORLD_CLOCK_CITIES.length).toBeGreaterThan(40);
    for (const city of WORLD_CLOCK_CITIES) {
      expect(city.names["zh-CN"] && city.names.en && city.names.ja).toBeTruthy();
      expect(() => new Intl.DateTimeFormat("en",{timeZone:city.timeZone}).format(0)).not.toThrow();
    }
  });
  it("uses real time-zone rules, including daylight saving", () => {
    const format = (zone:string, instant:string) => new Intl.DateTimeFormat("en", {timeZone:zone,hour:"2-digit",minute:"2-digit",hourCycle:"h23"}).format(new Date(instant));
    expect(format("America/New_York","2026-07-01T12:00:00Z")).toBe("08:00");
    expect(format("America/New_York","2026-01-01T12:00:00Z")).toBe("07:00");
    expect(resolveWorldClockZone("Asia/Tokyo")).toBe("Asia/Tokyo");
  });
});

import { describe, expect, it } from "vitest";
import { calendarDayDifference, calendarSevenDays, calendarMonthDays, calendarMonthStart, calendarToday } from "./calendarDates";

describe("calendar dates", () => {
  it("shows two preceding days and four following days across the year boundary", () => {
    const today = Date.UTC(2027, 0, 1);
    expect(calendarSevenDays(today).map(day => new Date(day).toISOString().slice(0, 10))).toEqual(["2026-12-30", "2026-12-31", "2027-01-01", "2027-01-02", "2027-01-03", "2027-01-04", "2027-01-05"]);
    expect(calendarMonthStart(today, -1)).toBe(Date.UTC(2026, 11, 1));
  });
  it("counts civil days correctly across leap days and daylight saving changes", () => {
    const today = calendarToday(Date.parse("2024-03-10T09:30:00Z"), "America/Los_Angeles");
    const next = calendarToday(Date.parse("2024-03-11T08:30:00Z"), "America/Los_Angeles");
    expect(calendarDayDifference(next, today)).toBe(1);
    expect(calendarDayDifference(Date.UTC(2024, 1, 29), Date.UTC(2024, 2, 1))).toBe(-1);
    expect(calendarMonthDays(Date.UTC(2024, 1, 1)).filter(day => day !== null)).toHaveLength(29);
    expect(calendarToday(Date.parse("2026-10-27T16:30:00Z"), "Asia/Tokyo")).toBe(Date.UTC(2026, 9, 28));
    expect(calendarToday(Date.parse("2026-10-27T16:30:00Z"), "America/Los_Angeles")).toBe(Date.UTC(2026, 9, 27));
  });
});

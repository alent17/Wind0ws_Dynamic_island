/** Calendar arithmetic uses UTC midnight so DST changes never alter day counts. */
export const CALENDAR_DAY_MS = 86_400_000;

export function calendarToday(timestamp: number, timeZone = "system"): number {
  const parts = new Intl.DateTimeFormat("en-US-u-ca-gregory-nu-latn", {
    ...(timeZone === "system" ? {} : {timeZone}), year:"numeric", month:"numeric", day:"numeric",
  }).formatToParts(new Date(timestamp));
  const value = (type: string) => Number(parts.find(part => part.type === type)?.value);
  return Date.UTC(value("year"), value("month") - 1, value("day"));
}

export function calendarSevenDays(today: number): number[] {
  return Array.from({length:7}, (_, index) => today + (index - 2) * CALENDAR_DAY_MS);
}

export function calendarMonthStart(day: number, offset = 0): number {
  const date = new Date(day);
  return Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + offset, 1);
}

export function calendarMonthDays(monthStart: number): (number | null)[] {
  const date = new Date(monthStart);
  const start = (date.getUTCDay() + 6) % 7;
  const length = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + 1, 0)).getUTCDate();
  return Array.from({length:42}, (_, index) => {
    const day = index - start + 1;
    return day > 0 && day <= length ? Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), day) : null;
  });
}

export function calendarDayDifference(day: number, today: number): number {
  return Math.round((day - today) / CALENDAR_DAY_MS);
}

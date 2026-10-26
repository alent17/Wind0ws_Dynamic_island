import type { ResolvedLocale } from "$lib/i18n";

export const CLOCK_TIME_ZONES = [
  { value: "Asia/Shanghai", names: { "zh-CN": "北京", en: "Beijing", ja: "北京" } },
  { value: "Asia/Tokyo", names: { "zh-CN": "东京", en: "Tokyo", ja: "東京" } },
  { value: "Asia/Seoul", names: { "zh-CN": "首尔", en: "Seoul", ja: "ソウル" } },
  { value: "Asia/Hong_Kong", names: { "zh-CN": "香港", en: "Hong Kong", ja: "香港" } },
  { value: "Asia/Singapore", names: { "zh-CN": "新加坡", en: "Singapore", ja: "シンガポール" } },
  { value: "Asia/Bangkok", names: { "zh-CN": "曼谷", en: "Bangkok", ja: "バンコク" } },
  { value: "Asia/Kolkata", names: { "zh-CN": "孟买", en: "Mumbai", ja: "ムンバイ" } },
  { value: "Asia/Dubai", names: { "zh-CN": "迪拜", en: "Dubai", ja: "ドバイ" } },
  { value: "Europe/London", names: { "zh-CN": "伦敦", en: "London", ja: "ロンドン" } },
  { value: "Europe/Paris", names: { "zh-CN": "巴黎", en: "Paris", ja: "パリ" } },
  { value: "Europe/Berlin", names: { "zh-CN": "柏林", en: "Berlin", ja: "ベルリン" } },
  { value: "Europe/Moscow", names: { "zh-CN": "莫斯科", en: "Moscow", ja: "モスクワ" } },
  { value: "Africa/Cairo", names: { "zh-CN": "开罗", en: "Cairo", ja: "カイロ" } },
  { value: "America/New_York", names: { "zh-CN": "纽约", en: "New York", ja: "ニューヨーク" } },
  { value: "America/Chicago", names: { "zh-CN": "芝加哥", en: "Chicago", ja: "シカゴ" } },
  { value: "America/Denver", names: { "zh-CN": "丹佛", en: "Denver", ja: "デンバー" } },
  { value: "America/Los_Angeles", names: { "zh-CN": "洛杉矶", en: "Los Angeles", ja: "ロサンゼルス" } },
  { value: "America/Toronto", names: { "zh-CN": "多伦多", en: "Toronto", ja: "トロント" } },
  { value: "America/Vancouver", names: { "zh-CN": "温哥华", en: "Vancouver", ja: "バンクーバー" } },
  { value: "America/Sao_Paulo", names: { "zh-CN": "圣保罗", en: "São Paulo", ja: "サンパウロ" } },
  { value: "Australia/Sydney", names: { "zh-CN": "悉尼", en: "Sydney", ja: "シドニー" } },
  { value: "Pacific/Auckland", names: { "zh-CN": "奥克兰", en: "Auckland", ja: "オークランド" } },
  { value: "Pacific/Honolulu", names: { "zh-CN": "太平洋（檀香山）", en: "Pacific (Honolulu)", ja: "太平洋（ホノルル）" } },
  { value: "UTC", names: { "zh-CN": "协调世界时", en: "UTC", ja: "協定世界時" } },
] as const;

export function clockTimeZoneOption(timeZone: (typeof CLOCK_TIME_ZONES)[number]["value"], locale: ResolvedLocale, date = new Date()): { value: string; label: string } {
  const offset = new Intl.DateTimeFormat("en", { timeZone, timeZoneName: "shortOffset" })
    .formatToParts(date).find((part) => part.type === "timeZoneName")?.value ?? "GMT";
  const city = CLOCK_TIME_ZONES.find((zone) => zone.value === timeZone)?.names[locale] ?? timeZone;
  return { value: timeZone, label: `${city} · ${offset}` };
}

export function clockTimeZoneCity(timeZone: string, locale: ResolvedLocale): string {
  const resolved = timeZone === "system" ? Intl.DateTimeFormat().resolvedOptions().timeZone : timeZone;
  return CLOCK_TIME_ZONES.find((zone) => zone.value === resolved)?.names[locale] ?? resolved.split("/").at(-1)?.replaceAll("_", " ") ?? resolved;
}

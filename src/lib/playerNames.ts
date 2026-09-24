/** Names for saved SMTC IDs when Windows no longer reports the session. */
export function playerNameFromId(id: string): string {
  const lower = id.toLowerCase();
  if (lower.includes("cloudmusic")) return "网易云音乐";
  if (lower.includes("spotify")) return "Spotify";
  if (lower.includes("bilibili")) return "Bilibili";
  if (lower.includes("qqmusic")) return "QQ 音乐";
  if (lower.includes("apple") && lower.includes("music")) return "Apple Music";
  if (lower.includes("zunemusic")) return "Windows 媒体播放器";
  if (lower.startsWith("chrome")) return browserName("Chrome", id);
  if (lower.startsWith("msedge")) return browserName("Microsoft Edge", id);
  const app = id.split("!")[0].split("\\").pop() ?? id;
  return app.replace(/_[^_]+$/, "").replace(/\.exe$/i, "").replaceAll(".", " ") || id;
}

function browserName(name: string, id: string): string {
  const profile = /\.Profile(\d+)/i.exec(id);
  return profile ? `${name} · 个人资料 ${profile[1]}` : name;
}

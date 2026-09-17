import type { FontId } from "$lib/api/types";

const stacks: Record<FontId, string> = {
  system: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  misans: '"MiSans", "Segoe UI", sans-serif',
  "source-han-serif-cn-bold": '"Source Han Serif CN", "SimSun", serif',
  "alibaba-puhuiti-heavy": '"Alibaba PuHuiTi Heavy", "Microsoft YaHei", sans-serif',
};

export function applyAppFont(fontId: FontId | string | undefined) {
  const resolved = (fontId && fontId in stacks ? fontId : "misans") as FontId;
  document.documentElement.style.setProperty("--app-font", stacks[resolved]);
}

export const FONT_OPTIONS: Array<{ id: FontId; label: string }> = [
  { id: "system", label: "系统默认" },
  { id: "misans", label: "MiSans" },
  { id: "source-han-serif-cn-bold", label: "思源宋体 Bold" },
  { id: "alibaba-puhuiti-heavy", label: "阿里巴巴普惠体 Heavy" },
];

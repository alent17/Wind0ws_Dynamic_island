# 旧 UI 残留清理（2026-10-24）

通过 TypeScript 静态导入、动态导入、Svelte 组件引用和资源路径检查确认当前入口，删除不可达的旧 UI 残留。当前界面的结构、样式参数和功能控制链路保留。

## 当前 UI 入口

| 界面 | 入口与组件 |
| --- | --- |
| 主灵动岛 | `index.html` → `src/main.ts` → `src/App.svelte` → `IslandSurface.svelte` |
| 设置 | `studio.html` → `src/studio.ts` → `src/Studio.svelte` |
| 悬浮播放器 | `src/main.ts` 的 `window=floating` 分支 → `FloatingWindow.svelte` |
| 独立倒计时 | `src/main.ts` 的 `window=timer` 分支 → `TimerWindow.svelte` |

`App.svelte` 仍是当前窗口、媒体、计时器及设置的控制层；`ComicClock` 仍由当前时钟页引用。Spotify 演示图标和托盘图标仍有实际调用，保留。

## 删除的旧文件

- `src/lib/components/media/PlayerControls.svelte`
- `src/lib/components/volume/ArcVolumeControl.svelte`
- `src/assets/icons/apple_music.svg`
- `src/assets/icons/bilibili.svg`
- `src/assets/icons/default_music.svg`
- `src/assets/icons/netease.svg`
- `src/assets/icons/qqmusic.svg`
- `static/vite.svg`
- `static/tauri.svg`
- `static/svelte.svg`

以上共 10 个文件，18,022 字节；旧组件及其样式一起删除。历史审计脚本中作为记录使用的旧文件名不构成运行时依赖。

清理了当前 UI 文件中已无引用的 `Spectrum`、`Minimize2`、`VolumeX`、`clampSeekPosition` 和 `IslandGeometry` 导入。

将 `static/favicon.png` 移至当前 Vite 的静态资源目录 `public/favicon.png`，保留 `index.html` 的 favicon 引用。`static` 中已无旧资源；删除其空目录的命令被自动审批拒绝，仅返回 `blocked by policy`，目录保留。

旧文件在删除前已备份至系统临时目录的 `isle-legacy-ui-backup-2026-10-24.zip`，未在项目中另留一套旧 UI。

## 验证

- 运行源码及 UI 测试中无已删除组件或图标的引用。
- `pnpm run check`：0 错误、0 警告。
- `pnpm exec vitest run`：28 个测试文件、165 项测试通过。
- `pnpm run build`：生产构建成功，favicon 随构建输出。
- Playwright：低动效布局切换和 Hover 返回两项回归通过。
- 进度组件旧测试仍要求显示已由用户取消的红字提示，已同步为当前契约：保留未知时间 `--:--`，不显示待确认提示，不提供无效进度拖拽。

这些验证覆盖清理后的前端引用、构建和相关交互；未以此宣称新的原生性能收益。

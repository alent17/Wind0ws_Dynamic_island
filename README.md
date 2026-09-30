# Isle — 原生 Windows UI

[简体中文](README.md) · [English](README.en.md) · [日本語](README.ja.md)

main 默认使用 **Rust、Win32、Direct2D、DirectWrite、DXGI 和 DirectComposition**。原生版本 1.0.11，运行时不需要 Tauri 或 WebView2。

## 功能

- Windows 系统媒体会话：曲目、封面、播放控制、进度和播放器选择。
- 灵动岛收起/展开、四向贴边、实时六段频谱与动画偏好。
- 独立原生悬浮播放器与倒计时窗口。
- 系统音量和输出设备、时钟时区、天气与城市搜索。
- 原生设置、多显示器、DPI 和基础无障碍接口。

原生主岛预览：

![原生灵动岛](docs/images/native-prototype/top.png)

托盘、MV 视频和部分旧版捕获保护尚未迁移；完整迁移与验证记录见 [native/README.md](native/README.md) 和 [迁移清单](docs/native-ui-execution-checklist.md)。

## 启动与构建

需要 Windows 10/11 x64、Rust MSVC、Visual Studio C++ Build Tools 和 Windows SDK。生成安装包还需要 NSIS。

无需 Node.js 即可启动：

```powershell
cargo run --release --manifest-path native/Cargo.toml --bin isle-native
```

根目录命令默认指向原生程序：

```powershell
npm run dev
npm run check
npm test
npm run build
npm run bundle:windows
```

输出：`dist/Isle_1.0.11_x64-setup.exe`、`dist/Isle_1.0.11_native_x64.zip`。便携运行时保留 EXE 旁的 `fonts` 和许可文件。安装方法见 [INSTALL.md](INSTALL.md)。

默认连接真实媒体；F8 打开设置，Alt+F4 退出。`--open-floating` 启动悬浮播放器，`--open-timer` 打开倒计时；`--demo` 使用演示数据。

配置保存到 `%APPDATA%/IsleNative/settings.json`。首次读取旧版配置，首次迁移保存保留原始备份。性能数据见 [性能记录](docs/performance/README.md)，100 MB 整组内存目标尚未完成。

## 旧版 WebView 源码

旧版保留于 `src/` 与 `src-tauri/`，使用显式命令：

```powershell
npm install
npm run web:dev
npm run web:check
npm run web:test
npm run web:bundle:windows
```

旧版 Tauri 开发入口仍可通过 `npm run tauri dev` 使用。旧版截图保存在 `docs/screenshots/`，不代表当前原生 UI。

## 隐私与网络

媒体和系统音量控制在本机完成。天气使用 Open-Meteo；封面补全可能访问网易云音乐服务。原生频谱仅分析系统输出，不读取麦克风或保存原始声音。

## 贡献与许可证

提交前运行原生检查、测试和相关窗口验证；旧版前端改动运行 `web:check`、`web:test`。不提交个人配置或构建缓存。

[MIT License](LICENSE) · [alent17](https://github.com/alent17)
<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Isle icon">

# Isle

**A Windows media island that stays beautifully within reach.**

[简体中文](README.md) · [English](README.en.md) · [日本語](README.ja.md)

[Download v1.0.0](https://github.com/alent17/Wind0ws_Dynamic_island/releases/tag/v1.0.0) · [Installation](INSTALL.md) · [Report an issue](https://github.com/alent17/Wind0ws_Dynamic_island/issues)

</div>

---

Isle is a Dynamic Island-inspired desktop media controller and floating player for Windows 10/11. It reads track, playback, and timeline data from Windows media sessions and presents them in a lightweight, persistent, customizable interface.

![Isle Studio and the expanded island](docs/screenshots/isle-studio.png)

## Highlights

- Windows SMTC playback controls, seeking, and player priority
- Compact, hover, expanded, and auto-hidden island states
- Floating or four-edge attached layouts with multi-monitor support
- Live system-audio FFT spectrum or smooth generated animation
- High-resolution artwork matching and a resizable floating player synchronized with the island
- Weather, date, network, CPU, memory, battery, and custom idle content
- System volume, output-device selection, timer, tray, and startup integration
- Capture privacy options for screenshots, recording, games, and screen sharing
- Simplified Chinese, English, and Japanese UI

## Floating player

The floating player is an independent, immersive media view that can be opened from Isle Studio or the island's shortcut controls.

- Drag it anywhere and resize it freely; compact and large layouts adapt automatically.
- Track, play state, timeline, play/pause, previous, and next controls stay synchronized with the island.
- Prefer high-resolution artwork, or silently loop a matched 30-second video preview when the MV option is enabled.
- Generate a background gradient from the artwork's representative color or use a fixed custom color.
- Supports always-on-top, close, position reset, and capture protection for screenshots, recording, and screen sharing.

| Floating player | Media view |
|:--:|:--:|
| ![Floating player](docs/screenshots/floating-player.png) | ![Floating player media view](docs/screenshots/floating-player-compact.png) |

## Install

Download `Isle_1.0.0_x64-setup.exe` from [GitHub Releases](https://github.com/alent17/Wind0ws_Dynamic_island/releases/latest), run the installer, then play media from an app that supports Windows system media controls. Open Isle Studio from the system tray to customize the experience.

Requires Windows 10/11 x64 and Microsoft Edge WebView2 Runtime. Player capabilities vary with the information exposed through Windows SMTC.

## Build from source

Install Node.js 18+, Rust stable, and Visual Studio 2022 Build Tools with Desktop development with C++.

```powershell
npm install
npm run check
npm test -- --run
cargo test --manifest-path src-tauri/Cargo.toml
npm run bundle:windows
```

Built with Tauri 2, Svelte 5, TypeScript, Vite, Rust, Windows API / SMTC, CPAL, RustFFT, and Vitest.

## Privacy

Core media control stays local. Weather uses Open-Meteo; high-resolution artwork may use Apple iTunes Search and an existing unauthenticated NetEase Cloud Music endpoint. Isle does not request Windows location permission.

## Contributing

Issues and pull requests are welcome. Please run the frontend checks, frontend tests, and Rust tests before submitting.

**Contributors**: [@alent17](https://github.com/alent17) · [ChatGPT](https://chatgpt.com/)

Licensed under the [MIT License](LICENSE).

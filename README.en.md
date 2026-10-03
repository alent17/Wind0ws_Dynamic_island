# Isle — Windows Dynamic Island (Tauri)

This repository maintains the **Svelte, Tauri 2, and WebView2** Windows desktop app. The separate Rust/Win32 native UI now lives in [Wind0ws_Dynamic_island-native](https://github.com/alent17/Wind0ws_Dynamic_island-native).

## Development

Requirements: Node.js, pnpm (or npm), Rust MSVC, Visual Studio C++ Build Tools, and the Windows SDK.

```powershell
pnpm install
pnpm dev
```

Common commands: `pnpm check`, `pnpm test`, `pnpm test:ui`, `pnpm build`, and `pnpm bundle:windows`. Use `pnpm tauri dev` to launch the Tauri development app directly.

## Project layout

- `src/`: Svelte UI
- `src-tauri/`: Tauri/Rust backend and app configuration
- `ui-tests/`: Playwright UI tests and migration reference captures
- `docs/`: project documentation, release notes, and Tauri performance samples

See [INSTALL.md](INSTALL.md) for installation and upgrade notes, or visit the [native UI repository](https://github.com/alent17/Wind0ws_Dynamic_island-native) for its source and documentation.

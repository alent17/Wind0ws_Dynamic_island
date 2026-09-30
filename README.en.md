# Isle — Native Windows UI

[简体中文](README.md) · [English](README.en.md) · [日本語](README.ja.md)

The default application on main is **Isle Native 1.0.11**, built with Rust, Win32, Direct2D, DirectWrite and DirectComposition. It does not require WebView2 at runtime.

## Run and build

Requires Windows 10/11 x64, Rust MSVC, Visual Studio C++ Build Tools and the Windows SDK. NSIS is required to create the installer.

```powershell
cargo run --release --manifest-path native/Cargo.toml --bin isle-native
```

Root commands now target the native application:

```powershell
npm run dev
npm run check
npm test
npm run bundle:windows
```

Outputs: `dist/Isle_1.0.11_x64-setup.exe` and `dist/Isle_1.0.11_native_x64.zip`. Keep the fonts and license files next to the portable executable.

Real Windows media sessions are enabled by default. F8 opens settings; Alt+F4 exits. Use `--open-floating` to open the native floating player or `--demo` for sample data.

Native features include media controls, spectrum, a floating player, a timer, system audio, time zones, weather and settings. Tray integration, MV video and some legacy capture protection remain unported. See [native documentation](native/README.md) and [installation instructions](INSTALL.md).

Native settings are stored in `%APPDATA%/IsleNative/settings.json`. The first migration can read legacy settings and backs up the original data when saving.

## Legacy WebView application

The legacy source remains in `src/` and `src-tauri/`. Use `web:dev`, `web:check`, `web:test` and `web:bundle:windows` explicitly. Historical screenshots in `docs/screenshots/` show the legacy UI.

[MIT License](LICENSE)
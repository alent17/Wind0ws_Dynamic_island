# Isle — Windows ネイティブ UI

[简体中文](README.md) · [English](README.en.md) · [日本語](README.ja.md)

main の既定アプリは **Isle Native 1.0.11** です。Rust、Win32、Direct2D、DirectWrite、DirectComposition を使用し、実行時に WebView2 は不要です。

## 起動とビルド

Windows 10/11 x64、Rust MSVC、Visual Studio C++ Build Tools、Windows SDK が必要です。インストーラー作成には NSIS が必要です。

```powershell
cargo run --release --manifest-path native/Cargo.toml --bin isle-native
```

プロジェクト直下のコマンドはネイティブ版を使用します。

```powershell
npm run dev
npm run check
npm test
npm run bundle:windows
```

出力は `dist/Isle_1.0.11_x64-setup.exe` と `dist/Isle_1.0.11_native_x64.zip` です。ポータブル版では EXE と同じ場所に fonts とライセンスファイルを保持してください。

既定で実際の Windows メディアセッションに接続します。F8 で設定、Alt+F4 で終了します。`--open-floating` は浮動プレイヤーを開き、`--demo` はデモデータを使用します。

メディア操作、スペクトラム、浮動プレイヤー、タイマー、システム音量、時計、天気、設定を実装しています。トレイ、MV 動画、一部の旧版キャプチャ保護は未移行です。[ネイティブ版の説明](native/README.md) と [インストール方法](INSTALL.md) を参照してください。

設定は `%APPDATA%/IsleNative/settings.json` に保存します。初回移行では旧版設定を読み取り、保存時に元データをバックアップします。

## 旧 WebView 版

旧版ソースは `src/` と `src-tauri/` に保持しています。`web:dev`、`web:check`、`web:test`、`web:bundle:windows` を明示的に使用してください。`docs/screenshots/` は旧版 UI の画像です。

[MIT License](LICENSE)
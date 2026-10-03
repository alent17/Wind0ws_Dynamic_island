# Isle — Windows Dynamic Island（Tauri）

このリポジトリでは **Svelte、Tauri 2、WebView2** を使った Windows デスクトップアプリを管理します。Rust/Win32 のネイティブ UI は[別リポジトリ](https://github.com/alent17/Wind0ws_Dynamic_island-native)に分離しました。

## 開発

Node.js、pnpm（または npm）、Rust MSVC、Visual Studio C++ Build Tools、Windows SDK が必要です。

```powershell
pnpm install
pnpm dev
```

主なコマンド：`pnpm check`、`pnpm test`、`pnpm test:ui`、`pnpm build`、`pnpm bundle:windows`。Tauri 開発アプリを直接起動するには `pnpm tauri dev` を実行します。

`src/` は Svelte UI、`src-tauri/` は Tauri/Rust バックエンドと設定、`ui-tests/` は Playwright テストです。インストール方法は [INSTALL.md](INSTALL.md)、ネイティブ UI のソースと文書は[ネイティブ UI リポジトリ](https://github.com/alent17/Wind0ws_Dynamic_island-native)を参照してください。

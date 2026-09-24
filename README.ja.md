<div align="center">

<img src="src-tauri/icons/128x128.png" width="96" alt="Isle icon">

# Isle

**Windows のメディア情報を、美しい Dynamic Island に。**

[简体中文](README.md) · [English](README.en.md) · [日本語](README.ja.md)

[v1.0.2 をダウンロード](https://github.com/alent17/Wind0ws_Dynamic_island/releases/tag/v1.0.2) · [インストール](INSTALL.md) · [問題を報告](https://github.com/alent17/Wind0ws_Dynamic_island/issues)

</div>

---

Isle は Windows 10/11 向けの Dynamic Island 風デスクトップ・メディアコントローラー兼フローティングプレーヤーです。Windows のメディアセッションから曲名、再生状態、タイムラインを取得し、軽量で常駐可能なカスタマイズ UI に表示します。

![Isle Studio と展開したアイランド](docs/screenshots/isle-studio.png)

## 主な機能

- Windows SMTC による再生、一時停止、前後の曲、シーク操作
- 最後に再生した曲を記憶し、アイランドからプレーヤーを開いて再生を再開
- 収納、ホバー、展開、自動非表示の各状態
- フローティングまたは画面四辺への接続、マルチディスプレイ対応
- システム音声に連動する FFT スペクトラムとスムーズなランダム表示
- 高解像度カバー検索とアイランドに同期するサイズ変更可能なフローティングプレーヤー
- 天気、日付、通信速度、CPU、メモリ、バッテリー、カスタム文字
- 音量、出力デバイス、タイマー、トレイ、スタートアップ連携
- スクリーンショット、録画、ゲーム、画面共有向けのプライバシー設定
- 简体中文、English、日本語の UI

## フローティングプレーヤー

フローティングプレーヤーは、Isle Studio またはアイランドのショートカットから開ける独立した没入型メディアビューです。

- 自由にドラッグ、サイズ変更でき、コンパクト表示と大型表示へ自動的に適応します。
- 曲、再生状態、タイムライン、再生・一時停止、前後の曲の操作をアイランドと同期します。
- 高解像度カバーを優先表示し、MV 機能を有効にすると一致した30秒のプレビューを無音でループ再生します。
- カバーの代表色から背景グラデーションを生成するか、固定色を設定できます。
- 常に最前面、閉じる、位置のリセット、スクリーンショット・録画・画面共有のキャプチャ保護に対応します。

| フローティングプレーヤー | メディアビュー |
|:--:|:--:|
| ![フローティングプレーヤー](docs/screenshots/floating-player.png) | ![フローティングプレーヤーのメディアビュー](docs/screenshots/floating-player-compact.png) |

## インストール

[GitHub Releases](https://github.com/alent17/Wind0ws_Dynamic_island/releases/latest) から `Isle_1.0.2_x64-setup.exe` をダウンロードして実行します。Windows のメディアコントロールに対応したアプリで音楽を再生し、システムトレイから Isle Studio を開いて設定してください。

Windows 10/11 x64 と Microsoft Edge WebView2 Runtime が必要です。利用できる操作は各プレーヤーの Windows SMTC 対応状況によって異なります。

## ソースからビルド

Node.js 18+、Rust stable、Visual Studio 2022 Build Tools（Desktop development with C++）を用意してください。

```powershell
npm install
npm run check
npm test -- --run
cargo test --manifest-path src-tauri/Cargo.toml
npm run bundle:windows
```

Tauri 2、Svelte 5、TypeScript、Vite、Rust、Windows API / SMTC、CPAL、RustFFT、Vitest を使用しています。

## プライバシー

メディア操作はローカルで完結します。天気には Open-Meteo、高解像度カバーには Apple iTunes Search と既存の認証不要 NetEase Cloud Music エンドポイントを利用する場合があります。Windows の位置情報権限は要求しません。

## コントリビューション

Issue と Pull Request を歓迎します。送信前にフロントエンド検査、テスト、Rust テストを実行してください。

**コントリビューター**：[@alent17](https://github.com/alent17) · [ChatGPT](https://chatgpt.com/)

[MIT License](LICENSE) のもとで公開されています。

# Isle 原生版安装

main 默认发行 Windows x64 原生程序，不需要 WebView2。

## 安装与升级

运行 `Isle_1.0.11_x64-setup.exe`，选择安装目录。更新前关闭旧程序；从旧 Tauri 版更新时需在旧版托盘选择“退出”。如使用此前的安装目录，安装器会替换 `isle.exe` 为原生程序，保留用户配置。

便携包 `Isle_1.0.11_native_x64.zip` 解压后运行 `isle.exe`；保留同目录的 `fonts/` 与许可文件。

原生版默认连接真实系统媒体会话。F8 打开设置，Alt+F4 退出；原生版本目前没有系统托盘。悬浮播放器可在工具栏打开，也可执行 `isle.exe --open-floating`。

原生配置位于 `%APPDATA%/IsleNative/settings.json`，首次可读取旧版配置并在迁移保存时备份。卸载不会删除用户配置。

## 从源码运行与打包

需要 Rust MSVC、Visual Studio C++ Build Tools、Windows SDK；打包需要 NSIS。

```powershell
cargo run --release --manifest-path native/Cargo.toml --bin isle-native
cargo test --manifest-path native/Cargo.toml --workspace
npm run bundle:windows
```

也可不使用 npm：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File native/scripts/package-windows.ps1
```

脚本自动寻找 PATH、标准安装目录或现有工具缓存中的 NSIS，可使用 `-MakensisPath` 指定。EXE、安装包和便携 ZIP 输出在 `dist/`。

## 确认版本

查看安装目录 `isle.exe` 的文件属性，应显示 ProductVersion **1.0.11**、FileDescription **Isle Native**。原生程序不会启动 WebView2 子进程。旧目录中残留的 WebView2 缓存不影响原生运行。

## 旧版 WebView 构建

旧版需要 Node.js、Rust 和 WebView2，其命令是 `npm run web:bundle:windows`；通过 `npm run tauri dev` 开发。它不是 main 的默认发行程序。
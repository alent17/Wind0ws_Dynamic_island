# Isle 安装与故障排查

## 安装发行版

1. 从发布包中运行 `Isle_*_x64-setup.exe`。
2. 按向导选择安装目录并完成安装。当前安装模式为当前用户安装，不需要管理员权限。
3. 首次启动后，Isle 会驻留系统托盘；关闭主界面不等于退出程序。
4. 如需开机启动，从系统托盘打开 Isle Studio，在“应用行为”中开启“开机启动”。

如果系统提示缺少 WebView2，请先安装 Microsoft Edge WebView2 Runtime，再重新启动 Isle。

## 更新安装

直接运行新版本安装包即可覆盖更新。更新前建议从系统托盘选择“退出”，确保旧版本完全结束；这样可以避免旧进程继续占用窗口或后端文件。

## 从源码运行

需要 Windows 10/11、Node.js 18+、Rust stable、Visual Studio 2022 Build Tools（“使用 C++ 的桌面开发”）和 WebView2 Runtime。

```powershell
npm install
npm run tauri dev
```

生成 Windows 安装包：

```powershell
npm run bundle:windows
```

安装包输出在 `src-tauri/target/release/bundle/nsis/`。

## 功能说明

- 展开灵动岛后，功能栏提供系统音量滑块；拖动即可调整 Windows 主音量。音频设备切换仍使用 Windows 音量面板或系统设置。
- 倒计时窗口已改为启动时预加载并复用，打开时只执行显示和聚焦，不再同步创建新的 WebView。
- 倒计时运行期间跨窗口状态每秒同步一次；点击开始、暂停、继续、加时或重置会立即同步。

## 倒计时或悬浮窗无法打开

如果看到窗口标题显示“未响应”或界面空白：

1. 先从系统托盘退出 Isle，不要只关闭窗口。
2. 重新启动应用，再打开倒计时。
3. 如果仍然无响应，重新运行最新安装包覆盖安装。
4. 最后再考虑重置用户配置；操作前请备份 `%APPDATA%` 下 Isle 的设置文件，因为这会丢失个性化设置。

## 仍然无法启动时

请记录 Windows 版本、Isle 版本、使用的播放器，以及是否在点击倒计时前就已经卡顿，并附上启动后是否能从托盘退出。这些信息有助于区分 WebView2、播放器媒体会话和配置文件问题。

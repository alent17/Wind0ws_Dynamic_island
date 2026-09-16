# Isle

Isle 是一个面向 Windows 10/11 的桌面灵动岛与悬浮音乐播放器。它通过 Windows 系统媒体会话读取当前曲目、播放状态和时间线，并使用 Tauri 2、Svelte 5 与 Rust 提供轻量、常驻且可定制的播放界面。

> 当前项目处于早期开发阶段。不同播放器对 Windows SMTC 的支持程度不同，曲目信息、进度和可用控制可能存在差异。

## 功能

- 系统媒体会话：自动读取支持 SMTC 的播放器，并提供播放、暂停、上一首、下一首和可用的进度跳转。
- 灵动岛形态：支持收起、悬停、展开和自动隐藏状态；收起长度可在 80–300 px 之间调整。
- 屏幕贴边：支持上、右、下、左四个方向以及沿边位置调整；贴边展开保持与屏幕边缘连接。
- 空闲内容：没有媒体会话时，收起态显示居中时钟，展开态可轮播日期、天气、网络、CPU、内存、电池和自定义文本。
- 天气城市：通过 Open-Meteo 搜索城市并保存精确经纬度，可区分同名地点，不申请 Windows 定位权限。
- 音频频谱：提供系统音频实时 FFT 与平滑随机动画两种模式；暂停后频谱自然回落。
- 高清封面：根据播放器来源优先尝试网易云或 Apple iTunes Search，并在匹配置信度不足时保留系统原始封面。
- 悬浮播放器：独立、可调整大小的封面播放器，与主岛共享媒体时间线和进度跳转逻辑。
- Isle Studio：集中设置布局、边缘、字体、播放器优先级、频谱、空闲内容、显示器和应用行为。
- Capture Mode：可分别配置截图、录屏、全屏游戏和屏幕共享场景；截图/全屏触发临时收起，Windows 捕获保护会把窗口从录制或共享画面中排除。
- 系统音频活动：调节 Windows 音量时自动显示音量与当前输出设备，展开后可调节音量并切换扬声器、耳机或蓝牙输出。
- 系统集成：支持多显示器、始终置顶、开机启动和系统托盘。

## 近期改进

- 统一主岛与悬浮窗的媒体时钟，过滤暂停时 SMTC 偶发上报的零进度，并保持暂停前的有效位置。
- 进度条支持拖动即时预览，释放后只提交一次跳转；不可跳转的会话仅显示进度。
- 修复四向贴边展开缝隙，贴边交互不再使用会拉开屏幕边缘的缩放反馈。
- 改进中文城市搜索、行政区消歧和经纬度缓存隔离。
- 将高清封面解析集中到 Rust 后端，加入来源优先级、候选匹配、请求超时、换歌防串图、限流和 24 小时缓存。
- 新增实时与随机频谱模式，并调整实时 FFT 的噪声底、响应速度和平滑效果。

## 技术栈

- Tauri 2
- Svelte 5（Runes）
- TypeScript、Vite、Tailwind CSS
- Rust、Windows API / SMTC
- CPAL、RustFFT
- Vitest

## 环境要求

- Windows 10 或 Windows 11
- Node.js 18 或更高版本
- Rust stable 工具链
- Visual Studio 2022 Build Tools，安装“使用 C++ 的桌面开发”工作负载
- Microsoft Edge WebView2 Runtime

## 本地开发

安装依赖：

```powershell
npm install
```

启动完整桌面应用：

```powershell
npm run tauri dev
```

只启动前端预览：

```powershell
npm run dev
```

运行检查：

```powershell
npm run check
npm test -- --run
cargo test --manifest-path src-tauri/Cargo.toml
```

构建生产安装包：

```powershell
npm run bundle:windows
```

构建结果位于 `src-tauri/target/release/bundle/nsis/`。安装向导允许用户选择安装目录，并在完成安装前询问是否随 Windows 开机启动。

## 使用方式

1. 启动任意支持 Windows 系统媒体控制的播放器并播放媒体。
2. 点击灵动岛可展开或收起；展开后可使用媒体控制与进度条。
3. 从系统托盘打开 Isle Studio，选择播放器、显示器、岛体形态、贴边方向和其他偏好。
4. 如需悬浮播放器，可在 Studio 中打开，并通过窗口边缘调整尺寸。
5. 应用窗口关闭后仍可能驻留系统托盘；需要完全退出或加载新后端版本时，请从托盘菜单退出。

## 项目结构

```text
.
├─ src/
│  ├─ App.svelte                 # 主灵动岛窗口
│  ├─ FloatingWindow.svelte      # 独立悬浮播放器
│  ├─ Studio.svelte              # 设置页面
│  ├─ lib/IslandSurface.svelte   # 主岛共享表面组件
│  └─ lib/                       # 媒体时钟、频谱、API 与几何逻辑
├─ src-tauri/
│  └─ src/
│     ├─ commands/               # Tauri 命令
│     ├─ services/               # 媒体、天气、封面与缓存服务
│     └─ models/                 # 前后端共享数据模型
├─ DESIGN.md
├─ PRODUCT.md
└─ README.md
```

## 网络服务与隐私

核心媒体控制在本机完成。启用相关功能时，应用可能访问：

- Open-Meteo：城市搜索与天气数据。
- Apple iTunes Search API：无需用户令牌的歌曲封面搜索。
- 网易云音乐现有免鉴权搜索端点：歌曲信息与封面补全。该端点不是本项目验证的官方开放 API，可能随时不可用。

天气设置只保存用户主动选择的地点名称及经纬度，不请求 Windows 定位权限。第三方服务失败时，媒体界面会尽量保留 Windows SMTC 提供的原始数据。

## 已知限制

- 媒体能力取决于播放器公开给 Windows SMTC 的信息。
- 某些浏览器、直播流或播放器不会提供可跳转进度或准确时长。
- 实时频谱依赖可用的 Windows 音频设备与驱动。
- 高清封面依赖第三方搜索结果，匹配失败时会回退到系统封面。

## 贡献

欢迎提交 Issue 或 Pull Request。提交前请至少运行前端检查、前端测试和 Rust 测试，并避免提交本地缓存、构建产物或个人配置。

## License

项目包元数据声明为 MIT。正式分发前建议在仓库根目录补充独立的 `LICENSE` 文件。

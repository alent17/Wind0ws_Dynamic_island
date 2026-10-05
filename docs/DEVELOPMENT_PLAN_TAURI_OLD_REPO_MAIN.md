# Wind0ws Dynamic Island — 旧 Tauri 主仓库恢复与 Native 功能迁移计划

> 目标：重新以旧 Tauri 仓库 `alent17/Wind0ws_Dynamic_island` 作为主开发仓库，在其现有 Tauri / Svelte UI 基础上继续开发，并将 `alent17/Wind0ws_Dynamic_island-native` 中已经验证成熟的 Rust / Win32 / 播放器能力按模块迁回。  
> 原则：**旧 Tauri 仓库负责主工程和 UI，Native 仓库只作为功能移植来源；Rust 管功能，Svelte 管界面。**

---


---

## 0. 主仓库与迁移策略

### 主开发仓库

```text
https://github.com/alent17/Wind0ws_Dynamic_island
```

旧 Tauri 仓库重新作为最终产品和主开发仓库，继续保留：

- Tauri 工程结构
- Svelte 前端
- 灵动岛 UI
- 设置页面
- CSS / Web Animation
- 前端组件和交互
- Tauri 窗口体系
- 前后端通信
- 已经验证过的视觉方案

### Native 功能来源仓库

```text
https://github.com/alent17/Wind0ws_Dynamic_island-native
```

Native 仓库不再作为最终产品主仓库，只作为功能迁移来源，重点提取：

- Rust Core 改进
- Win32 系统能力
- SMTC
- 网易云音乐专用控制
- 其他播放器控制改进
- 系统音量
- 开机启动
- 截图隐藏
- DPI / 多显示器相关实现
- 其他已经验证成熟的系统逻辑

### 原则上不迁移

- Native UI
- Native 设置页面
- Native UI 动画
- Native UI 布局代码
- 与 Svelte UI 重复的视图层代码
- 仅为 Native UI 服务的窗口封装
- 已被旧 Tauri 版本更好实现的重复逻辑

### 最终关系

```text
旧 Tauri 仓库
Wind0ws_Dynamic_island
        │
        │ 继续作为主项目
        ↓
Tauri 2 + Svelte UI
        +
Rust Backend
        +
必要 Win32
        ↑
        │ 按模块迁移
        │
Native 仓库
Wind0ws_Dynamic_island-native
```

> **UI 留在旧 Tauri 仓库，Native 仓库只迁成熟功能，不迁 UI。**

---

## P-1 — 两仓库迁移审计

正式迁移前，Codex 必须同时读取：

- `alent17/Wind0ws_Dynamic_island`
- `alent17/Wind0ws_Dynamic_island-native`

并在旧 Tauri 主仓库生成：

```text
docs/MIGRATION_AUDIT.md
```

### 审计标记

| 标记 | 含义 |
|---|---|
| ✅ | 直接保留旧 Tauri 实现 |
| ⬆️ | 从 Native 仓库迁入 |
| 🔄 | 两边对比后合并 / 重构 |
| 🗑 | 删除 / 不迁移 |
| 🆕 | 重新实现 |

### 审计范围

- [x] Tauri 配置
- [x] Svelte 前端
- [x] 灵动岛 UI
- [x] 设置页面
- [x] 动画系统
- [x] 状态管理
- [x] Rust 后端
- [x] SMTC
- [x] 网易云控制
- [x] 其他播放器控制
- [x] 系统音量
- [x] 开机启动
- [x] 截图隐藏
- [x] 窗口控制
- [x] DPI / 多显示器
- [x] 配置持久化
- [x] Native UI
- [x] 重复依赖
- [x] 重复 Rust 模块
- [x] 可删除代码

### 审计必须回答

1. 旧 Tauri 仓库哪些部分可以原样保留？
2. Native 仓库哪些功能明显更新或更稳定？
3. 哪些 Rust 模块需要从 Native 仓库迁回？
4. 哪些模块两边都有实现，应该选哪一个？
5. 哪些 Native UI 代码完全不应迁移？
6. 是否存在 Cargo / npm / Tauri 依赖版本冲突？
7. 是否存在配置文件格式差异？
8. 哪些模块风险最高？
9. 第一批最安全的迁移模块是什么？
10. 哪些功能需要迁移后重新测试？

### 审计完成前禁止

- 不允许直接把 Native 仓库 merge 到旧 Tauri 仓库
- 不允许整仓复制 Native 源码
- 不允许覆盖旧 Tauri UI
- 不允许大规模删除旧 Tauri 代码
- 不允许在未对比实现前替换 Rust 模块
- 不允许直接修改 `main` 做实验性迁移

---

## P0 — Git 迁移分支策略

旧 Tauri 仓库继续作为主仓库。

```text
Wind0ws_Dynamic_island
│
├─ main
│
├─ backup/pre-native-return
│
└─ migration/native-features
```

### 操作原则

1. 为旧 Tauri 仓库当前 `main` 创建备份分支或 tag。
2. 从 `main` 创建 `migration/native-features`。
3. 所有 Native 功能迁移先在迁移分支完成。
4. 每迁移一个模块就独立测试。
5. 每个模块尽量独立 commit。
6. 不使用整仓 `git merge native/main`。
7. 迁移完成并通过验收后再合并回 `main`。

### 第一批建议迁移

- [ ] 网易云控制
- [ ] SMTC
- [ ] 其他播放器控制改进
- [ ] 系统音量
- [ ] 开机启动
- [ ] 截图隐藏
- [ ] 多显示器 / DPI 相关能力

### 暂时不要迁移

- [ ] Native UI
- [ ] Native 设置页
- [ ] Native 动画系统
- [ ] Native UI 状态管理


## 1. 总体架构

```text
┌─────────────────────────────┐
│        Tauri / Svelte       │
│                             │
│  灵动岛 UI                  │
│  设置页面                   │
│  音乐控制 UI                │
│  倒计时 / 音量 / 时钟       │
│  CSS / Web Animation        │
└──────────────┬──────────────┘
               │ invoke / event
               ↓
┌─────────────────────────────┐
│          Rust Core          │
│                             │
│  播放器状态                  │
│  网易云控制                  │
│  SMTC / Win32               │
│  系统音量                    │
│  开机启动                    │
│  截图隐藏                    │
└──────────────┬──────────────┘
               │
               ↓
┌─────────────────────────────┐
│       Windows Native        │
│ HWND / Win32 / COM / API    │
└─────────────────────────────┘
```

---

## 2. 开发计划表

| 阶段 | 任务 | 具体内容 | 验收标准 | 状态 |
|---|---|---|---|---|
| **P0** | 旧 Tauri 主仓库备份 | 给旧 Tauri `main` 打 tag / 建备份分支 | Tauri 基线可随时恢复 | ☑ |
| **P0** | 两仓库迁移审计 | 对比旧 Tauri 与 Native 仓库 | 生成 `docs/MIGRATION_AUDIT.md` | ☑ |
| **P0** | 创建迁移分支 | 创建 `migration/native-features` | Native 功能不直接修改 `main` | ☑ |
| **P1** | 验证旧 Tauri 基线 | 执行 Svelte 诊断、前端构建及 Tauri Release NSIS 构建 | 0 诊断错误；安装包构建成功 | ☑ |
| **P1** | 运行时窗口验收 | 启动 Release EXE 并验证不透明、无边框、置顶、任务栏和 DPI 行为 | 实机及缩放测试通过 | ☐ |
| **P1** | 无边框窗口 | 移除系统标题栏，自定义窗口外观 | 无系统边框、无多余白边 | ☐ |
| **P1** | 不透明窗口 | 灵动岛窗口使用实色背景（按用户要求取消透明窗口） | 窗口显示稳定、无异常底色 | ☐ |
| **P1** | 置顶 | 灵动岛保持 Always On Top | 普通窗口无法盖住灵动岛 | ☐ |
| **P1** | 任务栏隐藏 | 灵动岛主窗口不进入任务栏 | Alt+Tab / 任务栏行为符合预期 | ☐ |
| **P1** | DPI 适配 | 100% / 125% / 150% / 多显示器测试 | 不错位、不模糊 | ☐ |
| **P2** | 建立 Island Store | 创建统一状态管理 | UI 不直接读取后端杂乱状态 | ☐ |
| **P2** | 状态机 | 建立 `idle / media / expanded / timer / volume` 等状态 | 状态切换路径明确 | ☐ |
| **P2** | Rust → Svelte 通信 | 后端只通过 event / invoke 提供数据 | UI 与 Rust 解耦 | ☐ |
| **P2** | Settings Store | 设置统一存储 | 修改设置后即时生效并持久化 | ☐ |
| **P3** | 重做灵动岛基础 UI | 只完成黑色胶囊、尺寸、圆角、布局 | 静态 UI 达到设计要求 | ☐ |
| **P3** | 左侧媒体区域 | 专辑封面、歌曲、歌手 | 文本溢出、长歌名正常 | ☐ |
| **P3** | 右侧功能栏 | 倒计时 / 设置 / 音量 / 时钟 | 与岛体风格统一 | ☐ |
| **P3** | Hover 状态 | 鼠标进入 / 离开时展开收起 | 无跳变、无闪烁 | ☐ |
| **P3** | 点击区域 | 为不同控件定义独立点击区 | 不出现误触 | ☐ |
| **P4** | 灵动岛形变动画 | `width / height / radius / transform` 统一动画 | 收起、展开自然连续 | ☐ |
| **P4** | 左下角锚定动画 | 保留“向右上生长”逻辑 | 锚点视觉不漂移 | ☐ |
| **P4** | 内容进出动画 | 封面、文字、按钮分别控制 opacity / transform | 不抢动画、不突然出现 | ☐ |
| **P4** | 状态切换动画 | Media → Timer → Volume 等 | 状态间不闪、不重叠 | ☐ |
| **P4** | 动画中断处理 | 快速 Hover / 离开 / 切歌时处理动画打断 | 不出现尺寸卡死 | ☐ |
| **P5** | 恢复 SMTC | 获取 Windows 当前媒体信息 | 支持系统播放器 | ☐ |
| **P5** | 恢复网易云控制 | 使用已确定的网易云专用方案 | 网易云播放 / 暂停 / 上下曲正常 | ☐ |
| **P5** | 其他播放器 | 保持原有控制方式不变 | Spotify 等播放器正常 | ☐ |
| **P5** | 专辑封面更新 | 切歌实时更新封面 | 不残留上一首封面 | ☐ |
| **P5** | 播放状态同步 | Pause / Playing / Stopped | UI 状态实时一致 | ☐ |
| **P6** | 音量模块 | 获取和调整系统音量 | UI 与 Windows 音量双向同步 | ☐ |
| **P6** | 倒计时模块 | 新建 / 暂停 / 结束提醒 | 收起后仍正常计时 | ☐ |
| **P6** | 时钟模块 | 时间显示和格式设置 | 不造成持续高 CPU | ☐ |
| **P6** | 开机启动 | 恢复自动启动 | 开关可控、重启有效 | ☐ |
| **P6** | 截图隐藏 | 使用 Win32 原生方案 | 截图时符合设定 | ☐ |
| **P7** | 重做设置页面 | 设置窗口完全使用 Svelte UI | 不再出现 Native 设置页崩坏 | ☐ |
| **P7** | 自定义标题栏 | 拖动、关闭、最小化 | 无边框但基础窗口功能完整 | ☐ |
| **P7** | 外观设置 | 圆角、位置、功能显示等 | 修改即时预览 | ☐ |
| **P7** | 模块开关 | 音乐 / 时钟 / 倒计时等可独立显示 | 配置正确保存 | ☐ |
| **P8** | 性能治理 | 检查事件监听、Timer、Store 更新频率 | 空闲 CPU 保持低占用 | ☐ |
| **P8** | 内存治理 | 避免封面图片、事件监听泄漏 | 长时间运行无明显增长 | ☐ |
| **P8** | EXE 测试 | Debug 和 Release 对比 | Release 不出现额外问题 | ☐ |
| **P8** | 多显示器测试 | 主副屏、缩放不同、分辨率不同 | 灵动岛位置正确 | ☐ |
| **P8** | 睡眠恢复 | Windows 睡眠 / 唤醒后测试 | 不丢播放器、不丢窗口 | ☐ |
| **P9** | 清理旧 Native UI | 删除确认不再使用的 Native UI 层 | 不留下双 UI 架构 | ☐ |
| **P9** | 重构目录 | 整理 `src` / `src-tauri` | 结构清晰 | ☐ |
| **P9** | 编写开发文档 | 架构、模块、事件、状态机说明 | 后续 Codex 能直接理解 | ☐ |
| **P9** | 发布候选版 | 打包完整安装版本 | 可长期日常使用 | ☐ |

---

## 3. 推荐目录结构

```text
Wind0ws_Dynamic_island/
│
├─ src/
│  ├─ components/
│  │  ├─ island/
│  │  ├─ media/
│  │  ├─ timer/
│  │  ├─ volume/
│  │  └─ settings/
│  │
│  ├─ stores/
│  │  ├─ island.ts
│  │  ├─ media.ts
│  │  ├─ timer.ts
│  │  └─ settings.ts
│  │
│  ├─ animations/
│  ├─ routes/
│  └─ lib/
│
├─ src-tauri/
│  └─ src/
│     ├─ media/
│     │  ├─ netease.rs
│     │  ├─ smtc.rs
│     │  └─ player.rs
│     │
│     ├─ system/
│     │  ├─ volume.rs
│     │  ├─ startup.rs
│     │  ├─ screenshot.rs
│     │  └─ display.rs
│     │
│     ├─ window/
│     │  ├─ island.rs
│     │  └─ win32.rs
│     │
│     ├─ commands/
│     └─ lib.rs
│
└─ docs/
   ├─ ARCHITECTURE.md
   ├─ DEVELOPMENT_PLAN.md
   └─ STATE_MACHINE.md
```

---

## 4. 开发优先级

严格按照以下顺序推进，避免功能、UI、窗口逻辑同时修改：

```text
P0 备份 / 清点
        ↓
P1 Tauri 窗口
        ↓
P2 状态管理
        ↓
P3 静态 UI
        ↓
P4 动画
        ↓
P5 播放器
        ↓
P6 系统功能
        ↓
P7 设置页面
        ↓
P8 稳定性
        ↓
P9 清理 / 发布
```

### 约束

- **P3 完成之前，不开始复杂动画。**
- **P4 完成之前，不继续往灵动岛 UI 上堆新功能。**
- UI 不直接操作播放器或系统能力。
- Rust 后端不负责具体 UI 布局。
- 所有状态变化必须经过统一 Store / State Machine。
- Win32 只用于 Tauri / WebView 无法可靠完成的功能。

---

## 5. Milestone 1 — Tauri UI 基座

第一阶段只完成基础框架，不接入复杂业务。

- [ ] 当前 Native 项目完整备份
- [ ] 创建 Tauri 2 + Svelte 前端
- [ ] 保留现有 Rust 后端
- [ ] 灵动岛透明无边框窗口成功启动
- [ ] Always On Top
- [ ] 不进入任务栏
- [ ] 正确固定到屏幕位置
- [ ] DPI 正常
- [ ] 完成最基础黑色灵动岛胶囊
- [ ] 设置窗口能够正常打开
- [ ] Debug / Release 均不崩溃

### Milestone 1 完成后暂停

此阶段暂时不要：

- 接网易云
- 做复杂动画
- 做倒计时
- 重做所有功能
- 清理全部旧代码

先确认 **Tauri 窗口层 + Svelte UI 基础架构稳定**，再进入下一阶段。

---

## 6. Milestone 2 — 灵动岛 UI 与动画

- [ ] 完成灵动岛完整静态布局
- [ ] 完成媒体区域
- [ ] 完成右侧功能栏
- [ ] 建立 Island State Machine
- [ ] 完成 Hover 展开 / 收起
- [ ] 完成左下角锚定向右上生长
- [ ] 完成内容淡入 / 位移动画
- [ ] 完成快速 Hover 动画中断处理
- [ ] 完成 Media / Timer / Volume 状态切换
- [ ] 确认窗口尺寸变化与前端动画同步

---

## 7. Milestone 3 — 媒体系统

- [ ] 恢复 Windows SMTC
- [ ] 恢复网易云音乐专用控制方案
- [ ] 保持其他播放器原有逻辑
- [ ] 播放 / 暂停
- [ ] 上一首 / 下一首
- [ ] 歌曲名称
- [ ] 歌手
- [ ] 专辑封面
- [ ] 播放状态同步
- [ ] 切歌时 UI 不闪烁

---

## 8. Milestone 4 — 系统功能

- [ ] 系统音量
- [ ] 倒计时
- [ ] 时钟
- [ ] 开机启动
- [ ] 截图隐藏
- [ ] 多显示器
- [ ] DPI
- [ ] 睡眠 / 唤醒恢复
- [ ] 窗口位置持久化

---

## 9. Milestone 5 — 设置页面

- [ ] 设置页完全迁移到 Svelte
- [ ] 无边框窗口
- [ ] 自定义标题栏
- [ ] 最小化
- [ ] 关闭
- [ ] 拖动
- [ ] 外观设置
- [ ] 灵动岛位置设置
- [ ] 功能模块开关
- [ ] 动画相关设置
- [ ] 设置即时预览
- [ ] 配置持久化
- [ ] 设置页长时间运行不崩溃

---

## 10. Milestone 6 — 稳定性与发布

- [ ] 检查事件监听泄漏
- [ ] 检查 Timer 更新频率
- [ ] 检查 Store 无意义重复更新
- [ ] 检查封面资源释放
- [ ] 检查 WebView 内存占用
- [ ] Debug 测试
- [ ] Release 测试
- [ ] 多显示器测试
- [ ] 不同 DPI 测试
- [ ] 睡眠唤醒测试
- [ ] 长时间挂机测试
- [ ] 清理旧 Native UI
- [ ] 整理项目结构
- [ ] 完善开发文档
- [ ] 打包 Release Candidate

---

## 11. 核心开发原则

### 前端负责

- UI 布局
- 动画
- 状态展示
- 用户交互
- 设置页面
- 灵动岛视觉表现

### Rust 负责

- Windows 系统 API
- 播放器控制
- 网易云控制
- SMTC
- 音量
- 开机启动
- 截图隐藏
- 多显示器与窗口能力
- 配置持久化底层能力

### 禁止

```text
Rust 直接控制大量 UI 细节
UI 直接操作 Win32
每个功能自己修改窗口尺寸
每个组件维护一套自己的展开状态
多个模块同时控制动画
播放器状态直接耦合 UI DOM
```

统一关系：

```text
Windows / Player
      ↓
   Rust Core
      ↓
 event / invoke
      ↓
     Store
      ↓
 State Machine
      ↓
 Svelte UI
      ↓
 Animation
```

---

## 12. 最终目标

最终架构应满足：

- UI 可以快速修改
- 动画可以独立调试
- Windows 原生能力完整保留
- 网易云拥有单独控制方案
- 其他播放器维持通用控制逻辑
- 设置页面稳定
- 灵动岛与右侧功能区域视觉统一
- 不因增加新功能破坏窗口系统
- 不因修改 UI 影响播放器后端
- 后续 Codex 能根据计划逐项完成并勾选状态

---

**当前推荐起点：先在旧 Tauri 仓库执行 `P-1 两仓库迁移审计 → P0 Git 分支准备 → 再逐模块迁移 Native 功能`。**

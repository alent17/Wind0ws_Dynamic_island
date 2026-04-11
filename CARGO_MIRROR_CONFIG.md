# 配置 Rust/Cargo 国内镜像

## 问题描述
下载 Rust 依赖时超时：
```
[28] Timeout was reached (Operation too slow. Less than 10 bytes/sec transferred the last 30 seconds)
```

## 解决方案：使用国内镜像

### 方法 1：配置 Cargo 镜像（推荐）

1. 打开或创建 Cargo 配置文件：
   - Windows: `C:\Users\你的用户名\.cargo\config.toml`
   - 或者在项目目录创建 `.cargo\config.toml`

2. 添加以下配置：

```toml
# 使用中科大镜像
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"

# 或者使用清华大学镜像
# [source.crates-io]
# replace-with = 'tuna'
# 
# [source.tuna]
# registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"

# 或者使用 RustChina 镜像
# [source.crates-io]
# replace-with = 'rsproxy-sparse'
#
# [source.rsproxy-sparse]
# registry = "sparse://mirrors.rsproxy.cn/crates.io-index/"
```

### 方法 2：使用环境变量（临时）

```bash
# PowerShell
$env:CARGO_REGISTRIES_CRATES_IO_PROTOCOL="git"
$env:CARGO_NET_GIT_FETCH_WITH_CLI="true"
npm run tauri dev
```

### 方法 3：配置 Git 协议

```bash
# 配置 git 使用 https 协议
git config --global url."https://github.com/".insteadOf "git://github.com/"

# 配置 crates.io 使用 git 协议
cargo config --add registries.crates-io.protocol=git
```

### 方法 4：使用 rsproxy.cn 镜像

创建 `.cargo\config.toml` 文件：

```toml
[source.crates-io]
replace-with = 'rsproxy'

[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"

[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"

[net]
git-fetch-with-cli = true
```

## 快速操作

让我为你创建配置文件：

```bash
# 在项目目录创建 .cargo 目录
mkdir .cargo

# 创建 config.toml 文件
```

## 验证配置

配置完成后，运行：

```bash
cargo --version
cargo update
```

如果看到依赖开始下载，说明配置成功。

## 常见问题

### Q: 配置文件位置？
A: 优先级从高到低：
1. 项目目录：`C:\Users\admin\Desktop\project\Wind0ws_Dynamic_island\.cargo\config.toml`
2. 用户目录：`C:\Users\admin\.cargo\config.toml`
3. 全局配置：`C:\Users\admin\.cargo\config`

### Q: 配置后仍然很慢？
A: 尝试：
1. 切换到其他镜像源
2. 使用手机热点
3. 等待网络高峰过去

### Q: 如何恢复默认配置？
A: 删除或重命名 `config.toml` 文件即可。

---

*创建时间：2026-04-10*

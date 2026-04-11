# 解决 Cargo 文件锁问题

## 问题症状

```
Blocking waiting for file lock on package cache
```

项目启动时卡在这个消息，无法继续编译。

## 原因分析

这个问题通常由以下原因造成：
1. 多个 cargo 进程同时运行
2. 之前的 cargo 进程异常退出，锁文件未释放
3. 网络问题导致依赖下载中断，锁文件未清理

## 快速解决方案

### 方案 1：手动清理（推荐）

在 PowerShell 中执行：

```powershell
# 1. 终止所有 cargo 和 rustc 进程
taskkill /F /IM cargo.exe
taskkill /F /IM rustc.exe

# 2. 删除锁文件
Remove-Item -Path "$env:USERPROFILE\.cargo\.package-cache" -Force -ErrorAction SilentlyContinue

# 3. 清理目标目录
Remove-Item -Path "src-tauri\target" -Recurse -Force -ErrorAction SilentlyContinue

# 4. 等待 3 秒
Start-Sleep -Seconds 3

# 5. 重新启动项目
npm run tauri dev
```

### 方案 2：使用批处理脚本

创建 `fix-cargo-lock.bat` 文件：

```batch
@echo off
echo 正在清理 Cargo 锁文件...
taskkill /F /IM cargo.exe 2>nul
taskkill /F /IM rustc.exe 2>nul
del /F /Q "%USERPROFILE%\.cargo\package-cache" 2>nul
rmdir /S /Q "src-tauri\target" 2>nul
echo 等待 3 秒...
timeout /t 3 /nobreak >nul
echo 清理完成！正在启动项目...
npm run tauri dev
```

双击运行即可。

### 方案 3：使用 PowerShell 脚本

创建 `Fix-CargoLock.ps1` 文件：

```powershell
Write-Host "正在清理 Cargo 锁文件..." -ForegroundColor Yellow
taskkill /F /IM cargo.exe 2>$null
taskkill /F /IM rustc.exe 2>$null
Remove-Item -Path "$env:USERPROFILE\.cargo\.package-cache" -Force -ErrorAction SilentlyContinue
Remove-Item -Path "src-tauri\target" -Recurse -Force -ErrorAction SilentlyContinue
Write-Host "等待 3 秒..." -ForegroundColor Yellow
Start-Sleep -Seconds 3
Write-Host "清理完成！正在启动项目..." -ForegroundColor Green
npm run tauri dev
```

右键选择 "使用 PowerShell 运行"。

## 预防措施

### 1. 配置镜像源（重要）

在 `.cargo\config.toml` 中配置国内镜像，避免下载超时：

```toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"

[net]
git-fetch-with-cli = true
```

### 2. 不要同时运行多个 cargo 进程

- 不要同时打开多个终端运行 `npm run tauri dev`
- 等待一个进程完全退出后再启动新的

### 3. 正常退出应用

- 使用 Ctrl+C 停止开发服务器
- 等待进程完全退出后再重新启动

## 验证解决

成功解决后，应该看到：

```
   Compiling ...
   Finished dev [unoptimized + debuginfo] target(s) in ...
   Running `...`
```

并且应用窗口正常打开。

## 其他可能的问题

### 如果仍然卡住

1. **检查网络**：确保网络连接正常
2. **检查磁盘空间**：确保有足够的磁盘空间
3. **重启电脑**：有时重启可以解决顽固的锁文件问题
4. **更新 Rust**：`rustup update`

### 如果编译错误

可能是依赖下载不完整，执行：

```powershell
cargo clean
cargo update
npm run tauri dev
```

## 联系支持

如果问题持续，可以：
- 查看 Cargo 日志：`%USERPROFILE%\.cargo\home\registry\index`
- 在 GitHub  issue 中报告
- 在 Rust 论坛寻求帮助

---

*创建时间：2026-04-10*

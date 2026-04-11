@echo off
chcp 65001 >nul
echo ========================================
echo   Cargo 锁文件清理工具
echo ========================================
echo.
echo 正在清理 Cargo 锁文件...
echo.

echo [1/5] 终止 cargo 进程...
taskkill /F /IM cargo.exe 2>nul
if %errorlevel% equ 0 (
    echo ✓ cargo 进程已终止
) else (
    echo - 未找到 cargo 进程
)
echo.

echo [2/5] 终止 rustc 进程...
taskkill /F /IM rustc.exe 2>nul
if %errorlevel% equ 0 (
    echo ✓ rustc 进程已终止
) else (
    echo - 未找到 rustc 进程
)
echo.

echo [3/5] 清理 package-cache...
del /F /Q "%USERPROFILE%\.cargo\package-cache" 2>nul
if %errorlevel% equ 0 (
    echo ✓ package-cache 已清理
) else (
    echo - package-cache 不存在或无法删除
)
echo.

echo [4/5] 清理 target 目录...
rmdir /S /Q "src-tauri\target" 2>nul
if %errorlevel% equ 0 (
    echo ✓ target 目录已清理
) else (
    echo - target 目录不存在或无法删除
)
echo.

echo [5/5] 等待 3 秒...
timeout /t 3 /nobreak >nul
echo.

echo ========================================
echo   清理完成！
echo ========================================
echo.
echo 正在启动项目...
echo.

npm run tauri dev

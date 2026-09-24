//! 设置服务模块
//!
//! 提供应用设置的读写和开机自启动管理

use crate::error::{AppError, AppResult};
use crate::models::AppPreferences;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use tauri::{AppHandle, Manager};

const AUTOSTART_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const AUTOSTART_VALUE: &str = "Isle";
const LEGACY_AUTOSTART_VALUE: &str = "Wind0wsDynamicIsland";
const AUTOSTART_ARGUMENT: &str = "--startup";

/// 从配置文件读取设置
///
/// 配置文件位于应用数据目录下的 settings.json
/// 如果文件不存在或解析失败，返回 None
pub fn read_settings_file(app: &AppHandle) -> Option<AppPreferences> {
    let config_dir = app.path().app_data_dir().ok()?;
    let config_path = config_dir.join("settings.json");

    if !config_path.exists() {
        return None;
    }

    let content = fs::read_to_string(config_path).ok()?;
    let mut value: serde_json::Value = serde_json::from_str(&content).ok()?;
    if let Some(settings) = value.as_object_mut() {
        // Older releases allowed several families; keep the setting key but use MiSans everywhere.
        settings.insert(
            "fontId".to_string(),
            serde_json::Value::String("misans".to_string()),
        );
        // Older releases shared one topmost preference between the island
        // and floating player. Seed the new independent value from that
        // preference once so existing users keep their current behavior.
        if !settings.contains_key("floatingWindowAlwaysOnTop") {
            let was_always_on_top = settings
                .get("alwaysOnTop")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            settings.insert(
                "floatingWindowAlwaysOnTop".to_string(),
                serde_json::Value::Bool(was_always_on_top),
            );
        }
    }
    serde_json::from_value(value).ok()
}

/// 将设置写入配置文件
///
/// 自动创建配置目录（如果不存在）
/// 使用美观的 JSON 格式（带缩进）
pub fn write_settings_file(app: &AppHandle, settings: &AppPreferences) -> AppResult<()> {
    let config_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::config(format!("获取配置目录失败：{}", e)))?;

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| AppError::config(format!("创建配置目录失败：{}", e)))?;
    }

    let config_path = config_dir.join("settings.json");
    let content = serde_json::to_string_pretty(settings).map_err(AppError::Serialization)?;

    fs::write(config_path, content).map_err(AppError::Io)
}

/// 设置开机自启动
///
/// 通过修改当前用户的 Windows 注册表实现。
///
/// 启动项使用当前安装的可执行文件路径，并附带一个明确的启动参数。
/// 这样升级后可以重新写入新路径，也不会依赖 `reg.exe` 的本地化输出格式。
pub fn set_auto_start(enable: bool) -> AppResult<()> {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        if enable {
            let (run_key, _) = hkcu
                .create_subkey(AUTOSTART_KEY)
                .map_err(|e| AppError::config(format!("打开开机启动注册表失败：{}", e)))?;
            let exe_path = std::env::current_exe()
                .map_err(|e| AppError::config(format!("获取可执行文件路径失败：{}", e)))?;
            let command = startup_command(&exe_path);

            run_key
                .set_value(AUTOSTART_VALUE, &command)
                .map_err(|e| AppError::config(format!("写入开机启动项失败：{}", e)))?;

            // 清理早期版本使用的名称，避免同一个应用登录时启动两次。
            let _ = run_key.delete_value(LEGACY_AUTOSTART_VALUE);
        } else {
            match hkcu.open_subkey_with_flags(AUTOSTART_KEY, winreg::enums::KEY_SET_VALUE) {
                Ok(run_key) => {
                    for value_name in [AUTOSTART_VALUE, LEGACY_AUTOSTART_VALUE] {
                        if let Err(error) = run_key.delete_value(value_name) {
                            if error.kind() != ErrorKind::NotFound {
                                return Err(AppError::config(format!(
                                    "删除开机启动项失败：{}",
                                    error
                                )));
                            }
                        }
                    }
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(AppError::config(format!(
                        "打开开机启动注册表失败：{}",
                        error
                    )))
                }
            }
        }

        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = enable;
        Err(AppError::config("开机启动仅支持 Windows"))
    }
}

/// 检查是否已设置开机自启动
///
/// 通过查询注册表并比较启动命令中的可执行文件路径判断启动项是否有效。
pub fn get_auto_start() -> AppResult<bool> {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;

        let current_exe = std::env::current_exe()
            .map_err(|e| AppError::config(format!("获取可执行文件路径失败：{}", e)))?;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = match hkcu.open_subkey(AUTOSTART_KEY) {
            Ok(key) => key,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(AppError::config(format!(
                    "读取开机启动注册表失败：{}",
                    error
                )))
            }
        };

        for value_name in [AUTOSTART_VALUE, LEGACY_AUTOSTART_VALUE] {
            if let Ok(command) = run_key.get_value::<String, _>(value_name) {
                if normalize_windows_path(command_executable(&command))
                    == normalize_windows_path(&current_exe.to_string_lossy())
                {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

fn startup_command(exe_path: &Path) -> String {
    format!("\"{}\" {}", exe_path.display(), AUTOSTART_ARGUMENT)
}

fn command_executable(command: &str) -> &str {
    let command = command.trim();
    if let Some(quoted) = command.strip_prefix('"') {
        return quoted.split('"').next().unwrap_or_default();
    }

    command.split_whitespace().next().unwrap_or_default()
}

fn normalize_windows_path(path: &str) -> String {
    path.trim()
        .trim_matches('"')
        .replace('/', "\\")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::{command_executable, normalize_windows_path, startup_command};
    use std::path::Path;

    #[test]
    fn normalizes_quoted_windows_paths_for_registry_comparison() {
        assert_eq!(
            normalize_windows_path(r#""C:/Apps/Isle/Isle.exe""#),
            r#"c:\apps\isle\isle.exe"#
        );
    }

    #[test]
    fn extracts_executable_from_startup_command() {
        assert_eq!(
            command_executable(r#""C:\\Apps\\Isle\\isle.exe" --startup"#),
            r"C:\\Apps\\Isle\\isle.exe"
        );
        assert_eq!(
            command_executable(r#"C:\\Apps\\Isle\\isle.exe --startup"#),
            r"C:\\Apps\\Isle\\isle.exe"
        );
    }

    #[test]
    fn builds_quoted_startup_command() {
        assert_eq!(
            startup_command(Path::new(r"C:\Apps\Isle\isle.exe")),
            r#""C:\Apps\Isle\isle.exe" --startup"#
        );
    }
}

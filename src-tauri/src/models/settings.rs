//! 应用设置数据模型
//!
//! 定义用户可配置的所有设置项
//! 支持持久化存储和前端同步

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 应用程序设置
///
/// 包含所有用户可配置的选项，使用 camelCase 序列化以匹配前端命名规范
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// 主题名称（original, dark, light 等）
    pub island_theme: String,
    /// 是否自动隐藏窗口
    pub auto_hide: bool,
    /// 是否显示音频频谱
    pub show_spectrum: bool,
    /// 是否启用动画效果
    pub enable_animations: bool,
    /// 窗口不透明度 (0-255)
    pub window_opacity: u8,
    /// 是否始终置顶
    pub always_on_top: bool,
    /// 是否启用硬件加速
    pub hardware_acceleration: bool,
    /// 是否减少动画（无障碍选项）
    pub reduce_animations: bool,
    /// 是否显示调试信息
    pub show_debug_info: bool,
    /// 日志级别（Info, Debug, Trace 等）
    pub log_level: String,
    /// 当前显示器索引
    pub monitor_index: u32,
    /// 播放器优先级权重配置
    /// Key: 播放器名称，Value: 权重值
    pub player_weights: HashMap<String, u32>,
    /// 悬浮窗口 X 坐标
    pub floating_window_x: Option<i32>,
    /// 悬浮窗口 Y 坐标
    pub floating_window_y: Option<i32>,
    /// 悬浮窗口宽度
    pub floating_window_width: Option<u32>,
    /// 悬浮窗口高度
    pub floating_window_height: Option<u32>,
    /// 是否启用 MV 播放
    pub enable_mv_playback: bool,
    /// 是否锁定悬浮窗口位置
    pub lock_floating_window: bool,
    /// 是否启用高清封面
    pub enable_hd_cover: bool,
    /// 是否启用像素艺术效果
    pub enable_pixel_art: bool,
    /// 是否启用半色调效果
    pub enable_halftone: bool,
    /// 自定义缓存目录路径
    pub cache_directory: Option<String>,
    /// 是否开机自启动
    pub auto_start: bool,
    /// 是否隐藏设置按钮
    pub hide_settings_button: bool,
    /// 是否隐藏显示器选择器
    pub hide_monitor_selector: bool,
    /// 是否隐藏悬浮窗口
    pub hide_floating_window: bool,
    /// 展开状态下的圆角半径
    pub expanded_corner_radius: u32,
    /// 是否始终显示顶部栏
    pub always_show_top_bar: bool,
}

/// 默认设置实现
impl Default for AppSettings {
    fn default() -> Self {
        // 初始化播放器权重默认值
        let mut player_weights = HashMap::new();
        player_weights.insert("netease".to_string(), 50);
        player_weights.insert("spotify".to_string(), 50);
        player_weights.insert("bilibili".to_string(), 50);
        player_weights.insert("qqmusic".to_string(), 50);
        player_weights.insert("apple".to_string(), 50);
        player_weights.insert("generic".to_string(), 10);

        Self {
            island_theme: "original".to_string(),
            auto_hide: true,
            show_spectrum: true,
            enable_animations: true,
            window_opacity: 255,
            always_on_top: true,
            hardware_acceleration: true,
            reduce_animations: false,
            show_debug_info: false,
            log_level: "Info".to_string(),
            monitor_index: 0,
            player_weights,
            floating_window_x: None,
            floating_window_y: None,
            floating_window_width: None,
            floating_window_height: None,
            enable_mv_playback: true,
            lock_floating_window: false,
            enable_hd_cover: true,
            enable_pixel_art: false,
            enable_halftone: false,
            cache_directory: None,
            auto_start: false,
            hide_settings_button: false,
            hide_monitor_selector: false,
            hide_floating_window: false,
            expanded_corner_radius: 16,
            always_show_top_bar: true,
        }
    }
}

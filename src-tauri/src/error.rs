//! 统一错误处理模块
//!
//! 提供应用程序级别的错误类型定义和处理机制
//! 支持错误码、错误消息序列化，便于前后端通信

use serde::Serialize;
use std::fmt;
use std::io;
use std::string::FromUtf8Error;

/// 应用程序统一错误类型
///
/// 错误码范围规划：
/// - 1000-1999: 系统级错误（IO、序列化、锁等）
/// - 2000-2999: 业务级错误（媒体、缓存、网络等）
/// - 3000+: 自定义业务错误
#[derive(Debug)]
pub enum AppError {
    /// IO 错误：文件读写、目录操作等
    Io(io::Error),
    /// 序列化错误：JSON 解析/生成失败
    Serialization(serde_json::Error),
    /// UTF-8 转换错误：字符串编码问题
    Utf8(FromUtf8Error),
    /// 锁获取失败：Mutex 毒化或竞争
    Lock(String),
    /// 媒体信息获取失败：Windows 媒体会话访问问题
    Media(String),
    /// 缓存错误：缓存读写、清理失败
    Cache(String),
    /// 网络请求失败：HTTP 请求、API 调用错误
    Network(String),
    /// 窗口操作失败：窗口创建、显示、移动等
    Window(String),
    /// 配置错误：设置文件解析、保存失败
    Config(String),
    /// 解析错误：数据格式转换失败
    Parse(String),
    /// 资源未找到：文件、媒体、缓存不存在
    NotFound(String),
    /// 业务错误：自定义错误码和消息
    Business { code: u32, message: String },
}

/// 实现 Display trait，提供用户友好的错误消息
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO 错误: {}", e),
            AppError::Serialization(e) => write!(f, "序列化错误: {}", e),
            AppError::Utf8(e) => write!(f, "UTF-8 转换错误: {}", e),
            AppError::Lock(msg) => write!(f, "锁获取失败: {}", msg),
            AppError::Media(msg) => write!(f, "媒体信息获取失败: {}", msg),
            AppError::Cache(msg) => write!(f, "缓存错误: {}", msg),
            AppError::Network(msg) => write!(f, "网络请求失败: {}", msg),
            AppError::Window(msg) => write!(f, "窗口操作失败: {}", msg),
            AppError::Config(msg) => write!(f, "配置错误: {}", msg),
            AppError::Parse(msg) => write!(f, "解析错误: {}", msg),
            AppError::NotFound(msg) => write!(f, "资源未找到: {}", msg),
            AppError::Business { code, message } => write!(f, "[{}] {}", code, message),
        }
    }
}

/// 实现 Error trait，支持错误链
impl std::error::Error for AppError {}

/// 自动转换 IO 错误
impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        AppError::Io(e)
    }
}

/// 自动转换序列化错误
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serialization(e)
    }
}

/// 自动转换 UTF-8 错误
impl From<FromUtf8Error> for AppError {
    fn from(e: FromUtf8Error) -> Self {
        AppError::Utf8(e)
    }
}

/// 序列化实现，用于 IPC 通信传递错误信息
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("code", &self.code())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

impl AppError {
    /// 获取错误码
    ///
    /// 错误码用于前端统一处理不同类型的错误
    pub fn code(&self) -> u32 {
        match self {
            AppError::Io(_) => 1001,
            AppError::Serialization(_) => 1002,
            AppError::Utf8(_) => 1003,
            AppError::Lock(_) => 1004,
            AppError::Media(_) => 2001,
            AppError::Cache(_) => 2002,
            AppError::Network(_) => 2003,
            AppError::Window(_) => 2004,
            AppError::Config(_) => 2005,
            AppError::Parse(_) => 2006,
            AppError::NotFound(_) => 2007,
            AppError::Business { code, .. } => *code,
        }
    }

    /// 创建业务错误
    ///
    /// # 参数
    /// - `code`: 自定义错误码（建议 3000+）
    /// - `message`: 错误消息
    pub fn business(code: u32, message: impl Into<String>) -> Self {
        AppError::Business {
            code,
            message: message.into(),
        }
    }

    /// 创建锁错误
    pub fn lock(msg: impl Into<String>) -> Self {
        AppError::Lock(msg.into())
    }

    /// 创建媒体错误
    pub fn media(msg: impl Into<String>) -> Self {
        AppError::Media(msg.into())
    }

    /// 创建缓存错误
    pub fn cache(msg: impl Into<String>) -> Self {
        AppError::Cache(msg.into())
    }

    /// 创建网络错误
    pub fn network(msg: impl Into<String>) -> Self {
        AppError::Network(msg.into())
    }

    /// 创建窗口错误
    pub fn window(msg: impl Into<String>) -> Self {
        AppError::Window(msg.into())
    }

    /// 创建配置错误
    pub fn config(msg: impl Into<String>) -> Self {
        AppError::Config(msg.into())
    }

    /// 创建解析错误
    pub fn parse(msg: impl Into<String>) -> Self {
        AppError::Parse(msg.into())
    }

    /// 创建资源未找到错误
    pub fn not_found(msg: impl Into<String>) -> Self {
        AppError::NotFound(msg.into())
    }

    /// 创建 IO 错误（自定义消息）
    pub fn io(msg: impl Into<String>) -> Self {
        AppError::Io(io::Error::new(io::ErrorKind::Other, msg.into()))
    }
}

/// 自动转换 Mutex 毒化错误
///
/// 当持有 Mutex 的线程 panic 时，Mutex 会被"毒化"
impl<T> From<std::sync::PoisonError<std::sync::MutexGuard<'_, T>>> for AppError {
    fn from(_: std::sync::PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
        AppError::Lock("Mutex poisoned".to_string())
    }
}

/// 应用程序 Result 类型别名
///
/// 统一使用 AppError 作为错误类型，简化函数签名
pub type AppResult<T> = Result<T, AppError>;

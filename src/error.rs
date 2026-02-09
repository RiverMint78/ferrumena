use thiserror::Error;

/// Ferrumena 的核心错误类型
#[derive(Debug, Error)]
pub enum FerrumenaError {
    /// 1. 网络层错误
    /// 网络断开、DNS 解析失败或连接超时等触发
    #[error("网络传输失败: {0}")]
    Network(#[from] reqwest::Error),

    /// 2. 磁盘与 IO 错误
    /// 硬盘存在问题、文件夹没权限或保存图片失败时触发
    #[error("文件操作失败: {0}")]
    Io(#[from] std::io::Error),

    /// 3. DOM 解析错误
    /// 当 Philomena 网页结构发生变化，导致 CSS 选择器失效时触发
    #[error("网页解析失败: 找不到元素 [{selector}] (位置: {location})")]
    DomParse { selector: String, location: String },

    /// 4. 业务逻辑错误
    /// 例如：图片已被删除、该 ID 不存在、或者触发了 404 等
    #[error("业务逻辑错误: {0}")]
    Logic(String),

    /// 5. 环境配置错误
    /// 例如：.env 文件里没有设置 User-Agent
    #[error("配置缺失: {0}")]
    Config(String),

    /// 6. 其他未知错误
    #[error("未知错误: {0}")]
    Unknown(String),
}

/// 项目统一使用 FerrumenaError 作为错误类型
pub type Result<T> = std::result::Result<T, FerrumenaError>;

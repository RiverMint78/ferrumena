use serde::Deserialize;
use std::path::PathBuf;

/// Ferrumena 的核心配置结构体
#[derive(Debug, Deserialize)]
pub struct FerrumenaConfig {
    /// 目标站点，默认 https://trixiebooru.org/
    #[serde(default = "default_base_url")]
    pub base_url: String,

    /// 过滤器 ID，默认 100073
    #[serde(default = "default_filter_id")]
    pub filter_id: u32,

    /// 用户代理 UA
    #[serde(default = "default_user_agent")]
    pub user_agent: String,

    /// 其它 Cookie，默认为空
    #[serde(default)]
    pub cookie: String,

    /// 每秒请求数 (RPS)
    #[serde(default = "default_rps")]
    pub rps: u32,

    /// 并发下载任务数
    #[serde(default = "default_concurrency")]
    pub concurrency: u32,

    /// 文件保存路径
    #[serde(default = "default_save_path")]
    pub save_path: PathBuf,
}

// 默认值提供函数

fn default_base_url() -> String {
    "https://trixiebooru.org/".to_string()
}

fn default_filter_id() -> u32 {
    100073
}

fn default_user_agent() -> String {
    // 编译时获取 Cargo.toml 中的版本号
    format!(
        "Ferrumena/{} (+https://github.com/RiverMint78/ferrumena)",
        env!("CARGO_PKG_VERSION")
    )
}

fn default_rps() -> u32 {
    8
}

fn default_concurrency() -> u32 {
    4
}

fn default_save_path() -> PathBuf {
    PathBuf::from("./downloads")
}

// --- 实现加载逻辑 ---

impl FerrumenaConfig {
    /// 加载： Default -> .env -> Environment
    pub fn load() -> Self {
        // 1. 尝试加载 .env 文件（如果存在）
        let _ = dotenvy::dotenv();

        // 2. 尝试从环境变量（带 FERRUMENA_ 前缀）解析
        // envy 字段自动匹配 FERRUMENA_<UPPER> -> <lower>
        envy::prefixed("FERRUMENA_")
            .from_env::<FerrumenaConfig>()
            .unwrap_or_else(|e| {
                // 打印警告，使用默认值
                eprintln!("警告: 环境变量解析部分失败 ({})，将使用默认配置。", e);
                Self::default()
            })
    }
}

/// 实现 Default Trait
impl Default for FerrumenaConfig {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            filter_id: default_filter_id(),
            user_agent: default_user_agent(),
            cookie: String::new(),
            rps: default_rps(),
            concurrency: default_concurrency(),
            save_path: default_save_path(),
        }
    }
}

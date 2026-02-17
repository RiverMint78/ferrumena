use serde::Deserialize;
use std::path::PathBuf;

use crate::cli;

/// Ferrumena 的核心配置结构体
#[derive(Debug, Deserialize)]
pub struct FerrumenaConfig {
    /// 目标站点，默认 <https://trixiebooru.org>
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
    32
}

fn default_save_path() -> PathBuf {
    PathBuf::from("./downloads")
}

impl FerrumenaConfig {
    /// 加载： Default -> .env -> Environment
    pub fn load() -> Self {
        // 加载 .env 文件（如果存在）
        let _ = dotenvy::dotenv();

        envy::prefixed("FERRUMENA_")
            .from_env::<FerrumenaConfig>()
            .unwrap_or_else(|e| {
                eprintln!("警告: 环境变量解析失败 ({})，将使用默认配置。", e);
                Self::default()
            })
    }

    /// 应用命令行参数覆盖
    pub fn merge_with_cli(mut self, args: &cli::Args) -> Self {
        if let Some(ref url) = args.base_url {
            self.base_url = url.clone();
        }
        if let Some(id) = args.filter_id {
            self.filter_id = id;
        }
        if let Some(ref ua) = args.user_agent {
            self.user_agent = ua.clone();
        }
        if let Some(ref ck) = args.cookie {
            self.cookie = ck.clone();
        }
        if let Some(r) = args.rps {
            self.rps = r;
        }
        if let Some(c) = args.concurrency {
            self.concurrency = c;
        }
        if let Some(ref p) = args.save_path {
            self.save_path = p.clone();
        }

        // 不允许空 UA 和 空 URL
        if self.user_agent.trim().is_empty() {
            self.user_agent = default_user_agent();
        }
        if self.base_url.trim().is_empty() {
            self.base_url = default_base_url();
        }

        self
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

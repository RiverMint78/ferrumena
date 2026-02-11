use crate::api::models::PageResponse;
use crate::cli::Args;
use crate::config::FerrumenaConfig;
use crate::error::{FerrumenaError, Result};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use reqwest::{Client, header};
use std::num::NonZeroU32;
use std::sync::Arc;

// 速度限制器
type SharedLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

pub struct PhilomenaClient {
    pub(crate) client: Client,   // reqwest 客户端
    pub config: FerrumenaConfig, // 内部所有 config
    limiter: SharedLimiter,
}

impl PhilomenaClient {
    pub fn new(config: FerrumenaConfig) -> Result<Self> {
        let mut headers = header::HeaderMap::new();

        // 1. 设置 User-Agent
        let ua_val = header::HeaderValue::from_str(&config.user_agent)
            .map_err(|e| FerrumenaError::Config(format!("User-Agent 构建失败: {}", e)))?;
        headers.insert(header::USER_AGENT, ua_val);

        // 2. 处理 Cookie 逻辑
        let mut cookie_raw = config.cookie.clone();
        if !cookie_raw.is_empty() {
            if !cookie_raw.ends_with(';') {
                cookie_raw.push(';');
            }
            cookie_raw.push(' ');
        }
        // 注入 filter_id
        cookie_raw.push_str(&format!("filter_id={};", config.filter_id));

        // 构造 Cookie Header
        let cookie_val = header::HeaderValue::from_str(&cookie_raw)
            .map_err(|e| FerrumenaError::Config(format!("Cookie 构造失败: {}", e)))?;
        headers.insert(header::COOKIE, cookie_val);

        // 3. 构造限速器
        let rps = NonZeroU32::new(config.rps)
            .ok_or_else(|| FerrumenaError::Config("RPS (每秒请求数) 必须大于 0".to_string()))?;
        let limiter = Arc::new(RateLimiter::direct(Quota::per_second(rps)));

        // 4. 构建 Reqwest Client
        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            config,
            limiter,
        })
    }

    /// 获取首页 HTML
    pub async fn fetch_home(&self) -> Result<String> {
        self.limiter.until_ready().await;

        let resp = self
            .client
            .get(&self.config.base_url)
            .send()
            .await?
            .text()
            .await?;
        Ok(resp)
    }

    /// 获取搜索结果页（占位实现）
    pub async fn fetch_page(&self, _page: u32, _args: &Args) -> Result<PageResponse> {
        self.limiter.until_ready().await;

        // TODO: 实现真实的 API/HTML 解析逻辑
        Ok(PageResponse {
            total: 0,
            images: Vec::new(),
        })
    }
}

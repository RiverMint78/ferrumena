use crate::api::models::PageResponse;
use crate::config::FerrumenaConfig;
use crate::error::{FerrumenaError, Result};
use crate::cli::Args;
use reqwest::{Client, header};

pub struct PhilomenaClient {
    pub(crate) client: Client,   // reqwest 客户端
    pub config: FerrumenaConfig, // 内部所有 config
}

impl PhilomenaClient {
    pub fn new(config: FerrumenaConfig) -> Result<Self> {
        let mut headers = header::HeaderMap::new();

        // 1. 设置 User-Agent
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(&config.user_agent)
                .map_err(|_| FerrumenaError::Config("UA 包含非法字符".into()))?,
        );

        // 2. 处理 Cookie 逻辑
        let mut cookie_raw = config.cookie.clone();
        if !cookie_raw.is_empty() && !cookie_raw.ends_with(';') {
            cookie_raw.push(';');
        }
        // 注入 filter_id
        cookie_raw.push_str(&format!(" filter_id={};", config.filter_id));

        headers.insert(
            header::COOKIE,
            header::HeaderValue::from_str(&cookie_raw)
                .map_err(|_| FerrumenaError::Config("Cookie 构造失败".into()))?,
        );

        // 3. 构建 Reqwest Client
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(FerrumenaError::Network)?;

        Ok(Self { client, config })
    }

    /// 获取首页 HTML
    pub async fn fetch_home(&self) -> Result<String> {
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
        // TODO: 实现真实的 API/HTML 解析逻辑
        Ok(PageResponse {
            total: 0,
            images: Vec::new(),
        })
    }
}

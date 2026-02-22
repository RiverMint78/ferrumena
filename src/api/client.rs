use crate::api::models::{ImageItem, PageResponse};
use crate::cli::Args;
use crate::config::FerrumenaConfig;
use crate::error::{FerrumenaError, Result};
use crate::utils::{extract_total_from_first_page, parse_representations, pick_view_url};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use reqwest::{Client, header};
use scraper::Selector;
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

    /// 获取搜索结果页
    pub async fn fetch_page(&self, page: u32, args: &Args) -> Result<PageResponse> {
        self.limiter.until_ready().await;

        // 构建搜索 URL
        let search_url = format!(
            "{}search?page={}&per_page={}&sd={}&sf={}&filter_id={}&q={}",
            self.config.base_url,
            page,
            args.per_page,
            args.sort_direction,
            args.sort_field.to_api_string(),
            args.filter_id.unwrap_or(self.config.filter_id),
            urlencoding::encode(&args.query),
        );

        // 获取 HTML
        let html = self.client.get(&search_url).send().await?.text().await?;

        // 解析 HTML
        let document = scraper::Html::parse_document(&html);
        let selector =
            Selector::parse("div.image-container").map_err(|_| FerrumenaError::DomParse {
                selector: "div.image-container".to_string(),
                location: "search results".to_string(),
            })?;

        let mut images = Vec::new();
        for element in document.select(&selector) {
            // 提取 image-id
            let id_str =
                element
                    .value()
                    .attr("data-image-id")
                    .ok_or_else(|| FerrumenaError::DomParse {
                        selector: "data-image-id".to_string(),
                        location: "image-container".to_string(),
                    })?;

            let id: u32 = id_str
                .parse()
                .map_err(|_| FerrumenaError::Logic(format!("无效的图片 ID: {}", id_str)))?;

            // 提取 representations (data-uris 是 HTML 转义的 JSON)
            let uris_str =
                element
                    .value()
                    .attr("data-uris")
                    .ok_or_else(|| FerrumenaError::DomParse {
                        selector: "data-uris".to_string(),
                        location: "image-container".to_string(),
                    })?;

            // 反序列化 HTML 转义的 JSON (&quot; -> "), 并构建 representations 字典
            let representations = parse_representations(uris_str, id)?;

            // 从 representations 提取 view_url（优先级: full > tall > 第一个）
            let view_url = pick_view_url(&representations);

            // 从 URL 推断文件格式
            let format = view_url
                .split('.')
                .last()
                .ok_or_else(|| {
                    FerrumenaError::Logic(format!("无法推断文件格式，URL 不含扩展名: {}", view_url))
                })?
                .to_string();

            images.push(ImageItem {
                id,
                representations,
                view_url,
                format,
            });
        }

        // 在第一页提取总数信息
        let total = if page == 1 {
            Some(extract_total_from_first_page(&document)?)
        } else {
            None
        };

        Ok(PageResponse { total, images })
    }
}

use crate::error::{FerrumenaError, Result};
use scraper::Selector;
use std::collections::HashMap;

pub fn compact_url_for_log(url: &str) -> String {
    if let Some(scheme_pos) = url.find("://") {
        let rest = &url[scheme_pos + 3..];
        if let Some(path_pos) = rest.find('/') {
            return rest[path_pos..].to_string();
        }
        return "/".to_string();
    }

    url.to_string()
}

pub fn parse_representations(uris_str: &str, id: u32) -> Result<HashMap<String, String>> {
    let uris_unescaped = uris_str.replace("&quot;", "\"");
    serde_json::from_str(&uris_unescaped)
        .map_err(|e| FerrumenaError::Logic(format!("无法解析图片 ID {} 的 URIs: {}", id, e)))
}

pub fn pick_view_url(representations: &HashMap<String, String>) -> String {
    representations
        .get("full")
        .or_else(|| representations.get("tall"))
        .or_else(|| representations.values().next())
        .cloned()
        .unwrap_or_default()
}

pub fn extract_total_from_first_page(document: &scraper::Html) -> Result<u32> {
    let page_info_selector =
        Selector::parse("span.page__info strong").map_err(|_| FerrumenaError::DomParse {
            selector: "span.page__info strong".to_string(),
            location: "page info".to_string(),
        })?;

    let strong_elements: Vec<_> = document.select(&page_info_selector).collect();
    if strong_elements.len() < 2 {
        return Err(FerrumenaError::DomParse {
            selector: "span.page__info strong[1]".to_string(),
            location: "缺少第二个 <strong> 元素，无法提取总数".to_string(),
        });
    }

    let total_text = strong_elements[1].inner_html().trim().to_string();
    total_text
        .parse::<u32>()
        .map_err(|_| FerrumenaError::Logic(format!("无法解析总数信息: {}", total_text)))
}

use std::collections::HashMap;

/// 传递给下载器的图片任务
pub struct DownloadTask {
    pub id: u32,
    pub url: String,
    pub file_ext: String,
}

/// 单页图片数据
pub struct PageResponse {
    /// 总数，仅在 page=1 时提取，否则为 None
    pub total: Option<u32>,
    pub images: Vec<ImageItem>,
}

/// 图片条目
pub struct ImageItem {
    pub id: u32,
    pub representations: HashMap<String, String>,
    pub view_url: String,
    pub format: String,
}

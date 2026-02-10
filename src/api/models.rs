use std::collections::HashMap;

/// 传递给下载器的图片任务
pub struct DownloadTask {
    pub id: u32,
    pub url: String,
    pub file_ext: String,
}

/// 单页图片数据（占位）
pub struct PageResponse {
    pub total: u32,
    pub images: Vec<ImageItem>,
}

/// 图片条目（占位）
pub struct ImageItem {
    pub id: u32,
    pub representations: HashMap<String, String>,
    pub view_url: String,
    pub format: String,
}

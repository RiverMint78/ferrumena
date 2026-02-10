use clap::{Parser, ValueEnum};
use rand::RngExt;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Ferrumena: Philomena 异步下载器")]
pub struct Args {
    /// 搜索句
    /// (例: "pony OR safe")
    /// 搜索句法请参考：https://trixiebooru.org/pages/search_syntax
    #[arg(short, long, default_value = "safe")]
    pub query: String,

    /// 排序字段
    #[arg(short = 'f', long, visible_alias = "sf", value_enum, default_value_t = SortField::Id)]
    pub sort_field: SortField,

    /// 排序方向
    #[arg(short = 'd', long, visible_alias = "sd", value_enum, default_value_t = SortOrder::Desc)]
    pub sort_direction: SortOrder,

    /// 每页图片数量 (1-50)
    #[arg(
        short,
        long,
        default_value_t = 50,
        value_parser = clap::value_parser!(u32).range(1..=50)
    )]
    pub per_page: u32,

    /// 本次运行的最大下载张数
    /// 默认：所有结果
    #[arg(short, long)]
    pub limit: Option<u32>,

    /// [ENV] 目标站点 URL
    #[arg(long)]
    pub base_url: Option<String>,

    /// [ENV] 过滤器 ID
    #[arg(long)]
    pub filter_id: Option<u32>,

    /// [ENV] 用户代理 UA
    #[arg(long, visible_alias = "ua")]
    pub user_agent: Option<String>,

    /// [ENV] Cookie 字符串
    #[arg(long)]
    pub cookie: Option<String>,

    /// [ENV] 每秒请求数 (RPS)
    #[arg(short, long)]
    pub rps: Option<u32>,

    /// [ENV] 并发下载任务数
    #[arg(short, long)]
    pub concurrency: Option<u32>,

    /// [ENV] 文件保存路径
    #[arg(short = 'o', long)]
    pub save_path: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum SortField {
    Id,
    UpdatedAt,
    AspectRatio,
    Faves,
    Upvotes,
    Downvotes,
    Score,
    Relevance,
    Random,
    Width,
    Height,
    CommentCount,
    TagCount,
    Pixels,
    Size,
    Duration,
}

impl SortField {
    pub fn to_api_string(&self) -> String {
        match self {
            // Relevance -> _score
            Self::Relevance => "_score".to_string(),
            // Random -> random:随机数
            Self::Random => {
                let seed: u32 = rand::rng().random();
                format!("random:{}", seed)
            }
            // 多词字段
            Self::UpdatedAt => "updated_at".to_string(),
            Self::AspectRatio => "aspect_ratio".to_string(),
            Self::CommentCount => "comment_count".to_string(),
            Self::TagCount => "tag_count".to_string(),
            // 单词字段
            _ => format!("{:?}", self).to_lowercase(),
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum SortOrder {
    Asc,
    Desc,
}

#![allow(dead_code)]
mod api;
mod cli;
mod config;
mod error;
use clap::Parser;

#[tokio::main]
async fn main() -> error::Result<()> {
    // 加载配置
    let cfg = config::FerrumenaConfig::load();
    let args = cli::Args::parse();
    let cfg = cfg.merge_with_cli(&args);
    println!("{:#?}", cfg);

    // 初始化客户端
    let api_client = api::PhilomenaClient::new(cfg)?;

    // 处理 limit 的显示逻辑
    let limit_display = args
        .limit
        .map(|l| format!("{} 张", l))
        .unwrap_or_else(|| "无限制 (全部)".to_string());

    println!(
        "开始搜索: {}\n排序方式: {} ({:?})\n目标数量: {}\n每页抓取: {} 张",
        args.query,
        args.sort_field.to_api_string(),
        args.sort_direction,
        limit_display,
        args.per_page
    );

    match api_client.fetch_home().await {
        Ok(html) => println!(
            "{:#?}\nIs CF? : {}",
            html.len(),
            html.contains("Just a moment...")
        ),
        Err(e) => eprintln!("{:#?}", e),
    }

    Ok(())
}

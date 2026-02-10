#![allow(dead_code)]
mod api;
mod config;
mod error;

#[tokio::main]
async fn main() -> error::Result<()> {
    // 1. 加载配置
    let cfg = config::FerrumenaConfig::load();

    println!("{:#?}", cfg);

    // 2. 初始化客户端
    let api_client = match api::PhilomenaClient::new(cfg) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("初始化失败: {:#?}", e);
            std::process::exit(1);
        }
    };

    match api_client.fetch_home().await {
        Ok(html) => println!("{:#?}", html.len()),
        Err(e) => eprintln!("{:#?}", e),
    }

    Ok(())
}

mod api;
mod config;
mod error;

#[tokio::main]
async fn main() -> error::Result<()> {
    // 1. 加载配置
    let cfg = config::FerrumenaConfig::load();

    // 2. 初始化客户端
    let api_client = api::PhilomenaClient::new(cfg)?;

    if let Err(e) = api_client.fetch_home().await {
        eprint!("{:#?}", e);
    }

    Ok(())
}

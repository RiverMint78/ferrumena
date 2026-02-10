mod api;
mod config;
mod error;

#[tokio::main]
async fn main() -> error::Result<()> {
    // 1. 加载配置
    let cfg = config::FerrumenaConfig::load();

    // 2. 初始化客户端
    let api_client = api::PhilomenaClient::new(cfg)?;

    println!("客户端已就绪，目标站点：{}", api_client.config.base_url);

    // 3. 测试抓取
    let html = api_client.fetch_home().await?;
    println!("抓取成功，HTML 长度: {}", html.len());

    Ok(())
}

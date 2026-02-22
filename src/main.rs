mod api;
mod cli;
mod config;
mod downloader;
mod error;
mod utils;
use clap::Parser;
use downloader::Downloader;

#[tokio::main]
async fn main() -> error::Result<()> {
    // 加载配置
    let cfg = config::FerrumenaConfig::load();
    let args = cli::Args::parse();
    let cfg = cfg.merge_with_cli(&args);

    // 打印配置信息
    println!("================================================================");
    println!("               🦄 Ferrumena v{}", env!("CARGO_PKG_VERSION"));
    println!("        异步 Philomena 图片下载器, built with 🦀");
    println!("================================================================\n");

    println!("🔧 运行配置");
    println!("  ├─ 🌐 目标站点: {}", cfg.base_url);
    println!("  ├─ 🔍 搜索句: {}", args.query);
    println!(
        "  ├─ 📊 排序: {} ({})",
        args.sort_field.to_api_string(),
        args.sort_direction
    );
    println!("  └─ 🎫 Filter ID: {}", cfg.filter_id);

    println!("\n⚙️  性能参数");
    println!("  ├─ 📄 每页图片: {} 张", args.per_page);
    println!("  ├─ 🚀 下载并发: {} Workers", cfg.concurrency);
    println!("  └─ ⚡ 爬页限速: {} 请求/秒", cfg.rps);

    println!("\n💾 存储设置");
    let limit_display = args
        .limit
        .map(|l| format!("{} 张", l))
        .unwrap_or_else(|| "全部".to_string());
    println!("  ├─ 📁 保存路径: {}", cfg.save_path.display());
    println!("  └─ 🎯 目标数量: {}", limit_display);
    println!();

    // 初始化客户端
    println!("─────────────────────────────────────────────────────────────");
    println!("🔌 正在初始化 API 客户端...");
    let api_client = match api::PhilomenaClient::new(cfg) {
        Ok(client) => {
            println!("   ✅ 客户端初始化成功");
            client
        }
        Err(e) => {
            eprintln!("   ❌ 初始化失败: {:#?}", e);
            std::process::exit(1);
        }
    };

    // 测试连接
    println!("\n🧪 正在测试网络连接...");
    match api_client.fetch_home().await {
        Ok(html) => {
            let is_cf = html.contains("Just a moment...");
            if is_cf {
                println!("   ⚠️  检测到 Cloudflare 防护，可能需要配置 Cookie/UA 才能继续");
            } else {
                println!("   ✅ 网络连接正常，响应大小: {} bytes", html.len());
            }
        }
        Err(e) => {
            eprintln!("   ❌ 网络连接失败: {:#?}", e);
            std::process::exit(1);
        }
    }

    // 启动下载器
    println!("\n🚀 正在启动下载引擎...");
    let downloader = match Downloader::new(api_client, args).await {
        Ok(dl) => {
            println!("   ✅ 下载器初始化完成");
            dl
        }
        Err(e) => {
            eprintln!("   ❌ 下载器初始化失败: {:#?}", e);
            std::process::exit(1);
        }
    };

    println!("─────────────────────────────────────────────────────────────\n");

    match downloader.run().await {
        Ok(_) => {
            println!("\n=====================================================");
            println!("              ✨ 下载任务已完成 ✨");
            println!("         感谢使用 Ferrumena 图片下载器！");
            println!("=====================================================\n");
        }
        Err(e) => {
            eprintln!("\n=====================================================");
            eprintln!("                ❌ 下载过程中出错");
            eprintln!("=====================================================");
            eprintln!("错误详情: {:#?}\n", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

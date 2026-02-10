// 1. 声明模块
mod config;
mod error;

// 2. 引入我们需要使用的类型
use config::FerrumenaConfig;

fn main() {
    let cfg = FerrumenaConfig::load();

    println!("--- Ferrumena 初始化测试 ---");
    println!("目标站点: {}", cfg.base_url);
    println!("过滤器ID: {}", cfg.filter_id);
    println!("最大并发: {}", cfg.concurrency);
    println!("RPS限制:  {}", cfg.rps);
    println!("保存路径: {:?}", cfg.save_path);
    println!("用户代理: {}", cfg.user_agent);
    println!("Cookie: {}", cfg.cookie);
}

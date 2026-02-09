// 公开子模块
pub mod client;
pub mod models;

// 重定向导出
// 可以用 api::PhilomenaClient 替换 api::client::PhilomenaClient
pub use client::PhilomenaClient;

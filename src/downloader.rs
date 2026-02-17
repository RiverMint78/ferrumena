use crate::api::models::DownloadTask;
use crate::cli::Args;
use crate::{api::client::PhilomenaClient, error::Result};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

pub struct Downloader {
    client: Arc<PhilomenaClient>,
    args: Args,
    // 存储本地已存在的 ID，用于去重
    existing_ids: Arc<HashSet<u32>>,
}

impl Downloader {
    pub fn new(client: PhilomenaClient, args: Args) -> Result<Self> {
        // 递归路径创建
        let save_path = &client.config.save_path;
        std::fs::create_dir_all(save_path)?;

        // 扫描目录获取已有 ID
        let existing_ids = Self::scan_existing_files(save_path);
        Ok(Self {
            client: Arc::new(client),
            args,
            existing_ids: Arc::new(existing_ids),
        })
    }

    /// 扫描文件夹，提取已存在的图片 ID
    fn scan_existing_files(save_path: &Path) -> HashSet<u32> {
        let entries = match std::fs::read_dir(save_path) {
            Ok(en) => en,
            Err(err) => {
                println!("读取路径 {} 出错：{}", save_path.display(), err);
                return HashSet::new();
            }
        };
        entries
            .flatten()
            .filter(|entry| {
                // 只保留文件
                entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
            })
            .filter_map(|entry| {
                // 获取文件名 -> 获取主名 -> 转换字符串 -> 解析数字
                entry.path().file_stem()?.to_str()?.parse::<u32>().ok()
            })
            .collect()
    }

    pub async fn run(self) -> Result<()> {
        // 确定抓取范围
        let first_page = self.client.fetch_page(1, &self.args).await?;
        let total_images = first_page.total;

        // 计算实际需要抓取的图片总数
        let target_count = match self.args.limit {
            Some(l) => u32::min(l, total_images),
            None => total_images,
        };

        // 计算总页数
        let per_page = self.args.per_page;
        let total_pages = (target_count + per_page - 1) / per_page;

        println!("ℹ️ 计划抓取 {} 张图片，共 {} 页", target_count, total_pages);

        // 建立通信管道
        // mpsc 通道：Page Worker 生产图片链接，Image Worker 消费
        let (tx, rx) = mpsc::channel::<DownloadTask>(256);
        let rx = Arc::new(Mutex::new(rx));

        // 启动并行任务
        let mut worker_handles = vec![];

        // A. 页面抓取任务
        let client_c = Arc::clone(&self.client);
        let args_c = self.args.clone();
        let tx_c = tx.clone();
        let page_handle = tokio::spawn(async move {
            let mut failure_count = 0;
            let max_failures = 5;

            for page in 1..=total_pages {
                match client_c.fetch_page(page, &args_c).await {
                    Ok(resp) => {
                        failure_count = 0; // 成功, 重置计数

                        for img in resp.images {
                            let task = DownloadTask {
                                id: img.id,
                                url: img
                                    .representations
                                    .get("full")
                                    .cloned()
                                    .unwrap_or(img.view_url),
                                file_ext: img.format,
                            };
                            let _ = tx_c.send(task).await;
                        }
                    }
                    Err(e) => {
                        failure_count += 1;
                        println!(
                            "⚠️  页面 {} 抓取失败: {:#?} ({}/{})",
                            page, e, failure_count, max_failures
                        );

                        if failure_count >= max_failures {
                            eprintln!("❌ 连续失败 {} 次，停止爬取页面 No.{}", max_failures, page);
                            break;
                        }
                    }
                }
            }
            drop(tx_c); // 生产者关闭
        });
        worker_handles.push(page_handle);

        // B. 图片下载任务
        let concurrency = self.client.config.concurrency;
        let client_c = Arc::clone(&self.client);

        for i in 0..concurrency {
            let rx_c = Arc::clone(&rx);
            let existing_ids_c = Arc::clone(&self.existing_ids);
            let client_cc = Arc::clone(&client_c);

            let handle = tokio::spawn(async move {
                while let Some(task) = {
                    let mut lock = rx_c.lock().await;
                    lock.recv().await
                } {
                    // 1. 检查去重
                    if existing_ids_c.contains(&task.id) {
                        println!("⏭️  Worker {} 跳过已存在: ID {}", i, task.id);
                        continue;
                    }

                    // 2. 执行下载
                    let file_name = format!("{}.{}", task.id, task.file_ext);
                    let file_path = client_cc.config.save_path.join(&file_name);

                    match client_cc.client.get(&task.url).send().await {
                        Ok(resp) => match resp.bytes().await {
                            Ok(bytes) => match std::fs::write(&file_path, bytes) {
                                Ok(_) => println!(
                                    "💾 Worker {} 下载完成: {} (ID: {})",
                                    i, file_name, task.id
                                ),
                                Err(e) => eprintln!(
                                    "⚠️ Worker {} 保存文件失败: {} - {:#?}",
                                    i, file_name, e
                                ),
                            },
                            Err(e) => {
                                eprintln!("⚠️ Worker {} 读取响应失败: {} - {:#?}", i, file_name, e)
                            }
                        },
                        Err(e) => eprintln!("⚠️ Worker {} 下载失败: {} - {:#?}", i, file_name, e),
                    }
                }
            });
            worker_handles.push(handle);
        }

        // 等待所有任务完成
        for h in worker_handles {
            let _ = h.await;
        }

        Ok(())
    }
}

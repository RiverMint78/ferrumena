use crate::api::models::DownloadTask;
use crate::cli::Args;
use crate::{api::client::PhilomenaClient, error::Result};
use crate::utils::compact_url_for_log;
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
    pub async fn new(client: PhilomenaClient, args: Args) -> Result<Self> {
        // 递归路径创建
        let save_path = client.config.save_path.clone();
        tokio::fs::create_dir_all(&save_path).await?;

        // 扫描目录获取已有 ID
        let existing_ids = Self::scan_existing_files(&save_path).await;
        Ok(Self {
            client: Arc::new(client),
            args,
            existing_ids: Arc::new(existing_ids),
        })
    }

    /// 扫描文件夹，提取已存在的图片 ID
    async fn scan_existing_files(save_path: &Path) -> HashSet<u32> {
        let mut entries = match tokio::fs::read_dir(save_path).await {
            Ok(en) => en,
            Err(err) => {
                println!("❓  读取路径 {} 出错：{}", save_path.display(), err);
                return HashSet::new();
            }
        };

        let mut ids = HashSet::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            // 只保留文件
            if let Ok(file_type) = entry.file_type().await {
                if !file_type.is_file() {
                    continue;
                }
            }

            // 获取文件名 -> 获取主名 -> 转换字符串 -> 解析数字
            if let Some(file_stem) = entry.path().file_stem() {
                if let Some(id_str) = file_stem.to_str() {
                    if let Ok(id) = id_str.parse::<u32>() {
                        ids.insert(id);
                    }
                }
            }
        }
        ids
    }

    pub async fn run(self) -> Result<()> {
        // 确定抓取范围
        let first_page = self.client.fetch_page(1, &self.args).await?;
        let total_images = first_page.total.ok_or_else(|| {
            crate::error::FerrumenaError::Logic("第一页未获取到总数信息".to_string())
        })?;

        // 计算实际需要抓取的图片总数
        let target_count = match self.args.limit {
            Some(l) => u32::min(l, total_images),
            None => total_images,
        };

        // 计算总页数
        let per_page = self.args.per_page;
        let total_pages = (target_count + per_page - 1) / per_page;

        println!(
            "ℹ️  计划抓取 {} 张图片，共 {} 页",
            target_count, total_pages
        );

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
        let max_failures = self.client.config.max_failures;
        let representation = self.client.config.representation.clone();
        drop(tx); // 立即 drop 原始 tx，只保留 tx_c
        let page_handle = tokio::spawn(async move {
            let mut failure_count: u32 = 0;

            for page in 1..=total_pages {
                match client_c.fetch_page(page, &args_c).await {
                    Ok(resp) => {
                        failure_count = 0; // 成功, 重置计数

                        for img in resp.images {
                            let url = if let Some(url) =
                                img.representations.get(representation.as_str()).cloned()
                            {
                                url
                            } else {
                                let compact_view_url = compact_url_for_log(&img.view_url);
                                println!(
                                    "⚠️  图片 ID {} 不存在 representation='{}'，已回退到 view_url: {}",
                                    img.id, representation, compact_view_url
                                );
                                img.view_url.clone()
                            };

                            let task = DownloadTask {
                                id: img.id,
                                url,
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
                            println!("❌  连续失败 {} 次，停止爬取页面 No.{}", max_failures, page);
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
                loop {
                    // 从 channel 接收任务
                    let task = {
                        let mut lock = rx_c.lock().await;
                        lock.recv().await
                    };

                    // 如果 channel 已关闭，退出循环
                    let task = match task {
                        Some(t) => t,
                        None => break,
                    };

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
                            Ok(bytes) => match tokio::fs::write(&file_path, bytes).await {
                                Ok(_) => println!(
                                    "💾  Worker {} 下载完成: {} (ID: {})",
                                    i, file_name, task.id
                                ),
                                Err(e) => println!(
                                    "⚠️  Worker {} 保存文件失败: {} - {:#?}",
                                    i, file_name, e
                                ),
                            },
                            Err(e) => {
                                println!("⚠️  Worker {} 读取响应失败: {} - {:#?}", i, file_name, e)
                            }
                        },
                        Err(e) => println!("⚠️  Worker {} 下载失败: {} - {:#?}", i, file_name, e),
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

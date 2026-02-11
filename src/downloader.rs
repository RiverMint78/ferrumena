use crate::api::models::DownloadTask;
use crate::cli::Args;
use crate::{api::client::PhilomenaClient, error::FerrumenaError};
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
    pub fn new(client: PhilomenaClient, args: Args) -> Result<Self, FerrumenaError> {
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

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
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

        println!("计划抓取 {} 张图片，共 {} 页", target_count, total_pages);

        // 建立通信管道
        // mpsc 通道：Page Worker 生产图片链接，Image Worker 消费
        let (tx, rx) = mpsc::channel::<DownloadTask>(100);
        let rx = Arc::new(Mutex::new(rx));

        // 启动并行任务
        let mut worker_handles = vec![];

        // A. 页面抓取任务
        let client_c = Arc::clone(&self.client);
        let args_c = self.args.clone();
        let tx_c = tx.clone();
        let page_handle = tokio::spawn(async move {
            for page in 1..=total_pages {
                if let Ok(resp) = client_c.fetch_page(page, &args_c).await {
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
                } else {
                    // TODO: 累计失败限度逻辑
                }
            }
            drop(tx_c); // 生产者关闭
        });
        worker_handles.push(page_handle);

        // B. 图片下载任务 (启动多个并发 Worker)
        let concurrency = 4; // 可从配置读取
        for i in 0..concurrency {
            let rx_c = Arc::clone(&rx);
            let existing_ids_c = Arc::clone(&self.existing_ids);

            let handle = tokio::spawn(async move {
                while let Some(task) = {
                    let mut lock = rx_c.lock().await;
                    lock.recv().await
                } {
                    // 1. 检查去重
                    if existing_ids_c.contains(&task.id) {
                        continue;
                    }

                    // 2. 执行下载
                    println!("Worker {} 正在下载 ID: {}", i, task.id);
                    // TODO: 调用 reqwest 下载并保存
                    // if fails > limit { break; }
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

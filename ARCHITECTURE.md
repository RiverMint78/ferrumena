# 项目文件架构

```plaintext
ferrumena/
├── .env                # 环境变量
├── .gitignore          # gitignore
├── Cargo.toml          # Cargo.toml
├── LICENSE             # LICENSE
└── src/                # 业务逻辑核心
    ├── main.rs         # 入口文件：负责启动运行时，串联各功能模块
    ├── cli.rs          # 命令行接口：定义参数解析逻辑（Clap）
    ├── config.rs       # 配置管理：读取环境变量或配置文件
    ├── error.rs        # 错误处理：定义项目统一的自定义错误枚举
    ├── api.rs          # API 模块入口：负责导出 api/ 下的子模块
    ├── api/
    │   ├── client.rs   # 核心客户端：封装 Reqwest，处理连接与频率限制
    │   └── models.rs   # 数据模型：基于 Serde 的 JSON 映射结构体
    ├── downloader.rs   # 下载引擎：高性能异步 IO，管理并发任务
    └── utils.rs        # 工具集：文件名脱敏、路径转换等通用小工具
```

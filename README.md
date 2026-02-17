# 🦄 Ferrumena

**Ferrumena** 是一个高性能的异步 Philomena 图片批量下载器，使用 Rust 和 Tokio 构建。支持所有 Philomena-based 图片站点（Derpibooru、Ponerpics 等）。

![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)
![License](https://img.shields.io/badge/License-Apache%202.0-blue)
![Status](https://img.shields.io/badge/Status-Beta-yellow)

## ✨ 主要特性

- 🚀 **完全异步** - 基于 Tokio 的高并发下载引擎
- 🌐 **多站点支持** - 适配所有 Philomena-based 网站
- 🔍 **灵活搜索** - 支持 Philomena 搜索语法和无登录过滤器
- ⚡ **智能限速** - 内置请求限速和并发控制，友好访问，且不使用API
- 📊 **详细日志** - 完整的运行状态和错误提示

---

## 🚀 快速开始

### 方式一：直接下载预编译版本（推荐新手）

1. 前往 [Releases 页面](https://github.com/RiverMint78/ferrumena/releases)
2. 下载最新版本的 `.exe` 文件：
   - `ferrumena-v0.2.0-beta-windows-x64.exe` - 性能优化版
   - `ferrumena-v0.2.0-beta-windows-x64-small.exe` - 小体积版

3. 在下载后的目录打开 PowerShell/CMD，执行：

   ```bash
   .\ferrumena.exe -q "搜索句" -l 10
   ```

---

### 方式二：从源码编译

#### 前置条件

- 安装 [Rust](https://rustup.rs/)（自动包含 Cargo）
- Windows 10+ 或其他操作系统

#### 编译步骤

```bash
# 克隆仓库
git clone https://github.com/RiverMint78/ferrumena.git
cd ferrumena

# 编译（开发版）
cargo build
./target/debug/ferrumena.exe -q "搜索句"

# 编译（性能最优）
cargo build --release
./target/release/ferrumena.exe -q "搜索句"

# 编译（小体积版）
cargo build --profile dist
./target/dist/ferrumena.exe -q "搜索句"
```

---

## 📖 使用教程

### 基础用法

```bash
# 搜索并下载前10张图片
ferrumena.exe -q "pony" -l 10

# 查看完整命令行帮助
ferrumena.exe --help
```

### 完整命令参数

```bash
ferrumena.exe [OPTIONS]
```

#### 搜索和排序参数

| 参数 | 短名 | 说明 | 默认值 | 取值范围 |
|------|------|------|--------|---------|
| `--query` | `-q` | 搜索句（Philomena 句法） | `safe` | 任意搜索表达式 |
| `--limit` | `-l` | 本次运行的最大下载张数 | 全部结果 | 正整数 |
| `--sort-field` | `-f` / `--sf` | 排序字段 | `id` | 见下表 |
| `--sort-direction` | `-d` / `--sd` | 排序方向 | `desc` | `asc` / `desc` |
| `--per-page` | `-p` | 每页图片数（推荐50） | `50` | 1-50 |

#### 排序字段详解

| 字段 | 说明 |
|------|------|
| `id` | 图片 ID（默认） |
| `random` | 随机排序（由 Ferrumena 随机产生种子） |
| `updated-at` | 最后更新时间 |
| `creation-date` | 创建日期 |
| `score` | 评分 |
| `faves` | 收藏数 |
| `upvotes` | 点赞数 |
| `downvotes` | 点踩数 |
| `relevance` | 关联度 |
| `aspect-ratio` | 宽高比 |
| `width` | 宽度 |
| `height` | 高度 |
| `comment-count` | 评论数 |
| `tag-count` | 标签数 |
| `pixels` | 像素数 |
| `size` | 文件大小 |
| `duration` | 时长 |

#### 站点和网络参数

| 参数 | 说明 | 默认值 | 来源优先级 |
|------|------|--------|---------|
| `--base-url` | 目标站点 URL | `https://trixiebooru.org/` | .env → 环境变量 → 命令行 |
| `--filter-id` | 过滤器 ID（内容分级控制） | `100073` | .env → 环境变量 → 命令行 |
| `--user-agent` / `--ua` | 自定义 User-Agent | `Ferrumena/v版本号` | .env → 环境变量 → 命令行 |
| `--cookie` | Cookie 字符串（用于登录等） | 空 | .env → 环境变量 → 命令行 |
| `--rps` / `-r` | 每秒请求数（RPS 限速） | `8` | .env → 环境变量 → 命令行 |
| `--concurrency` / `-c` | 并发下载任务数 | `32` | .env → 环境变量 → 命令行 |
| `--save-path` / `-o` | 文件保存路径 | `./downloads` | .env → 环境变量 → 命令行 |

#### 常见过滤器 ID

| ID | 描述 | 用途 |
|---|---|---|
| `56027` | Everything（完全不过滤） | 下载所有内容 |
| `100073` | Default（默认） | 默认内容过滤 |
| 其他 | 自定义过滤 | 访问网站 `/filters` 查看 |

### 高级例子

```bash
# 下载评分最高的200张图片
ferrumena.exe -q "pony" -l 200 -f score -d desc

# 按随机顺序下载50张，高并发
ferrumena.exe -q "cute" -l 50 -f random -c 64

# 下载最新上传的图片（评分>=100）
ferrumena.exe -q "pony, score.gte:100" -l 100 -f updated-at -d desc

# 切换到 Derpibooru
ferrumena.exe -q "safe, -grimdark, score.gte:500" \
  --base-url "https://derpibooru.org/" \
  --filter-id 56027 \
  -l 100

# 自定义并发和速率（快速下载）
ferrumena.exe -q "pony" -l 500 -c 64 -r 16 -o "D:/my_downloads/"

# 使用特定用户代理和 Cookie（通过登录身份下载）
ferrumena.exe -q "*" --user-agent "MyCustomUA/1.0" --cookie "user_remember_me=xxx; filter_id=xxx..." -l 50
```

---

## ⚙️ 配置文件

### 配置加载优先级

Ferrumena 按以下顺序加载配置（后面的覆盖前面的）：

1. **代码默认值** - 硬编码的默认值
2. **`.env` 文件** - 项目目录下的 `.env` 文件（如存在）
3. **环境变量** - 系统/Shell 环境变量（前缀 `FERRUMENA_`）
4. **命令行参数** - CLI 参数（最高优先级）

**示例：**

```bash
# .env 中设置
FERRUMENA_RPS=8
FERRUMENA_CONCURRENCY=32

# 环境变量覆盖 .env
export FERRUMENA_CONCURRENCY=64

# 命令行参数覆盖一切
ferrumena.exe -q "pony" -c 128  # 并发数为 128
```

### 环境变量和 .env 配置

复制 `.env.example` 为 `.env`：

```bash
cp .env.example .env
```

编辑 `.env` 文件：

```dotenv
# === 站点配置 ===

# 目标站点 URL（默认: https://trixiebooru.org/）
FERRUMENA_BASE_URL=https://trixiebooru.org/

# 过滤器 ID（默认: 100073）
# - 100073: Default（默认）
# - 56027: Everything（完全不过滤）
# 更多过滤器请访问目标站点的 /filters 页面
FERRUMENA_FILTER_ID=100073

# 登录 Cookie
# 从浏览器开发者工具复制 Cookie 字符串
FERRUMENA_COOKIE=

# === 身份识别 ===

# User-Agent（默认: Ferrumena/版本号）
# 若被阻止可尝试自定义
FERRUMENA_USER_AGENT=

# === 频率限制与并发 ===

# 每秒请求数 (RPS)，默认: 8
# 范围建议: 4-16（根据网站限制调整）
# 值越高请求越快，但可能被识别为爬虫和被限流
FERRUMENA_RPS=8

# 并发下载任务数，默认: 32
# 值越高下载越快，但会占用更多内存和网络带宽
FERRUMENA_CONCURRENCY=32

# === 存储配置 ===

# 图片下载后的存放目录，默认: ./downloads
# 支持绝对路径和相对路径
FERRUMENA_SAVE_PATH=./downloads
```

---

## 📊 实际使用例子

> 强烈推荐查看 <https://trixiebooru.org/pages/search_syntax> 了解全部搜索句法

### 例子 1：备份高分作品

```bash
ferrumena.exe -q "score.gte:500, pony" -l 100 -f score -d desc
```

下载评分在 500 以上、带有 pony 标签的前 100 张图片。

### 例子 2：收集最新上传

```bash
ferrumena.exe -q "created_at.gte:1 month ago" -l 50
```

下载最近一个月上传的前 50 张图片。

### 例子 3：大量下载（设置高并发）

编辑 `.env`：

```dotenv
# 或者通过命令行参数，请查看 --help
FERRUMENA_RPS=16
FERRUMENA_CONCURRENCY=64
```

然后执行：

```bash
ferrumena.exe -q "pony" -l 1000
```

---

## ⚠️ 常见问题

### Q：下载中断了怎么办？

**A：** Ferrumena 不支持断点续传。重新运行相同命令会重新开始下载。

### Q：提示 "检测到 Cloudflare 防护" 怎么办？

**A：**

1. 手动访问目标网站一次（通过 Cloudflare 验证）
2. 复制浏览器的 Cookie 到 `.env` 中的 `FERRUMENA_COOKIE`
3. 重试下载

### Q：怎样只下载指定类型的文件？

**A：** 某些 Philomena 站点支持搜索语法过滤，例如：

```bash
ferrumena.exe -q "mime_type:*gif"
```

### Q：为什么下载很慢？

**A：** 检查 `.env` 中的配置：

- 增加 `FERRUMENA_CONCURRENCY` 增加并发数（或者通过命令行）
- 增加 `FERRUMENA_RPS` 提高请求频率（或者通过命令行）
- 检查网络连接

### Q：支持 Linux/Mac 吗？

**A：** 支持。从源码编译即可。

---

## 📋 文件结构

```
ferrumena/
├── Cargo.toml              # 项目配置
├── src/
│   ├── main.rs            # 主程序
│   ├── api/               # API 客户端
│   ├── cli/               # 命令行参数
│   ├── config/            # 配置管理
│   ├── downloader/        # 下载引擎
│   └── error/             # 错误处理
├── .env.example           # 配置模板
├── downloads/             # 默认下载文件夹
└── README.md             # 本文件
```

---

## 🤝 贡献

本项目为个人练习，暂时不接受PR。

欢迎提出任何问题。

---

## 📜 许可证

本项目采用 **Apache License 2.0** 许可证。详见 [LICENSE](LICENSE) 文件。

---

## ⭐ 致谢

感谢以下项目的支持：

- [Tokio](https://tokio.rs/) - 异步运行时
- [Reqwest](https://github.com/seanmonstar/reqwest) - HTTP 客户端
- [Clap](https://github.com/clap-rs/clap) - 命令行参数解析
- [Scraper](https://github.com/causal-agent/scraper) - HTML 解析

---

## 📞 联系方式

遇到问题？

- 📧 Email: <67481978@qq.com>
- 🐙 GitHub: [@RiverMint78](https://github.com/RiverMint78)

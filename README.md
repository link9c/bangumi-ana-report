# Bangumi 收藏分析报告生成器

一个 Rust CLI 工具，用于分析 Bangumi 用户收藏数据并生成精美的 HTML 报告。

## 功能特性

- 获取用户收藏数据（动画、书籍、游戏、音乐、真人）
- 统计分析：评分分布、标签分析、时间线、完成率等
- AI 驱动的品味分析（使用 OpenAI 兼容 API）
- 生成单文件 HTML 报告，内嵌 Chart.js 图表
- 深色主题，现代设计，响应式布局

## 安装

```bash
cargo build --release
```

## 配置

复制 `.env.example` 为 `.env` 并填入配置：

```bash
cp .env.example .env
```

配置项：
- `BGM_TOKEN`: Bangumi 访问令牌
- `AI_API_KEY`: OpenAI 兼容 API 密钥
- `AI_BASE_URL`: API 基础 URL（默认：https://api.openai.com/v1）
- `AI_MODEL`: 使用的模型（默认：gpt-4o-mini）

## 使用

```bash
# 使用指定用户名
./target/release/bangumi-ana-report --username your_username

# 使用当前登录用户
./target/release/bangumi-ana-report

# 指定输出文件
./target/release/bangumi-ana-report --username your_username --output my_report.html

# 详细输出
./target/release/bangumi-ana-report --username your_username --verbose
```

## 报告内容

1. 用户信息概览
2. 收藏类型分布（饼图）
3. 收藏状态分布（柱状图）
4. 评分分析（直方图、平均分、最高评分作品）
5. 标签云 / 热门标签
6. 收藏时间线
7. AI 生成的品味分析
8. 生成时间戳

## 技术栈

- Rust
- reqwest (HTTP 客户端)
- serde (JSON 序列化)
- tokio (异步运行时)
- clap (CLI 参数解析)
- Chart.js (图表库)

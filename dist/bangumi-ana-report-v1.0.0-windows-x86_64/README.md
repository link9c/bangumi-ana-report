# Bangumi 收藏分析报告生成器

一个 Rust CLI 工具，用于分析 Bangumi 用户收藏数据并生成精美的 HTML 报告。

## 功能特性

- 获取用户收藏数据（动画、书籍、游戏、音乐、真人）
- 统计分析：评分分布、标签分析、时间线、完成率等
- AI 驱动的品味分析（使用 OpenAI 兼容 API）
- 生成单文件 HTML 报告，内嵌 Chart.js 图表
- 深色主题，现代设计，响应式布局

## 快速开始（Release 二进制，推荐）

无需编译，直接下载可执行文件使用。

1) 下载
- 前往 Releases 页面获取预编译二进制：https://github.com/link9c/bangumi-ana-report/releases
- 选择与你系统架构匹配的压缩包（文件名以发布页为准），例如：
  - Windows x86_64：bangumi-ana-report-vX.Y.Z-windows-x86_64.zip
  - macOS Apple Silicon (arm64)：bangumi-ana-report-vX.Y.Z-macos-aarch64.zip
  - macOS Intel (x86_64)：bangumi-ana-report-vX.Y.Z-macos-x86_64.zip
  - Linux x86_64：bangumi-ana-report-vX.Y.Z-linux-x86_64.tar.gz

2) 解压
- Windows：右键解压 zip，得到 bangumi-ana-report.exe
- macOS/Linux：使用 unzip 或 tar 解压，得到可执行文件 bangumi-ana-report

3) 配置凭据
- 必需：BGM_TOKEN（Bangumi 访问令牌）
- 可选：AI_API_KEY（启用 AI 品味分析）、AI_BASE_URL（默认 https://api.openai.com/v1）、AI_MODEL（默认 gpt-4o-mini）

方式 A：使用 .env 文件（推荐）
- 复制 .env.example 为 .env，并填写至少 BGM_TOKEN

Linux/macOS:
```
cp .env.example .env
# 编辑 .env：
BGM_TOKEN=your_bangumi_token
AI_API_KEY=sk-xxxx                 # 可选
AI_BASE_URL=https://api.openai.com/v1  # 可选
AI_MODEL=gpt-4o-mini               # 可选
```

Windows PowerShell:
```
copy .env.example .env
# 用记事本/编辑器修改 .env
```

方式 B：使用环境变量（无需 .env）
- Linux/macOS：`BGM_TOKEN=your_token ./bangumi-ana-report --username your_username`
- Windows PowerShell：`$env:BGM_TOKEN="your_token"; .\bangumi-ana-report.exe --username your_username`

4) 运行示例

- Linux/macOS（使用 .env）：
```
./bangumi-ana-report --username your_username --output report.html
```

- Windows PowerShell（使用 .env）：
```
.\bangumi-ana-report.exe --username your_username --output report.html
```

- 不传 --username 时，会使用 BGM_TOKEN 对应的当前账号：
```
# Linux/macOS
./bangumi-ana-report
# Windows
.\bangumi-ana-report.exe
```

- 启用 AI 品味分析（设置 AI_API_KEY 即可）：
```
# Linux/macOS（一次性设置环境变量）
AI_API_KEY=sk-xxxx ./bangumi-ana-report --username your_username

# Windows PowerShell
$env:AI_API_KEY="sk-xxxx"; .\bangumi-ana-report.exe --username your_username
```

5) 将可执行文件加入 PATH（可选）
- macOS/Linux：
```
chmod +x ./bangumi-ana-report
sudo mv ./bangumi-ana-report /usr/local/bin/
# 或 mv 到 ~/.local/bin 并确保 PATH 已包含该目录
```
- Windows：将 bangumi-ana-report.exe 所在目录加入系统环境变量 Path。

6) 平台注意事项
- macOS Gatekeeper：若提示“无法验证开发者”
```
xattr -d com.apple.quarantine ./bangumi-ana-report
# 或 右键 -> 打开 -> 仍要打开
```
- Windows SmartScreen：如出现拦截，点击“更多信息”->“仍要运行”
```
Unblock-File .\bangumi-ana-report.exe
```

7) 校验下载文件（如发布页提供 SHA256）
- Linux/macOS：`sha256sum <文件>` 或 `shasum -a 256 <文件>`
- Windows PowerShell：`Get-FileHash <文件> -Algorithm SHA256`

## 命令行参数

- `-u, --username <name>` 指定 Bangumi 用户名（不提供时使用 BGM_TOKEN 对应账号）
- `-o, --output <path>` 指定输出 HTML 文件路径（默认：report.html）
- `-v, --verbose`      显示更详细的日志
- `-h, --help`         查看帮助信息
- `-V, --version`      查看版本号

## 配置

复制 `.env.example` 为 `.env` 并填入配置：

```
cp .env.example .env
```

配置项：
- `BGM_TOKEN`（必需）：Bangumi 访问令牌
- `AI_API_KEY`（可选）：OpenAI 兼容 API 密钥（启用 AI 品味分析）
- `AI_BASE_URL`（可选）：API 基础 URL（默认：https://api.openai.com/v1）
- `AI_MODEL`（可选）：使用的模型（默认：gpt-4o-mini）

## 开发者构建

如果你想从源码构建：

```
cargo build --release
# 二进制在：./target/release/bangumi-ana-report
```

## 使用（开发构建二进制）

```
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
7. AI 生成的品味分析（如启用）
8. 生成时间戳

## 技术栈

- Rust
- reqwest (HTTP 客户端)
- serde (JSON 序列化)
- tokio (异步运行时)
- clap (CLI 参数解析)
- Chart.js (图表库)

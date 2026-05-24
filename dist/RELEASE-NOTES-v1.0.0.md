# v1.0.0 — Bangumi 收藏分析报告生成器

这是首个公开 Release，提供 Linux 与 Windows 预编译二进制，可直接运行生成 HTML 报告。

## 主要功能
- 获取用户收藏数据（动画、书籍、游戏、音乐、真人）
- 统计分析：评分分布、标签分析、时间线、完成率等
- AI 驱动的品味分析（可选，使用 OpenAI 兼容 API）
- 生成单文件 HTML 报告（内嵌 Chart.js，深色主题，响应式）

## 下载
- Linux x86_64：`bangumi-ana-report-v1.0.0-linux-x86_64.tar.gz`
- Windows x86_64：`bangumi-ana-report-v1.0.0-windows-x86_64.zip`

SHA256 校验：
```
5266fc7cdb87395fa57fcaf65e9df3bab61641574fa9ef75ebe2ca083c98acaf  bangumi-ana-report-v1.0.0-linux-x86_64.tar.gz
025398f2b54dcdb3ca9f1918f96f0621e2c670f8e15939f2d26a8e40db35e3b7  bangumi-ana-report-v1.0.0-windows-x86_64.zip
```

## 快速开始（Release 二进制）
1) 解压
- Windows：右键解压 zip，得到 `bangumi-ana-report.exe`
- Linux：`tar -xzf bangumi-ana-report-v1.0.0-linux-x86_64.tar.gz`

2) 配置凭据
- 必需：`BGM_TOKEN`（Bangumi 访问令牌）
- 可选：`AI_API_KEY`、`AI_BASE_URL`（默认 `https://api.openai.com/v1`）、`AI_MODEL`（默认 `gpt-4o-mini`）

使用 .env（推荐）：
```
cp .env.example .env
# 编辑 .env 至少填入：BGM_TOKEN=your_bangumi_token
```

或用环境变量（Linux 示例）：
```
BGM_TOKEN=your_token ./bangumi-ana-report --username your_username --output report.html
```

3) 运行示例
- Linux：`./bangumi-ana-report --username your_username --output report.html`
- Windows PowerShell：`./bangumi-ana-report.exe --username your_username --output report.html`

不传 `--username` 时将使用 `BGM_TOKEN` 对应账号。

更多用法与平台注意事项，请见 README。

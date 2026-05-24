use anyhow::Result;
use clap::Parser;
use colored::Colorize;

mod ai;
mod analysis;
mod api;
mod report;

#[derive(Parser, Debug)]
#[command(name = "bangumi-ana-report")]
#[command(version = "1.0.1")]
#[command(about = "Bangumi 用户收藏分析报告生成器", long_about = None)]
struct Cli {
    /// Bangumi 用户名（默认使用当前登录用户）
    #[arg(short, long)]
    username: Option<String>,

    /// 输出 HTML 文件路径
    #[arg(short, long, default_value = "report.html")]
    output: String,

    /// 显示详细输出
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    // Load environment variables
    let bgm_token = std::env::var("BGM_TOKEN").unwrap_or_else(|_| {
        eprintln!(
            "{}",
            "警告: 未设置 BGM_TOKEN 环境变量，请在 .env 文件中配置".yellow()
        );
        String::new()
    });

    if bgm_token.is_empty() {
        eprintln!(
            "{}",
            "错误: 需要 BGM_TOKEN 才能访问 Bangumi API，请在 .env 文件中配置".red()
        );
        std::process::exit(1);
    }

    let ai_api_key = std::env::var("AI_API_KEY").ok();
    let ai_base_url = std::env::var("AI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let ai_model = std::env::var("AI_MODEL")
        .unwrap_or_else(|_| "gpt-4o-mini".to_string());

    println!("{}", "🎵 Bangumi 收藏分析报告生成器".cyan().bold());
    println!("{}", "=".repeat(40).dimmed());

    // Create API client
    let client = api::BangumiClient::new(&bgm_token);

    // Get user info
    let username = if let Some(ref username) = cli.username {
        println!("{} {}", "正在获取用户信息:".green(), username);
        username.clone()
    } else {
        println!("{}", "正在获取当前用户信息...".green());
        let me = client.get_me().await?;
        println!("{} {} ({})", "当前用户:".green(), me.nickname, me.username);
        me.username.clone()
    };

    // Fetch collections
    println!("{}", "正在获取收藏数据...".green());
    let collections = client
        .get_user_collections(&username, None, None, cli.verbose)
        .await?;

    if collections.is_empty() {
        eprintln!("{}", "警告: 未找到任何收藏数据".yellow());
    } else {
        println!("{} {} 条收藏记录", "获取成功:".green(), collections.len());
    }

    // Get full user info for report
    let user = if cli.username.is_some() {
        client.get_user(&username).await?
    } else {
        client.get_me().await?
    };

    // Analyze collections
    println!("{}", "正在分析收藏数据...".green());
    let analysis_result = analysis::analyze_collections(&collections);

    // AI analysis
    let ai_analysis = if let Some(ref api_key) = ai_api_key {
        if !api_key.is_empty() && api_key != "your_ai_api_key" {
            println!("{}", "正在进行 AI 品味分析...".green());
            let summary = analysis::format_analysis_summary(&analysis_result, &username);

            match ai::get_ai_analysis(api_key, &ai_base_url, &ai_model, &summary).await {
                Ok(result) => {
                    println!("{}", "AI 分析完成!".green());
                    Some(result)
                }
                Err(e) => {
                    eprintln!("{} {}", "AI 分析失败:".yellow(), e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        if cli.verbose {
            println!("{}", "未配置 AI_API_KEY，跳过 AI 分析".yellow());
        }
        None
    };

    // Generate report
    println!("{}", "正在生成 HTML 报告...".green());
    let html = report::generate_html_report(&user, &analysis_result, ai_analysis.as_deref());

    // Write to file
    std::fs::write(&cli.output, &html)?;
    println!(
        "{} {}",
        "报告已生成:".green().bold(),
        std::fs::canonicalize(&cli.output)
            .unwrap_or_else(|_| cli.output.clone().into())
            .display()
    );

    println!("{}", "=".repeat(40).dimmed());
    println!("{}", "✅ 完成!".green().bold());

    Ok(())
}

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    max_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageContent,
}

#[derive(Debug, Deserialize)]
struct ChatMessageContent {
    content: String,
}

pub async fn get_ai_analysis(
    api_key: &str,
    base_url: &str,
    model: &str,
    summary: &str,
) -> Result<String> {
    let client = Client::new();

    let prompt = format!(
        "你是一个专业的文化品味分析师。以下是一个用户的 Bangumi（番组计划）收藏数据统计信息。\n\n\
         请用中文从以下几个方面进行深入分析，写一篇详细而有趣的报告：\n\n\
         1. **品味画像**：根据收藏类型和评分，总结用户的整体品味特征\n\
         2. **类型偏好分析**：分析用户在不同类型（动画、书籍、游戏等）上的偏好\n\
         3. **评分倾向分析**：用户的评分是否偏高或偏低，评分标准如何\n\
         4. **标签偏好分析**：从热门标签推断用户的兴趣领域和审美偏好\n\
         5. **收藏习惯**：从完成率、收藏时间线等分析用户的使用习惯\n\
         6. **推荐建议**：基于以上分析，给出 3-5 个具体的推荐方向\n\n\
         请用轻松有趣的语气撰写，可以适当使用比喻和形象的描述。\
         每个部分使用 ### 标题，整体结构清晰。\n\n\
         以下是用户的收藏数据：\n\n{}",
        summary
    );

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "你是一个专业的文化品味分析师，擅长从数据中洞察个人偏好和品味特征。请用中文回答。".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ],
        temperature: 0.7,
        max_tokens: std::env::var("AI_MAX_TOKENS")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(6000),
    };

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to call AI API")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("AI API error: {} - {}", status, body);
    }

    let chat_resp: ChatResponse = resp
        .json()
        .await
        .context("Failed to parse AI response")?;

    let content = chat_resp
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "AI 分析未返回结果".to_string());

    Ok(content)
}

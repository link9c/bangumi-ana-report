use crate::analysis::AnalysisResult;
use crate::api::User;
use chrono::Local;

pub fn generate_html_report(
    user: &User,
    analysis: &AnalysisResult,
    ai_analysis: Option<&str>,
) -> String {
    let avatar_url = user
        .avatar
        .as_ref()
        .and_then(|a| a.large.as_deref().or(a.medium.as_deref().or(a.small.as_deref())))
        .unwrap_or("https://bgm.tv/img/no_icon_subject.png");

    let sign = user.sign.as_deref().unwrap_or("");
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Type distribution
    let type_labels = serde_json::to_string(&analysis.type_distribution.keys().collect::<Vec<_>>()).unwrap_or_default();
    let type_values = serde_json::to_string(&analysis.type_distribution.values().collect::<Vec<_>>()).unwrap_or_default();

    // Status distribution - flatten for chart
    let mut status_labels = Vec::new();
    let mut status_values = Vec::new();
    let status_names = ["想看", "看过", "在看", "搁置", "抛弃"];
    for (type_name, status_map) in &analysis.status_distribution {
        status_labels.push(type_name.clone());
        for status_name in &status_names {
            status_values.push(status_map.get(*status_name).copied().unwrap_or(0));
        }
    }
    let status_labels_json = serde_json::to_string(&status_labels).unwrap_or_default();
    let status_values_json = serde_json::to_string(&status_values).unwrap_or_default();

    // Rating distribution
    let rating_labels: Vec<String> = (0..=10).map(|i| i.to_string()).collect();
    let rating_labels_json = serde_json::to_string(&rating_labels).unwrap_or_default();
    let rating_values_json = serde_json::to_string(&analysis.rating_distribution).unwrap_or_default();

    // Timeline
    let timeline_labels: Vec<&str> = analysis.timeline.iter().map(|t| t.month.as_str()).collect();
    let timeline_values: Vec<usize> = analysis.timeline.iter().map(|t| t.count).collect();
    let timeline_labels_json = serde_json::to_string(&timeline_labels).unwrap_or_default();
    let timeline_values_json = serde_json::to_string(&timeline_values).unwrap_or_default();

    // Top tags
    let tag_labels: Vec<&str> = analysis.top_tags.iter().take(15).map(|(name, _)| name.as_str()).collect();
    let tag_values: Vec<usize> = analysis.top_tags.iter().take(15).map(|(_, count)| *count).collect();
    let tag_labels_json = serde_json::to_string(&tag_labels).unwrap_or_default();
    let tag_values_json = serde_json::to_string(&tag_values).unwrap_or_default();

    // Release year distribution
    let year_labels: Vec<&str> = analysis.release_year_dist.iter().map(|(y, _)| y.as_str()).collect();
    let year_values: Vec<usize> = analysis.release_year_dist.iter().map(|(_, c)| *c).collect();
    let year_labels_json = serde_json::to_string(&year_labels).unwrap_or_default();
    let year_values_json = serde_json::to_string(&year_values).unwrap_or_default();

    // Rank distribution
    let rank_labels = serde_json::to_string(&vec!["Top 100", "Top 500", "Top 1000", "有排名", "无排名"]).unwrap_or_default();
    let rank_values = serde_json::to_string(&vec![
        analysis.rank_distribution.top_100,
        analysis.rank_distribution.top_500,
        analysis.rank_distribution.top_1000,
        analysis.rank_distribution.ranked,
        analysis.rank_distribution.unranked,
    ]).unwrap_or_default();

    // Score comparison chart data
    let score_labels: Vec<String> = analysis.hidden_gems.iter().chain(analysis.overrated.iter())
        .map(|s| s.name_cn.as_deref().filter(|n| !n.is_empty()).unwrap_or(&s.name).to_string())
        .collect();
    let score_labels_json = serde_json::to_string(&score_labels).unwrap_or_default();
    let user_scores: Vec<f64> = analysis.hidden_gems.iter().chain(analysis.overrated.iter())
        .map(|s| s.user_score as f64)
        .collect();
    let user_scores_json = serde_json::to_string(&user_scores).unwrap_or_default();
    let community_scores: Vec<f64> = analysis.hidden_gems.iter().chain(analysis.overrated.iter())
        .map(|s| s.community_score)
        .collect();
    let community_scores_json = serde_json::to_string(&community_scores).unwrap_or_default();

    // Top rated items HTML with cover images
    let top_rated_html: String = analysis
        .top_rated
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let display_name = item
                .name_cn
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(&item.name);
            let cover = item.cover_url.as_deref().unwrap_or("");
            let community = item.community_score
                .map(|s| format!("{:.1}", s))
                .unwrap_or_else(|| "-".to_string());
            let rank_str = item.rank
                .map(|r| if r > 0 { format!("#{}", r) } else { "-".to_string() })
                .unwrap_or_else(|| "-".to_string());
            let badge_class = if item.rating >= 8 { "high" } else if item.rating >= 5 { "mid" } else { "low" };
            let cover_html = if !cover.is_empty() {
                format!(r#"<img src="{}" class="item-cover" alt="{}" loading="lazy" onerror="this.style.display='none'">"#, cover, display_name)
            } else {
                String::new()
            };
            format!(
                r#"<tr>
                    <td>{}</td>
                    <td class="item-name-cell">
                        {}
                        <div>
                            <a href="https://bgm.tv/subject/{}" target="_blank">{}</a>
                            <span class="item-type">{}</span>
                        </div>
                    </td>
                    <td><span class="score-badge badge-{}">{}</span></td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>"#,
                i + 1,
                cover_html,
                item.subject_id,
                display_name,
                item.subject_type,
                badge_class,
                item.rating,
                community,
                rank_str
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Score comparison HTML
    let score_comparison_html: String = {
        let mut html = String::new();
        if !analysis.hidden_gems.is_empty() {
            html.push_str(r#"<div class="score-group"><h3>💎 沧海遗珠（你的评分 >> 社区评分）</h3><div class="score-list">"#);
            for item in &analysis.hidden_gems {
                let display_name = item.name_cn.as_deref().filter(|s| !s.is_empty()).unwrap_or(&item.name);
                html.push_str(&format!(
                    r#"<div class="score-item gem">
                        <a href="https://bgm.tv/subject/{}" target="_blank">{}</a>
                        <span class="score-compare">你 {} vs 社区 {:.1} <span class="diff-pos">+{:.1}</span></span>
                    </div>"#,
                    item.subject_id, display_name, item.user_score, item.community_score, item.diff
                ));
            }
            html.push_str("</div></div>");
        }
        if !analysis.overrated.is_empty() {
            html.push_str(r#"<div class="score-group"><h3>📉 过誉之作（你的评分 << 社区评分）</h3><div class="score-list">"#);
            for item in &analysis.overrated {
                let display_name = item.name_cn.as_deref().filter(|s| !s.is_empty()).unwrap_or(&item.name);
                html.push_str(&format!(
                    r#"<div class="score-item overrated">
                        <a href="https://bgm.tv/subject/{}" target="_blank">{}</a>
                        <span class="score-compare">你 {} vs 社区 {:.1} <span class="diff-neg">{:.1}</span></span>
                    </div>"#,
                    item.subject_id, display_name, item.user_score, item.community_score, item.diff
                ));
            }
            html.push_str("</div></div>");
        }
        if html.is_empty() {
            html = "<p>暂无足够评分数据进行对比分析</p>".to_string();
        }
        html
    };

    // AI analysis HTML
    let ai_html = if let Some(ai) = ai_analysis {
        let html = markdown_to_html(ai);
        format!(
            r#"<section class="section" id="ai-analysis">
                <h2>🤖 AI 品味分析</h2>
                <div class="ai-content">{}</div>
            </section>"#,
            html
        )
    } else {
        String::new()
    };

    // Stats
    let completion_rate_str = format!("{:.1}", analysis.completion_stats.completion_rate);
    let book_completion_str = format!("{:.1}", analysis.book_completion.completion_rate);
    let avg_community_str = format!("{:.1}", analysis.avg_community_score);
    let avg_popularity_str = format!("{:.0}", analysis.popularity_stats.avg_collection_total);

    let template = include_str!("template.html");

    template
        .replace("{{AVATAR_URL}}", avatar_url)
        .replace("{{USER_NICKNAME}}", &user.nickname)
        .replace("{{USER_SIGN}}", sign)
        .replace("{{TOTAL_COUNT}}", &analysis.total_count.to_string())
        .replace("{{AVERAGE_RATING}}", &format!("{:.1}", analysis.average_rating))
        .replace("{{MEDIAN_RATING}}", &format!("{:.1}", analysis.median_rating))
        .replace("{{COMPLETION_RATE}}", &completion_rate_str)
        .replace("{{BOOK_COMPLETION_RATE}}", &book_completion_str)
        .replace("{{AVG_COMMUNITY_SCORE}}", &avg_community_str)
        .replace("{{AVG_POPULARITY}}", &avg_popularity_str)
        .replace("{{PRIVATE_COUNT}}", &analysis.private_count.to_string())
        .replace("{{COMMENT_COUNT}}", &analysis.comment_count.to_string())
        .replace("{{RANK_TOP100}}", &analysis.rank_distribution.top_100.to_string())
        .replace("{{RANK_TOP500}}", &analysis.rank_distribution.top_500.to_string())
        .replace("{{RANK_TOP1000}}", &analysis.rank_distribution.top_1000.to_string())
        .replace("{{TOP_RATED_HTML}}", &top_rated_html)
        .replace("{{SCORE_COMPARISON_HTML}}", &score_comparison_html)
        .replace("{{AI_ANALYSIS_HTML}}", &ai_html)
        .replace("{{GENERATED_TIME}}", &now)
        .replace("{{TYPE_LABELS}}", &type_labels)
        .replace("{{TYPE_VALUES}}", &type_values)
        .replace("{{RATING_LABELS}}", &rating_labels_json)
        .replace("{{RATING_VALUES}}", &rating_values_json)
        .replace("{{STATUS_LABELS}}", &status_labels_json)
        .replace("{{STATUS_VALUES}}", &status_values_json)
        .replace("{{TAG_LABELS}}", &tag_labels_json)
        .replace("{{TAG_VALUES}}", &tag_values_json)
        .replace("{{TIMELINE_LABELS}}", &timeline_labels_json)
        .replace("{{TIMELINE_VALUES}}", &timeline_values_json)
        .replace("{{YEAR_LABELS}}", &year_labels_json)
        .replace("{{YEAR_VALUES}}", &year_values_json)
        .replace("{{RANK_LABELS}}", &rank_labels)
        .replace("{{RANK_VALUES}}", &rank_values)
        .replace("{{SCORE_LABELS}}", &score_labels_json)
        .replace("{{USER_SCORES}}", &user_scores_json)
        .replace("{{COMMUNITY_SCORES}}", &community_scores_json)
}

fn markdown_to_html(md: &str) -> String {
    let mut html = String::new();
    let mut in_list = false;

    for line in md.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("### ") {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str(&format!("<h3>{}</h3>\n", &trimmed[4..]));
        } else if trimmed.starts_with("## ") {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str(&format!("<h3>{}</h3>\n", &trimmed[3..]));
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            if !in_list {
                html.push_str("<ul>\n");
                in_list = true;
            }
            let content = process_inline(&trimmed[2..]);
            html.push_str(&format!("  <li>{}</li>\n", content));
        } else if trimmed.is_empty() {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
        } else {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            let content = process_inline(trimmed);
            html.push_str(&format!("<p>{}</p>\n", content));
        }
    }

    if in_list {
        html.push_str("</ul>\n");
    }

    html
}

fn process_inline(text: &str) -> String {
    let mut output = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '*' && chars.peek() == Some(&'*') {
            chars.next();
            let mut bold_text = String::new();
            loop {
                match chars.next() {
                    Some('*') if chars.peek() == Some(&'*') => {
                        chars.next();
                        break;
                    }
                    Some(c) => bold_text.push(c),
                    None => break,
                }
            }
            output.push_str(&format!("<strong>{}</strong>", bold_text));
        } else {
            output.push(c);
        }
    }

    output
}

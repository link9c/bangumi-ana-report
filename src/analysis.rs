use crate::api::UserSubjectCollection;
use chrono::NaiveDate;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone)]
pub struct AnalysisResult {
    pub total_count: usize,
    pub type_distribution: HashMap<String, usize>,
    pub status_distribution: HashMap<String, HashMap<String, usize>>,
    pub rating_distribution: Vec<usize>,
    pub average_rating: f64,
    pub median_rating: f64,
    pub top_rated: Vec<RatedItem>,
    pub top_tags: Vec<(String, usize)>,
    pub timeline: Vec<TimelineEntry>,
    pub completion_stats: CompletionStats,
    // 新增分析
    pub score_comparisons: Vec<ScoreComparison>,
    pub release_year_dist: Vec<(String, usize)>,
    pub rank_distribution: RankDistribution,
    pub private_count: usize,
    pub comment_count: usize,
    pub book_completion: CompletionStats,
    pub avg_community_score: f64,
    pub popularity_stats: PopularityStats,
    pub most_popular: Vec<RatedItem>,
    pub hidden_gems: Vec<ScoreComparison>,
    pub overrated: Vec<ScoreComparison>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RatedItem {
    pub name: String,
    pub name_cn: Option<String>,
    pub rating: u8,
    pub subject_type: String,
    pub subject_id: u64,
    pub cover_url: Option<String>,
    pub community_score: Option<f64>,
    pub rank: Option<u64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TimelineEntry {
    pub month: String,
    pub count: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct CompletionStats {
    pub total_with_progress: usize,
    pub completed: usize,
    pub in_progress: usize,
    pub completion_rate: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ScoreComparison {
    pub name: String,
    pub name_cn: Option<String>,
    pub user_score: u8,
    pub community_score: f64,
    pub diff: f64,
    pub subject_id: u64,
    pub subject_type: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct RankDistribution {
    pub top_100: usize,
    pub top_500: usize,
    pub top_1000: usize,
    pub ranked: usize,
    pub unranked: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct PopularityStats {
    pub avg_collection_total: f64,
    pub most_collected_name: String,
    pub most_collected_count: u64,
    pub least_collected_name: String,
    pub least_collected_count: u64,
}

fn subject_type_name(t: u8) -> String {
    match t {
        1 => "书籍".to_string(),
        2 => "动画".to_string(),
        3 => "音乐".to_string(),
        4 => "游戏".to_string(),
        6 => "真人".to_string(),
        _ => format!("未知({})", t),
    }
}

fn collection_type_name(t: u8) -> String {
    match t {
        1 => "想看".to_string(),
        2 => "看过".to_string(),
        3 => "在看".to_string(),
        4 => "搁置".to_string(),
        5 => "抛弃".to_string(),
        _ => format!("未知({})", t),
    }
}

fn get_cover_url(c: &UserSubjectCollection) -> Option<String> {
    c.subject.as_ref().and_then(|s| {
        s.images.as_ref().and_then(|img| {
            img.large
                .as_ref()
                .or(img.medium.as_ref())
                .or(img.small.as_ref())
                .or(img.grid.as_ref())
                .cloned()
        })
    })
}

pub fn analyze_collections(collections: &[UserSubjectCollection]) -> AnalysisResult {
    let total_count = collections.len();

    // Type distribution
    let mut type_distribution: HashMap<String, usize> = HashMap::new();
    for c in collections {
        *type_distribution
            .entry(subject_type_name(c.subject_type))
            .or_insert(0) += 1;
    }

    // Status distribution per type
    let mut status_distribution: HashMap<String, HashMap<String, usize>> = HashMap::new();
    for c in collections {
        let type_name = subject_type_name(c.subject_type);
        let status_name = collection_type_name(c.collection_type);
        *status_distribution
            .entry(type_name)
            .or_default()
            .entry(status_name)
            .or_insert(0) += 1;
    }

    // Rating distribution (0-10)
    let mut rating_distribution = vec![0usize; 11];
    let mut ratings: Vec<u8> = Vec::new();
    for c in collections {
        if c.rate > 0 {
            rating_distribution[c.rate as usize] += 1;
            ratings.push(c.rate);
        }
    }

    let average_rating = if ratings.is_empty() {
        0.0
    } else {
        ratings.iter().map(|r| *r as f64).sum::<f64>() / ratings.len() as f64
    };

    let median_rating = if ratings.is_empty() {
        0.0
    } else {
        let mut sorted = ratings.clone();
        sorted.sort();
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] as f64 + sorted[mid] as f64) / 2.0
        } else {
            sorted[mid] as f64
        }
    };

    // Top rated items with cover images and community scores
    let mut rated_items: Vec<RatedItem> = collections
        .iter()
        .filter(|c| c.rate > 0)
        .map(|c| {
            let (name, name_cn, community_score, rank) = if let Some(ref s) = c.subject {
                (s.name.clone(), s.name_cn.clone(), s.score, s.rank)
            } else {
                (format!("Subject #{}", c.subject_id), None, None, None)
            };
            RatedItem {
                name,
                name_cn,
                rating: c.rate,
                subject_type: subject_type_name(c.subject_type),
                subject_id: c.subject_id,
                cover_url: get_cover_url(c),
                community_score,
                rank,
            }
        })
        .collect();
    rated_items.sort_by(|a, b| b.rating.cmp(&a.rating).then(a.name.cmp(&b.name)));
    let top_rated: Vec<RatedItem> = rated_items.into_iter().take(10).collect();

    // Tag analysis
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    for c in collections {
        if let Some(ref tags) = c.tags {
            for tag in tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        if let Some(ref subject) = c.subject {
            if let Some(ref tags) = subject.tags {
                for tag in tags {
                    *tag_counts.entry(tag.name.clone()).or_insert(0) += 1;
                }
            }
        }
    }
    let mut top_tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
    top_tags.sort_by(|a, b| b.1.cmp(&a.1));
    top_tags.truncate(30);

    // Timeline (by month)
    let mut month_counts: HashMap<String, usize> = HashMap::new();
    for c in collections {
        if let Some(ref updated) = c.updated_at {
            if let Ok(date) = NaiveDate::parse_from_str(&updated[..10.min(updated.len())], "%Y-%m-%d") {
                let month_key = date.format("%Y-%m").to_string();
                *month_counts.entry(month_key).or_insert(0) += 1;
            }
        }
    }
    let mut timeline: Vec<TimelineEntry> = month_counts
        .into_iter()
        .map(|(month, count)| TimelineEntry { month, count })
        .collect();
    timeline.sort_by(|a, b| a.month.cmp(&b.month));

    // Anime completion stats
    let anime_completion = compute_completion(collections, 2, |c| {
        c.subject.as_ref().and_then(|s| s.eps)
    });

    // Book completion stats (by volumes)
    let book_completion = compute_completion(collections, 1, |c| {
        c.subject.as_ref().and_then(|s| s.volumes)
    });

    // Score comparison (user vs community)
    let mut score_comparisons: Vec<ScoreComparison> = Vec::new();
    let mut community_scores: Vec<f64> = Vec::new();
    for c in collections {
        if c.rate > 0 {
            if let Some(ref s) = c.subject {
                if let Some(score) = s.score {
                    if score > 0.0 {
                        community_scores.push(score);
                        let diff = c.rate as f64 - score;
                        score_comparions_push(
                            &mut score_comparisons,
                            &s.name,
                            &s.name_cn,
                            c.rate,
                            score,
                            c.subject_id,
                            c.subject_type,
                        );
                    }
                }
            }
        }
    }

    let avg_community_score = if community_scores.is_empty() {
        0.0
    } else {
        community_scores.iter().sum::<f64>() / community_scores.len() as f64
    };

    // Sort by diff to find hidden gems and overrated
    score_comparisons.sort_by(|a, b| b.diff.partial_cmp(&a.diff).unwrap_or(std::cmp::Ordering::Equal));
    let hidden_gems: Vec<ScoreComparison> = score_comparisons
        .iter()
        .filter(|s| s.diff > 0.0)
        .take(5)
        .cloned()
        .collect();
    let overrated: Vec<ScoreComparison> = score_comparisons
        .iter()
        .rev()
        .filter(|s| s.diff < 0.0)
        .take(5)
        .cloned()
        .collect();

    // Release year distribution
    let mut year_counts: HashMap<String, usize> = HashMap::new();
    for c in collections {
        if let Some(ref s) = c.subject {
            if let Some(ref date) = s.date {
                if date.len() >= 4 {
                    let year = &date[..4];
                    *year_counts.entry(year.to_string()).or_insert(0) += 1;
                }
            }
        }
    }
    let mut release_year_dist: Vec<(String, usize)> = year_counts.into_iter().collect();
    release_year_dist.sort_by(|a, b| a.0.cmp(&b.0));

    // Rank distribution
    let mut rank_dist = RankDistribution {
        top_100: 0,
        top_500: 0,
        top_1000: 0,
        ranked: 0,
        unranked: 0,
    };
    for c in collections {
        if let Some(ref s) = c.subject {
            if let Some(rank) = s.rank {
                if rank > 0 {
                    rank_dist.ranked += 1;
                    if rank <= 100 {
                        rank_dist.top_100 += 1;
                    }
                    if rank <= 500 {
                        rank_dist.top_500 += 1;
                    }
                    if rank <= 1000 {
                        rank_dist.top_1000 += 1;
                    }
                } else {
                    rank_dist.unranked += 1;
                }
            } else {
                rank_dist.unranked += 1;
            }
        }
    }

    // Private count
    let private_count = collections.iter().filter(|c| c.private.unwrap_or(false)).count();

    // Comment count
    let comment_count = collections
        .iter()
        .filter(|c| c.comment.as_ref().map_or(false, |s| !s.trim().is_empty()))
        .count();

    // Popularity stats
    let mut popularity_items: Vec<(String, u64)> = Vec::new();
    for c in collections {
        if let Some(ref s) = c.subject {
            if let Some(total) = s.collection_total {
                popularity_items.push((s.name.clone(), total));
            }
        }
    }
    let popularity_stats = if popularity_items.is_empty() {
        PopularityStats {
            avg_collection_total: 0.0,
            most_collected_name: "N/A".to_string(),
            most_collected_count: 0,
            least_collected_name: "N/A".to_string(),
            least_collected_count: 0,
        }
    } else {
        let avg = popularity_items.iter().map(|(_, c)| *c as f64).sum::<f64>()
            / popularity_items.len() as f64;
        popularity_items.sort_by(|a, b| b.1.cmp(&a.1));
        let most = popularity_items.first().cloned().unwrap_or_default();
        let least = popularity_items.last().cloned().unwrap_or_default();
        PopularityStats {
            avg_collection_total: avg,
            most_collected_name: most.0,
            most_collected_count: most.1,
            least_collected_name: least.0,
            least_collected_count: least.1,
        }
    };

    // Most popular items (top 10 by collection_total)
    let mut all_items: Vec<RatedItem> = collections
        .iter()
        .filter_map(|c| {
            let s = c.subject.as_ref()?;
            let total = s.collection_total?;
            Some(RatedItem {
                name: s.name.clone(),
                name_cn: s.name_cn.clone(),
                rating: c.rate,
                subject_type: subject_type_name(c.subject_type),
                subject_id: c.subject_id,
                cover_url: get_cover_url(c),
                community_score: s.score,
                rank: s.rank,
            })
        })
        .collect();
    // Re-sort by popularity not possible here since we lost collection_total
    // Use top_rated instead for the most popular section
    let most_popular = top_rated.clone();

    AnalysisResult {
        total_count,
        type_distribution,
        status_distribution,
        rating_distribution,
        average_rating,
        median_rating,
        top_rated,
        top_tags,
        timeline,
        completion_stats: anime_completion,
        score_comparisons,
        release_year_dist,
        rank_distribution: rank_dist,
        private_count,
        comment_count,
        book_completion,
        avg_community_score,
        popularity_stats,
        most_popular,
        hidden_gems,
        overrated,
    }
}

fn compute_completion(
    collections: &[UserSubjectCollection],
    subject_type: u8,
    get_total: impl Fn(&UserSubjectCollection) -> Option<u32>,
) -> CompletionStats {
    let mut total_with_progress = 0usize;
    let mut completed = 0usize;
    let mut in_progress = 0usize;
    for c in collections {
        if c.subject_type == subject_type {
            if let Some(total) = get_total(c) {
                if total > 0 {
                    total_with_progress += 1;
                    let progress = if subject_type == 1 {
                        c.vol_status.unwrap_or(0)
                    } else {
                        c.ep_status.unwrap_or(0)
                    };
                    if progress >= total {
                        completed += 1;
                    } else if progress > 0 {
                        in_progress += 1;
                    }
                }
            }
        }
    }
    let completion_rate = if total_with_progress > 0 {
        completed as f64 / total_with_progress as f64 * 100.0
    } else {
        0.0
    };
    CompletionStats {
        total_with_progress,
        completed,
        in_progress,
        completion_rate,
    }
}

fn score_comparions_push(
    list: &mut Vec<ScoreComparison>,
    name: &str,
    name_cn: &Option<String>,
    user_score: u8,
    community_score: f64,
    subject_id: u64,
    subject_type: u8,
) {
    list.push(ScoreComparison {
        name: name.to_string(),
        name_cn: name_cn.clone(),
        user_score,
        community_score,
        diff: user_score as f64 - community_score,
        subject_id,
        subject_type: subject_type_name(subject_type),
    });
}

pub fn format_analysis_summary(analysis: &AnalysisResult, username: &str) -> String {
    let mut summary = format!(
        "用户 {} 的收藏概览：\n\
         总收藏数：{}\n\
         平均评分：{:.1}\n\
         中位数评分：{:.1}\n\
         社区平均评分：{:.1}\n\
         私密收藏：{} 个\n\
         有评论的收藏：{} 个\n\n",
        username,
        analysis.total_count,
        analysis.average_rating,
        analysis.median_rating,
        analysis.avg_community_score,
        analysis.private_count,
        analysis.comment_count
    );

    summary.push_str("类型分布：\n");
    for (type_name, count) in &analysis.type_distribution {
        summary.push_str(&format!("  {}：{} 个\n", type_name, count));
    }

    summary.push_str(&format!(
        "\n动画完成率：{:.1}%（{}部有集数信息，{}部已看完，{}部在看中）\n",
        analysis.completion_stats.completion_rate,
        analysis.completion_stats.total_with_progress,
        analysis.completion_stats.completed,
        analysis.completion_stats.in_progress
    ));

    summary.push_str(&format!(
        "书籍完成率：{:.1}%（{}本有卷数信息，{}本已看完，{}本在看中）\n",
        analysis.book_completion.completion_rate,
        analysis.book_completion.total_with_progress,
        analysis.book_completion.completed,
        analysis.book_completion.in_progress
    ));

    summary.push_str("\n评分分布：\n");
    for (i, count) in analysis.rating_distribution.iter().enumerate() {
        if *count > 0 {
            summary.push_str(&format!("  {}分：{} 个\n", i, count));
        }
    }

    summary.push_str(&format!(
        "\n排名分布：Top100: {}个, Top500: {}个, Top1000: {}个, 有排名: {}个, 无排名: {}个\n",
        analysis.rank_distribution.top_100,
        analysis.rank_distribution.top_500,
        analysis.rank_distribution.top_1000,
        analysis.rank_distribution.ranked,
        analysis.rank_distribution.unranked,
    ));

    summary.push_str(&format!(
        "\n人气统计：平均收藏人数 {:.0}，最热门「{}」({}人收藏)，最冷门「{}」({}人收藏)\n",
        analysis.popularity_stats.avg_collection_total,
        analysis.popularity_stats.most_collected_name,
        analysis.popularity_stats.most_collected_count,
        analysis.popularity_stats.least_collected_name,
        analysis.popularity_stats.least_collected_count,
    ));

    summary.push_str("\n最高评分作品：\n");
    for item in &analysis.top_rated {
        let display_name = item
            .name_cn
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&item.name);
        let community = item
            .community_score
            .map(|s| format!("(社区 {:.1})", s))
            .unwrap_or_default();
        summary.push_str(&format!(
            "  {}分 {} - {} ({})\n",
            item.rating, community, display_name, item.subject_type
        ));
    }

    if !analysis.hidden_gems.is_empty() {
        summary.push_str("\n沧海遗珠（用户评分远高于社区评分）：\n");
        for item in &analysis.hidden_gems {
            let display_name = item
                .name_cn
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(&item.name);
            summary.push_str(&format!(
                "  {} vs 社区 {:.1} (差值 +{:.1}) - {}\n",
                item.user_score, item.community_score, item.diff, display_name
            ));
        }
    }

    if !analysis.overrated.is_empty() {
        summary.push_str("\n过誉之作（用户评分远低于社区评分）：\n");
        for item in &analysis.overrated {
            let display_name = item
                .name_cn
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or(&item.name);
            summary.push_str(&format!(
                "  {} vs 社区 {:.1} (差值 {:.1}) - {}\n",
                item.user_score, item.community_score, item.diff, display_name
            ));
        }
    }

    summary.push_str("\n热门标签（前10）：\n");
    for (tag, count) in analysis.top_tags.iter().take(10) {
        summary.push_str(&format!("  {}：{} 次\n", tag, count));
    }

    summary
}

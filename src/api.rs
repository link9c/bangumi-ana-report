use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

const BANGUMI_API_BASE: &str = "https://api.bgm.tv";

#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub nickname: String,
    pub avatar: Option<Avatar>,
    pub sign: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Avatar {
    pub large: Option<String>,
    pub medium: Option<String>,
    pub small: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserSubjectCollection {
    pub subject_id: u64,
    pub subject_type: u8,
    pub rate: u8,
    #[serde(rename = "type")]
    pub collection_type: u8,
    pub comment: Option<String>,
    pub tags: Option<Vec<String>>,
    pub ep_status: Option<u32>,
    pub vol_status: Option<u32>,
    pub updated_at: Option<String>,
    pub private: Option<bool>,
    pub subject: Option<SlimSubject>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SlimSubject {
    pub id: u64,
    #[serde(rename = "type")]
    pub subject_type: u8,
    pub name: String,
    pub name_cn: Option<String>,
    pub short_summary: Option<String>,
    pub date: Option<String>,
    pub images: Option<SubjectImages>,
    pub volumes: Option<u32>,
    pub eps: Option<u32>,
    pub collection_total: Option<u64>,
    pub score: Option<f64>,
    pub rank: Option<u64>,
    pub tags: Option<Vec<SubjectTag>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SubjectImages {
    pub large: Option<String>,
    pub medium: Option<String>,
    pub small: Option<String>,
    pub grid: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SubjectTag {
    pub name: String,
    pub count: u64,
}

#[derive(Debug, Deserialize)]
struct PagedResponse<T> {
    total: u64,
    data: Vec<T>,
}

pub struct BangumiClient {
    client: Client,
    token: String,
}

impl BangumiClient {
    pub fn new(token: &str) -> Self {
        let client = Client::builder()
            .user_agent("bangumi-ana-report/1.0")
            .build()
            .expect("Failed to create HTTP client");
        Self {
            client,
            token: token.to_string(),
        }
    }

    pub async fn get_me(&self) -> Result<User> {
        let url = format!("{}/v0/me", BANGUMI_API_BASE);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .context("Failed to fetch user info")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get user info: {} - {}", status, body);
        }

        let user: User = resp.json().await.context("Failed to parse user info")?;
        Ok(user)
    }

    pub async fn get_user(&self, username: &str) -> Result<User> {
        let url = format!("{}/v0/users/{}", BANGUMI_API_BASE, username);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .context("Failed to fetch user info")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get user {}: {} - {}", username, status, body);
        }

        let user: User = resp.json().await.context("Failed to parse user info")?;
        Ok(user)
    }

    pub async fn get_user_collections(
        &self,
        username: &str,
        subject_type: Option<u8>,
        collection_type: Option<u8>,
        verbose: bool,
    ) -> Result<Vec<UserSubjectCollection>> {
        let mut all_collections = Vec::new();
        let mut offset = 0u64;
        let limit = 50u64;
        let mut total = 0u64;

        loop {
            let mut url = format!(
                "{}/v0/users/{}/collections?limit={}&offset={}",
                BANGUMI_API_BASE, username, limit, offset
            );

            if let Some(st) = subject_type {
                url.push_str(&format!("&subject_type={}", st));
            }
            if let Some(ct) = collection_type {
                url.push_str(&format!("&type={}", ct));
            }

            if verbose {
                print!("\r正在获取收藏数据... {}/{}", all_collections.len(), total);
                use std::io::Write;
                std::io::stdout().flush().ok();
            }

            let resp = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .send()
                .await
                .context("Failed to fetch collections")?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("Failed to get collections: {} - {}", status, body);
            }

            let paged: PagedResponse<UserSubjectCollection> =
                resp.json().await.context("Failed to parse collections")?;

            total = paged.total;
            let count = paged.data.len();
            all_collections.extend(paged.data);

            if count == 0 || all_collections.len() >= total as usize {
                break;
            }

            offset += limit;
        }

        if verbose {
            println!("\r收藏数据获取完成，共 {} 条记录", all_collections.len());
        }

        Ok(all_collections)
    }
}

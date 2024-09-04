use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct ArticleSearch {
    id: String,
    collection_id: String,
    category_ids: Vec<String>,
    slug: String,
    name: String,
    preview: String,
    url: String,
    docs_url: String,
    number: i32,
    status: String,
    visibility: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    articles: Articles,
}

#[derive(Debug, Serialize, Deserialize)]
struct Articles {
    page: i32,
    pages: i32,
    count: i32,
    items: Vec<ArticleSearch>,
}

pub struct ExternalApiService {
    client: Client,
    base_url: String,
    api_key: String,
}

impl ExternalApiService {
    pub fn new(api_key: String) -> Self {
        ExternalApiService {
            client: Client::new(),
            base_url: "https://docsapi.helpscout.net/v1".to_string(),
            api_key,
        }
    }

    pub async fn search_articles(&self, query: &str) -> Result<Vec<ArticleSearch>, Box<dyn Error>> {
        let url = format!("{}/search/articles", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&[("query", query)])
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?
            .json::<ApiResponse>()
            .await?;

        Ok(response.articles.items)
    }
}

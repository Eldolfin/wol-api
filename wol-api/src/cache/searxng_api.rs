use anyhow::Context as _;
use serde::Deserialize;
use serde::Serialize;

const SEARXNG_BASEURL: &str = "https://search.eldolfin.top";

// thanks https://transform.tools/json-to-rust-serde
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    // pub title: String,
    // pub url: String,
    // pub template: String,
    // #[serde(rename = "thumbnail_src")]
    // pub thumbnail_src: Option<String>,
    #[serde(rename = "img_src")]
    pub img_src: String,
    pub resolution: Option<String>,
    #[serde(rename = "img_format")]
    pub img_format: Option<String>,
    // pub engine: String,
    // #[serde(rename = "parsed_url")]
    // pub parsed_url: Vec<String>,
    // pub engines: Vec<String>,
    // pub positions: Vec<i64>,
    // pub content: Option<String>,
    // pub source: Option<String>,
    pub score: f32,
    pub category: String,
    // pub author: Option<String>,
}

pub async fn query_image(search_query: String) -> anyhow::Result<SearchResponse> {
    let resp = reqwest::Client::default()
        .get(SEARXNG_BASEURL)
        .query(&[
            ("search", ""),
            ("category_images", ""),
            ("format", "json"),
            ("q", &search_query),
        ])
        .send()
        .await
        .with_context(|| format!("Failed to query searxng instance `{SEARXNG_BASEURL}`"))?;
    // debug!("searxng query response: {:#?}", &resp);
    resp.json()
        .await
        .context("Failed to parse searxng response (maybe format changed?)")
}

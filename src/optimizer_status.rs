use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Optimizer {
    Vacuum,
    Merge,
    Indexing,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationStatus {
    Done,
    Optimizing,
    Cancelled,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerSegment {
    pub uuid: String,
    pub points_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerTreeProgression {
    pub name: String,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
    #[serde(default)]
    pub duration_sec: Option<f64>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub done: Option<u64>,
    #[serde(default)]
    pub children: Option<Vec<OptimizerTreeProgression>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerProgress {
    pub uuid: String,
    pub optimizer: Optimizer,
    #[serde(default)]
    pub status: Option<OptimizationStatus>,
    pub segments: Vec<OptimizerSegment>,
    #[serde(default)]
    pub progress: Option<OptimizerTreeProgression>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OptimizerSummary {
    pub queued_optimizations: u32,
    pub queued_points: u32,
    pub queued_segments: u32,
    pub idle_segments: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerResult {
    pub summary: OptimizerSummary,
    pub running: Vec<OptimizerProgress>,
    pub completed: Vec<OptimizerProgress>,
    pub queued: Vec<OptimizerProgress>,
    pub idle_segments: Vec<OptimizerSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizerResponse {
    pub result: OptimizerResult,
}

pub async fn get_optimizations_status(
    base_url: &str,
    api_key: Option<&str>,
    collection_name: &str,
) -> Result<OptimizerResponse, reqwest::Error> {
    let client = Client::new();
    let mut header_map = HeaderMap::new();
    if let Some(key) = api_key {
        header_map.insert(
            "api-key",
            HeaderValue::from_str(key).expect("Header value should convert without problems"),
        );
    }
    let response: OptimizerResponse = client
        .get(format!(
            "{}/collections/{}/optimizations",
            base_url, collection_name
        ))
        .headers(header_map)
        .query(&[("with", "completed,idle,queued")])
        .send()
        .await?
        .json()
        .await?;

    Ok(response)
}

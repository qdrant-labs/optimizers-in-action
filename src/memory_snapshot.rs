use serde::{Deserialize, Serialize};

/// disk/ram/cached bytes for one storage component, as reported by the
/// collection memory endpoint. `expected_cache_bytes` is how much of it Qdrant
/// wants resident in the page cache; `cached_bytes` is how much actually is.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct MemoryUsage {
    pub disk_bytes: u64,
    pub ram_bytes: u64,
    pub cached_bytes: u64,
    pub expected_cache_bytes: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VectorMemory {
    pub name: String,
    pub storage: MemoryUsage,
    pub index: MemoryUsage,
    pub quantized: Option<MemoryUsage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PayloadIndexMemory {
    pub name: String,
    pub usage: MemoryUsage,
}

/// Per-collection memory/storage breakdown, read from the REST
/// `/collections/{name}/memory` endpoint - much more precise than the global
/// jemalloc stats on `/telemetry`, and scoped to the collection under test.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CollectionMemory {
    pub total: MemoryUsage,
    #[serde(default)]
    pub vectors: Vec<VectorMemory>,
    #[serde(default)]
    pub sparse_vectors: Vec<VectorMemory>,
    #[serde(default)]
    pub payload: MemoryUsage,
    #[serde(default)]
    pub payload_index: Vec<PayloadIndexMemory>,
}

#[derive(Debug, Deserialize)]
struct CollectionMemoryResponse {
    result: CollectionMemory,
}

/// The gRPC client talks to port 6334; this benchmark's REST calls (memory,
/// telemetry) go through port 6333 instead.
fn grpc_url_to_rest_url(qdrant_api_url: &str) -> String {
    qdrant_api_url.replace(":6334", ":6333")
}

/// Best-effort: memory stats are a diagnostic extra, so a REST-side failure
/// (e.g. no REST access from this deployment, or an older server without this
/// endpoint) shouldn't fail the whole benchmark.
pub async fn fetch_collection_memory(
    qdrant_api_url: &str,
    qdrant_api_key: Option<&str>,
    collection_name: &str,
) -> Result<CollectionMemory, reqwest::Error> {
    let rest_url = grpc_url_to_rest_url(qdrant_api_url);
    let mut request =
        reqwest::Client::new().get(format!("{rest_url}/collections/{collection_name}/memory"));
    if let Some(key) = qdrant_api_key {
        request = request.header("api-key", key);
    }
    let response = request.send().await?;
    let result = response.json::<CollectionMemoryResponse>().await?;
    Ok(result.result)
}

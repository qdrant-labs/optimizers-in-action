use std::time::Instant;

use qdrant_client::{Qdrant, QdrantError, qdrant::SearchPointsBuilder};

use crate::metrics::LatencyStats;

pub async fn run_search(
    base_url: &str,
    api_key: Option<&str>,
    collection_name: &str,
    queries: &[Vec<f32>],
    limit: u64,
    repeat: usize,
    verbose: bool,
) -> Result<LatencyStats, QdrantError> {
    let client = Qdrant::from_url(base_url)
        .api_key(api_key)
        .timeout(6000)
        .build()?;
    let mut durations = Vec::with_capacity(queries.len() * repeat);
    let wall_start = Instant::now();

    for round in 0..repeat {
        for (i, query) in queries.iter().enumerate() {
            let request = SearchPointsBuilder::new(collection_name, query.clone(), limit)
                .vector_name("dense")
                .build();
            let start = Instant::now();
            client.search_points(request).await?;
            let elapsed = start.elapsed();
            if verbose {
                println!("round {round} query {i} latency {elapsed:?}");
            }
            durations.push(elapsed);
        }
    }

    Ok(LatencyStats::from_durations(
        durations,
        wall_start.elapsed(),
    ))
}

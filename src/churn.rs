use qdrant_client::{
    Qdrant, QdrantError,
    qdrant::{DeletePointsBuilder, ScrollPointsBuilder},
};

/// Scrolls the collection in id order and deletes the first `fraction` of
/// its points. Scrolling always visits ids in the same order for a given
/// collection state, so repeated runs against freshly re-loaded data delete
/// the same logical slice, keeping vacuum-trigger experiments comparable.
pub async fn churn_points(
    base_url: &str,
    api_key: Option<&str>,
    collection_name: &str,
    fraction: f64,
    batch_size: u32,
    verbose: bool,
) -> Result<u64, QdrantError> {
    let client = Qdrant::from_url(base_url).api_key(api_key).build()?;

    let info = client.collection_info(collection_name).await?;
    let total = info
        .result
        .and_then(|r| r.points_count)
        .unwrap_or_default();
    let target = ((total as f64) * fraction).round() as u64;

    let mut ids = Vec::new();
    let mut offset = None;
    while (ids.len() as u64) < target {
        let mut builder = ScrollPointsBuilder::new(collection_name)
            .limit(batch_size)
            .with_payload(false)
            .with_vectors(false);
        if let Some(off) = offset.take() {
            builder = builder.offset(off);
        }
        let resp = client.scroll(builder).await?;
        if resp.result.is_empty() {
            break;
        }
        for point in resp.result {
            if let Some(id) = point.id {
                ids.push(id);
            }
            if (ids.len() as u64) >= target {
                break;
            }
        }
        offset = resp.next_page_offset;
        if offset.is_none() {
            break;
        }
    }

    let deleted = ids.len() as u64;
    if deleted > 0 {
        client
            .delete_points(
                DeletePointsBuilder::new(collection_name)
                    .points(ids)
                    .wait(true),
            )
            .await?;
    }
    if verbose {
        println!("Deleted {deleted} / {total} points (target fraction {fraction})");
    }
    Ok(deleted)
}

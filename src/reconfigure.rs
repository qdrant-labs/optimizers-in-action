use qdrant_client::{
    Qdrant, QdrantError,
    qdrant::{HnswConfigDiffBuilder, OptimizersConfigDiffBuilder, UpdateCollectionBuilder},
};

/// Qdrant's own default `indexing_threshold_kb`, used as a convenience value
/// when re-enabling indexing on a collection that was created with
/// indexing disabled for bulk loading.
pub const DEFAULT_INDEXING_THRESHOLD_KB: u64 = 10_000;

#[allow(clippy::too_many_arguments)]
pub async fn reconfigure_collection(
    base_url: &str,
    api_key: Option<&str>,
    collection_name: &str,
    indexing_threshold_kb: Option<u64>,
    prevent_unoptimized: Option<bool>,
    delete_threshold: Option<f64>,
    vacuum_min_vectors_number: Option<u64>,
    default_segment_number: Option<u64>,
    max_segment_size_kb: Option<u64>,
    optimizers_threads: Option<u64>,
    indexing_threads: Option<u64>,
) -> Result<(), QdrantError> {
    let client = Qdrant::from_url(base_url).api_key(api_key).build()?;

    let mut optimizers_conf = OptimizersConfigDiffBuilder::default();
    let mut change_optimizers = false;
    if let Some(it) = indexing_threshold_kb {
        optimizers_conf = optimizers_conf.indexing_threshold(it);
        change_optimizers = true;
    }
    if let Some(pu) = prevent_unoptimized {
        optimizers_conf = optimizers_conf.prevent_unoptimized(pu);
        change_optimizers = true;
    }
    if let Some(dt) = delete_threshold {
        optimizers_conf = optimizers_conf.deleted_threshold(dt);
        change_optimizers = true;
    }
    if let Some(vc) = vacuum_min_vectors_number {
        optimizers_conf = optimizers_conf.vacuum_min_vector_number(vc);
        change_optimizers = true;
    }
    if let Some(ds) = default_segment_number {
        optimizers_conf = optimizers_conf.default_segment_number(ds);
        change_optimizers = true;
    }
    if let Some(sz) = max_segment_size_kb {
        optimizers_conf = optimizers_conf.max_segment_size(sz);
        change_optimizers = true;
    }
    if let Some(ot) = optimizers_threads {
        optimizers_conf = optimizers_conf.max_optimization_threads(ot);
        change_optimizers = true;
    }

    let mut update_builder = UpdateCollectionBuilder::new(collection_name);
    if change_optimizers {
        update_builder = update_builder.optimizers_config(optimizers_conf.build());
    }
    if let Some(it) = indexing_threads {
        update_builder = update_builder.hnsw_config(
            HnswConfigDiffBuilder::default()
                .max_indexing_threads(it)
                .build(),
        );
    }

    client.update_collection(update_builder.build()).await?;
    Ok(())
}

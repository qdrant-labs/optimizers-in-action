use qdrant_client::{
    Qdrant, QdrantError,
    qdrant::{
        CreateCollectionBuilder, Distance, HnswConfigDiffBuilder, OptimizersConfigDiffBuilder,
        VectorParamsBuilder,
    },
};

pub const NOINDEX_THRESH_KB: u64 = 400_000; // 768 dim vector is ~3KB, and we have 100_000 vectors, so we put here 400_000 to be sure no indexing occur while uploading

pub async fn create_collection(
    base_url: &str,
    api_key: Option<&str>,
    collection_name: &str,
    disable_indexing: bool,
    prevent_unoptimized: bool,
    delete_threshold: Option<f64>,
    vacuum_min_vectors_number: Option<u64>,
    default_segment_number: Option<u64>,
    max_segment_size_kb: Option<u64>,
    optimizers_threads: Option<u64>,
    indexing_threads: Option<u64>,
) -> Result<(), QdrantError> {
    let client = Qdrant::from_url(base_url).api_key(api_key).build()?;
    let mut coll_builder = CreateCollectionBuilder::default()
        .collection_name(collection_name)
        .vectors_config(VectorParamsBuilder::new(768, Distance::Cosine));
    let mut optimizers_conf = OptimizersConfigDiffBuilder::default();
    if disable_indexing {
        optimizers_conf = optimizers_conf.indexing_threshold(NOINDEX_THRESH_KB);
    }
    if prevent_unoptimized {
        optimizers_conf = optimizers_conf.prevent_unoptimized(prevent_unoptimized);
    }
    if let Some(dt) = delete_threshold {
        optimizers_conf = optimizers_conf.deleted_threshold(dt);
    }
    if let Some(vc) = vacuum_min_vectors_number {
        optimizers_conf = optimizers_conf.vacuum_min_vector_number(vc);
    }
    if let Some(ds) = default_segment_number {
        optimizers_conf = optimizers_conf.default_segment_number(ds);
    }
    if let Some(sz) = max_segment_size_kb {
        optimizers_conf = optimizers_conf.max_segment_size(sz);
    }
    if let Some(ot) = optimizers_threads {
        optimizers_conf = optimizers_conf.max_optimization_threads(ot);
    }
    let mut hnsw_conf = HnswConfigDiffBuilder::default();
    if let Some(it) = indexing_threads {
        hnsw_conf = hnsw_conf.max_indexing_threads(it);
    }
    coll_builder = coll_builder
        .optimizers_config(optimizers_conf.build())
        .hnsw_config(hnsw_conf.build());
    client.create_collection(coll_builder.build()).await?;
    Ok(())
}

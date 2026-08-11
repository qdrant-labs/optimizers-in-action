use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

use datafusion::{
    arrow::{
        array::{Array, AsArray, Float32Array},
        datatypes::Float32Type,
    },
    error::DataFusionError,
    execution::{config::SessionConfig, context::SessionContext, options::ParquetReadOptions},
};
use futures::StreamExt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QuerySetError {
    #[error(transparent)]
    DataFusion(#[from] DataFusionError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// Deterministically strides through `parquet_path`, taking every `stride`-th
/// row's embedding, up to `limit` vectors. Writing the same stride/limit
/// against the same file always yields the same query set, so latency
/// numbers stay comparable across experiment runs.
pub async fn sample_queries(
    parquet_path: &str,
    out_path: &str,
    stride: usize,
    limit: usize,
) -> Result<usize, QuerySetError> {
    let mut session_config = SessionConfig::new();
    session_config
        .options_mut()
        .execution
        .parquet
        .schema_force_view_types = false;
    let ctx = SessionContext::new_with_config(session_config);
    ctx.register_parquet("data", parquet_path, ParquetReadOptions::default())
        .await?;

    let df = ctx.sql("SELECT emb FROM data").await?;
    let mut stream = df.execute_stream().await?;

    let file = File::create(out_path)?;
    let mut writer = BufWriter::new(file);
    let mut row_idx: usize = 0;
    let mut collected = 0usize;

    'outer: while let Some(batch) = stream.next().await {
        let batch = batch?;
        let embedding_col = batch.column(0).as_list::<i32>();
        for i in 0..embedding_col.len() {
            let take = row_idx.is_multiple_of(stride);
            row_idx += 1;
            if !take || embedding_col.is_null(i) {
                continue;
            }
            let values = embedding_col.value(i);
            let float_array: &Float32Array = values.as_primitive::<Float32Type>();
            let embedding: Vec<f32> = float_array.values().to_vec();
            writeln!(writer, "{}", serde_json::to_string(&embedding)?)?;
            collected += 1;
            if collected >= limit {
                break 'outer;
            }
        }
    }
    writer.flush()?;
    Ok(collected)
}

pub fn load_queries(path: &str) -> Result<Vec<Vec<f32>>, QuerySetError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut queries = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        queries.push(serde_json::from_str(&line)?);
    }
    Ok(queries)
}

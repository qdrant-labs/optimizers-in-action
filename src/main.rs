mod bench;
mod churn;
mod create_collection;
mod memory_snapshot;
mod metrics;
mod optimizer_status;
mod query_set;
mod reconfigure;
mod search;
mod upload;

use std::env;

use clap::{Parser, Subcommand};

use crate::{
    bench::run_bench,
    churn::churn_points,
    create_collection::create_collection,
    memory_snapshot::fetch_collection_memory,
    optimizer_status::get_optimizations_status,
    query_set::{load_queries, sample_queries},
    reconfigure::reconfigure_collection,
    search::run_search,
    upload::load_embeddings,
};

#[derive(Debug, Subcommand)]
enum Commands {
    OptStatus {
        collection: String,
        #[arg(long, default_value = None)]
        api_key: Option<String>,
        #[arg(long, default_value = None)]
        base_url: Option<String>,
    },
    MemStatus {
        collection: String,
        #[arg(long, default_value = None)]
        api_key: Option<String>,
        #[arg(long, default_value = None)]
        base_url: Option<String>,
    },
    Create {
        collection: String,
        #[arg(long, default_value = None)]
        base_url: Option<String>,
        #[arg(long, default_value = None)]
        api_key: Option<String>,
        #[arg(long, default_value_t = false)]
        no_disable_indexing: bool,
        #[arg(long, default_value_t = false)]
        prevent_unoptimized: bool,
        #[arg(long, default_value = None)]
        delete_threshold: Option<f64>,
        #[arg(long, default_value = None)]
        vacuum_min_vectors_number: Option<u64>,
        #[arg(long, default_value = None)]
        default_segment_number: Option<u64>,
        #[arg(long, default_value = None)]
        max_segment_size_kb: Option<u64>,
        #[arg(long, default_value = None)]
        optimizers_threads: Option<u64>,
        #[arg(long, default_value = None)]
        indexing_threads: Option<u64>,
    },
    Upload {
        data_file: String,
        collection: String,
        #[arg(long, default_value = None)]
        api_key: Option<String>,
        #[arg(long, default_value = None)]
        base_url: Option<String>,
        #[arg(long, default_value_t = false)]
        verbose: bool,
    },
    /// Deterministically sample a held-out query set from a parquet file.
    SampleQueries {
        data_file: String,
        out: String,
        #[arg(long, default_value_t = 200)]
        stride: usize,
        #[arg(long, default_value_t = 1000)]
        limit: usize,
    },
    /// Run one-shot search-latency measurement against a fixed query set.
    Search {
        collection: String,
        query_file: String,
        #[arg(long, default_value = None)]
        api_key: Option<String>,
        #[arg(long, default_value = None)]
        base_url: Option<String>,
        #[arg(long, default_value_t = 10)]
        limit: u64,
        #[arg(long, default_value_t = 1)]
        repeat: usize,
        #[arg(long, default_value_t = false)]
        verbose: bool,
    },
    /// Update optimizer/HNSW settings on a live collection without recreating it.
    Reconfigure {
        collection: String,
        #[arg(long, default_value = None)]
        api_key: Option<String>,
        #[arg(long, default_value = None)]
        base_url: Option<String>,
        #[arg(long, default_value = None)]
        indexing_threshold_kb: Option<u64>,
        /// Convenience flag: reset indexing_threshold_kb to Qdrant's own default (10000).
        #[arg(long, default_value_t = false)]
        enable_indexing: bool,
        #[arg(long, default_value_t = false)]
        prevent_unoptimized: bool,
        #[arg(long, default_value = None)]
        delete_threshold: Option<f64>,
        #[arg(long, default_value = None)]
        vacuum_min_vectors_number: Option<u64>,
        #[arg(long, default_value = None)]
        default_segment_number: Option<u64>,
        #[arg(long, default_value = None)]
        max_segment_size_kb: Option<u64>,
        #[arg(long, default_value = None)]
        optimizers_threads: Option<u64>,
        #[arg(long, default_value = None)]
        indexing_threads: Option<u64>,
    },
    /// Delete a fraction of the collection's points, in deterministic scroll order,
    /// to drive the vacuum optimizer as a controlled event.
    Churn {
        collection: String,
        #[arg(long, default_value = None)]
        api_key: Option<String>,
        #[arg(long, default_value = None)]
        base_url: Option<String>,
        #[arg(long, default_value_t = 0.25)]
        fraction: f64,
        #[arg(long, default_value_t = 500)]
        batch_size: u32,
        #[arg(long, default_value_t = false)]
        verbose: bool,
    },
    /// Upload to completion, then search continuously while optimizers drain
    /// the post-load backlog, then run a fixed number of passes once
    /// optimizers report idle, for a single experiment configuration.
    Bench {
        data_file: String,
        collection: String,
        query_file: String,
        out: String,
        #[arg(long, default_value = None)]
        api_key: Option<String>,
        #[arg(long, default_value = None)]
        base_url: Option<String>,
        #[arg(long, default_value_t = 10)]
        search_limit: u64,
        #[arg(long, default_value_t = 2000)]
        poll_interval_ms: u64,
        #[arg(long, default_value_t = 3)]
        idle_stability_rounds: u32,
        /// Safety fallback only: give up waiting for optimizers to go idle after this long.
        #[arg(long, default_value_t = 3600)]
        drain_timeout_secs: u64,
        /// Query-set passes to run once optimizers are confirmed idle.
        #[arg(long, default_value_t = 5)]
        steady_repeat: usize,
        /// Poll /collections/{name}/memory alongside the search loop. Off by
        /// default: existing result files predate this and simply have no
        /// memory events, rather than needing an explanation for missing them.
        #[arg(long, default_value_t = false)]
        enable_memory_monitoring: bool,
    },
}

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

fn resolve_base_url(base_url: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    Ok(match base_url {
        Some(b) => b,
        None => env::var("QDRANT_URL")?,
    })
}

fn resolve_api_key(api_key: Option<String>) -> Option<String> {
    match api_key {
        Some(a) => Some(a),
        None => env::var("QDRANT_API_KEY").ok(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.cmd {
        Commands::OptStatus {
            collection,
            api_key,
            base_url,
        } => {
            let bu = resolve_base_url(base_url)?;
            let key = resolve_api_key(api_key);
            let opt = get_optimizations_status(&bu, key.as_deref(), &collection).await?;
            println!("{}", serde_json::to_string_pretty(&opt)?);
        }
        Commands::MemStatus {
            collection,
            api_key,
            base_url,
        } => {
            let bu = resolve_base_url(base_url)?;
            let key = resolve_api_key(api_key);
            let mem = fetch_collection_memory(&bu, key.as_deref(), &collection).await?;
            println!("{}", serde_json::to_string_pretty(&mem)?);
        }
        Commands::Create {
            collection,
            base_url,
            api_key,
            no_disable_indexing,
            prevent_unoptimized,
            delete_threshold,
            vacuum_min_vectors_number,
            default_segment_number,
            max_segment_size_kb,
            optimizers_threads,
            indexing_threads,
        } => {
            let bu = resolve_base_url(base_url)?;
            let key = resolve_api_key(api_key);
            create_collection(
                &bu,
                key.as_deref(),
                &collection,
                !no_disable_indexing,
                prevent_unoptimized,
                delete_threshold,
                vacuum_min_vectors_number,
                default_segment_number,
                max_segment_size_kb,
                optimizers_threads,
                indexing_threads,
            )
            .await?;
        }
        Commands::Upload {
            data_file,
            collection,
            api_key,
            base_url,
            verbose,
        } => {
            let bu = resolve_base_url(base_url)?;
            let key = resolve_api_key(api_key);
            load_embeddings(&data_file, &bu, key.as_deref(), &collection, verbose).await?;
        }
        Commands::SampleQueries {
            data_file,
            out,
            stride,
            limit,
        } => {
            let collected = sample_queries(&data_file, &out, stride, limit).await?;
            println!("Sampled {collected} query vectors into {out}");
        }
        Commands::Search {
            collection,
            query_file,
            api_key,
            base_url,
            limit,
            repeat,
            verbose,
        } => {
            let bu = resolve_base_url(base_url)?;
            let key = resolve_api_key(api_key);
            let queries = load_queries(&query_file)?;
            let stats = run_search(
                &bu,
                key.as_deref(),
                &collection,
                &queries,
                limit,
                repeat,
                verbose,
            )
            .await?;
            println!(
                "search: n={} min={:?} p50={:?} p95={:?} p99={:?} max={:?} mean={:?} ({:.1} qps)",
                stats.count,
                stats.min,
                stats.p50,
                stats.p95,
                stats.p99,
                stats.max,
                stats.mean,
                stats.throughput(),
            );
        }
        Commands::Reconfigure {
            collection,
            api_key,
            base_url,
            indexing_threshold_kb,
            enable_indexing,
            prevent_unoptimized,
            delete_threshold,
            vacuum_min_vectors_number,
            default_segment_number,
            max_segment_size_kb,
            optimizers_threads,
            indexing_threads,
        } => {
            let bu = resolve_base_url(base_url)?;
            let key = resolve_api_key(api_key);
            let indexing_threshold_kb = indexing_threshold_kb.or(if enable_indexing {
                Some(reconfigure::DEFAULT_INDEXING_THRESHOLD_KB)
            } else {
                None
            });
            reconfigure_collection(
                &bu,
                key.as_deref(),
                &collection,
                indexing_threshold_kb,
                prevent_unoptimized.then_some(true),
                delete_threshold,
                vacuum_min_vectors_number,
                default_segment_number,
                max_segment_size_kb,
                optimizers_threads,
                indexing_threads,
            )
            .await?;
            println!("Reconfigured {collection}");
        }
        Commands::Churn {
            collection,
            api_key,
            base_url,
            fraction,
            batch_size,
            verbose,
        } => {
            let bu = resolve_base_url(base_url)?;
            let key = resolve_api_key(api_key);
            let deleted = churn_points(
                &bu,
                key.as_deref(),
                &collection,
                fraction,
                batch_size,
                verbose,
            )
            .await?;
            println!("Deleted {deleted} points from {collection}");
        }
        Commands::Bench {
            data_file,
            collection,
            query_file,
            out,
            api_key,
            base_url,
            search_limit,
            poll_interval_ms,
            idle_stability_rounds,
            drain_timeout_secs,
            steady_repeat,
            enable_memory_monitoring,
        } => {
            let bu = resolve_base_url(base_url)?;
            let key = resolve_api_key(api_key);
            run_bench(
                &bu,
                key.as_deref(),
                &collection,
                &data_file,
                &query_file,
                search_limit,
                poll_interval_ms,
                idle_stability_rounds,
                drain_timeout_secs,
                steady_repeat,
                enable_memory_monitoring,
                &out,
            )
            .await?;
        }
    }

    Ok(())
}

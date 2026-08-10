mod create_collection;
mod optimizer_status;

use std::env;

use clap::{Parser, Subcommand};

use crate::{create_collection::create_collection, optimizer_status::get_optimizations_status};

#[derive(Debug, Subcommand)]
enum Commands {
    OptStatus {
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
}

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
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
            let bu = match base_url {
                Some(b) => b,
                None => env::var("QDRANT_URL")?,
            };
            let key = match api_key {
                Some(a) => Some(a),
                None => env::var("QDRANT_API_KEY").ok(),
            };
            get_optimizations_status(&bu, key.as_deref(), &collection).await?;
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
            let bu = match base_url {
                Some(b) => b,
                None => env::var("QDRANT_URL")?,
            };
            let key = match api_key {
                Some(a) => Some(a),
                None => env::var("QDRANT_API_KEY").ok(),
            };
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
    }

    Ok(())
}

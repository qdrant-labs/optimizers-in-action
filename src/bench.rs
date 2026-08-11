use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::{Duration, Instant};

use qdrant_client::{Qdrant, QdrantError, qdrant::SearchPointsBuilder};
use serde::Serialize;
use thiserror::Error;
use tokio::sync::mpsc;

use crate::metrics::LatencyStats;
use crate::optimizer_status::{Optimizer, get_optimizations_status};
use crate::query_set::{QuerySetError, load_queries};
use crate::upload::{LoadError, load_embeddings};

const LOADING: u8 = 0;
const STEADY: u8 = 1;
const DONE: u8 = 2;

#[derive(Debug, Error)]
pub enum BenchError {
    #[error(transparent)]
    Qdrant(#[from] QdrantError),
    #[error(transparent)]
    Load(#[from] LoadError),
    #[error(transparent)]
    QuerySet(#[from] QuerySetError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum PhaseTag {
    Loading,
    Steady,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum BenchEvent {
    Search {
        t: f64,
        phase: PhaseTag,
        latency_ms: f64,
    },
    Optimizer {
        t: f64,
        phase: PhaseTag,
        queued_optimizations: u32,
        queued_points: u32,
        queued_segments: u32,
        idle_segments: u32,
        running: usize,
        running_optimizers: Vec<Optimizer>,
        queued_optimizers: Vec<Optimizer>,
        completed_optimizers: Vec<Optimizer>,
    },
    PhaseChange {
        t: f64,
        phase: PhaseTag,
    },
}

fn phase_tag(p: u8) -> PhaseTag {
    if p == LOADING {
        PhaseTag::Loading
    } else {
        PhaseTag::Steady
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn run_bench(
    base_url: &str,
    api_key: Option<&str>,
    collection_name: &str,
    data_file: &str,
    query_file: &str,
    search_limit: u64,
    poll_interval_ms: u64,
    idle_stability_rounds: u32,
    steady_timeout_secs: u64,
    out_path: &str,
) -> Result<(), BenchError> {
    let queries = load_queries(query_file)?;
    let poll_interval = Duration::from_millis(poll_interval_ms);
    let phase = Arc::new(AtomicU8::new(LOADING));
    let (tx, mut rx) = mpsc::unbounded_channel::<BenchEvent>();
    let start = Instant::now();

    let aggregator = tokio::spawn(async move {
        let mut events = Vec::new();
        while let Some(ev) = rx.recv().await {
            events.push(ev);
        }
        events
    });

    // Continuous search load: runs across both phases, tagging each
    // measurement with the phase active at the moment the request fired.
    let search_client = Qdrant::from_url(base_url).api_key(api_key).build()?;
    let search_phase = phase.clone();
    let search_tx = tx.clone();
    let search_collection = collection_name.to_string();
    let search_handle = tokio::spawn(async move {
        let mut idx = 0usize;
        loop {
            let p = search_phase.load(Ordering::Relaxed);
            if p == DONE {
                break;
            }
            let query = queries[idx % queries.len()].clone();
            idx += 1;
            let request = SearchPointsBuilder::new(search_collection.clone(), query, search_limit)
                .vector_name("dense")
                .build();
            let t0 = Instant::now();
            let res = search_client.search_points(request).await;
            let latency_ms = t0.elapsed().as_secs_f64() * 1000.0;
            if res.is_ok() {
                let _ = search_tx.send(BenchEvent::Search {
                    t: start.elapsed().as_secs_f64(),
                    phase: phase_tag(p),
                    latency_ms,
                });
            }
        }
    });

    // Optimizer-status poller for the loading phase only; the steady phase
    // is polled inline below since that loop also decides when to stop.
    let poll_phase = phase.clone();
    let poll_tx = tx.clone();
    let poll_base_url = base_url.to_string();
    let poll_api_key = api_key.map(str::to_string);
    let poll_collection = collection_name.to_string();
    let loading_poller = tokio::spawn(async move {
        loop {
            if poll_phase.load(Ordering::Relaxed) != LOADING {
                break;
            }
            if let Ok(resp) =
                get_optimizations_status(&poll_base_url, poll_api_key.as_deref(), &poll_collection)
                    .await
            {
                let s = resp.result.summary;
                let mut running_optimizers = vec![];
                let mut queued_optimizers = vec![];
                let mut completed_optimizers = vec![];
                if !resp.result.running.is_empty() {
                    for r in &resp.result.running {
                        running_optimizers.push(r.optimizer);
                    }
                }
                if !resp.result.completed.is_empty() {
                    for r in resp.result.completed {
                        completed_optimizers.push(r.optimizer);
                    }
                }
                if !resp.result.queued.is_empty() {
                    for r in resp.result.queued {
                        queued_optimizers.push(r.optimizer);
                    }
                }
                let _ = poll_tx.send(BenchEvent::Optimizer {
                    t: start.elapsed().as_secs_f64(),
                    phase: PhaseTag::Loading,
                    queued_optimizations: s.queued_optimizations,
                    queued_points: s.queued_points,
                    queued_segments: s.queued_segments,
                    idle_segments: s.idle_segments,
                    running: resp.result.running.len(),
                    running_optimizers,
                    queued_optimizers,
                    completed_optimizers,
                });
            }
            tokio::time::sleep(poll_interval).await;
        }
    });

    load_embeddings(data_file, base_url, api_key, collection_name, false).await?;

    phase.store(STEADY, Ordering::Relaxed);
    tx.send(BenchEvent::PhaseChange {
        t: start.elapsed().as_secs_f64(),
        phase: PhaseTag::Steady,
    })
    .ok();
    loading_poller.await.ok();

    let mut stable_rounds = 0u32;
    let steady_deadline = Instant::now() + Duration::from_secs(steady_timeout_secs);
    loop {
        if let Ok(resp) = get_optimizations_status(base_url, api_key, collection_name).await {
            let s = resp.result.summary;
            let is_idle = s.queued_optimizations == 0 && resp.result.running.is_empty();
            let mut running_optimizers = vec![];
            let mut queued_optimizers = vec![];
            let mut completed_optimizers = vec![];
            if !resp.result.running.is_empty() {
                for r in &resp.result.running {
                    running_optimizers.push(r.optimizer);
                }
            }
            if !resp.result.completed.is_empty() {
                for r in resp.result.completed {
                    completed_optimizers.push(r.optimizer);
                }
            }
            if !resp.result.queued.is_empty() {
                for r in resp.result.queued {
                    queued_optimizers.push(r.optimizer);
                }
            }
            tx.send(BenchEvent::Optimizer {
                t: start.elapsed().as_secs_f64(),
                phase: PhaseTag::Steady,
                queued_optimizations: s.queued_optimizations,
                queued_points: s.queued_points,
                queued_segments: s.queued_segments,
                idle_segments: s.idle_segments,
                running: resp.result.running.len(),
                running_optimizers,
                completed_optimizers,
                queued_optimizers,
            })
            .ok();
            stable_rounds = if is_idle { stable_rounds + 1 } else { 0 };
            if stable_rounds >= idle_stability_rounds {
                break;
            }
        }
        if Instant::now() >= steady_deadline {
            println!("steady-phase timeout reached before optimizers went idle");
            break;
        }
        tokio::time::sleep(poll_interval).await;
    }

    phase.store(DONE, Ordering::Relaxed);
    drop(tx);
    search_handle.await.ok();
    let events = aggregator.await.unwrap_or_default();

    write_report(out_path, &events)?;
    print_summary(&events);
    Ok(())
}

fn write_report(out_path: &str, events: &[BenchEvent]) -> Result<(), BenchError> {
    let file = File::create(out_path)?;
    let mut writer = BufWriter::new(file);
    for event in events {
        writeln!(writer, "{}", serde_json::to_string(event)?)?;
    }
    writer.flush()?;
    Ok(())
}

fn print_summary(events: &[BenchEvent]) {
    let mut loading = Vec::new();
    let mut steady = Vec::new();
    for event in events {
        if let BenchEvent::Search {
            phase, latency_ms, ..
        } = event
        {
            let duration = Duration::from_secs_f64(latency_ms / 1000.0);
            match phase {
                PhaseTag::Loading => loading.push(duration),
                PhaseTag::Steady => steady.push(duration),
            }
        }
    }

    for (label, durations) in [("loading", loading), ("steady", steady)] {
        if durations.is_empty() {
            println!("{label}: no search samples collected");
            continue;
        }
        let wall = durations.iter().sum();
        let stats = LatencyStats::from_durations(durations, wall);
        println!(
            "{label}: n={} min={:?} p50={:?} p95={:?} p99={:?} max={:?} mean={:?}",
            stats.count, stats.min, stats.p50, stats.p95, stats.p99, stats.max, stats.mean,
        );
    }
}

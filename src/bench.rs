use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::{Duration, Instant};

use qdrant_client::{Qdrant, QdrantError, qdrant::SearchPointsBuilder};
use serde::Serialize;
use thiserror::Error;
use tokio::sync::mpsc;

use crate::memory_snapshot::fetch_collection_memory;
use crate::metrics::LatencyStats;
use crate::optimizer_status::{Optimizer, get_optimizations_status};
use crate::query_set::{QuerySetError, load_queries};
use crate::upload::{LoadError, load_embeddings};

const DRAINING: u8 = 0;
const DONE: u8 = 1;

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
    /// Upload has finished; optimizers are still draining the backlog it left behind.
    Draining,
    /// Optimizers reported idle (nothing running or queued); fixed-repeat measurement.
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
    /// Collection-level memory/cache breakdown, from the `/collections/{name}/memory`
    /// endpoint. Only emitted when `enable_memory_monitoring` is set; existing
    /// result files predate this and simply have no events of this kind.
    Memory {
        t: f64,
        phase: PhaseTag,
        disk_bytes: u64,
        ram_bytes: u64,
        cached_bytes: u64,
        expected_cache_bytes: u64,
    },
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
    drain_timeout_secs: u64,
    steady_repeat: usize,
    enable_memory_monitoring: bool,
    out_path: &str,
) -> Result<(), BenchError> {
    let queries = load_queries(query_file)?;
    let poll_interval = Duration::from_millis(poll_interval_ms);
    let start = Instant::now();
    let (tx, mut rx) = mpsc::unbounded_channel::<BenchEvent>();

    let aggregator = tokio::spawn(async move {
        let mut events = Vec::new();
        while let Some(ev) = rx.recv().await {
            events.push(ev);
        }
        events
    });

    // Phase 0: upload runs to completion with no concurrent search traffic,
    // so search latency is never measured against a partially-loaded collection.
    load_embeddings(data_file, base_url, api_key, collection_name, false).await?;

    // Phase 1 ("draining"): search continuously against the fully-loaded
    // collection while polling /optimizations, until it reports nothing
    // running or queued (or the safety timeout below is hit).
    tx.send(BenchEvent::PhaseChange {
        t: start.elapsed().as_secs_f64(),
        phase: PhaseTag::Draining,
    })
    .ok();

    let phase = Arc::new(AtomicU8::new(DRAINING));
    let search_client = Qdrant::from_url(base_url).api_key(api_key).build()?;
    let search_phase = phase.clone();
    let search_tx = tx.clone();
    let search_collection = collection_name.to_string();
    let draining_queries = queries.clone();
    let search_handle = tokio::spawn(async move {
        let mut idx = 0usize;
        loop {
            if search_phase.load(Ordering::Relaxed) == DONE {
                break;
            }
            let query = draining_queries[idx % draining_queries.len()].clone();
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
                    phase: PhaseTag::Draining,
                    latency_ms,
                });
            }
        }
    });

    // Optional memory poller: runs for the whole bench, draining and steady
    // alike, since the cache warm-up we're trying to see happens inside
    // steady. It tags each snapshot using the same `phase` flag search_handle
    // watches: DRAINING means draining is still in progress, anything else
    // means we're past it (search_handle has already stopped and the steady
    // loop below is what's issuing queries). `finished` is a separate signal
    // for when to stop polling altogether, since `phase` alone can't say
    // "steady is done too" without being repurposed away from search_handle's
    // simpler on/off use of it.
    let finished = Arc::new(AtomicBool::new(false));
    let memory_handle = if enable_memory_monitoring {
        let mem_phase = phase.clone();
        let mem_finished = finished.clone();
        let mem_tx = tx.clone();
        let mem_base_url = base_url.to_string();
        let mem_api_key = api_key.map(str::to_string);
        let mem_collection = collection_name.to_string();
        Some(tokio::spawn(async move {
            loop {
                if mem_finished.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok(mem) =
                    fetch_collection_memory(&mem_base_url, mem_api_key.as_deref(), &mem_collection)
                        .await
                {
                    let phase_tag = if mem_phase.load(Ordering::Relaxed) == DRAINING {
                        PhaseTag::Draining
                    } else {
                        PhaseTag::Steady
                    };
                    let _ = mem_tx.send(BenchEvent::Memory {
                        t: start.elapsed().as_secs_f64(),
                        phase: phase_tag,
                        disk_bytes: mem.total.disk_bytes,
                        ram_bytes: mem.total.ram_bytes,
                        cached_bytes: mem.total.cached_bytes,
                        expected_cache_bytes: mem.total.expected_cache_bytes,
                    });
                }
                tokio::time::sleep(poll_interval).await;
            }
        }))
    } else {
        None
    };

    let mut stable_rounds = 0u32;
    let drain_deadline = Instant::now() + Duration::from_secs(drain_timeout_secs);
    loop {
        if let Ok(resp) = get_optimizations_status(base_url, api_key, collection_name).await {
            let s = resp.result.summary;
            let is_idle = s.queued_optimizations == 0 && resp.result.running.is_empty();
            let running_optimizers = resp.result.running.iter().map(|r| r.optimizer).collect();
            let queued_optimizers = resp.result.queued.iter().map(|r| r.optimizer).collect();
            let completed_optimizers = resp.result.completed.iter().map(|r| r.optimizer).collect();
            tx.send(BenchEvent::Optimizer {
                t: start.elapsed().as_secs_f64(),
                phase: PhaseTag::Draining,
                queued_optimizations: s.queued_optimizations,
                queued_points: s.queued_points,
                queued_segments: s.queued_segments,
                idle_segments: s.idle_segments,
                running: resp.result.running.len(),
                running_optimizers,
                queued_optimizers,
                completed_optimizers,
            })
            .ok();
            stable_rounds = if is_idle { stable_rounds + 1 } else { 0 };
            if stable_rounds >= idle_stability_rounds {
                break;
            }
        }
        if Instant::now() >= drain_deadline {
            println!("drain-phase timeout reached before optimizers went idle");
            break;
        }
        tokio::time::sleep(poll_interval).await;
    }

    phase.store(DONE, Ordering::Relaxed);
    search_handle.await.ok();

    // Phase 2 ("steady"): optimizers confirmed idle; run a fixed number of
    // passes over the query set as the genuine steady-state measurement.
    tx.send(BenchEvent::PhaseChange {
        t: start.elapsed().as_secs_f64(),
        phase: PhaseTag::Steady,
    })
    .ok();
    let steady_client = Qdrant::from_url(base_url).api_key(api_key).build()?;
    for query in queries.iter().cycle().take(queries.len() * steady_repeat) {
        let request = SearchPointsBuilder::new(collection_name, query.clone(), search_limit)
            .vector_name("dense")
            .build();
        let t0 = Instant::now();
        let res = steady_client.search_points(request).await;
        let latency_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if res.is_ok() {
            tx.send(BenchEvent::Search {
                t: start.elapsed().as_secs_f64(),
                phase: PhaseTag::Steady,
                latency_ms,
            })
            .ok();
        }
    }

    finished.store(true, Ordering::Relaxed);
    if let Some(handle) = memory_handle {
        handle.await.ok();
    }

    drop(tx);
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
    let mut draining = Vec::new();
    let mut steady = Vec::new();
    for event in events {
        if let BenchEvent::Search {
            phase, latency_ms, ..
        } = event
        {
            let duration = Duration::from_secs_f64(latency_ms / 1000.0);
            match phase {
                PhaseTag::Draining => draining.push(duration),
                PhaseTag::Steady => steady.push(duration),
            }
        }
    }

    for (label, durations) in [("draining", draining), ("steady", steady)] {
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

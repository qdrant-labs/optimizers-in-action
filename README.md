# optimizers-in-action

Experiments quantifying how Qdrant's background optimizers (indexing, merge,
vacuum) affect search latency under read-write contention, using the
MS MARCO passage embeddings dataset (~1.76M x 1024-dim vectors).

## Layout

- `src/`: Rust CLI (`optimizers-in-action`) for driving the experiments.
- `scripts/`: shell scripts that run the experiment matrix end-to-end.
- `configs/base.yaml`, `compose.yaml`: local Qdrant via Docker Compose.
- `packages/download-data/`: Python helper (`uv run download-data`) to fetch
  the parquet dataset via Hugging Face.
- `packages/results-analysis-scripts/`: Python helper (`uv run
  results-analysis-scripts`) that turns the JSONL logs in `results/` into an
  HTML report with per-run latency and memory charts.
- `results/`: experiment output (JSONL logs, latency reports), gitignored
  except for a few checked-in examples.

## Setup

1. Download the dataset: `cd packages/download-data && uv run download-data`.
2. Set `QDRANT_URL` (the **gRPC** endpoint, e.g. `http://localhost:6334` or a
   Cloud cluster's `:6334` URL) and, if needed, `QDRANT_API_KEY`, e.g. via a
   `.env` you `source`. `scripts/base.env` holds the dataset path
   (`PARQUET_FILE`) and is sourced automatically by the scripts.
3. If running locally: `docker compose up -d`.

## CLI commands

- `create <collection> [flags]`: create a collection with a given optimizer
  config (indexing threshold, `prevent_unoptimized`, vacuum/merge thresholds,
  optimizer/indexing thread caps). Indexing is disabled by default; pass
  `--no-disable-indexing` to leave it on.
- `upload <data_file> <collection>`: bulk-load embeddings from parquet.
- `sample-queries <data_file> <out>`: deterministically sample a held-out
  query set to a JSONL file, reused across all experiment runs.
- `search <collection> <query_file>`: one-shot search-latency measurement.
  `--repeat` runs multiple passes over the query set, `--verbose` logs every
  query's latency instead of just the summary.
- `bench <data_file> <collection> <query_file> <out>`: run a single
  experiment configuration in three stages: upload to completion with no
  search traffic, then search continuously while polling
  `/collections/{collection_name}/optimizations` until the post-load backlog
  drains, then a fixed number of passes once optimizers report idle. Tags
  every search sample `draining` or `steady` accordingly and writes a
  timestamped JSONL event log. Pass `--enable-memory-monitoring` to also poll
  `/collections/{collection_name}/memory` throughout (used for the delayed
  indexing runs, to check search latency against the vector cache).
- `reconfigure <collection> [flags]`: change optimizer/HNSW config on a live
  collection without recreating it. `--enable-indexing` is a convenience flag
  that resets `indexing_threshold_kb` to Qdrant's own default.
- `churn <collection> --fraction 0.25`: deterministically delete a fraction
  of points (to trigger the vacuum optimizer as a controlled event).
- `opt-status <collection>`: dump the current
  `/collections/{collection_name}/optimizations` status.
- `mem-status <collection>`: dump the current
  `/collections/{collection_name}/memory` status.

Run `cargo run -- <command> --help` for full flag lists.

## Running the experiments

```
./scripts/run_all.sh
```

This samples the query set once, then runs each experiment group. Or run
groups individually:

- `scripts/group_a.sh`: indexing threshold / `prevent_unoptimized`
- `scripts/group_b.sh`: segment count / max segment size
- `scripts/group_c.sh`: optimizer/indexing thread budget
- `scripts/group_d.sh`: vacuum thresholds (load, churn, re-measure)
- `scripts/group_e.sh`: delayed indexing, then `reconfigure --enable-indexing`
  to trigger the deferred indexing burst as its own event, re-measure

Each config gets its own collection and its own `results/bench-*.jsonl`, so
groups don't need a storage reset between runs. `scripts/bench_one.sh
<collection> <out.jsonl> [create flags...] [-- [bench flags...]]` runs a
single config manually; flags before `--` go to `create`, flags after go to
`bench`.

## Generating the report

Once the runs you care about have `results/bench-*.jsonl` files:

```
cd packages/results-analysis-scripts && uv run results-analysis-scripts
```

This reads every result file it finds, skips any run it doesn't, and writes
`results/report.html` with per-run latency charts (and memory charts for runs
benched with `--enable-memory-monitoring`).

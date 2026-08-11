# optimizers-in-action

Experiments quantifying how Qdrant's background optimizers (indexing, merge,
vacuum) affect search latency under read-write contention, using the
MS MARCO passage embeddings dataset (~1.76M x 1024-dim vectors).

## Layout

- `src/` — Rust CLI (`optimizers-in-action`) for driving the experiments.
- `scripts/` — shell scripts that run the experiment matrix end-to-end.
- `configs/base.yaml`, `compose.yaml` — local Qdrant via Docker Compose.
- `packages/download-data/` — Python helper (`uv run download-data`) to fetch
  the parquet dataset via Hugging Face.
- `results/` — experiment output (JSONL logs, latency reports), gitignored
  except for a few checked-in examples.

## Setup

1. Download the dataset: `cd packages/download-data && uv run download-data`.
2. Set `QDRANT_URL` (the **gRPC** endpoint, e.g. `http://localhost:6334` or a
   Cloud cluster's `:6334` URL) and, if needed, `QDRANT_API_KEY` — e.g. via a
   `.env` you `source`. `scripts/base.env` holds the dataset path
   (`PARQUET_FILE`) and is sourced automatically by the scripts.
3. If running locally: `docker compose up -d`.

## CLI commands

- `create <collection> [flags]` — create a collection with a given optimizer
  config (indexing threshold, prevent_unoptimized, vacuum/merge thresholds,
  optimizer/indexing thread caps).
- `upload <data_file> <collection>` — bulk-load embeddings from parquet.
- `sample-queries <data_file> <out>` — deterministically sample a held-out
  query set to a JSONL file, reused across all experiment runs.
- `search <collection> <query_file>` — one-shot search-latency measurement.
- `bench <data_file> <collection> <query_file> <out>` — run upload and search
  concurrently; tags every search sample as `loading` (upload in flight) or
  `steady` (optimizers idle), polls `/optimizations` throughout, and writes a
  timestamped JSONL event log plus per-phase latency percentiles.
- `reconfigure <collection> [flags]` — change optimizer/HNSW config on a live
  collection without recreating it.
- `churn <collection> --fraction 0.25` — deterministically delete a fraction
  of points (to trigger the vacuum optimizer as a controlled event).
- `opt-status <collection>` — dump the current `/optimizations` status.

Run `cargo run -- <command> --help` for full flag lists.

## Running the experiments

```
./scripts/run_all.sh
```

This samples the query set once, then runs each experiment group. Or run
groups individually:

- `scripts/group_a.sh` — indexing threshold / `prevent_unoptimized`
- `scripts/group_b.sh` — segment count / max segment size
- `scripts/group_c.sh` — optimizer/indexing thread budget
- `scripts/group_d.sh` — vacuum thresholds (load, churn, re-measure)
- `scripts/group_e.sh` — delayed indexing, then `reconfigure --enable-indexing`
  to trigger the deferred indexing burst as its own event, re-measure

Each config gets its own collection and its own `results/bench-*.jsonl`, so
groups don't need a storage reset between runs. `scripts/bench_one.sh
<collection> <out.jsonl> [create flags...]` runs a single config manually.

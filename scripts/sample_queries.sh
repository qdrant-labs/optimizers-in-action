#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/base.env

cargo run --release -- sample-queries "$PARQUET_FILE" results/queries.jsonl --stride 200 --limit 1000

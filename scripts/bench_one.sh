#!/usr/bin/env bash
# usage: bench_one.sh <collection> <out.jsonl> [create flags...]
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/base.env

collection="$1"; out="$2"; shift 2

cargo run --release -- create "$collection" "$@"
cargo run --release -- bench "$PARQUET_FILE" "$collection" results/queries.jsonl "$out"

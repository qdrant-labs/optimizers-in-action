#!/usr/bin/env bash
# usage: bench_one.sh <collection> <out.jsonl> [create flags...] [-- [bench flags...]]
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/base.env

collection="$1"; out="$2"; shift 2

create_flags=()
bench_flags=()
past_sep=false
for arg in "$@"; do
  if [ "$arg" = "--" ]; then
    past_sep=true
    continue
  fi
  if [ "$past_sep" = true ]; then
    bench_flags+=("$arg")
  else
    create_flags+=("$arg")
  fi
done

cargo run --release -- create "$collection" "${create_flags[@]}"
cargo run --release -- bench "$PARQUET_FILE" "$collection" results/queries.jsonl "$out" "${bench_flags[@]}"

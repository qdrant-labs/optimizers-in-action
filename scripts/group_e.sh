#!/usr/bin/env bash
# delayed indexing -> reconfigure to enable indexing -> observe the indexing-burst latency
set -euo pipefail
cd "$(dirname "$0")/.."

run_one() {
  collection="$1"; out="$2"; shift 2
  ./scripts/bench_one.sh "$collection" "$out" "$@"
  cargo run --release -- reconfigure "$collection" --enable-indexing
  cargo run --release -- opt-status "$collection" > "${out%.jsonl}-optimizers-status.json"
  cargo run --release -- search "$collection" results/queries.jsonl --repeat 5 --verbose > "${out%.jsonl}-post-reconfigure.log"
}

run_one opt-e1-delayed-to-indexed results/bench-e1-delayed-to-indexed.jsonl -- --enable-memory-monitoring
run_one opt-e2-delayed-to-indexed-prevent-unopt results/bench-e2-delayed-to-indexed-prevent-unopt.jsonl --prevent-unoptimized -- --enable-memory-monitoring

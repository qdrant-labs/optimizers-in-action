#!/usr/bin/env bash
# vacuum thresholds — load, churn, then observe search latency + optimizer status
set -euo pipefail
cd "$(dirname "$0")/.."

run_one() {
  collection="$1"; out="$2"; shift 2
  ./scripts/bench_one.sh "$collection" "$out" --no-disable-indexing "$@"
  cargo run --release -- churn "$collection" --fraction 0.25 --verbose
  cargo run --release -- opt-status "$collection"
  cargo run --release -- search "$collection" results/queries.jsonl --repeat 5 --verbose > "${out%.jsonl}-post-churn.log"
}

run_one vac-d1-default results/bench-d1-default-vacuum.jsonl --delete-threshold 0.2 --vacuum-min-vectors-number 1000
run_one vac-d2-aggressive results/bench-d2-aggressive-vacuum.jsonl --delete-threshold 0.5 --vacuum-min-vectors-number 100

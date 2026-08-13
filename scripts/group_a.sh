#!/usr/bin/env bash
# indexing threshold / prevent_unoptimized
set -euo pipefail
cd "$(dirname "$0")/.."

./scripts/bench_one.sh opt-a1-continuous results/bench-a1-continuous.jsonl --no-disable-indexing
./scripts/bench_one.sh opt-a2-delayed results/bench-a2-delayed.jsonl -- --enable-memory-monitoring
./scripts/bench_one.sh opt-a3-prevent-unopt results/bench-a3-prevent-unopt.jsonl --no-disable-indexing --prevent-unoptimized

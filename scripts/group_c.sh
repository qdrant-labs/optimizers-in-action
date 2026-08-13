#!/usr/bin/env bash
# optimization thread budget
set -euo pipefail
cd "$(dirname "$0")/.."

./scripts/bench_one.sh thr-c1-serial results/bench-c1-serial-threads.jsonl --no-disable-indexing --optimizers-threads 1 --indexing-threads 1
./scripts/bench_one.sh thr-c2-default results/bench-c2-default-threads.jsonl --no-disable-indexing

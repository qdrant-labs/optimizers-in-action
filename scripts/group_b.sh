#!/usr/bin/env bash
# segment count / size
set -euo pipefail
cd "$(dirname "$0")/.."

./scripts/bench_one.sh seg-b1-single results/bench-b1-single-segment.jsonl --no-disable-indexing --default-segment-number 1
./scripts/bench_one.sh seg-b2-default results/bench-b2-default-segments.jsonl --no-disable-indexing
./scripts/bench_one.sh seg-b3-many results/bench-b3-many-segments.jsonl --no-disable-indexing --default-segment-number "$(( $(nproc) * 4 ))"
./scripts/bench_one.sh seg-b4-small-max results/bench-b4-small-max-segment.jsonl --no-disable-indexing --max-segment-size-kb 100000

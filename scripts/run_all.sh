#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

./scripts/sample_queries.sh
./scripts/group_a.sh
./scripts/group_b.sh
./scripts/group_c.sh
./scripts/group_d.sh
./scripts/group_e.sh

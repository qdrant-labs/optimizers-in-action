from .stats import compute_stats

OPTIMIZER_ORDER = ["indexing", "merge", "vacuum"]


def _bucketize(search_events, optimizer_events, start_t, end_t, target_buckets=90):
    span = end_t - start_t
    if span <= 0:
        return []
    bucket_count = max(10, min(target_buckets, int(span // 3) or 10))
    bw = span / bucket_count
    latencies = [[] for _ in range(bucket_count)]
    for t, latency_ms in search_events:
        idx = min(bucket_count - 1, max(0, int((t - start_t) // bw)))
        latencies[idx].append(latency_ms)

    opt_sorted = sorted(optimizer_events, key=lambda x: x[0])
    j = 0
    current = set()
    buckets = []
    for i in range(bucket_count):
        bucket_start = start_t + i * bw
        while j < len(opt_sorted) and opt_sorted[j][0] <= bucket_start:
            current = opt_sorted[j][1]
            j += 1
        lats = latencies[i]
        stats = compute_stats(lats) if lats else None
        buckets.append(
            {
                "t": bucket_start,
                "p50": stats.p50 if stats else None,
                "p95": stats.p95 if stats else None,
                "n": len(lats),
                "optimizers": set(current),
            }
        )
    return buckets


def analyze_run(events):
    """Bench now runs in three stages: upload to completion (no search
    traffic), then continuous search while draining the post-load optimizer
    backlog, then a fixed number of passes once optimizers report idle. The
    event log carries two phase_change markers: "draining" (upload just
    finished) and "steady" (optimizers confirmed idle). Each phase is
    bucketed independently so its chart gets its own time/latency scale.
    `memory_series` is only non-empty for runs benched with
    `--enable-memory-monitoring` (the delayed-indexing runs, A2/E1/E2);
    everything else simply has no `memory` events to collect.
    """
    upload_duration = 0.0
    steady_start_t = None
    max_t = 0.0
    draining_optimizers = set()
    steady_optimizers = set()
    draining_latencies = []
    steady_latencies = []
    draining_search_events = []
    steady_search_events = []
    draining_optimizer_events = []
    steady_optimizer_events = []
    memory_series = []

    for e in events:
        max_t = max(max_t, e["t"])
        kind = e["kind"]
        if kind == "phase_change":
            if e["phase"] == "draining":
                upload_duration = e["t"]
            elif e["phase"] == "steady":
                steady_start_t = e["t"]
        elif kind == "search":
            if e["phase"] == "draining":
                draining_latencies.append(e["latency_ms"])
                draining_search_events.append((e["t"], e["latency_ms"]))
            else:
                steady_latencies.append(e["latency_ms"])
                steady_search_events.append((e["t"], e["latency_ms"]))
        elif kind == "optimizer":
            running = set(e.get("running_optimizers", []))
            if e["phase"] == "draining":
                draining_optimizers |= running
                draining_optimizer_events.append((e["t"], running))
            else:
                steady_optimizers |= running
                steady_optimizer_events.append((e["t"], running))
        elif kind == "memory":
            memory_series.append(
                {
                    "t": e["t"],
                    "disk_bytes": e["disk_bytes"],
                    "ram_bytes": e["ram_bytes"],
                    "cached_bytes": e["cached_bytes"],
                    "expected_cache_bytes": e["expected_cache_bytes"],
                }
            )

    memory_series.sort(key=lambda m: m["t"])
    drain_end = steady_start_t if steady_start_t is not None else max_t

    return {
        "draining_stats": compute_stats(draining_latencies),
        "steady_stats": compute_stats(steady_latencies),
        "upload_duration": upload_duration,
        "drain_duration": drain_end - upload_duration,
        "total_duration": max_t,
        "draining_optimizers": draining_optimizers,
        "steady_optimizers": steady_optimizers,
        "draining_buckets": _bucketize(
            draining_search_events, draining_optimizer_events, upload_duration, drain_end
        ),
        "steady_buckets": _bucketize(steady_search_events, steady_optimizer_events, drain_end, max_t),
        "phase_change_t": steady_start_t,
        "memory_series": memory_series,
    }

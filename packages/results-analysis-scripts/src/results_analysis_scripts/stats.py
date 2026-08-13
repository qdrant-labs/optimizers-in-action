import math
from dataclasses import dataclass


@dataclass
class LatencyStats:
    count: int
    min: float
    p50: float
    p95: float
    p99: float
    max: float
    mean: float


def _percentile(sorted_vals, p):
    idx = int(math.floor((len(sorted_vals) - 1) * p + 0.5))
    return sorted_vals[idx]


def compute_stats(latencies_ms):
    if not latencies_ms:
        return None
    vals = sorted(latencies_ms)
    n = len(vals)
    return LatencyStats(
        count=n,
        min=vals[0],
        p50=_percentile(vals, 0.50),
        p95=_percentile(vals, 0.95),
        p99=_percentile(vals, 0.99),
        max=vals[-1],
        mean=sum(vals) / n,
    )

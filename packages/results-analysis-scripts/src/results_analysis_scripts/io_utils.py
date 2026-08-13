import json
import re

_SUMMARY_RE = re.compile(
    r"search: n=(?P<n>\d+) min=(?P<min>\S+) p50=(?P<p50>\S+) p95=(?P<p95>\S+) "
    r"p99=(?P<p99>\S+) max=(?P<max>\S+) mean=(?P<mean>\S+) \((?P<qps>[\d.]+) qps\)"
)


def load_jsonl(path):
    events = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line:
                events.append(json.loads(line))
    return events


def load_json(path):
    with open(path) as f:
        return json.load(f)


def _duration_to_ms(text):
    text = text.strip()
    for suffix, div in (("ns", 1_000_000.0), ("µs", 1_000.0), ("us", 1_000.0), ("ms", 1.0)):
        if text.endswith(suffix):
            return float(text[: -len(suffix)]) / div
    if text.endswith("s"):
        return float(text[:-1]) * 1000.0
    raise ValueError(f"unrecognized duration: {text!r}")


def parse_post_log_summary(path):
    with open(path) as f:
        for line in f:
            m = _SUMMARY_RE.search(line)
            if m:
                return {
                    "n": int(m["n"]),
                    "min": _duration_to_ms(m["min"]),
                    "p50": _duration_to_ms(m["p50"]),
                    "p95": _duration_to_ms(m["p95"]),
                    "p99": _duration_to_ms(m["p99"]),
                    "max": _duration_to_ms(m["max"]),
                    "mean": _duration_to_ms(m["mean"]),
                    "qps": float(m["qps"]),
                }
    return None

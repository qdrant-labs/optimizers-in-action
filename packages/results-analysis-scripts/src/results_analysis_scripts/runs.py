from dataclasses import dataclass


@dataclass
class Run:
    key: str
    group: str
    label: str
    description: str
    jsonl: str
    status_json: str | None = None
    post_log: str | None = None
    post_log_label: str | None = None


RUNS = [
    Run(
        "a1",
        "A",
        "A1 · continuous indexing",
        "Qdrant's own default: indexing runs continuously during upload (--no-disable-indexing).",
        "bench-a1-continuous.jsonl",
    ),
    Run(
        "a2",
        "A",
        "A2 · delayed indexing",
        "Indexing disabled during upload (this tool's default); never re-enabled in this run.",
        "bench-a2-delayed.jsonl",
    ),
    Run(
        "a3",
        "A",
        "A3 · prevent-unoptimized",
        "Continuous indexing + prevent_unoptimized=true (queries skip unindexed segments).",
        "bench-a3-prevent-unopt.jsonl",
    ),
    Run(
        "b1",
        "B",
        "B1 · single segment",
        "default_segment_number=1, continuous indexing.",
        "bench-b1-single-segment.jsonl",
    ),
    Run(
        "b2",
        "B",
        "B2 · default segments",
        "Default segment count (~num CPUs), continuous indexing.",
        "bench-b2-default-segments.jsonl",
    ),
    Run(
        "b3",
        "B",
        "B3 · many segments",
        "default_segment_number = 4x CPU count, continuous indexing.",
        "bench-b3-many-segments.jsonl",
    ),
    Run(
        "b4",
        "B",
        "B4 · small max segment",
        "max_segment_size_kb=100000, continuous indexing.",
        "bench-b4-small-max-segment.jsonl",
    ),
    Run(
        "c1",
        "C",
        "C1 · serial optimizer threads",
        "max_optimization_threads=1, max_indexing_threads=1.",
        "bench-c1-serial-threads.jsonl",
    ),
    Run(
        "c2",
        "C",
        "C2 · default threads",
        "Default optimizer / indexing thread budget.",
        "bench-c2-default-threads.jsonl",
    ),
    Run(
        "c3",
        "C",
        "C3 · parallel optimizer threads",
        "max_optimization_threads=num CPUs, max_indexing_threads=4.",
        "bench-c3-parallel-threads.jsonl",
    ),
    Run(
        "d1",
        "D",
        "D1 · default vacuum thresholds",
        "deleted_threshold=0.2, vacuum_min_vector_number=1000; then delete 25% of points and re-measure.",
        "bench-d1-default-vacuum.jsonl",
        "bench-d1-default-vacuum-optimizers-status.json",
        "bench-d1-default-vacuum-post-churn.log",
        "after churn",
    ),
    Run(
        "d2",
        "D",
        "D2 · aggressive vacuum thresholds",
        "deleted_threshold=0.5, vacuum_min_vector_number=100; then delete 25% of points and re-measure.",
        "bench-d2-aggressive-vacuum.jsonl",
        "bench-d2-aggressive-vacuum-optimizers-status.json",
        "bench-d2-aggressive-vacuum-post-churn.log",
        "after churn",
    ),
    Run(
        "e1",
        "E",
        "E1 · delayed indexing, then enabled",
        "Indexing disabled during load; reconfigure --enable-indexing; then re-measure.",
        "bench-e1-delayed-to-indexed.jsonl",
        "bench-e1-delayed-to-indexed-optimizers-status.json",
        "bench-e1-delayed-to-indexed-post-reconfigure.log",
        "after reconfigure",
    ),
    Run(
        "e2",
        "E",
        "E2 · delayed indexing, then enabled + prevent-unopt",
        "Same as E1, plus prevent_unoptimized=true.",
        "bench-e2-delayed-to-indexed-prevent-unopt.jsonl",
        "bench-e2-delayed-to-indexed-prevent-unopt-optimizers-status.json",
        "bench-e2-delayed-to-indexed-prevent-unopt-post-reconfigure.log",
        "after reconfigure",
    ),
]

GROUPS = [
    ("A", "Indexing threshold / prevent_unoptimized"),
    ("B", "Segment count & size"),
    ("C", "Optimizer / indexing thread budget"),
    ("D", "Vacuum thresholds"),
    ("E", "Delayed indexing, then reconfigured"),
]

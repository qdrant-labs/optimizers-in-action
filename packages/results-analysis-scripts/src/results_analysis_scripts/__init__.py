import sys
from pathlib import Path

from .analysis import analyze_run
from .io_utils import load_json, load_jsonl, parse_post_log_summary
from .report import build_html
from .runs import RUNS


def _find_repo_root(start):
    for p in [start, *start.parents]:
        if (p / "Cargo.toml").exists():
            return p
    raise RuntimeError("could not locate repo root (no Cargo.toml found)")


def _status_snapshot(status_json):
    result = status_json["result"]
    running = {r["optimizer"] for r in result.get("running", [])}
    summary = result["summary"]
    return {
        "running": running,
        "queued_optimizations": summary["queued_optimizations"],
        "queued_points": summary["queued_points"],
    }


def main():
    repo_root = _find_repo_root(Path(__file__).resolve())
    results_dir = repo_root / "results"
    out_path = results_dir / "report.html"

    analyzed_by_key = {}
    runs_by_group = {}
    for run in RUNS:
        jsonl_path = results_dir / run.jsonl
        if not jsonl_path.exists():
            print(f"skipping {run.key}: missing {jsonl_path}", file=sys.stderr)
            continue

        analyzed = analyze_run(load_jsonl(jsonl_path))

        if run.post_log and (results_dir / run.post_log).exists():
            analyzed["post_log"] = parse_post_log_summary(results_dir / run.post_log)
        if run.status_json and (results_dir / run.status_json).exists():
            analyzed["status_snapshot"] = _status_snapshot(
                load_json(results_dir / run.status_json)
            )

        analyzed_by_key[run.key] = analyzed
        runs_by_group.setdefault(run.group, []).append(run)

    html = build_html(analyzed_by_key, runs_by_group)
    out_path.write_text(html)
    print(f"wrote {out_path}")

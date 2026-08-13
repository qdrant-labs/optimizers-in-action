import json
import math

from .analysis import OPTIMIZER_ORDER

WIDTH = 720
LEFT = 74
RIGHT = 12
TOP = 10
MAIN_H = 130
STRIP_H = 14
STRIP_GAP = 2
XAXIS_H = 18
HEIGHT = TOP + MAIN_H + STRIP_GAP + len(OPTIMIZER_ORDER) * (STRIP_H + STRIP_GAP) + XAXIS_H

PLOT_W = WIDTH - LEFT - RIGHT

OPT_CLASS = {"indexing": "opt-indexing", "merge": "opt-merge", "vacuum": "opt-vacuum"}

MEM_MAIN_H = 160
MEM_HEIGHT = TOP + MEM_MAIN_H + XAXIS_H


def _log_ticks(min_v, max_v):
    min_v = max(min_v, 0.01)
    max_v = max(max_v, min_v * 1.01)
    lo = math.floor(math.log10(min_v))
    hi = math.ceil(math.log10(max_v))
    if hi == lo:
        hi = lo + 1
    return [10**e for e in range(lo, hi + 1)]


def _linear_ticks(max_v, target=4):
    if max_v <= 0:
        return [0.0, 1.0]
    raw_step = max_v / target
    magnitude = 10 ** math.floor(math.log10(raw_step))
    step = magnitude
    for m in (1, 2, 5, 10):
        step = m * magnitude
        if step >= raw_step:
            break
    ticks = [0.0]
    while ticks[-1] < max_v:
        ticks.append(ticks[-1] + step)
    return ticks


def _fmt_bytes(v):
    if v >= 1e9:
        return f"{v / 1e9:.1f}GB"
    if v >= 1e6:
        return f"{v / 1e6:.0f}MB"
    if v >= 1e3:
        return f"{v / 1e3:.0f}KB"
    return f"{v:.0f}B"


def _fmt_ms(v):
    if v >= 1000:
        return f"{v / 1000:g}s"
    return f"{v:g}ms"


def _fmt_t(t):
    return f"{t:g}s"


def _render_single(key, buckets, start_t, end_t):
    time_span = end_t - start_t
    if not buckets or time_span <= 0:
        return "<p class='no-data'>no search samples recorded</p>"

    known_p95 = [b["p95"] for b in buckets if b["p95"] is not None]
    known_p50 = [b["p50"] for b in buckets if b["p50"] is not None]
    if not known_p95:
        return "<p class='no-data'>no search samples recorded</p>"

    min_v = min(known_p50 + known_p95)
    max_v = max(known_p95)
    ticks = _log_ticks(min_v, max_v)
    lo, hi = math.log10(ticks[0]), math.log10(ticks[-1])
    span = hi - lo

    def x(t):
        return LEFT + ((t - start_t) / time_span) * PLOT_W

    def y(v):
        v = max(v, ticks[0])
        frac = (math.log10(v) - lo) / span
        return TOP + MAIN_H - frac * MAIN_H

    def polylines(field):
        segs = []
        cur = []
        for b in buckets:
            v = b[field]
            if v is None:
                if len(cur) > 1:
                    segs.append(cur)
                cur = []
                continue
            cur.append((x(b["t"]), y(v)))
        if len(cur) > 1:
            segs.append(cur)
        return segs

    parts = [
        f'<svg class="chart-svg" viewBox="0 0 {WIDTH} {HEIGHT}" '
        f'width="{WIDTH}" height="{HEIGHT}" data-key="{key}" '
        f'data-start="{start_t}" data-total="{end_t}" data-left="{LEFT}" data-plotw="{PLOT_W}">'
    ]

    for tk in ticks:
        gy = y(tk)
        parts.append(
            f'<line class="gridline" x1="{LEFT}" y1="{gy:.1f}" x2="{WIDTH - RIGHT}" y2="{gy:.1f}" />'
        )
        parts.append(
            f'<text class="tick-label" x="{LEFT - 6}" y="{gy:.1f}" text-anchor="end" dominant-baseline="middle">{_fmt_ms(tk)}</text>'
        )

    for field, cls in (("p50", "line-p50"), ("p95", "line-p95")):
        for seg in polylines(field):
            points = " ".join(f"{px:.1f},{py:.1f}" for px, py in seg)
            parts.append(f'<polyline class="{cls}" points="{points}" />')

    strip_top = TOP + MAIN_H + STRIP_GAP
    for i, opt in enumerate(OPTIMIZER_ORDER):
        row_y = strip_top + i * (STRIP_H + STRIP_GAP)
        parts.append(
            f'<text class="strip-label" x="{LEFT - 6}" y="{row_y + STRIP_H / 2:.1f}" '
            f'text-anchor="end" dominant-baseline="middle">{opt}</text>'
        )
        bw = max(PLOT_W / len(buckets), 1.0)
        for b in buckets:
            bx = x(b["t"])
            on = opt in b["optimizers"]
            cls = OPT_CLASS[opt] if on else "opt-off"
            parts.append(
                f'<rect class="{cls}" x="{bx:.1f}" y="{row_y}" width="{bw:.1f}" height="{STRIP_H}" />'
            )

    axis_y = strip_top + len(OPTIMIZER_ORDER) * (STRIP_H + STRIP_GAP) + 12
    for frac in (0, 0.25, 0.5, 0.75, 1.0):
        t = start_t + frac * time_span
        tx = x(t)
        parts.append(
            f'<text class="tick-label" x="{tx:.1f}" y="{axis_y}" text-anchor="middle">{_fmt_t(t)}</text>'
        )

    hit_h = strip_top + len(OPTIMIZER_ORDER) * (STRIP_H + STRIP_GAP) - TOP
    parts.append(f'<line class="crosshair" x1="0" y1="{TOP}" x2="0" y2="{TOP + hit_h}" style="opacity:0" />')
    parts.append(
        f'<rect class="hit-layer" x="{LEFT}" y="{TOP}" width="{PLOT_W}" height="{hit_h}" fill="transparent" />'
    )
    parts.append("</svg>")

    data = [
        {
            "t": b["t"],
            "p50": b["p50"],
            "p95": b["p95"],
            "n": b["n"],
            "optimizers": sorted(b["optimizers"]),
        }
        for b in buckets
    ]
    parts.append(
        f'<script type="application/json" class="chart-data" data-key="{key}">{json.dumps(data)}</script>'
    )

    return f'<div class="chart" data-key="{key}">' + "".join(parts) + "</div>"


def render_memory_chart(run_key, memory_series, start_t, drain_end, end_t):
    """One continuous chart across draining + steady (unlike the latency
    charts, which are deliberately split per phase): the cache warm-up this
    is meant to explain straddles the draining/steady boundary, so splitting
    it in two would hide the transition. Only called for runs benched with
    `--enable-memory-monitoring`; every other run has an empty
    `memory_series` and renders nothing.
    """
    time_span = end_t - start_t
    if not memory_series or time_span <= 0:
        return ""

    max_v = max(
        max(m["disk_bytes"], m["ram_bytes"], m["cached_bytes"], m["expected_cache_bytes"])
        for m in memory_series
    )
    ticks = _linear_ticks(max_v)
    top_v = ticks[-1] if ticks[-1] > 0 else 1.0

    def x(t):
        return LEFT + ((t - start_t) / time_span) * PLOT_W

    def y(v):
        return TOP + MEM_MAIN_H - (v / top_v) * MEM_MAIN_H

    def polyline(field):
        pts = [(x(m["t"]), y(m[field])) for m in memory_series]
        return " ".join(f"{px:.1f},{py:.1f}" for px, py in pts)

    key = f"{run_key}-memory"
    parts = [
        f'<svg class="chart-svg" viewBox="0 0 {WIDTH} {MEM_HEIGHT}" '
        f'width="{WIDTH}" height="{MEM_HEIGHT}" data-key="{key}">'
    ]

    for tk in ticks:
        gy = y(tk)
        parts.append(
            f'<line class="gridline" x1="{LEFT}" y1="{gy:.1f}" x2="{WIDTH - RIGHT}" y2="{gy:.1f}" />'
        )
        parts.append(
            f'<text class="tick-label" x="{LEFT - 6}" y="{gy:.1f}" text-anchor="end" dominant-baseline="middle">{_fmt_bytes(tk)}</text>'
        )

    if start_t < drain_end < end_t:
        px = x(drain_end)
        parts.append(
            f'<line class="phase-line" x1="{px:.1f}" y1="{TOP}" x2="{px:.1f}" y2="{TOP + MEM_MAIN_H}" />'
        )
        parts.append(f'<text class="phase-label" x="{px + 4:.1f}" y="{TOP + 10}">steady →</text>')

    for field, cls in (
        ("disk_bytes", "mem-disk"),
        ("ram_bytes", "mem-ram"),
        ("expected_cache_bytes", "mem-expected"),
        ("cached_bytes", "mem-cached"),
    ):
        parts.append(f'<polyline class="{cls}" points="{polyline(field)}" />')

    axis_y = TOP + MEM_MAIN_H + 12
    for frac in (0, 0.25, 0.5, 0.75, 1.0):
        t = start_t + frac * time_span
        parts.append(
            f'<text class="tick-label" x="{x(t):.1f}" y="{axis_y}" text-anchor="middle">{_fmt_t(t)}</text>'
        )

    parts.append("</svg>")

    caption = (
        "<p class='mem-caption'>"
        "<span class='mem-key mem-key-cached'>cached</span> vs "
        "<span class='mem-key mem-key-expected'>expected cache</span> vs "
        "<span class='mem-key mem-key-disk'>on disk</span> vs "
        "<span class='mem-key mem-key-ram'>in RAM</span>"
        "</p>"
    )

    return f'<div class="chart">' + "".join(parts) + "</div>" + caption


def render_phase_charts(run_key, draining_buckets, steady_buckets, upload_duration, drain_end, total_duration):
    draining_chart = _render_single(f"{run_key}-draining", draining_buckets, upload_duration, drain_end)
    steady_chart = _render_single(f"{run_key}-steady", steady_buckets, drain_end, total_duration)
    return (
        "<div class='chart-pair'>"
        f"<div class='chart-slot'><h4 class='chart-title'>draining</h4>{draining_chart}</div>"
        f"<div class='chart-slot'><h4 class='chart-title'>steady</h4>{steady_chart}</div>"
        "</div>"
    )

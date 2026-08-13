from .charts import render_memory_chart, render_phase_charts
from .runs import GROUPS

CSS = """
:root {
  color-scheme: light;
  --page: #f9f9f7;
  --surface: #fcfcfb;
  --text-primary: #0b0b0b;
  --text-secondary: #52514e;
  --text-muted: #898781;
  --gridline: #e1e0d9;
  --baseline: #c3c2b7;
  --border: rgba(11,11,11,0.10);
  --opt-indexing: #2a78d6;
  --opt-merge: #eb6834;
  --opt-vacuum: #1baf7a;
  --opt-off: #e1e0d9;
}
@media (prefers-color-scheme: dark) {
  :root {
    color-scheme: dark;
    --page: #0d0d0d;
    --surface: #1a1a19;
    --text-primary: #ffffff;
    --text-secondary: #c3c2b7;
    --text-muted: #898781;
    --gridline: #2c2c2a;
    --baseline: #383835;
    --border: rgba(255,255,255,0.10);
    --opt-indexing: #3987e5;
    --opt-merge: #d95926;
    --opt-vacuum: #199e70;
    --opt-off: #2c2c2a;
  }
}
* { box-sizing: border-box; }
body {
  margin: 0;
  background: var(--page);
  color: var(--text-primary);
  font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
  line-height: 1.45;
}
main { max-width: 900px; margin: 0 auto; padding: 32px 20px 80px; }
h1 { font-size: 1.6rem; margin-bottom: 4px; }
h2 { font-size: 1.25rem; margin: 40px 0 4px; border-bottom: 1px solid var(--border); padding-bottom: 8px; }
h2 .group-desc { display: block; font-size: 0.9rem; font-weight: 400; color: var(--text-secondary); margin-top: 2px; }
h3 { font-size: 1.02rem; margin: 28px 0 2px; }
p.intro { color: var(--text-secondary); max-width: 640px; }
p.run-desc { color: var(--text-secondary); font-size: 0.92rem; margin: 2px 0 12px; }
p.no-data { color: var(--text-muted); font-style: italic; }
.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 16px 18px 8px;
  margin-bottom: 20px;
}
table { border-collapse: collapse; width: 100%; font-size: 0.86rem; margin-bottom: 8px; }
table.stats th, table.stats td {
  text-align: right;
  padding: 3px 8px;
  font-variant-numeric: tabular-nums;
}
table.stats th:first-child, table.stats td:first-child { text-align: left; font-variant-numeric: normal; }
table.stats thead th { color: var(--text-muted); font-weight: 500; border-bottom: 1px solid var(--gridline); }
table.stats tbody th { color: var(--text-secondary); font-weight: 500; }
table.stats tbody tr:not(:last-child) td, table.stats tbody tr:not(:last-child) th { border-bottom: 1px solid var(--gridline); }
.meta-line { color: var(--text-muted); font-size: 0.82rem; margin: 4px 0 14px; }
.legend { display: flex; gap: 18px; align-items: center; margin: 10px 0 24px; font-size: 0.86rem; color: var(--text-secondary); flex-wrap: wrap; }
.legend .swatch { display: inline-block; width: 12px; height: 12px; border-radius: 2px; margin-right: 6px; vertical-align: -1px; }
.legend .sw-indexing { background: var(--opt-indexing); }
.legend .sw-merge { background: var(--opt-merge); }
.legend .sw-vacuum { background: var(--opt-vacuum); }
.legend .sw-p95 { background: var(--text-primary); }
.legend .sw-p50 { background: var(--text-secondary); }
.chart-pair { display: flex; flex-direction: column; gap: 16px; }
.chart-slot { min-width: 0; }
.chart-title {
  margin: 0 0 4px;
  font-size: 0.78rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.03em;
  color: var(--text-muted);
}
.chart { position: relative; overflow-x: auto; }
.chart-svg { display: block; max-width: 100%; height: auto; }
.gridline { stroke: var(--gridline); stroke-width: 1; }
.tick-label { fill: var(--text-muted); font-size: 9px; }
.line-p95 { fill: none; stroke: var(--text-primary); stroke-width: 2; stroke-linejoin: round; stroke-linecap: round; }
.line-p50 { fill: none; stroke: var(--text-secondary); stroke-width: 1.5; stroke-linejoin: round; stroke-linecap: round; opacity: 0.8; }
.strip-label { fill: var(--text-muted); font-size: 9px; }
.opt-indexing { fill: var(--opt-indexing); }
.opt-merge { fill: var(--opt-merge); }
.opt-vacuum { fill: var(--opt-vacuum); }
.opt-off { fill: var(--opt-off); }
.crosshair { stroke: var(--text-muted); stroke-width: 1; }
.hit-layer { cursor: crosshair; }
.phase-line { stroke: var(--baseline); stroke-width: 1; stroke-dasharray: 3 3; }
.phase-label { fill: var(--text-muted); font-size: 9px; }
.mem-cached { fill: none; stroke: var(--text-primary); stroke-width: 2; stroke-linejoin: round; stroke-linecap: round; }
.mem-expected { fill: none; stroke: var(--text-secondary); stroke-width: 1.5; stroke-dasharray: 6 3; }
.mem-disk { fill: none; stroke: var(--text-muted); stroke-width: 1; stroke-dasharray: 1 3; opacity: 0.8; }
.mem-ram { fill: none; stroke: var(--text-muted); stroke-width: 1; stroke-dasharray: 7 2 2 2; opacity: 0.8; }
.mem-caption { font-size: 0.78rem; color: var(--text-muted); margin: 6px 0 4px; }
.mem-key { margin: 0 2px; }
.mem-key-cached { color: var(--text-primary); font-weight: 600; }
.mem-key-expected { color: var(--text-secondary); }
.mem-key-disk, .mem-key-ram { color: var(--text-muted); }
#tooltip {
  position: fixed;
  display: none;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 10px;
  font-size: 0.8rem;
  box-shadow: 0 4px 14px rgba(0,0,0,0.18);
  pointer-events: none;
  z-index: 10;
  min-width: 140px;
}
.tooltip-row { display: flex; justify-content: space-between; gap: 12px; }
.tooltip-label { color: var(--text-secondary); text-transform: capitalize; }
.followup { margin-top: 14px; padding-top: 12px; border-top: 1px dashed var(--gridline); font-size: 0.86rem; }
.followup h4 { margin: 0 0 6px; font-size: 0.88rem; color: var(--text-secondary); }
"""

TOOLTIP_JS = """
(function () {
  var tooltip = document.getElementById('tooltip');
  document.querySelectorAll('.chart').forEach(function (chartDiv) {
    var svg = chartDiv.querySelector('.chart-svg');
    var hit = svg.querySelector('.hit-layer');
    var crosshair = svg.querySelector('.crosshair');
    var dataScript = chartDiv.querySelector('.chart-data');
    if (!dataScript) return;
    var data = JSON.parse(dataScript.textContent);
    var left = parseFloat(svg.dataset.left);
    var plotw = parseFloat(svg.dataset.plotw);

    function handleMove(evt) {
      var rect = svg.getBoundingClientRect();
      var scale = rect.width / svg.viewBox.baseVal.width;
      var svgX = (evt.clientX - rect.left) / scale;
      var frac = (svgX - left) / plotw;
      frac = Math.max(0, Math.min(1, frac));
      var idx = Math.round(frac * (data.length - 1));
      var d = data[idx];
      if (!d) return;
      var cx = left + frac * plotw;
      crosshair.setAttribute('x1', cx);
      crosshair.setAttribute('x2', cx);
      crosshair.style.opacity = 1;

      var optimizers = d.optimizers.length ? d.optimizers.join(', ') : 'idle';
      tooltip.textContent = '';
      var rows = [
        ['t', d.t.toFixed(1) + 's'],
        ['p50', d.p50 != null ? d.p50.toFixed(1) + 'ms' : String.fromCharCode(8211)],
        ['p95', d.p95 != null ? d.p95.toFixed(1) + 'ms' : String.fromCharCode(8211)],
        ['n', String(d.n)],
        ['optimizers', optimizers],
      ];
      rows.forEach(function (pair) {
        var row = document.createElement('div');
        row.className = 'tooltip-row';
        var label = document.createElement('span');
        label.className = 'tooltip-label';
        label.textContent = pair[0];
        var value = document.createElement('strong');
        value.textContent = pair[1];
        row.appendChild(label);
        row.appendChild(value);
        tooltip.appendChild(row);
      });
      tooltip.style.display = 'block';
      tooltip.style.left = (evt.clientX + 14) + 'px';
      tooltip.style.top = (evt.clientY + 14) + 'px';
    }

    function handleLeave() {
      tooltip.style.display = 'none';
      crosshair.style.opacity = 0;
    }

    hit.addEventListener('pointermove', handleMove);
    hit.addEventListener('pointerleave', handleLeave);
  });
})();
"""


def _fmt_ms(v):
    if v is None:
        return "–"
    if v >= 1000:
        return f"{v / 1000:.2f} s"
    return f"{v:.1f} ms"


def _fmt_s(v):
    if v is None:
        return "–"
    return f"{v:.1f} s"


def _stats_row(label, stats):
    if stats is None:
        return f"<tr><th>{label}</th><td colspan='6'>no samples</td></tr>"
    return (
        f"<tr><th>{label}</th>"
        f"<td>{stats.count}</td>"
        f"<td>{_fmt_ms(stats.p50)}</td>"
        f"<td>{_fmt_ms(stats.p95)}</td>"
        f"<td>{_fmt_ms(stats.p99)}</td>"
        f"<td>{_fmt_ms(stats.max)}</td>"
        f"<td>{_fmt_ms(stats.mean)}</td></tr>"
    )


def _optimizers_label(opts):
    return ", ".join(sorted(opts)) if opts else "none observed"


def _run_section(run, analyzed):
    a = analyzed
    stats_table = (
        "<table class='stats'><thead><tr>"
        "<th>phase</th><th>n</th><th>p50</th><th>p95</th><th>p99</th><th>max</th><th>mean</th>"
        "</tr></thead><tbody>"
        + _stats_row("draining", a["draining_stats"])
        + _stats_row("steady", a["steady_stats"])
        + "</tbody></table>"
    )
    drain_end = a["upload_duration"] + a["drain_duration"]
    meta = (
        f"<div class='meta-line'>upload: 0–{_fmt_s(a['upload_duration'])} (no search traffic) &middot; "
        f"draining: {_fmt_s(a['upload_duration'])}–{_fmt_s(drain_end)} "
        f"(optimizers seen: {_optimizers_label(a['draining_optimizers'])}) &middot; "
        f"steady: {_fmt_s(drain_end)}–{_fmt_s(a['total_duration'])} "
        f"(optimizers seen: {_optimizers_label(a['steady_optimizers'])})</div>"
    )
    chart = render_phase_charts(
        run.key,
        a["draining_buckets"],
        a["steady_buckets"],
        a["upload_duration"],
        a["upload_duration"] + a["drain_duration"],
        a["total_duration"],
    )

    memory_block = ""
    if a.get("memory_series"):
        memory_chart = render_memory_chart(
            run.key, a["memory_series"], a["upload_duration"], drain_end, a["total_duration"]
        )
        if memory_chart:
            memory_block = f"<h4 class='chart-title'>cache (disk vs RAM vs page cache)</h4>{memory_chart}"

    followup = ""
    if a.get("post_log") or a.get("status_snapshot"):
        rows = ""
        if a.get("post_log"):
            pl = a["post_log"]
            rows += (
                f"<tr><th>search {run.post_log_label}</th>"
                f"<td>{pl['n']}</td><td>{_fmt_ms(pl['p50'])}</td><td>{_fmt_ms(pl['p95'])}</td>"
                f"<td>{_fmt_ms(pl['p99'])}</td><td>{_fmt_ms(pl['max'])}</td><td>{_fmt_ms(pl['mean'])}</td></tr>"
            )
        table = ""
        if rows:
            table = (
                "<table class='stats'><thead><tr>"
                "<th>phase</th><th>n</th><th>p50</th><th>p95</th><th>p99</th><th>max</th><th>mean</th>"
                "</tr></thead><tbody>" + rows + "</tbody></table>"
            )
        snap_line = ""
        if a.get("status_snapshot"):
            snap = a["status_snapshot"]
            snap_line = (
                f"<div class='meta-line'>optimizer status {run.post_log_label}: "
                f"running={_optimizers_label(snap['running'])}, "
                f"queued_optimizations={snap['queued_optimizations']}, "
                f"queued_points={snap['queued_points']}</div>"
            )
        followup = f"<div class='followup'><h4>{run.post_log_label}</h4>{table}{snap_line}</div>"

    return (
        f"<h3>{run.label}</h3>"
        f"<p class='run-desc'>{run.description}</p>"
        f"<div class='card'>{stats_table}{meta}{chart}{memory_block}{followup}</div>"
    )


def build_html(analyzed_by_key, runs_by_group):
    body_groups = []
    for group_key, group_desc in GROUPS:
        runs = runs_by_group.get(group_key, [])
        if not runs:
            continue
        sections = "".join(
            _run_section(run, analyzed_by_key[run.key]) for run in runs
        )
        body_groups.append(
            f"<h2>Group {group_key}<span class='group-desc'>{group_desc}</span></h2>{sections}"
        )

    legend = (
        "<div class='legend'>"
        "<span><span class='swatch sw-p95'></span>p95 latency</span>"
        "<span><span class='swatch sw-p50'></span>p50 latency</span>"
        "<span><span class='swatch sw-indexing'></span>indexing running</span>"
        "<span><span class='swatch sw-merge'></span>merge running</span>"
        "<span><span class='swatch sw-vacuum'></span>vacuum running</span>"
        "</div>"
    )

    return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>optimizers-in-action — results report</title>
<style>{CSS}</style>
</head>
<body>
<main>
  <h1>optimizers-in-action — intermediate results report</h1>
  <p class="intro">Each run uploads ~1.76M MS MARCO passage embeddings into Qdrant
  to completion first, with no concurrent search traffic. Search then runs
  continuously against the fully-loaded collection while polling
  <code>/optimizations</code> every ~2s, tagged <strong>draining</strong> until
  optimizers report nothing running or queued, then <strong>steady</strong> for
  a fixed number of passes once genuinely idle. Charts cover the draining +
  steady window (the silent upload prefix is cropped) and show p50/p95 latency
  per time bucket against which optimizer (indexing / merge / vacuum) was
  running at that moment. Groups D and E chain a follow-up step (churn or
  reconfigure) onto the base run to trigger a specific optimizer as a
  controlled, isolated event. The delayed-indexing runs (A2, E1, E2) also poll
  <code>/collections/[name]/memory</code> and show a disk/RAM/page-cache chart;
  every other run has that monitoring off and simply has no such chart.</p>
  {legend}
  {"".join(body_groups)}
</main>
<div id="tooltip"></div>
<script>{TOOLTIP_JS}</script>
</body>
</html>
"""

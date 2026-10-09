"""Plot formal samples without reopening diagnostic targets."""
import json, datetime, sys
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

root = Path(sys.argv[1])
batch = sys.argv[2]
output = Path(sys.argv[3])
output.mkdir(parents=True, exist_ok=True)
def load(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))
def epoch(value):
    return datetime.datetime.fromisoformat(value.replace('Z', '+00:00')).timestamp() * 1000

fig, axes = plt.subplots(4, 2, figsize=(12, 12), sharex='col')
for column, variant in enumerate(('before', 'after')):
    for number, color in zip((1, 2, 3), ('#2563eb', '#ea580c', '#16a34a')):
        label = f'{variant}-r{number}'
        directory = root / batch / label
        events = [json.loads(line) for line in (directory / 'events.jsonl').read_text(encoding='utf-8').splitlines()]
        marks = {row['event']['command'].get('phase'): row['at'] for row in events if row['event']['kind'] == 'command'}
        start, recovery, end = (marks[key] for key in ('continuous-start', 'recovery-start', 'complete'))
        samples = load(directory / f'{label}.json')['samples']
        samples = [row for row in samples if start <= epoch(row['timestampUtc']) <= end]
        # Align at pause; negative time is continuous switching, positive recovery.
        x = [(epoch(row['timestampUtc']) - recovery) / 60_000 for row in samples]
        values = [
            [row['totalPrivateBytes'] / 2**20 for row in samples],
            [sum(p['privateBytes'] for p in row['processes'] if p['role'] == 'renderer') / 2**20 for row in samples],
            [sum(g['committedBytes'] for p in row['processes'] for g in p['gpuMemory']) / 2**20 for row in samples],
            [row['totalCpuPercentOfMachine'] for row in samples],
        ]
        for axis, series in zip(axes[:, column], values):
            axis.plot(x, series, label=f'Round {number}', color=color, linewidth=.8, alpha=.85)
    axes[0, column].set_title(f'{variant.title()} M3 — continuous renderer')
    for axis in axes[:, column]:
        axis.axvline(0, color='#64748b', linestyle='--', linewidth=1)
        axis.axvspan(0, 5, color='#e2e8f0', alpha=.4)
        axis.grid(alpha=.2)
    axes[-1, column].set_xlabel('Minutes relative to pause (0); recovery window shaded')
for axis, title in zip(axes[:, 0], ('Tree Private Commit (MiB)', 'Renderer Private Commit (MiB)', 'GPU committed (MiB)', 'CPU (% of machine)')):
    axis.set_ylabel(title)
for row in range(4):
    bottom = min(axes[row, 0].get_ylim()[0], axes[row, 1].get_ylim()[0])
    top = max(axes[row, 0].get_ylim()[1], axes[row, 1].get_ylim()[1])
    for axis in axes[row, :]:
        axis.set_ylim(max(0, bottom), top)
axes[0, 0].legend(loc='upper left', fontsize=9)
fig.suptitle('Matched Release Benchmark C: 100 tracks + 5-minute recovery; no attached profiler', fontsize=12)
fig.tight_layout(rect=(0, 0, 1, .97))
fig.savefig(output / 'release-c-curves.png', dpi=160)
fig.savefig(output / 'release-c-curves.svg')
svg = output / 'release-c-curves.svg'
svg.write_text('\n'.join(line.rstrip() for line in svg.read_text(encoding='utf-8').splitlines()) + '\n', encoding='utf-8')
plt.close(fig)

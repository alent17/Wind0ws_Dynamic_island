"""Publish compact experiment evidence, excluding raw profiles/private queue."""
import json, shutil, sys
from pathlib import Path

root, output, batch = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
output.mkdir(parents=True, exist_ok=True)
def load(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))
def save(name, data):
    (output / name).write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding='utf-8')
comparison = load(root / 'comparison.json')
quality = load(root / 'quality-comparison.json')
quality_batch=quality['diagnosticBatch']
assert quality['samples'] == 100
save('comparison.json', comparison)
save('quality-comparison.json', quality)
diagnostics = []
for variant in ('before', 'after'):
    for mode in ('continuous', 'reopen'):
        report = load(root / (quality_batch if mode=='continuous' else 'diagnostic') / f'{variant}-{mode}' / 'artwork' / 'artwork-acceptance.json')
        assert not report.get('failure') and not report['errors']
        assert len(report['tracks']) == 100
        assert all(row['state']['urlMatchesPixels'] is True for row in report['tracks'])
        assert len(report['reopens']) == (4 if mode == 'reopen' else 0)
        assert all(row['detachedCanvasNodes'] == 0 for row in report['snapshots'])
        diagnostics.append({
            'variant': variant, 'mode': mode, 'tracks': len(report['tracks']),
            'reopens': len(report['reopens']), 'pageErrors': report['errors'],
            'canvasPixelChecksPassed': True,
            'snapshots': [{'trackIndex': row['index'], 'detachedCanvasNodes': row['detachedCanvasNodes'],
                           'jsHeapUsedMiB': row['metrics'].get('JSHeapUsedSize', 0) / 2**20,
                           'canvasNodes': row['canvasNodes']} for row in report['snapshots']],
            'qualityPresentation': report.get('qualitySources', []),
        })
save('diagnostic-summary.json', diagnostics)

names = {
    'treePrivateCommitPeakMiB': '连续切歌进程树 Private Commit 峰值 / MiB',
    'rendererPrivateCommitPeakMiB': '连续切歌 Renderer Private Commit 峰值 / MiB',
    'gpuCommittedPeakMiB': '连续切歌 GPU committed 峰值 / MiB',
    'sceneGpuCommittedPeakMiB': '完整采样（含恢复）GPU committed 峰值 / MiB',
    'cpuMachineAveragePercent': '连续切歌 CPU 平均值 / 整机 %',
    'cpuMachineP95Percent': '连续切歌 CPU P95 / 整机 %',
    'firstPaintMedianMs': '首次绘制延迟中位数 / ms',
    'firstPaintP95Ms': '首次绘制延迟 P95 / ms',
    'hdPaintMedianMs': '末次绘制延迟中位数 / ms',
    'hdPaintP95Ms': '末次绘制延迟 P95 / ms',
    'recoveryLastMinuteTreeCommitMedianMiB': '恢复末分钟进程树 Commit / MiB',
    'recoveryLastMinuteRendererCommitMedianMiB': '恢复末分钟 Renderer Commit / MiB',
    'recoveryLastMinuteGpuCommittedMedianMiB': '恢复末分钟 GPU committed / MiB',
    'primaryCanvasBackingBytesMedian': '主封面 Canvas backing 中位数 / bytes',
    'downloadBufferedBytesPeak': '已跟踪下载缓冲峰值 / bytes',
    'decodedBuffersTrackedBytesPeak': '已跟踪解码缓冲峰值 / bytes',
    'encodedBuffersTrackedBytesPeak': '已跟踪编码输入缓冲峰值 / bytes',
}
lines = ['| 指标（三轮中位数） | 优化前 | 优化后 | 后 − 前 | 相对变化 |', '| --- | ---: | ---: | ---: | ---: |']
for key, name in names.items():
    row = comparison['threeRoundMedians'][key]
    delta = row['deltaPercent']
    percent = f'{delta:+.2f}%' if delta is not None else '不可计算（前值为 0）'
    lines.append(f"| {name} | {row['beforeMedian']:.2f} | {row['afterMedian']:.2f} | {row['delta']:+.2f} | {percent} |")
lines += ['', '以下逐轮峰值和 CPU 均取连续切歌段；完整采样 GPU 峰值另见 JSON。', '', '| 轮次 | 采样点 | 连续 / 恢复秒 | Tree 峰值 MiB | Renderer 峰值 MiB | GPU 峰值 MiB | CPU 平均整机 % |', '| --- | ---: | ---: | ---: | ---: | ---: | ---: |']
for row in comparison['runs']:
    lines.append(f"| {row['label']} | {row['sampleCount']} | {row['continuousSeconds']:.1f} / {row['recoverySeconds']:.1f} | {row['treePrivateCommitPeakMiB']:.2f} | {row['rendererPrivateCommitPeakMiB']:.2f} | {row['gpuCommittedPeakMiB']:.2f} | {row['cpuMachineAveragePercent']:.2f} |")
(output / 'METRICS.md').write_text('\n'.join(lines) + '\n', encoding='utf-8')

native = load(root / batch / 'before-r1' / 'before-r1.json')
save('environment.json', {**load(root/'host-environment.json'),'os': native['os'], 'osBuild': native['osBuild'], 'logicalCores': native['logicalCores'],
                         'processVersions': sorted({(row['name'], row['version']) for row in native['processHistory']}),
                         'gpuCounterStatus': native['gpuStatus'], 'rawBatch': batch,
                         'startedUtc': load(root / batch / 'before-r1' / 'run.json')['startedUtc'],
                         'endedUtc': load(root / batch / 'after-r3' / 'run.json')['endedUtc']})

cache_rows=[]
for label in (f'{variant}-r{number}' for variant in ('before','after') for number in (1,2,3)):
    cache=root/batch/label/'cache';formats={}
    for file in cache.rglob('*'):
        if not file.is_file() or file.suffix=='.json':continue
        with file.open('rb') as stream:header=stream.read(16)
        kind='PNG' if header.startswith(b'\x89PNG') else 'JPEG' if header.startswith(b'\xff\xd8') else 'other'
        row=formats.setdefault(kind,{'files':0,'bytes':0});row['files']+=1;row['bytes']+=file.stat().st_size
    cache_rows.append({'label':label,'scope':'Final on-disk media files after app exit, not an in-flight or browser-memory peak','formats':formats,'totalBytes':sum(row['bytes'] for row in formats.values())})
save('cache-final.json',cache_rows)

# Small inspection samples retain the actual rendering; source images stay local.
worst = sorted(quality['rows'], key=lambda row: row['renderComparison']['mse'], reverse=True)[:3]
for row in worst:
    for variant in ('before', 'after'):
        source = root / quality_batch / f'{variant}-continuous' / 'artwork' / f"quality-render-{row['index']}.png"
        shutil.copy2(source, output / f"quality-{row['index']}-{variant}.png")
save('quality-inspection-samples.json', worst)
print('Published six formal runs and four complete diagnostic passes; raw profiles and queue remain local.')

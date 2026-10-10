import json,statistics,datetime,sys
from pathlib import Path
root=Path(sys.argv[1] if len(sys.argv)>1 else 'dist/performance/m3-release-c-2026-10-09')
batch=sys.argv[2] if len(sys.argv)>2 else 'runs-final'
def load(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def median(values):return statistics.median(values) if values else None
def percentile(values,p):
    values=sorted(values)
    return values[min(len(values)-1,int((len(values)-1)*p))] if values else None
def timestamp(value):return datetime.datetime.fromisoformat(value.replace('Z','+00:00')).timestamp()*1000
def memory(sample,role):return sum(row['privateBytes'] for row in sample['processes'] if row['role']==role)/2**20
def gpu(sample):return sum(row['committedBytes'] for process in sample['processes'] for row in process['gpuMemory'])/2**20
results=[];identities={}
frozen={row['localUrl']:(row['bytes'],row['width'],row['height']) for row in load(root/'frozen-covers.json')}
for variant in ('before','after'):
 for number in (1,2,3):
    label=f'{variant}-r{number}';directory=root/batch/label
    report=load(directory/'run.json');assert report['formal'] and report.get('complete') and not report.get('failure'),label
    assert len(report['tracks'])==100,label
    native=load(directory/f'{label}.json');events=[json.loads(x) for x in (directory/'events.jsonl').read_text(encoding='utf-8').splitlines()]
    marks={row['event']['command'].get('phase'):row['at'] for row in events if row['event']['kind']=='command'}
    start=marks['continuous-start'];recovery=marks['recovery-start'];end=marks['complete']
    samples=[dict(row,at=timestamp(row['timestampUtc'])) for row in native['samples']]
    continuous=[row for row in samples if start<=row['at']<recovery];tail=[row for row in samples if end-60000<=row['at']<=end]
    recovery_minutes=[[row for row in samples if recovery+minute*60000<=row['at']<min(end,recovery+(minute+1)*60000)] for minute in range(5)]
    assert continuous and tail and 'available:' in native['gpuStatus'],label
    renderer_sets=[tuple(sorted(row['pid'] for row in sample['processes'] if row['role']=='renderer')) for sample in samples]
    assert len(set(renderer_sets))==1,(label,'Renderer process changed during continuous scene')
    assert all(row['first']['dpi']==1 and row['first']['windowWidth']==200 and row['first']['windowHeight']==395 for row in report['tracks']),label
    downloaded={};partial_cache_reads=[]
    for row in events:
        event=row['event']
        if event['kind']=='download-input':
            assert event['url'].startswith('http://127.0.0.2:9234/cover/'),(label,event['url'])
            key=event['url'].split('?')[0];value=(event['bytes'],event['hash64'])
            assert key in frozen,(label,'Unexpected input',key)
            if event['bytes']!=frozen[key][0]:
                assert event['bytes']<frozen[key][0],(label,'Oversized cache observation',key)
                partial_cache_reads.append({'url':key,'observedBytes':event['bytes'],'expectedBytes':frozen[key][0],'hash64':event['hash64']})
                continue
            assert key not in downloaded or downloaded[key]==value,(label,'Unstable input',key)
            downloaded[key]=value
    assert len(downloaded)==101,(label,'Missing frozen input',len(downloaded))
    assert all((row['last']['imageWidth'],row['last']['imageHeight'])==frozen[f'http://127.0.0.2:9234/cover/{row["index"]}'][1:] for row in report['tracks']),(label,'Final displayed HD dimensions differ from corpus')
    identities[label]=downloaded
    cpu=[row['totalCpuPercentOfMachine'] for row in continuous]
    peaks={name:max(row[name] for row in events) for name in ('downloadBufferedBytesPeak','decodedBuffersTrackedBytesPeak','encodedBuffersTrackedBytesPeak')}
    codec_events=[row['event'] for row in events if row['event']['kind']=='codec']
    data={'label':label,'variant':variant,'sampleCount':len(samples),'rendererPids':renderer_sets[0],'tracks':100,'continuousSeconds':(recovery-start)/1000,'recoverySeconds':(end-recovery)/1000,
          'treePrivateCommitPeakMiB':max(row['totalPrivateBytes'] for row in continuous)/2**20,
          'rendererPrivateCommitPeakMiB':max(memory(row,'renderer') for row in continuous),
          'gpuCommittedPeakMiB':max(gpu(row) for row in continuous),
          'sceneGpuCommittedPeakMiB':max(gpu(row) for row in samples if start<=row['at']<=end),
          'cpuMachineAveragePercent':statistics.mean(cpu),'cpuMachineP95Percent':percentile(cpu,.95),
          'firstPaintMedianMs':median([row['firstPaintLatencyMs'] for row in report['tracks']]),
          'firstPaintP95Ms':percentile([row['firstPaintLatencyMs'] for row in report['tracks']],.95),
          'hdPaintMedianMs':median([row['lastPaintLatencyMs'] for row in report['tracks']]),
          'hdPaintP95Ms':percentile([row['lastPaintLatencyMs'] for row in report['tracks']],.95),
          'recoveryLastMinuteTreeCommitMedianMiB':median([row['totalPrivateBytes']/2**20 for row in tail]),
          'recoveryLastMinuteRendererCommitMedianMiB':median([memory(row,'renderer') for row in tail]),
          'recoveryLastMinuteGpuCommittedMedianMiB':median([gpu(row) for row in tail]),
          'peakBuffers':peaks,'hdResolvedTracks':sum(row['hdResolved'] for row in report['tracks']),'partialPostCacheProbeReads':partial_cache_reads,'finalDisplayDimensionsMatchFrozenCorpus':True,
          'codecOperations':{'decodeCalls':max(row.get('decodeCalls',0) for row in events),'encodeCalls':max(row.get('encodeCalls',0) for row in events),'decodeTrackedWallMs':sum(row['milliseconds'] for row in codec_events if row['operation']=='decode'),'encodeTrackedWallMs':sum(row['milliseconds'] for row in codec_events if row['operation']=='encode'),'scope':'Known native codec operations over process lifetime; excludes browser decode and native header-only inspection. Concurrent wall durations are not CPU time.'},
          'recoveryMinuteMedians':[{'minute':minute+1,'samples':len(rows),'treeCommitMiB':median([row['totalPrivateBytes']/2**20 for row in rows]),'rendererCommitMiB':median([memory(row,'renderer') for row in rows]),'gpuCommitMiB':median([gpu(row) for row in rows]),'cpuMachinePercent':median([row['totalCpuPercentOfMachine'] for row in rows])} for minute,rows in enumerate(recovery_minutes)],
          'primaryCanvasBackingBytesMedian':median([row['last']['rasterWidth']*row['last']['rasterHeight']*4 for row in report['tracks']]),
          'perProcessPagefileHighWaterBytes':{str(row['pid']):max((process.get('peakPagefileUsageBytes',0) for sample in samples for process in sample['processes'] if process['pid']==row['pid']),default=None) for row in native['processHistory']}}
    results.append(data)
reference=identities['before-r1'];assert all(value==reference for value in identities.values()),'Cohort input bytes differ'
summary={}
fields=['treePrivateCommitPeakMiB','rendererPrivateCommitPeakMiB','gpuCommittedPeakMiB','sceneGpuCommittedPeakMiB','cpuMachineAveragePercent','cpuMachineP95Percent','firstPaintMedianMs','firstPaintP95Ms','hdPaintMedianMs','hdPaintP95Ms','recoveryLastMinuteTreeCommitMedianMiB','recoveryLastMinuteRendererCommitMedianMiB','recoveryLastMinuteGpuCommittedMedianMiB','primaryCanvasBackingBytesMedian']
for name in fields:
    before=median([row[name] for row in results if row['variant']=='before']);after=median([row[name] for row in results if row['variant']=='after'])
    summary[name]={'beforeMedian':before,'afterMedian':after,'delta':after-before,'deltaPercent':(after/before-1)*100 if before else None}
for name in ('downloadBufferedBytesPeak','decodedBuffersTrackedBytesPeak','encodedBuffersTrackedBytesPeak'):
    before=median([row['peakBuffers'][name] for row in results if row['variant']=='before']);after=median([row['peakBuffers'][name] for row in results if row['variant']=='after'])
    summary[name]={'beforeMedian':before,'afterMedian':after,'delta':after-before,'deltaPercent':(after/before-1)*100 if before else None}
output={'runs':results,'threeRoundMedians':summary,'identicalFullLengthInputObservationsAcrossAllSixRuns':True,'distinctInputs':len(reference),'limits':['Legacy batches may contain short mutable-cache probe reads, recorded separately. Current correction builds probe complete response bytes before publication on the reference side and a completed unique download temporary file on the candidate side. All 101 full-length input identities must agree across all six runs.','OS peaks are sampled; per-process pagefile high-water values must not be summed as a simultaneous process-tree peak.','Buffer counters track declared scopes only; decoder scratch and browser decoded-image allocations are excluded.','Paint timestamps mark native receipt of draw completion, not compositor presentation.','GPU recovery conclusions require the full curves; an endpoint alone does not prove a plateau or leak.']}
(root/'comparison.json').write_text(json.dumps(output,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(summary,indent=2))

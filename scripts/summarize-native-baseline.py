import argparse,json,math,statistics
from pathlib import Path
parser=argparse.ArgumentParser()
parser.add_argument("directory",type=Path)
args=parser.parse_args()
def pct(values,p):
    values=sorted(values)
    if not values:return None
    return values[min(len(values)-1,math.ceil(p*len(values))-1)]
def stats(values):
    values=[v for v in values if v is not None]
    return {"first":values[0],"last":values[-1],"mean":statistics.mean(values),"p95":pct(values,.95),"min":min(values),"max":max(values)} if values else None
results=[]
for file in sorted(args.directory.glob("release-idle-r[123].json")):
    data=json.loads(file.read_text(encoding="utf-8-sig"))
    samples=data["samples"]
    cpu=[s["totalCpuPercentOfMachine"] for s in samples if all(p["cpuSampleAvailable"] for p in s["processes"])]
    row={"label":data["label"],"warmupSeconds":data["warmupSeconds"],"actualSeconds":data["actualSeconds"],"samples":len(samples),"cpu":stats(cpu),"logicalCores":data["logicalCores"],"machineCpu":stats([s["machineCpuPercent"] for s in samples]),"privateBytes":stats([s["totalPrivateBytes"] for s in samples]),"privateWorkingSet":stats([s["totalPrivateWorkingSetBytes"] for s in samples]),"threads":stats([s["totalThreads"] for s in samples]),"handles":stats([s["totalHandles"] for s in samples]),"gpuStatus":data["gpuStatus"],"processCount":stats([s["processCount"] for s in samples])}
    row["gpuMemory"]={key:stats([sum(m.get(key,0) or 0 for p in s["processes"] for m in p["gpuMemory"]) for s in samples]) for key in ["dedicatedBytes","sharedBytes","committedBytes"]}
    engines={}
    for sample in samples:
        for proc in sample["processes"]:
            for engine in proc["gpuEngines"]:
                engines.setdefault(engine["engine"],[]).append(engine["utilizationPercent"])
    row["gpuEngines"]={name:stats(values) for name,values in engines.items()}
    row["candidateIdleCpuPass"]=row["cpu"]["mean"]<=.1 and row["cpu"]["p95"]<=.5 if row["cpu"] else None
    row["memoryInterpretation"]="Ten-minute endpoint changes do not establish absence of a leak; inspect warm-up/cache and per-role trends. GPU counters are reported per engine. CPU summary is a sampled mean, not a presentation/FPS metric."
    results.append(row)
report={"rounds":results,"formalThreeRoundCoverage":len(results)==3 and all(r["warmupSeconds"]>=300 and r["actualSeconds"]>=600 for r in results),"candidateBudget":"Original plan CPU <=0.1% machine mean and <=0.5% sampled p95. Not noise-calibrated without a desktop control run. No automatic M1/M2 acceptance."}
(args.directory/"idle-summary.json").write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding="utf-8")
print(json.dumps(report,ensure_ascii=False,indent=2))

"""Reproduce matched isolated sources from committed code, never the dirty tree."""
import argparse,io,tarfile,subprocess,shutil,json,hashlib
from pathlib import Path

parser=argparse.ArgumentParser();parser.add_argument('output');parser.add_argument('--candidate-working-tree',action='store_true');args=parser.parse_args()
root=Path(args.output).resolve();assert not root.exists(),'Use a new build source directory'
root.mkdir(parents=True)
paths=['src','src-tauri','public','static','package.json','package-lock.json','index.html','studio.html','svelte.config.js','vite.config.js','tsconfig.json','jsconfig.json']
archive=subprocess.check_output(['git','archive','0055ebe','--',*paths])
for variant in ('before','after'):
    dest=root/variant;dest.mkdir()
    with tarfile.open(fileobj=io.BytesIO(archive)) as files:files.extractall(dest,filter='data')
before=root/'before'
for name in ('cache','color','image','media','mod'):
    path=f'src-tauri/src/services/{name}.rs';(before/path).write_bytes(subprocess.check_output(['git','show','2340164:'+path]))
(before/'src-tauri/src/utils.rs').write_bytes(subprocess.check_output(['git','show','2340164:src-tauri/src/utils.rs']))
for name in ('src-tauri/src/services/http_budget.rs','src-tauri/src/services/image_budget.rs','src/lib/artworkBudget.ts','src/lib/artworkBudget.test.ts'):
    target=(before/name).resolve();assert target.is_relative_to(before);target.unlink()
# Reverse only M3's FloatingWindow changes. M2 imports/reception/revisions stay.
p=before/'src/FloatingWindow.svelte';s=p.read_text(encoding='utf-8')
s=s.replace('  import { ArtworkResultCache, artworkCacheKey, artworkCanvasSize } from "$lib/artworkBudget";\n','')
s=s.replace('  const processedImageCache = new ArtworkResultCache();\n  let fingerprintJobs = 0;','  const MAX_PROCESSED_IMAGES = 12;\n  const processedImageCache = new Map<string, string>();')
a=s.index('    if (disposed || processingPromises.size + fingerprintJobs');b=s.index('    const cached =',a)
s=s[:a]+'    const cacheKey = `${enablePixelArt ? "pixel" : "normal"}:${imageUrl}`;\n'+s[b:]
s=s.replace('      // The byte-bounded cache updates recency on lookup.','      processedImageCache.delete(cacheKey);\n      processedImageCache.set(cacheKey, cached);')
s=s.replace('        processedImageCache.set(cacheKey, processedBase64);','        processedImageCache.set(cacheKey, processedBase64);\n        while (processedImageCache.size > MAX_PROCESSED_IMAGES) {\n          const oldestKey = processedImageCache.keys().next().value;\n          if (oldestKey === undefined) break;\n          processedImageCache.delete(oldestKey);\n        }')
a=s.index('    const raster = artworkCanvasSize');b=s.index('\n\n',a)
s=s[:a]+'    canvas.width = img.width;\n    canvas.height = img.height;'+s[b:]
s=s.replace('ctx.drawImage(img, 0, 0, canvas.width, canvas.height);','ctx.drawImage(img, 0, 0);')
p.write_text(s,encoding='utf-8')
# Keep Cargo feature/dependency declarations identical: tokio sync is a shared
# dependency capability, not an enabled resource budget in the baseline.
fixture=Path(__file__).resolve().parent
if args.candidate_working_tree:
    workspace=fixture.parent.parent
    # Copy only reviewed M3 files; unrelated working-tree edits stay excluded.
    for name in ('src-tauri/src/services/image_budget.rs','src-tauri/src/services/cache.rs','src-tauri/src/services/media.rs','src/FloatingWindow.svelte','src/lib/artworkBudget.ts','src/lib/artworkBudget.test.ts'):
        shutil.copy2(workspace/name,root/'after'/name)
for script in ('instrument.py','add-input-probes.py','add-cover-replay.py','add-metadata-replay.py'):
    subprocess.run(['python',str(fixture/script),str(root)],check=True)
shared=['src-tauri/src/lib.rs','src/App.svelte','src/lib/mediaArtwork.ts','src/lib/mediaStore.ts','src-tauri/src/benchmark_c.rs','src/lib/benchmarkC.ts']
for name in shared:assert (root/'before'/name).read_bytes()==(root/'after'/name).read_bytes(),name
manifest={'afterBase':'0055ebe','baselineBudgetSource':'2340164','sharedM2Commit':'0055ebe','cargoLockSha256':hashlib.sha256((root/'after/src-tauri/Cargo.lock').read_bytes()).hexdigest(),'sharedFiles':shared}
(root/'matching-builds.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
print('Prepared matched sources. Junction node_modules to existing dependencies before building.')

"""Matched M4 pair: committed candidate B plus only reviewed M4 files."""
import hashlib, io, json, subprocess, tarfile, shutil, sys
from pathlib import Path
root=Path(sys.argv[1]).resolve();assert not root.exists();root.mkdir(parents=True)
workspace=Path(__file__).resolve().parents[2];fixture=Path(__file__).resolve().parent
paths=['src','src-tauri','public','static','package.json','package-lock.json','index.html','studio.html','svelte.config.js','vite.config.js','tsconfig.json','jsconfig.json']
archive=subprocess.check_output(['git','archive','95a1e2b','--',*paths])
reviewed=['src-tauri/Cargo.toml','src-tauri/Cargo.lock','src-tauri/src/commands/media.rs','src-tauri/src/lib.rs','src-tauri/src/services/artwork_jobs.rs','src-tauri/src/services/cache.rs','src-tauri/src/services/color.rs','src-tauri/src/services/http_budget.rs','src-tauri/src/services/image.rs','src-tauri/src/services/image_budget.rs','src-tauri/src/services/media.rs','src-tauri/src/services/mod.rs','src/App.svelte','src/FloatingWindow.svelte','src/lib/api/types.ts','src/lib/artworkOwner.ts','src/lib/artworkOwner.test.ts','src/lib/canvasArtwork.ts']
for variant in ('before','after'):
 dest=root/variant;dest.mkdir()
 with tarfile.open(fileobj=io.BytesIO(archive)) as files:files.extractall(dest,filter='data')
 if variant=='after':
  for name in reviewed:shutil.copy2(workspace/name,dest/name)
 # Same dependency graph and opt-in trace helper. Reference does not call the
 # owner registry, coalesce or cancel; its production B path remains intact.
 for name in ('src-tauri/Cargo.toml','src-tauri/Cargo.lock','src-tauri/src/services/artwork_jobs.rs'):shutil.copy2(workspace/name,dest/name)
 if variant=='before':
  p=dest/'src-tauri/src/services/mod.rs';s=p.read_text(encoding='utf-8').replace('pub(crate) mod apartment;','pub(crate) mod apartment;\npub(crate) mod artwork_jobs;');p.write_text(s,encoding='utf-8')
  for name,anchor,probe in [('image_budget','pub fn decode(bytes: &[u8]) -> AppResult<DynamicImage> {','super::artwork_jobs::trace_bytes("decode",bytes);'),('image_budget',"pub fn prepare_artwork(bytes: &[u8]) -> AppResult<PreparedArtwork<'_>> {",'super::artwork_jobs::trace_bytes("prepare",bytes);'),('color','    let img_data = load_image_data(image_path)?;','super::artwork_jobs::trace_bytes("color",&img_data);'),('http_budget','    save_response(response, directory, limit, permit).await','super::artwork_jobs::trace_bytes("download",url.as_bytes());')]:
   p=dest/f'src-tauri/src/services/{name}.rs';s=p.read_text(encoding='utf-8');assert anchor in s
   s=s.replace(anchor,anchor+'\n    '+probe,1) if name!='http_budget' else s.replace(anchor,'    '+probe+'\n'+anchor,1);p.write_text(s,encoding='utf-8')
for script,flags in [('instrument.py',['--paired-budgeted']),('add-input-probes.py',['--paired-budgeted']),('add-cover-replay.py',[]),('add-metadata-replay.py',[])]:subprocess.run(['python',str(fixture/script),str(root),*flags],check=True)
# Observe HD completion in the main consumer too: either shared consumer may
# finish first. This marker has the same semantics and adapter on both sides.
for variant in ('before','after'):
 dest=root/variant;p=dest/'src/App.svelte';s=p.read_text(encoding='utf-8');s=s.replace('<script lang="ts">','<script lang="ts">\n  import { reportBenchmarkC } from "$lib/benchmarkC";',1)
 anchor='              .then((resolved) => {';assert anchor in s
 s=s.replace(anchor,anchor+'\n                void reportBenchmarkC({kind:"hd-resolved",title:data.title,artist:data.artist,found:!!resolved?.url});',1);p.write_text(s,encoding='utf-8')
 hashes={str(p.relative_to(dest)).replace('\\','/'):hashlib.sha256(p.read_bytes()).hexdigest() for folder in ('src','src-tauri/src') for p in (dest/folder).rglob('*') if p.is_file()}
 (root/f'{variant}-source-hashes.json').write_text(json.dumps(hashes,indent=2),encoding='utf-8')
for name in ('Cargo.toml','Cargo.lock'):assert (root/f'before/src-tauri/{name}').read_bytes()==(root/f'after/src-tauri/{name}').read_bytes()
assert (root/'before/src-tauri/src/benchmark_c.rs').read_bytes()==(root/'after/src-tauri/src/benchmark_c.rs').read_bytes()
(root/'matching-builds.json').write_text(json.dumps({'baselineCommit':'95a1e2b','scope':'M3 candidate B versus M4; same M2/M3 budgets, dependencies and adapter','reviewedCandidateFiles':reviewed,'excludedUserFiles':['src/lib/components/island/IslandSurface.svelte','src/lib/components/media/PlayerControls.svelte']},indent=2),encoding='utf-8')
print('Prepared M4 matched sources from candidate B.')

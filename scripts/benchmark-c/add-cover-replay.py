from pathlib import Path
import sys,shutil,json,hashlib
root=Path(sys.argv[1]).resolve()
for variant in ('before','after'):
    base=root/variant
    shutil.copy2(Path(__file__).resolve().parent/'native.rs',base/'src-tauri/src/benchmark_c.rs')
    p=base/'src-tauri/src/services/media.rs';s=p.read_text(encoding='utf-8')
    start=s.index('async fn cover_from_netease(');pos=s.index('    Ok(',start)
    s=s[:pos]+'    if let Some(url)=crate::benchmark_c::cover_url(title) { return Ok(Some(ResolvedCover {url,provider:"benchmark-frozen".to_string()})); }\n'+s[pos:]
    p.write_text(s,encoding='utf-8')
    hashes={str(p.relative_to(base)).replace('\\','/'):hashlib.sha256(p.read_bytes()).hexdigest() for folder in ('src','src-tauri/src') for p in (base/folder).rglob('*') if p.is_file()}
    (root/f'{variant}-source-hashes.json').write_text(json.dumps(hashes,indent=2),encoding='utf-8')
print('Same cover provider replay inserted before real variant-specific download/cache pipeline')

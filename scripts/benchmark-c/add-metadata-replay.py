from pathlib import Path
import sys,shutil,json,hashlib
root=Path(sys.argv[1]).resolve()
for variant in ('before','after'):
    base=root/variant;shutil.copy2(Path(__file__).resolve().parent/'native.rs',base/'src-tauri/src/benchmark_c.rs')
    p=base/'src-tauri/src/services/media.rs';s=p.read_text(encoding='utf-8')
    start=s.index('async fn matched_netease_song(');pos=s.index(') -> AppResult<Option<Value>> {',start)+len(') -> AppResult<Option<Value>> {')
    s=s[:pos]+'\n    if let Some(song)=crate::benchmark_c::song_metadata(title) { return Ok(Some(song)); }'+s[pos:]
    p.write_text(s,encoding='utf-8')
    hashes={str(p.relative_to(base)).replace('\\','/'):hashlib.sha256(p.read_bytes()).hexdigest() for folder in ('src','src-tauri/src') for p in (base/folder).rglob('*') if p.is_file()}
    (root/f'{variant}-source-hashes.json').write_text(json.dumps(hashes,indent=2),encoding='utf-8')
print('Ancillary duration/search cover inputs now use same frozen metadata')

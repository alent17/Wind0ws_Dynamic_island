from pathlib import Path
import json,hashlib,sys,shutil
root=Path(sys.argv[1]).resolve()
for variant in ('before','after'):
    base=root/variant
    shutil.copy2(Path(__file__).resolve().parent/'native.rs',base/'src-tauri/src/benchmark_c.rs')
    if variant=='before':
        p=base/'src-tauri/src/services/cache.rs';s=p.read_text(encoding='utf-8')
        anchor='    save_cache_file(url, &bytes, content_type)'
        assert anchor in s
        s=s.replace(anchor,'    crate::benchmark_c::input_bytes(url, &bytes);\n'+anchor,1)
    else:
        p=base/'src-tauri/src/services/http_budget.rs';s=p.read_text(encoding='utf-8')
        anchor='    save_response(response, directory, limit, permit).await'
        assert anchor in s
        s=s.replace(anchor,'    let result=save_response(response, directory, limit, permit).await?;\n    crate::benchmark_c::input_file(url, &result.path);\n    Ok(result)',1)
    p.write_text(s,encoding='utf-8')
    hashes={str(p.relative_to(base)).replace('\\','/'):hashlib.sha256(p.read_bytes()).hexdigest() for folder in ('src','src-tauri/src') for p in (base/folder).rglob('*') if p.is_file()}
    (root/f'{variant}-source-hashes.json').write_text(json.dumps(hashes,indent=2),encoding='utf-8')
print('Identical transport input identity probe attached; outputs distinguish normalized images')

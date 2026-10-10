"""Apply the same opt-in adapter to already prepared before/after source trees.
Only isolated build trees are modified. Main application source is untouched.
"""
from pathlib import Path
import re, shutil, json, hashlib, sys

root=Path(sys.argv[1]).resolve()
fixture=Path(__file__).resolve().parent
for variant in ('before','after'):
    base=root/variant
    shutil.copy2(fixture/'native.rs',base/'src-tauri/src/benchmark_c.rs')
    shutil.copy2(fixture/'frontend.ts',base/'src/lib/benchmarkC.ts')
    p=base/'src-tauri/src/lib.rs';s=p.read_text(encoding='utf-8')
    s=s.replace('mod services;','mod services;\nmod benchmark_c;',1)
    s=s.replace('commands::get_media_info_cmd,','commands::get_media_info_cmd,\n            benchmark_c::benchmark_c_enabled,\n            benchmark_c::benchmark_c_report,',1)
    s=s.replace('.setup(|app| {','.setup(|app| {\n            benchmark_c::start(app.handle().clone());',1)
    p.write_text(s,encoding='utf-8')
    p=base/'src-tauri/src/services/settings.rs';s=p.read_text(encoding='utf-8')
    s=s.replace('pub fn set_auto_start(enable: bool) -> AppResult<()> {','pub fn set_auto_start(enable: bool) -> AppResult<()> {\n    if std::env::var_os("ISLE_BENCH_DIR").is_some() { return Ok(()); }',1)
    p.write_text(s,encoding='utf-8')
    p=base/'src-tauri/tauri.conf.json';c=json.loads(p.read_text(encoding='utf-8'));c['identifier']='com.isle-app.benchmarkc';p.write_text(json.dumps(c,ensure_ascii=False,indent=2),encoding='utf-8')
    p=base/'src/FloatingWindow.svelte';s=p.read_text(encoding='utf-8')
    s=s.replace('<script lang="ts">','<script lang="ts">\n  import { reportBenchmarkC } from "$lib/benchmarkC";',1)
    anchor='        source || "generic",\n      );'
    assert anchor in s
    s=s.replace(anchor,anchor+'\n      void reportBenchmarkC({kind:"hd-resolved",title,artist,found:!!resolved?.url});',1)
    draw='ctx.drawImage(img, 0, 0, canvas.width, canvas.height);' if variant=='after' else 'ctx.drawImage(img, 0, 0);'
    assert draw in s
    s=s.replace(draw,draw+'\n      if(canvas.classList.contains("album-art-new")) void reportBenchmarkC({kind:"paint",title:mediaState.title,artist:mediaState.artist,track:currentTrackKey,imageWidth:img.naturalWidth,imageHeight:img.naturalHeight,rasterWidth:canvas.width,rasterHeight:canvas.height,dpi:devicePixelRatio,windowWidth:innerWidth,windowHeight:innerHeight});',1)
    p.write_text(s,encoding='utf-8')
    # Baseline complete response buffers versus budgeted transport chunks.
    for name in ('cache','media'):
        p=base/f'src-tauri/src/services/{name}.rs';s=p.read_text(encoding='utf-8')
        s=re.sub(r'(let bytes = response\s*\.bytes\(\)[\s\S]*?\?;)',r'\1\n    let _transport_buffer = crate::benchmark_c::Buffer::new("download", bytes.len());',s)
        p.write_text(s,encoding='utf-8')
    if variant=='after':
        p=base/'src-tauri/src/services/http_budget.rs';s=p.read_text(encoding='utf-8')
        anchor='        let next = result'
        assert anchor in s
        s=s.replace(anchor,'        let _transport_buffer = crate::benchmark_c::Buffer::new("download", chunk.len());\n'+anchor,1)
        p.write_text(s,encoding='utf-8')
    # Track encoded inputs and known decoded image buffers at lexical owners.
    # This intentionally does not claim allocator/decoder scratch precision.
    for name in ('image','color'):
        p=base/f'src-tauri/src/services/{name}.rs';s=p.read_text(encoding='utf-8')
        s=re.sub(r'(let (img_data|bytes) = load_image_data\([^;]+;)',lambda m:m[0]+f'\n    let _encoded_buffer = crate::benchmark_c::Buffer::new("encoded", {m[2]}.len());',s)
        s=re.sub(r'(let (_?img|image) = (?:image::load_from_memory|(?:super::)?image_budget::(?:fit|decode))[^;]*;)',lambda m:m[0]+f'\n    let _decoded_buffer = crate::benchmark_c::decoded(&{m[2]});',s)
        p.write_text(s,encoding='utf-8')
    if variant=='after':
        p=base/'src-tauri/src/services/image_budget.rs';s=p.read_text(encoding='utf-8')
        s=s.replace('pub fn fit(img: DynamicImage) -> DynamicImage {','pub fn fit(img: DynamicImage) -> DynamicImage {\n    let _source_buffer = crate::benchmark_c::decoded(&img);',1)
        if 'pub fn prepare_artwork' in s:
            s=s.replace('        if let Some(content_type) = mime {','        if let Some(content_type) = mime {\n            let _preserved_buffer = crate::benchmark_c::decoded(&image);',1)
        s=s.replace('    reader(bytes)?\n        .decode()', '    let started=std::time::Instant::now();\n    let image=reader(bytes)?\n        .decode()',1)
        s=s.replace('.map_err(|e| AppError::parse(format!("图片解码失败：{e}")))\n}', '.map_err(|e| AppError::parse(format!("图片解码失败：{e}")))?;\n    crate::benchmark_c::codec("decode",started.elapsed().as_millis(),bytes.len(),image.as_bytes().len());\n    Ok(image)\n}',1)
        s=s.replace('pub fn encode_png(img: &DynamicImage) -> AppResult<Vec<u8>> {','pub fn encode_png(img: &DynamicImage) -> AppResult<Vec<u8>> {\n    let started=std::time::Instant::now();',1)
        s=s.replace('    Ok(output.0.into_inner())','    crate::benchmark_c::codec("encode",started.elapsed().as_millis(),img.as_bytes().len(),output.0.get_ref().len());\n    Ok(output.0.into_inner())',1)
        p.write_text(s,encoding='utf-8')
    hashes={str(p.relative_to(base)).replace('\\','/'):hashlib.sha256(p.read_bytes()).hexdigest() for folder in ('src','src-tauri/src') for p in (base/folder).rglob('*') if p.is_file()}
    (root/f'{variant}-source-hashes.json').write_text(json.dumps(hashes,indent=2),encoding='utf-8')
print('Both isolated builds instrumented; identical adapter and M2 retained')

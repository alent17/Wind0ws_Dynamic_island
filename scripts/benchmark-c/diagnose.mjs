import assert from 'node:assert/strict';
import {spawn,execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {mkdir,readFile,writeFile,copyFile,stat} from 'node:fs/promises';
import {join,resolve} from 'node:path';
const exec=promisify(execFile),variant=process.argv[2],mode=process.argv[3];
assert(['before','after'].includes(variant)&&['continuous','reopen'].includes(mode));
const batch=process.env.ISLE_BENCH_DIAGNOSTIC_BATCH||'diagnostic';assert(/^[a-z0-9-]+$/.test(batch));
const base=resolve(process.env.ISLE_BENCH_ROOT||'dist/performance/m3-release-c-2026-10-09'),output=join(base,batch,`${variant}-${mode}`);
await mkdir(output,{recursive:true});assert(!(await stat(join(output,'events.jsonl')).catch(()=>null)),'Fresh diagnostic output required');
await copyFile(join(base,'binaries',variant,'isle.exe'),join(output,'isle.exe'));
await copyFile(join(base,'frozen-covers.json'),join(output,'frozen-covers.json'));
const settings=await readFile(join(base,'settings.json'),'utf8'),config=join(process.env.APPDATA,'com.isle-app.benchmarkc');await mkdir(config,{recursive:true});await writeFile(join(config,'settings.json'),settings);
await exec(process.execPath,[resolve('scripts/benchmark-c/prepare-player.mjs')],{timeout:60000});
const env={...process.env,ISLE_BENCH_DIR:output,WEBVIEW2_USER_DATA_FOLDER:join(output,'webview'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9235 --remote-debugging-address=127.0.0.1',NO_PROXY:'127.0.0.1,127.0.0.2,localhost',no_proxy:'127.0.0.1,127.0.0.2,localhost'};
const app=spawn(join(output,'isle.exe'),[],{env,windowsHide:true,stdio:'ignore'});
try {
 await new Promise(done=>setTimeout(done,10000));
 const runtime='matched Release with same M2 and frozen corpus; CDP diagnostic pass, excluded from formal performance';
 const result=await exec(process.execPath,[resolve('scripts/accept-native-artwork.mjs'),join(output,'artwork')],{env:{...process.env,ISLE_ROOT_PID:String(app.pid),ISLE_CDP_ENDPOINT:'http://127.0.0.1:9235',ISLE_RUNTIME_DESCRIPTION:runtime,ARTWORK_REOPEN_INTERVAL:mode==='continuous'?'0':'25',ARTWORK_CAPTURE_QUALITY:mode==='continuous'?'1':'0',ARTWORK_ALLOW_SCREENSHOT_PIXEL_CHECK:'1'},timeout:1800000,maxBuffer:1024*1024});
 process.stdout.write(result.stdout);
}finally{if(app.exitCode===null)app.kill();}

import assert from 'node:assert/strict';
import { chromium } from '@playwright/test';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

// Explicitly authorized real-player exit/recovery; only processes whose
// executable equals M2_PLAYER_EXE are stopped. Restart even on a failed check.
assert(process.env.M2_PLAYER_EXE,'Set M2_PLAYER_EXE to the authorized player executable');
const exec=promisify(execFile),pause=ms=>new Promise(done=>setTimeout(done,ms));
const output=resolve(process.argv[2]||'dist/performance/m2-reopen-2026-10-08/session-recovery');
await mkdir(output,{recursive:true});
const browser=await chromium.connectOverCDP(process.env.ISLE_CDP_ENDPOINT||'http://127.0.0.1:9230');
const context=browser.contexts()[0],main=context.pages().find(page=>!page.url().includes('window='));
const invoke=(command,args={})=>main.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
const report={runtime:'isolated debug native WebView2; real player exit and restart',cases:[]};
let stopped=false,restarted=false;
const ps=cmd=>exec('powershell',['-NoProfile','-Command',cmd]);
const restart=async()=>{await ps("$ErrorActionPreference='Stop'; $m2PlayerPath=(Resolve-Path -LiteralPath $env:M2_PLAYER_EXE).Path; Start-Process -FilePath $m2PlayerPath -WindowStyle Hidden");restarted=true;};
async function state() {
 let media,queryError;
 try {media=await invoke('get_media_info_cmd');}
 catch(error){queryError=String(error);}
 const floating=context.pages().find(page=>page.url().includes('window=floating'));
 const dom=await floating.evaluate(()=>{const c=document.querySelector('canvas.album-art-new'),image=document.querySelector('img.compact-cover-image');return {title:document.querySelector('.track-title')?.textContent?.trim(),count:document.querySelectorAll('canvas.album-art-new').length,connected:!!c?.isConnected,ready:!!image?.complete&&!!image?.naturalWidth};});
 return {media:media?{queryOk:true,title:media.title,source:media.source,coverChars:media.albumArt?.length||0}:{queryOk:false,queryError},dom};
}
async function waitFor(name,predicate) {
 let value;for(let i=0;i<120;i++){value=await state();if(predicate(value)){report.cases.push({name,state:value});console.log(`Passed: ${name}`);return;}await pause(500);}
 throw new Error(`${name}: ${JSON.stringify(value)}`);
}
try {
 await invoke('open_floating_window');await pause(2500);
 await waitFor('before exit',s=>s.media.source&&s.media.coverChars&&s.dom.count===1&&s.dom.ready);
 await ps("$ErrorActionPreference='Stop'; $m2PlayerPath=(Resolve-Path -LiteralPath $env:M2_PLAYER_EXE).Path; $m2Players=@(Get-Process | Where-Object {$_.Path -eq $m2PlayerPath}); if(-not $m2Players.Count){throw 'Authorized player is not running'}; $m2Players | Stop-Process");stopped=true;
 await waitFor('real no-session clears Canvas',s=>s.media.queryOk&&!s.media.source&&s.dom.count===0);
 await restart();
 // Resume the existing queue through the player's real SMTC session.
 for(let i=0;i<30;i++){try{await exec('powershell',['-NoProfile','-File','scripts/media-session.ps1','-Action','Play']);if((await state()).media.source)break;}catch{}await pause(1000);}
 await waitFor('real session and cover restored',s=>s.media.source&&s.media.coverChars&&s.dom.title===s.media.title&&s.dom.count===1&&s.dom.connected&&s.dom.ready);
 const floating=context.pages().find(page=>page.url().includes('window=floating'));
 const cdp=await context.newCDPSession(floating),chunks=[];
 cdp.on('HeapProfiler.addHeapSnapshotChunk',event=>chunks.push(event.chunk));await cdp.send('HeapProfiler.takeHeapSnapshot',{reportProgress:false});
 const body=chunks.join('');await writeFile(resolve(output,'session-restored.heapsnapshot'),body);
 const heap=JSON.parse(body),f=heap.snapshot.meta.node_fields,w=f.length,n=f.indexOf('name'),d=f.indexOf('detachedness');let detached=0;
 for(let i=0;i<heap.nodes.length;i+=w){const name=heap.strings[heap.nodes[i+n]];if(name.includes('HTMLCanvasElement')&&(heap.nodes[i+d]===2||name.startsWith('Detached')))detached++;}
 assert.equal(detached,0);report.detachedCanvasNodes=detached;await cdp.detach();report.passed=true;
}catch(error){report.failure=String(error);throw error;}
finally{if(stopped&&!restarted)await restart();await writeFile(resolve(output,'session-recovery.json'),JSON.stringify(report,null,2));await browser.close();}

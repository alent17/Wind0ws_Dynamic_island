import assert from 'node:assert/strict';
import { chromium } from '@playwright/test';
import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

// Actual native floating WebView; opt-in latency only delays delivery of its
// real initial snapshot. Synthetic deltas supplement the real-player stress run.
const output=resolve(process.argv[2]||'dist/performance/m2-reopen-2026-10-08/reception');
await mkdir(output,{recursive:true});
const browser=await chromium.connectOverCDP(process.env.ISLE_CDP_ENDPOINT||'http://127.0.0.1:9230');
const context=browser.contexts()[0];
const main=context.pages().find(page=>!page.url().includes('window='));
assert(main,'Native main WebView required');
const invoke=(command,args={})=>main.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
const pause=ms=>new Promise(done=>setTimeout(done,ms));
const report={runtime:'isolated debug native custom-protocol WebView2; correctness, not Release performance',snapshotDelayMs:1800,cases:[],traces:[]};
const settings=await invoke('get_settings');
const media=await invoke('get_media_info_cmd');
assert(media.source && media.albumArt,'Real current player with artwork required');
const playerAction=async action=>promisify(execFile)('powershell',['-NoProfile','-File','scripts/media-session.ps1','-Action',action]);
const emit=payload=>invoke('plugin:event|emit',{event:'island-media-sync',payload});
let floating;
async function open() {
 await invoke('close_floating_window');await pause(500);
 await invoke('open_floating_window');
 for(let i=0;i<40;i++) {
  floating=context.pages().find(page=>page.url().includes('window=floating'));
  if(floating){await floating.waitForLoadState('domcontentloaded');return;}
  await pause(100);
 }
 throw new Error('Floating target missing');
}
async function state() {
 return floating.evaluate(()=>{
  const c=document.querySelector('canvas.album-art-new');
  const image=document.querySelector('img.compact-cover-image');
  return {title:document.querySelector('.track-title')?.textContent?.trim(),count:document.querySelectorAll('canvas.album-art-new').length,ready:!!image?.complete&&!!image?.naturalWidth,pixel:c?.width?Array.from(c.getContext('2d').getImageData(Math.floor(c.width/2),Math.floor(c.height/2),1,1).data):null};
 });
}
async function until(name,predicate) {
 let value;
 for(let i=0;i<60;i++){value=await state();if(predicate(value)){report.cases.push({name,state:value});console.log(`Passed: ${name}`);return value;}await pause(100);}
 throw new Error(`${name}: ${JSON.stringify(value)}`);
}
async function trace(name) {report.traces.push({name,events:await floating.evaluate(()=>window.__ISLE_MEDIA_TRACE__)});}
async function detached(name) {
 const cdp=await context.newCDPSession(floating),chunks=[];
 cdp.on('HeapProfiler.addHeapSnapshotChunk',event=>chunks.push(event.chunk));
 await cdp.send('HeapProfiler.takeHeapSnapshot',{reportProgress:false});
 const body=chunks.join('');await writeFile(resolve(output,`${name}.heapsnapshot`),body);
 const heap=JSON.parse(body),fields=heap.snapshot.meta.node_fields,w=fields.length,n=fields.indexOf('name'),d=fields.indexOf('detachedness');
 let count=0;for(let i=0;i<heap.nodes.length;i+=w){const label=heap.strings[heap.nodes[i+n]];if(label.includes('HTMLCanvasElement')&&(heap.nodes[i+d]===2||label.startsWith('Detached')))count++;}
 assert.equal(count,0,'Detached Canvas');report.cases.push({name:`${name}: Detached Canvas`,count});await cdp.detach();
}
try {
 await invoke('save_settings',{settings:{...settings,enableHdCover:false,enablePixelArt:false}});
 if(media.isPlaying)await playerAction('Pause');
 await context.addInitScript(()=>{window.__ISLE_MEDIA_TRACE__=[];window.__ISLE_MEDIA_SNAPSHOT_DELAY_MS__=1800;});
 for(let i=1;i<=5;i++) {await open();await pause(2500);await until(`same-track reopen ${i}`,s=>s.title===media.title&&s.count===1&&s.ready);await trace(`reopen ${i}`);}
 const colors=await main.evaluate(()=>["#ff0000","#0000ff","#00ff00"].map(color=>{const c=document.createElement('canvas');c.width=c.height=16;c.getContext('2d').fillStyle=color;c.getContext('2d').fillRect(0,0,16,16);return c.toDataURL();}));
 const base={...media,isPlaying:false};delete base.albumArt;
 await emit(base);await until('omitted cover preserves same-track artwork',s=>s.count===1&&s.ready);
 await emit({...base,albumArt:''});await until('explicit empty clears artwork',s=>s.count===0);
 await emit({...base,albumArt:colors[0]});await until('late same-track cover',s=>s.pixel?.[0]===255&&s.pixel?.[1]===0&&s.pixel?.[2]===0);
 await emit({...base,albumArt:colors[1]});await emit({...base,albumArt:colors[2]});
 await until('latest same-track replacement wins',s=>s.pixel?.[0]===0&&s.pixel?.[1]===255&&s.pixel?.[2]===0);
 await detached('same-track-updates');await trace('same-track-updates');
 await open();await pause(800);await emit({...base,albumArt:colors[1]});await pause(2000);
 await until('older initial snapshot cannot replace newer cover',s=>s.pixel?.[2]===255&&s.pixel?.[0]===0);
 await trace('stale-snapshot');
 await open();await pause(800);await emit({...base,albumArt:''});await pause(2000);
 await until('older initial snapshot cannot resurrect explicit clear',s=>s.count===0);await trace('clear-pending-snapshot');
 await emit({...base,albumArt:media.albumArt});await until('full cover restored',s=>s.count===1&&s.ready);
 await emit({...base,source:'',title:'',artist:'',albumArt:''});await until('no-session clears Canvas',s=>s.count===0);
 await emit({...base,albumArt:media.albumArt});await until('session snapshot restores Canvas',s=>s.title===media.title&&s.count===1&&s.ready);
 await detached('session-restored');await trace('session-restored');
 report.passed=true;
} catch(error){report.failure=String(error);throw error;}
finally {
 await writeFile(resolve(output,'reception.json'),JSON.stringify(report,null,2));
 await invoke('save_settings',{settings});
 if(media.isPlaying)await playerAction('Play');
 await invoke('close_floating_window');
 await browser.close();
}

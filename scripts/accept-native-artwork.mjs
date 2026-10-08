import { chromium } from '@playwright/test';
import { mkdir, writeFile, appendFile, readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
const exec=promisify(execFile);
const rootPid=Number(process.env.ISLE_ROOT_PID);
if(!rootPid)throw new Error('Set ISLE_ROOT_PID to the diagnostic Release process');
const endpoint=process.env.ISLE_CDP_ENDPOINT||'http://127.0.0.1:9228';
const output=resolve(process.argv[2]||'dist/performance/acceptance-2026-10-08/artwork');
await mkdir(output,{recursive:true});
const browser=await chromium.connectOverCDP(endpoint);
const context=browser.contexts()[0];
const main=context.pages().find(page=>!page.url().includes('window=floating') && /index|tauri|localhost/.test(page.url()));
if(!main) throw new Error('No native Isle main WebView2 target');
const invoke=(command,args={})=>main.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
const resume=Number(process.env.ARTWORK_RESUME_INDEX||0);
const report=resume ? JSON.parse(await readFile(resolve(output,'artwork-acceptance.json'),'utf8')) : {runtime:'Latest Release WebView2 with remote debugging; diagnostic pass separate from unprofiled Idle samples',startedUtc:new Date().toISOString(),tracks:[],snapshots:[],errors:[],expectedChanges:100};
const flush=()=>writeFile(resolve(output,'artwork-acceptance.json'),JSON.stringify(report,null,2));
const pause=ms=>new Promise(done=>setTimeout(done,ms));
async function nativePoint(index) {
 await exec('powershell',['-NoProfile','-File','scripts/measure-native.ps1','-RootPid',String(rootPid),'-Seconds','1','-IncludeGpu','-Label',`artwork-point-${index}`,'-OutputDirectory',output],{timeout:30000});
}
async function floatingPage() {
 for(let attempt=0;attempt<40;attempt++) {
  const page=context.pages().find(page=>page.url().includes('window=floating'));
  if(page) { await page.waitForLoadState('domcontentloaded'); return page; }
  await pause(250);
 }
 throw new Error('Floating WebView2 target did not open');
}
async function canvasState(page,expectedTitle) {
 return page.evaluate(async expected=>{
  const title=document.querySelector('.track-title')?.textContent?.trim();
  const canvas=document.querySelector('canvas.album-art-new');
  const image=document.querySelector('img.compact-cover-image');
  const result={title,titleMatches:title===expected,canvasCount:document.querySelectorAll('canvas.album-art-new').length,connected:canvas?.isConnected||false,width:canvas?.width||0,height:canvas?.height||0,imageReady:!!image?.complete && !!image?.naturalWidth,urlMatchesPixels:null};
  if(!canvas || !image?.src || !result.imageReady || !canvas.width || !canvas.height) return result;
  try {
   const reference=new Image();
   if(image.src.startsWith('http') && !image.src.includes('asset.localhost')) reference.crossOrigin='Anonymous';
   reference.src=image.src;await reference.decode();
   const target=document.createElement('canvas');target.width=canvas.width;target.height=canvas.height;
   const ref=target.getContext('2d');ref.drawImage(reference,0,0,target.width,target.height);
   const actual=canvas.getContext('2d');
   let equal=true;
   for(const ratio of [.2,.5,.8]) {
    const x=Math.floor(canvas.width*ratio),y=Math.floor(canvas.height*ratio);
    const a=actual.getImageData(x,y,1,1).data,b=ref.getImageData(x,y,1,1).data;
    if(a.some((channel,index)=>Math.abs(channel-b[index])>2))equal=false;
   }
   result.urlMatchesPixels=equal;target.width=target.height=0;
  } catch(error) { result.pixelCheckError=String(error); }
  return result;
 },expectedTitle);
}
async function settledArtwork(page,title) {
 let state;
 for(let attempt=0;attempt<60;attempt++) {
  state=await canvasState(page,title);
  if(state.titleMatches && state.canvasCount===1 && state.connected && state.imageReady && state.width>0 && state.height>0 && state.urlMatchesPixels!==false) return state;
  await pause(250);
 }
 throw new Error(`Artwork did not settle for latest track: ${JSON.stringify(state)}`);
}
async function heapSnapshot(page,index,postGc=false) {
 const cdp=await context.newCDPSession(page);
 await cdp.send('Performance.enable');
 if(postGc)await cdp.send('HeapProfiler.collectGarbage');
 const chunks=[];cdp.on('HeapProfiler.addHeapSnapshotChunk',event=>chunks.push(event.chunk));
 await cdp.send('HeapProfiler.takeHeapSnapshot',{reportProgress:false});
 const body=chunks.join('');
 const filename=`floating-${index}${postGc?'-post-gc':''}.heapsnapshot`;
 await writeFile(resolve(output,filename),body);
 const snapshot=JSON.parse(body),fields=snapshot.snapshot.meta.node_fields,width=fields.length;
 const nameField=fields.indexOf('name'),detachedField=fields.indexOf('detachedness');
 const canvases=[];
 for(let offset=0;offset<snapshot.nodes.length;offset+=width) {
  const name=snapshot.strings[snapshot.nodes[offset+nameField]];
  if(name.includes('HTMLCanvasElement')) canvases.push({name,detachedness:detachedField>=0?snapshot.nodes[offset+detachedField]:null});
 }
 const metrics=Object.fromEntries((await cdp.send('Performance.getMetrics')).metrics.map(metric=>[metric.name,metric.value]));
 const row={index,postGc,filename,metrics,canvasNodes:canvases,detachedCanvasNodes:canvases.filter(node=>node.detachedness===2||node.name.startsWith('Detached')).length};
 report.snapshots.push(row);await flush();await cdp.detach();
}
try {
 if(resume && report.tracks.length!==resume)throw new Error('Resume index does not match recorded track count');
 const settings=await invoke('get_settings');
 report.settings={pixelArt:settings.enablePixelArt,hdCover:settings.enableHdCover,captureHideOnScreenshot:settings.captureHideOnScreenshot,reduceAnimations:settings.reduceAnimations};
 if(settings.enablePixelArt) throw new Error('Pixel art enabled: direct reference-pixel comparison requires a separate pixel-art scenario');
 await invoke('open_floating_window');
 let floating=await floatingPage();
 floating.on('pageerror',error=>report.errors.push(String(error)));
 let media=await invoke('get_media_info_cmd');
 if(!media.source || !media.albumArt)throw new Error('Current real player must have a cover');
 if(!resume) {
 await settledArtwork(floating,media.title);
 await heapSnapshot(floating,0);
 await nativePoint(0);
 await floating.screenshot({path:resolve(output,'floating-0.png')});
 }
 for(let index=resume+1;index<=100;index++) {
  const previous=`${media.source}|${media.title}|${media.artist}`;
  await invoke('control_media',{action:'next'});
  let changed=false;
  for(let attempt=0;attempt<60;attempt++) {
   await pause(250);media=await invoke('get_media_info_cmd');
   if(`${media.source}|${media.title}|${media.artist}`!==previous && media.source) {changed=true;break;}
  }
  if(!changed) throw new Error(`Real player did not change track at step ${index}`);
  const state=await settledArtwork(floating,media.title);
  report.tracks.push({index,title:media.title,artist:media.artist,source:media.source,hasNativeCover:!!media.albumArt,state,utc:new Date().toISOString()});
  await flush();console.log(`Verified real track change ${index}/100: ${media.title}`);
  if(index%25===0) {
   await heapSnapshot(floating,index);
   await nativePoint(index);
   await floating.screenshot({path:resolve(output,`floating-${index}.png`)});
   await invoke('close_floating_window');
   await pause(1000);
   if(context.pages().some(page=>page.url().includes('window=floating')))throw new Error('Floating target retained after close');
   await invoke('open_floating_window');floating=await floatingPage();
   try { await settledArtwork(floating,media.title); } catch(error) {
    (report.reopenFailures ??= []).push({index,error:String(error)});await flush();
    console.log(`Recorded failed reopen at ${index}: ${error}`);
   }
  }
 }
 report.uniqueTracks=new Set(report.tracks.map(row=>`${row.source}|${row.title}|${row.artist}`)).size;
 report.phase='100-track-pass-completed; no-session and 5-minute recovery still required';await flush();
} catch(error) {
 report.failure=String(error);await flush();throw error;
} finally {
 // Disconnect diagnostics; leave the native application and player under user control.
 await browser.close();
}

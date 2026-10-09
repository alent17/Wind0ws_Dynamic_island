import { chromium } from '@playwright/test';
import { mkdir, writeFile, appendFile, readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import sharp from 'sharp';
const exec=promisify(execFile);
const rootPid=Number(process.env.ISLE_ROOT_PID);
if(!rootPid)throw new Error('Set ISLE_ROOT_PID to the native diagnostic process');
const endpoint=process.env.ISLE_CDP_ENDPOINT||'http://127.0.0.1:9228';
const output=resolve(process.argv[2]||'dist/performance/acceptance-2026-10-08/artwork');
const captureQuality=process.env.ARTWORK_CAPTURE_QUALITY==='1';
const qualityCorpus=captureQuality?JSON.parse(await readFile(resolve(output,'../frozen-covers.json'),'utf8')):[];
await mkdir(output,{recursive:true});
const browser=await chromium.connectOverCDP(endpoint);
const context=browser.contexts()[0];
const main=context.pages().find(page=>!page.url().includes('window=floating') && /index|tauri|localhost/.test(page.url()));
if(!main) throw new Error('No native Isle main WebView2 target');
const invoke=(command,args={})=>main.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
const resume=Number(process.env.ARTWORK_RESUME_INDEX||0);
const reopenInterval=Number(process.env.ARTWORK_REOPEN_INTERVAL??25);
if(![0,25].includes(reopenInterval))throw new Error('ARTWORK_REOPEN_INTERVAL must be 0 (continuous renderer) or 25');
const report=resume ? JSON.parse(await readFile(resolve(output,'artwork-acceptance.json'),'utf8')) : {runtime:process.env.ISLE_RUNTIME_DESCRIPTION||'Native WebView2 diagnostics; build type unspecified',startedUtc:new Date().toISOString(),tracks:[],snapshots:[],errors:[],reopens:[],expectedChanges:100};
report.reopens ??= [];
report.reopenInterval=reopenInterval;
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
 const state=await page.evaluate(async expected=>{
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
 if(process.env.ARTWORK_ALLOW_SCREENSHOT_PIXEL_CHECK==='1' && state.pixelCheckError?.includes('SecurityError') && state.connected && state.imageReady) {
  try {
   const url=await page.evaluate(()=>document.querySelector('img.compact-cover-image')?.src);
   const bytes=await diagnosticImageBytes(url);
   const actual=await page.locator('canvas.album-art-new').screenshot();
   await page.evaluate(async data=>{
    const original=document.querySelector('canvas.album-art-new'),box=original.getBoundingClientRect(),style=getComputedStyle(original);
    const image=new Image();image.src=data;await image.decode();
    const reference=document.createElement('canvas');reference.dataset.benchmarkReference='true';reference.width=original.width;reference.height=original.height;
    reference.getContext('2d').drawImage(image,0,0,reference.width,reference.height);
    Object.assign(reference.style,{position:'fixed',left:`${box.left}px`,top:`${box.top}px`,width:`${box.width}px`,height:`${box.height}px`,zIndex:'2147483647',pointerEvents:'none',borderRadius:style.borderRadius,filter:style.filter,willChange:style.willChange,backfaceVisibility:style.backfaceVisibility,imageRendering:style.imageRendering});
    document.body.append(reference);
   },`data:image/png;base64,${bytes.toString('base64')}`);
   const expected=await page.locator('canvas[data-benchmark-reference]').screenshot();
   const decode=async buffer=>sharp(buffer).removeAlpha().raw().toBuffer({resolveWithObject:true});
   const [a,b]=await Promise.all([decode(actual),decode(expected)]);
   let equal=a.info.width===b.info.width&&a.info.height===b.info.height;
   const checks=[];
   for(const ratio of [.2,.5,.8]) {
    const x=Math.floor(a.info.width*ratio),y=Math.floor(a.info.height*ratio),offset=(y*a.info.width+x)*a.info.channels;
    const first=[0,1,2].map(channel=>a.data[offset+channel]),last=[0,1,2].map(channel=>b.data[offset+channel]);checks.push({ratio,actual:first,expected:last});
    if(first.some((channel,index)=>Math.abs(channel-last[index])>2))equal=false;
   }
   // Compositor scaling of a promoted Canvas can differ from the reference
   // raster even at the same CSS size. Compare the full interior as well.
   let squared=0,channels=0;
   if(a.info.width===b.info.width&&a.info.height===b.info.height)for(let y=4;y<a.info.height-4;y++)for(let x=4;x<a.info.width-4;x++)for(let c=0;c<3;c++){const at=(y*a.info.width+x)*3+c;squared+=(a.data[at]-b.data[at])**2;channels++;}
   state.screenshotInteriorMse=channels?squared/channels:null;
   state.urlMatchesPixels=equal || (channels>0&&state.screenshotInteriorMse<=64);
   state.pixelCheckMethod='CSS-sized screenshot vs browser reference; exact three probes or interior RMS <= 8 RGB levels';
   if(!equal){state.screenshotPixelSamples=checks;state.screenshotSizes={actual:a.info,expected:b.info};await writeFile(resolve(output,'last-pixelcheck-actual.png'),actual);await writeFile(resolve(output,'last-pixelcheck-reference.png'),expected);}
  }catch(error){state.screenshotPixelCheckError=String(error);}
  finally{await page.evaluate(()=>document.querySelector('canvas[data-benchmark-reference]')?.remove());}
 }
 return state;
}
async function diagnosticImageBytes(url) {
 if(url?.startsWith('data:'))return Buffer.from(url.split(',')[1],'base64');
 if(url?.includes('asset.localhost')) {
  const path=decodeURIComponent(new URL(url).pathname).replace(/^\//,'');
  if(!/^[a-z]:[\\/]/i.test(path))throw new Error('Unexpected asset quality path');
  return readFile(path);
 }
 throw new Error('Expected local or data artwork for diagnostic quality capture');
}
async function settledArtwork(page,title) {
 let state;
 for(let attempt=0;attempt<60;attempt++) {
  state=await canvasState(page,title);
  if(state.titleMatches && state.canvasCount===1 && state.connected && state.imageReady && state.width>0 && state.height>0 && state.urlMatchesPixels===true) return state;
  await pause(250);
 }
 throw new Error(`Artwork did not settle for latest track: ${JSON.stringify(state)}`);
}
async function qualitySource(page,index) {
 if(!captureQuality)return;
 // A hover veil/play button is product UI, not image quality. Keep pointer
 // outside the floating viewport in both separate quality passes.
 await page.mouse.move(-100,-100);await pause(150);
 const title=(await page.locator('.track-title').textContent())?.trim();
 const expected=qualityCorpus.find(row=>row.title===title);if(!expected)throw new Error('Quality capture title is outside frozen corpus');
 for(let attempt=0;attempt<80;attempt++) {
  const width=await page.evaluate(()=>document.querySelector('img.compact-cover-image')?.naturalWidth);
  if(width===expected.width && (await canvasState(page,title)).urlMatchesPixels===true)break;
  if(attempt===79)throw new Error('Final HD artwork did not settle for quality capture');
  await pause(250);
 }
 const value=await page.evaluate(()=>({url:document.querySelector('img.compact-cover-image')?.src,width:document.querySelector('canvas.album-art-new')?.width,height:document.querySelector('canvas.album-art-new')?.height}));
 const bytes=await diagnosticImageBytes(value.url);
 await writeFile(resolve(output,`quality-source-${index}.img`),bytes);
 // Separate diagnostic pass: capture actual CSS-sized raster presentation too.
 // Source bytes alone cannot reveal a regression from a smaller Canvas backing.
 await pause(300);
 await page.locator('canvas.album-art-new').screenshot({path:resolve(output,`quality-render-${index}.png`)});
 const presentation=await page.locator('canvas.album-art-new').evaluate(canvas=>({cssWidth:canvas.getBoundingClientRect().width,cssHeight:canvas.getBoundingClientRect().height,dpi:devicePixelRatio,opacity:getComputedStyle(canvas).opacity}));
 (report.qualitySources ??= []).push({index,width:value.width,height:value.height,bytes:bytes.length,presentation});
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
 if(captureQuality){await floating.mouse.move(-100,-100);await pause(150);}
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
  await qualitySource(floating,index);
  report.tracks.push({index,title:media.title,artist:media.artist,source:media.source,hasNativeCover:!!media.albumArt,state,utc:new Date().toISOString()});
  await flush();console.log(`Verified real track change ${index}/100: ${media.title}`);
  if(index%25===0) {
   await heapSnapshot(floating,index);
   await nativePoint(index);
   await floating.screenshot({path:resolve(output,`floating-${index}.png`)});
   if(reopenInterval) {
   await invoke('close_floating_window');
   await pause(1000);
   if(context.pages().some(page=>page.url().includes('window=floating')))throw new Error('Floating target retained after close');
   await invoke('open_floating_window');floating=await floatingPage();
   floating.on('pageerror',error=>report.errors.push(String(error)));
   report.reopens.push({index,state:await settledArtwork(floating,media.title)});await flush();
   }
  }
 }
 report.uniqueTracks=new Set(report.tracks.map(row=>`${row.source}|${row.title}|${row.artist}`)).size;
 if(report.failure || report.reopenFailures?.length || report.errors.length || report.snapshots.some(row=>row.detachedCanvasNodes!==0) || report.tracks.length!==100 || report.tracks.some(row=>row.state.urlMatchesPixels!==true) || report.reopens.length!==(reopenInterval?4:0))throw new Error('Incomplete acceptance, prior failure, page errors or Detached Canvas detected; use a fresh output directory for a new pass');
 report.phase='100-track-and-reopen-pass-completed; no-session recovery requires separate validation';await flush();
} catch(error) {
 report.failure=String(error);await flush();throw error;
} finally {
 // Disconnect diagnostics; leave the native application and player under user control.
 await browser.close();
}

import { chromium } from '@playwright/test';
import { writeFile,mkdir } from 'node:fs/promises';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { resolve } from 'node:path';
const exec=promisify(execFile),out=resolve('dist/performance/acceptance-2026-10-08/recovery');
await mkdir(out,{recursive:true});
const browser=await chromium.connectOverCDP('http://127.0.0.1:9228'),context=browser.contexts()[0];
const main=context.pages().find(p=>!p.url().includes('window=floating'));
const invoke=(cmd,args={})=>main.evaluate(({cmd,args})=>window.__TAURI_INTERNALS__.invoke(cmd,args),{cmd,args});
const pid=process.env.ISLE_ROOT_PID;if(!pid)throw new Error('Missing root PID');
const report={startedUtc:new Date().toISOString(),deviceQueries:0,deviceErrors:[],phase:'device-stress',noSessionCases:[]};
const flush=()=>writeFile(resolve(out,'recovery.json'),JSON.stringify(report,null,2));
const wait=ms=>new Promise(done=>setTimeout(done,ms));
async function point(label,seconds=1) {
 await exec('powershell',['-NoProfile','-File','scripts/measure-native.ps1','-RootPid',pid,'-Seconds',String(seconds),'-IncludeGpu','-Label',label,'-OutputDirectory',out],{timeout:(seconds+45)*1000});
}
async function state() {
 const media=await invoke('get_media_info_cmd');
 const floating=context.pages().find(p=>p.url().includes('window=floating'));
 const dom=floating?await floating.evaluate(()=>({title:document.querySelector('.track-title')?.textContent,canvasCount:document.querySelectorAll('canvas.album-art-new').length,imageCount:document.querySelectorAll('.compact-cover-image').length})):null;
 return {media:{title:media.title,source:media.source,artBytes:media.albumArt?.length||0},dom};
}
try {
 await invoke('close_floating_window');
 await point('device-before');
 for(let i=0;i<1000;i++) {
  try { const devices=await invoke('list_audio_output_devices');if(!devices.length)throw new Error('No output devices'); }
  catch(error) { report.deviceErrors.push({index:i,error:String(error)}); }
  report.deviceQueries++;
  if(report.deviceQueries%250===0){await point(`device-${report.deviceQueries}`);await flush();console.log(`Device queries ${report.deviceQueries}/1000`);}
 }
 await invoke('open_floating_window');await wait(4000);
 report.noSessionCases.push({phase:'before-no-session',state:await state()});
 await flush();
 console.log('Ready for real player exit/no-session check.');
 report.phase='awaiting-real-player-exit';await flush();
} finally { await browser.close(); }

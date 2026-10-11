import assert from 'node:assert/strict';
import { chromium } from '@playwright/test';
import { spawn,execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir,copyFile,writeFile,readFile } from 'node:fs/promises';
import { resolve,join } from 'node:path';
import { createHash } from 'node:crypto';
const variant=process.argv[2];assert(['before','after'].includes(variant));
const base=resolve('dist/performance/m5-2026-10-12'),out=join(base,variant);
await mkdir(out,{recursive:true});
const exe=join(out,'isle.exe');await copyFile(join(base,'binaries',variant,'isle.exe'),exe);
const pause=ms=>new Promise(done=>setTimeout(done,ms)),exec=promisify(execFile);
const appData=join(out,'appdata'),config=join(appData,'com.isle-app.m5diag');await mkdir(config,{recursive:true});
const settings=JSON.parse(await readFile('dist/performance/m4-2026-10-10/settings.json','utf8'));
Object.assign(settings,{showSpectrum:false,idleContentEnabled:false,enableMvPlayback:false,enableHdCover:false,floatingUseAlbumColor:false,captureHideOnScreenshot:false,captureHideOnRecording:false,captureHideOnFullscreen:false,captureHideOnScreenShare:false,monitorIndex:0,floatingWindowX:null,floatingWindowY:null,floatingWindowWidth:200,floatingWindowHeight:395});
await writeFile(join(config,'settings.json'),JSON.stringify(settings,null,2));
const env={...process.env,APPDATA:appData,WEBVIEW2_USER_DATA_FOLDER:join(out,'webview'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9236 --remote-debugging-address=127.0.0.1',NO_PROXY:'127.0.0.1,localhost'};
const app=spawn(exe,[],{env,windowsHide:true,stdio:'ignore'});
const report={variant,runtime:'matched Release WebView2, isolated app data/profile, CDP forced-GC diagnostic; not formal CPU/GPU benchmark',baseline:'0f48301',binarySha256:createHash('sha256').update(await readFile(exe)).digest('hex'),warmupCycles:10,cycles:0,points:[],errors:[],imageErrors:[],requests:[],limits:{closedNodesGrowth:10,closedListenerGrowth:2,closedHeapGrowthBytes:1048576,treeHandlesGrowth:30,treeThreadsGrowth:8,treePrivateGrowthBytes:16777216}};
const flush=()=>writeFile(join(out,'results.json'),JSON.stringify(report,null,2));
let browser;
try {
 for(let i=0;i<60;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9236',{timeout:1000});break;}catch{await pause(500);}}
 assert(browser,'WebView2 CDP unavailable');const context=browser.contexts()[0];let main;
 for(let i=0;i<100&&!main;i++){
  if(app.exitCode!==null)throw new Error(`Release app exited before main WebView navigation: ${app.exitCode}`);
  main=context.pages().find(p=>/tauri\.localhost|tauri:\/\/localhost/.test(p.url())&&!p.url().includes('window=floating'));
  if(!main)await pause(200);
 }
 if(!main){const targets=context.pages().map(p=>p.url());const target=context.pages()[0];const details=target?await target.evaluate(()=>({url:location.href,title:document.title,body:document.body?.innerText?.slice(0,500)})).catch(e=>String(e)):'';throw new Error(`Main WebView target missing; CDP targets: ${targets.join(', ')}; details: ${JSON.stringify(details)}`);}
 const invoke=(cmd,args={})=>main.evaluate(({cmd,args})=>window.__TAURI_INTERNALS__.invoke(cmd,args),{cmd,args});
 await pause(5000);
 await invoke('update_settings',{patch:{showSpectrum:false,enableHdCover:false,idleContentEnabled:false,enableMvPlayback:false,floatingUseAlbumColor:false,captureHideOnScreenshot:false,captureHideOnRecording:false,captureHideOnFullscreen:false,captureHideOnScreenShare:false}});
 const activeSettings=await invoke('get_settings');assert(activeSettings.showSpectrum===false&&activeSettings.enableHdCover===false&&!activeSettings.idleContentEnabled,`Isolated low-activity settings did not apply: ${JSON.stringify(activeSettings)}`);
 await invoke('close_floating_window');
 const mainSession=await context.newCDPSession(main);await mainSession.send('Performance.enable');
 async function metrics(session){await session.send('HeapProfiler.collectGarbage');await session.send('HeapProfiler.collectGarbage');return Object.fromEntries((await session.send('Performance.getMetrics')).metrics.filter(m=>['Nodes','Documents','JSEventListeners','JSHeapUsedSize'].includes(m.name)).map(m=>[m.name,m.value]));}
 async function native(){const r=await exec(process.env.M5_PWSH||'pwsh',['-NoProfile','-File',resolve('scripts/m5-native-point.ps1'),'-RootPid',String(app.pid)],{timeout:30000});return JSON.parse(r.stdout.replace(/^\uFEFF/,''));}
 async function cycle(index,point=false){
  await invoke('open_floating_window');
  let floating;
  for(let i=0;i<60;i++){floating=context.pages().find(p=>p.url().includes('window=floating'));if(floating)break;await pause(100);}
  assert(floating,'Floating target missing');
  floating.on('pageerror',error=>report.errors.push({index,error:String(error)}));
  const requestStarts=new Map();
  floating.on('request',request=>{if(request.resourceType()==='image')requestStarts.set(request,Date.now());});
  floating.on('requestfinished',request=>{if(!requestStarts.has(request))return;const started=requestStarts.get(request),url=new URL(request.url());report.requests.push({index,status:'loaded',host:url.host,path:url.pathname.split('/').at(-1),startedAt:started,durationMs:Date.now()-started});requestStarts.delete(request);});
  floating.on('requestfailed',request=>{if(request.resourceType()!=='image')return;const started=requestStarts.get(request)??Date.now(),url=new URL(request.url());report.imageErrors.push({index,host:url.host,path:url.pathname.split('/').at(-1),startedAt:started,durationMs:Date.now()-started,error:request.failure(),at:Date.now()});requestStarts.delete(request);});
  await floating.waitForLoadState('domcontentloaded');
  await floating.waitForSelector('.player',{timeout:15000});
  await pause(250);
  let live;
  if(point){const session=await context.newCDPSession(floating);await session.send('Performance.enable');live=await metrics(session);await session.detach();}
  await invoke('close_floating_window');
  for(let i=0;i<60&&context.pages().some(p=>p.url().includes('window=floating'));i++)await pause(100);
  assert(!context.pages().some(p=>p.url().includes('window=floating')),'Floating renderer did not close');
  await pause(point?1500:150);
  if(point){report.points.push({index,live,closed:await metrics(mainSession),native:await native()});await flush();console.log(`${variant} ${index}/100`,JSON.stringify(report.points.at(-1)));}
 }
 for(let i=0;i<10;i++)await cycle(-10+i);
 await pause(3000);report.points.push({index:0,closed:await metrics(mainSession),native:await native()});await flush();
 for(let i=1;i<=100;i++){await cycle(i,i%10===0);report.cycles=i;await flush();}
 await pause(10000);report.recovery={closed:await metrics(mainSession),native:await native()};
 const first=report.points[0],last=report.recovery;
 report.growth={nodes:last.closed.Nodes-first.closed.Nodes,listeners:last.closed.JSEventListeners-first.closed.JSEventListeners,heap:last.closed.JSHeapUsedSize-first.closed.JSHeapUsedSize,handles:last.native.handles-first.native.handles,threads:last.native.threads-first.native.threads,privateBytes:last.native.privateBytes-first.native.privateBytes};
 assert.equal(report.errors.length,0,`WebView page errors: ${JSON.stringify(report.errors.slice(0,5))}`);
 assert.equal(report.imageErrors.length,0,`Image request failures: ${JSON.stringify(report.imageErrors.slice(0,5))}`);
 report.completed=true;await flush();
} catch(error){report.errors.push(String(error));await flush();throw error;}
finally{await browser?.close();if(app.exitCode===null)app.kill();}

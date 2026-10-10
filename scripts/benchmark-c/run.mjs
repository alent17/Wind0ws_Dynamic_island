import assert from 'node:assert/strict';
import {spawn,execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {mkdir,readFile,writeFile,copyFile,stat} from 'node:fs/promises';
import {resolve,join} from 'node:path';
import {createHash} from 'node:crypto';

const label=process.argv[2],variant=process.argv[3];
const pilot=process.env.ISLE_BENCH_PILOT==='1';
assert((pilot?/^(before|after)-pilot\d+$/:/^(before|after)-r[123]$/).test(label)&&['before','after'].includes(variant));
const batch=process.env.ISLE_BENCH_BATCH||'runs-final';assert(/^[a-z0-9-]+$/.test(batch));
const base=resolve(process.env.ISLE_BENCH_ROOT||'dist/performance/m3-release-c-2026-10-09'),output=join(base,batch,label);
await mkdir(output,{recursive:true});
assert(!(await stat(join(output,'events.jsonl')).catch(()=>null)),'Use a new output directory; prior samples must not be overwritten');
const binary=join(base,'binaries',variant,'isle.exe'),exe=join(output,'isle.exe');
await copyFile(binary,exe);
await copyFile(join(base,'frozen-covers.json'),join(output,'frozen-covers.json'));
const corpus=JSON.parse(await readFile(join(base,'corpus.json'),'utf8'));
const settings=JSON.parse(await readFile(join(base,'settings.json'),'utf8'));
const config=join(process.env.APPDATA,'com.isle-app.benchmarkc');await mkdir(config,{recursive:true});await writeFile(join(config,'settings.json'),JSON.stringify(settings));
const exec=promisify(execFile);
const pause=ms=>new Promise(done=>setTimeout(done,ms));
let app,sampler,sequence=0;
const report={label,variant,binarySha256:createHash('sha256').update(await readFile(binary)).digest('hex'),startedUtc:new Date().toISOString(),formal:!pilot,devTools:false,initialIsleCache:'empty fresh executable directory',warmupSeconds:pilot?10:60,recoverySeconds:pilot?10:300,minimumTrackSeconds:5,tracks:[],errors:[]};
async function events(){const body=await readFile(join(output,'events.jsonl'),'utf8').catch(()=> '');return body.split('\n').filter(Boolean).flatMap(line=>{try{return [JSON.parse(line)];}catch{return [];}});}
async function command(action,extra={}){const value={seq:++sequence,action,...extra};await writeFile(join(output,'command.json'),JSON.stringify(value));for(let i=0;i<50;i++){const row=(await events()).find(row=>row.event.kind==='command'&&row.event.command.seq===value.seq);if(row){assert(row.event.ok,JSON.stringify(row));return row;}await pause(200);}throw new Error('Native benchmark command timeout');}
async function playerAction(action){const timing=join(output,`control-${sequence}-${action}.json`);await exec('powershell',['-NoProfile','-File',resolve('scripts/media-session.ps1'),'-Action',action,'-TimingFile',timing],{timeout:30000});const rows=JSON.parse((await readFile(timing,'utf8')).replace(/^\uFEFF/,''));assert(rows.length===1,'Exactly one authorized player control required');return rows[0];}
const flush=()=>writeFile(join(output,'run.json'),JSON.stringify(report,null,2));
try {
 await exec(process.execPath,[resolve('scripts/benchmark-c/prepare-player.mjs')],{timeout:60000});
 const env={...process.env,ISLE_BENCH_DIR:output,WEBVIEW2_USER_DATA_FOLDER:join(output,'webview'),NO_PROXY:'127.0.0.1,127.0.0.2,localhost',no_proxy:'127.0.0.1,127.0.0.2,localhost'};delete env.WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS;
 app=spawn(exe,[],{env,windowsHide:true,stdio:'ignore'});report.pid=app.pid;
 for(let i=0;i<100;i++){if((await events()).some(row=>row.event.kind==='started'))break;if(i===99)throw new Error('Native process did not start');await pause(200);}
 assert((await events()).find(row=>row.event.kind==='started').event.debugAssertions===false,'Release build required');
 await command('open');await pause(report.warmupSeconds*1000);
 await command('mark',{phase:'continuous-start'});
 sampler=spawn('powershell',['-NoProfile','-File',resolve('scripts/measure-native.ps1'),'-RootPid',String(app.pid),'-Seconds','2400','-IncludeGpu','-Label',label,'-OutputDirectory',output,'-StopFile',join(output,'sampler.stop')],{windowsHide:true,stdio:['ignore','pipe','pipe']});
 sampler.stdout.on('data',data=>process.stdout.write(data));sampler.stderr.on('data',data=>process.stderr.write(data));
 for(const song of (pilot?corpus.slice(0,3):corpus)){
  const at=Date.now();await command('mark',{phase:'track',index:song.index,id:song.id,title:song.title});const control=await playerAction('Next');
  let paints=[],hd=[];
  for(let i=0;i<100;i++){
   const rows=(await events()).filter(row=>row.at>=at&&row.event.title===song.title);
   paints=rows.filter(row=>row.event.kind==='paint');hd=rows.filter(row=>row.event.kind==='hd-resolved');
   if(paints.length&&hd.length&&Date.now()-at>=5000)break;
   if(i===99)throw new Error(`Artwork/HD completion timeout: ${song.index} ${song.title}`);
   await pause(200);
  }
  assert(paints.length);const first=paints[0],last=paints.at(-1);
  assert(first.event.dpi===1&&first.event.windowWidth===200&&first.event.windowHeight===395,'DPI/window condition changed');
  report.tracks.push({index:song.index,id:song.id,title:song.title,driverLatencyMs:first.at-at,control,firstPaintLatencyMs:first.at-control.startedMs,lastPaintLatencyMs:last.at-control.startedMs,hdResolved:hd.at(-1).event.found,first:first.event,last:last.event,bufferPeaks:last});
  console.log(`${label} ${song.index}/100: ${song.title}; first paint ${first.at-at} ms; HD ${hd.at(-1).event.found}`);await flush();
 }
 await command('mark',{phase:'recovery-start'});await playerAction('Pause');
 if(pilot)await pause(10000);
 else for(let i=0;i<5;i++){await pause(60000);await command('mark',{phase:'recovery-minute',minute:i+1});console.log(`${label} recovery ${i+1}/5 minutes`);}
 await command('mark',{phase:'complete'});report.complete=true;
}catch(error){report.failure=String(error);throw error;}
finally{
 await writeFile(join(output,'sampler.stop'),'stop');
 if(sampler&&sampler.exitCode===null)await Promise.race([new Promise(done=>sampler.on('exit',done)),pause(15000)]);
 if(app&&app.exitCode===null)app.kill();
 report.endedUtc=new Date().toISOString();await flush();
}

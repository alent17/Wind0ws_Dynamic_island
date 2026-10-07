import { chromium } from '@playwright/test';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,extname} from 'node:path';
const label=process.argv[2]||'before', root=resolve(`dist/performance/${label}`);
const defaultScenes=['paused-compact','playing-compact','music-expanded','volume-detail','unmounted'];
const scenes=(process.env.PERF_SCENES||'').split(',').map(scene=>scene.trim()).filter(Boolean);
const selectedScenes=scenes.length?scenes:defaultScenes;
const sampleSeconds=Math.max(1,Number(process.env.PERF_SAMPLE_SECONDS)||60);
const server=createServer(async(req,res)=>{try{const path=resolve(root,'.'+decodeURIComponent(req.url.split('?')[0]));if(!path.startsWith(root))throw Error();res.setHeader('Content-Type',({'.js':'text/javascript','.css':'text/css','.html':'text/html','.svg':'image/svg+xml','.woff2':'font/woff2'})[extname(path)]||'application/octet-stream');res.end(await readFile(path));}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const browser=await chromium.launch({channel:'chrome',headless:true});
const results=[];
try {
for(const scene of selectedScenes){
 const context=await browser.newContext({viewport:{width:1280,height:760},reducedMotion:'no-preference'});
 const page=await context.newPage();
 const cdp=await context.newCDPSession(page);await cdp.send('Performance.enable');
 const metrics=async()=>Object.fromEntries((await cdp.send('Performance.getMetrics')).metrics.map(m=>[m.name,m.value]));
 try {
 const animate=scene==='playing-compact'||scene==='music-expanded';
 await page.goto(`http://127.0.0.1:${server.address().port}/ui-tests/island-fixture.html${animate?'?animate':''}`);
 await page.evaluate(({ scene, animate }) => window.configureIsland({
   mode: scene.endsWith('compact') ? 'compact' : 'expanded',
   playing: scene !== 'paused-compact', toolbar: scene === 'volume-detail', animate,
 }), {scene, animate});
 if(scene==='volume-detail') await page.locator('.feature-menu button').nth(1).click();
 if(scene==='unmounted')await page.evaluate(() => window.unmountIsland());
 await page.waitForTimeout(5000);
 const start=await metrics();const rows=[];
 for(let i=0;i<sampleSeconds;i++){await page.waitForTimeout(1000);rows.push(await metrics());}
 const end=rows.at(-1);results.push({scene,sampleSeconds,rendererTaskCpuPercent:100*(end.TaskDuration-start.TaskDuration)/(end.Timestamp-start.Timestamp),startHeap:start.JSHeapUsedSize,endHeap:end.JSHeapUsedSize,domNodes:end.Nodes,listeners:end.JSEventListeners});
await writeFile(`dist/performance/${label}.json`,JSON.stringify({runtime:'Chrome headless production fixture; playback scenes animate a synthetic random spectrum. Each scene uses a fresh browser context. Renderer TaskDuration and JS heap only; not native WebView process CPU or total memory. No forced GC in ordinary samples.',results},null,2));
 console.log(label,scene,results.at(-1));
 } finally { await context.close(); }
}
} finally { await browser.close();server.close(); }

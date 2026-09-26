import { chromium } from '@playwright/test';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,extname} from 'node:path';
const label=process.argv[2]||'before', root=resolve(`dist/performance/${label}`);
const server=createServer(async(req,res)=>{try{const path=resolve(root,'.'+decodeURIComponent(req.url.split('?')[0]));if(!path.startsWith(root))throw Error();res.setHeader('Content-Type',({'.js':'text/javascript','.css':'text/css','.html':'text/html','.svg':'image/svg+xml','.woff2':'font/woff2'})[extname(path)]||'application/octet-stream');res.end(await readFile(path));}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const browser=await chromium.launch({channel:'chrome',headless:true});
const page=await browser.newPage({viewport:{width:1280,height:760},reducedMotion:'no-preference'});
const cdp=await page.context().newCDPSession(page);await cdp.send('Performance.enable');
const metrics=async()=>Object.fromEntries((await cdp.send('Performance.getMetrics')).metrics.map(m=>[m.name,m.value]));
const results=[];
for(const scene of ['paused-compact','playing-compact','music-expanded','volume-detail','unmounted']){
 await page.goto(`http://127.0.0.1:${server.address().port}/ui-tests/island-fixture.html`);
 if(scene==='paused-compact')await page.locator('.controls .play').click();
 if(scene.endsWith('compact'))await page.getByTestId('compact-mode').click();
 if(scene==='volume-detail'){if(await page.locator('.expand-functions').count())await page.locator('.expand-functions').click();await page.locator('.feature-menu button').nth(1).click();}
 if(scene==='unmounted')await page.goto('about:blank');
 await page.waitForTimeout(5000);
 const start=await metrics();const rows=[];
 for(let i=0;i<60;i++){await page.waitForTimeout(1000);rows.push(await metrics());}
 const end=rows.at(-1);results.push({scene,sampleSeconds:60,rendererTaskCpuPercent:100*(end.TaskDuration-start.TaskDuration)/(end.Timestamp-start.Timestamp),startHeap:start.JSHeapUsedSize,endHeap:end.JSHeapUsedSize,domNodes:end.Nodes,listeners:end.JSEventListeners});
 await writeFile(`dist/performance/${label}.json`,JSON.stringify({runtime:'Chrome headless production fixture, not native WebView',results},null,2));
 console.log(label,scene,results.at(-1));
}
await browser.close();server.close();

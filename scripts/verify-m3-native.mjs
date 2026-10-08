import { chromium } from '@playwright/test';
import {writeFile,mkdir} from 'node:fs/promises';
import {resolve} from 'node:path';
const output=resolve('dist/performance/m3-2026-10-08');await mkdir(output,{recursive:true});
const browser=await chromium.connectOverCDP('http://127.0.0.1:9229');
const page=browser.contexts()[0].pages().find(page=>!page.url().includes('window='));
const invoke=async (command,args)=>{
 const reply=await page.evaluate(async ({command,args})=>{
  try {return {ok:true,value:await window.__TAURI_INTERNALS__.invoke(command,args)};}
  catch(error) {return {ok:false,error:typeof error==='object' && error ? {code:error.code,message:error.message||String(error)} : {message:String(error)}};}
 },{command,args});
 if(!reply.ok){const error=new Error(reply.error.message);error.code=reply.error.code;throw error;}
 return reply.value;
};
const path=name=>resolve('dist/m3-fixtures',name);
const results=[];
try {
 for(const pixelSize of [0,4294967295]) {
  let error;try{await invoke('pixelate_cover',{imagePath:path('wide.png'),pixelSize});}catch(caught){error=caught;}
  if(!error)throw new Error(`pixel_size ${pixelSize} was accepted`);
  results.push({case:`pixel_size=${pixelSize}`,rejected:true,error:{message:error.message,code:error.code}});
 }
 for(const filename of ['broken.png','over-bytes.png','over-edge.png','over-pixels.png']) {
  let error;try{await invoke('process_image',{imagePath:path(filename),enablePixelArt:false});}catch(caught){error=caught;}
  if(!error)throw new Error(`Unbounded input accepted: ${filename}`);
  results.push({case:filename,rejected:true,error:{message:error.message,code:error.code}});
 }
 for(const pixelated of [false,true]) {
  const value=await invoke('process_image',{imagePath:path('wide.png'),enablePixelArt:pixelated});
  const dimensions=await page.evaluate(async url=>{const image=new Image();image.src=url;await image.decode();return {width:image.naturalWidth,height:image.naturalHeight};},value);
  if(dimensions.width!==1280 || dimensions.height!==720 || !value.startsWith('data:image/png;base64,'))throw new Error('Normalized output violated its raster or MIME contract');
  results.push({case:`wide.png pixelated=${pixelated}`,dimensions,uriChars:value.length});
 }
 const afterErrors=await invoke('get_settings',{});
 results.push({case:'backend responds after all invalid inputs',success:!!afterErrors});
 console.log(JSON.stringify(results,null,2));
 await writeFile(resolve(output,'native-command-results.json'),JSON.stringify({runtime:'isolated debug custom-protocol native build; not a Release performance benchmark',results},null,2));
} finally { await browser.close(); }

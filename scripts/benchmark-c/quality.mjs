import {readFile,writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import sharp from 'sharp';
const root=resolve(process.env.ISLE_BENCH_ROOT||'dist/performance/m3-release-c-2026-10-09');
const batch=process.env.ISLE_BENCH_QUALITY_BATCH||'diagnostic';if(!/^[a-z0-9-]+$/.test(batch))throw new Error('Invalid quality batch');
const output=process.env.ISLE_BENCH_QUALITY_OUTPUT||'quality-comparison.json';if(!/^[a-z0-9.-]+\.json$/.test(output))throw new Error('Invalid quality output filename');
const manifest=JSON.parse(await readFile(`${root}/frozen-covers.json`,'utf8'));
const before=JSON.parse(await readFile(`${root}/${batch}/before-continuous/artwork/artwork-acceptance.json`,'utf8'));
const after=JSON.parse(await readFile(`${root}/${batch}/after-continuous/artwork/artwork-acceptance.json`,'utf8'));
for(const report of [before,after]) {
 if(report.failure || report.errors.length || report.tracks.length!==100 || report.qualitySources.length!==100)throw new Error('Incomplete quality diagnostic');
 if(typeof report.renderingDpr==='number' && report.qualitySources.some(row=>row.presentation.dpi!==report.renderingDpr))throw new Error('Observed rendering DPR differs from requested DPR');
}
const rows=[];
function compare(first,last){
 if(first.length!==last.length)throw new Error('Quality raster lengths differ');
 let squared=0,max=0,changed=0;for(let i=0;i<first.length;i++){const delta=Math.abs(first[i]-last[i]);squared+=delta*delta;max=Math.max(max,delta);if(delta)changed++;}
 const mse=squared/first.length;
 return {mse,psnrDb:mse?10*Math.log10(255*255/mse):null,maxChannelDifference:max,changedChannelFraction:changed/first.length};
}
for(const index of Array.from({length:100},(_,i)=>i+1)){
 const b=before.tracks.find(row=>row.index===index),a=after.tracks.find(row=>row.index===index);
 if(b.title!==a.title||b.title!==manifest.find(row=>row.index===index).title)throw new Error('Quality corpus order mismatch');
 const width=180,height=180;
 const load=async variant=>sharp(`${root}/${batch}/${variant}-continuous/artwork/quality-source-${index}.img`).resize(width,height,{fit:'fill',kernel:'lanczos3'}).removeAlpha().raw().toBuffer();
 const [first,last]=await Promise.all([load('before'),load('after')]);
 const screenshot=async variant=>{
  const file=`${root}/${batch}/${variant}-continuous/artwork/quality-render-${index}.png`;
  const image=sharp(file),info=await image.metadata();
  return {width:info.width,height:info.height,pixels:await image.removeAlpha().raw().toBuffer()};
 };
 const [beforeRender,afterRender]=await Promise.all([screenshot('before'),screenshot('after')]);
 if(beforeRender.width!==afterRender.width||beforeRender.height!==afterRender.height)throw new Error('Canvas presentation dimensions changed');
 rows.push({index,title:b.title,hdSourceUnavailable:!!manifest.find(row=>row.index===index).hdSourceUnavailable,comparison:'decoded actual displayed source images resized to fixed 180x180',...compare(first,last),renderComparison:{scope:'Actual CSS-sized Canvas element screenshots in a separate diagnostic pass',width:beforeRender.width,height:beforeRender.height,...compare(beforeRender.pixels,afterRender.pixels)}});
}
const result={scope:'Separate CDP diagnostic pass, same real-player queue and frozen input corpus. Source comparisons and actual Canvas presentation comparisons are listed separately.',diagnosticBatch:batch,samples:100,rows,worstMse:Math.max(...rows.map(row=>row.mse)),exactPixelMatches:rows.filter(row=>row.mse===0).length,renderWorstMse:Math.max(...rows.map(row=>row.renderComparison.mse)),renderExactPixelMatches:rows.filter(row=>row.renderComparison.mse===0).length};
await writeFile(`${root}/${output}`,JSON.stringify(result,null,2));console.log({samples:result.samples,worstMse:result.worstMse,exactPixelMatches:result.exactPixelMatches,renderWorstMse:result.renderWorstMse});

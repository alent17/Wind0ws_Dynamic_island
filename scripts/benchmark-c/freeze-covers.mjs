import {mkdir,readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import sharp from 'sharp';
const root=process.env.ISLE_BENCH_ROOT||'dist/performance/m3-release-c-2026-10-09';await mkdir(`${root}/covers`,{recursive:true});
const corpus=JSON.parse(await readFile(`${root}/corpus.json`,'utf8')),queue=JSON.parse(await readFile(`${root}/queue.json`,'utf8'));
const rows=[{index:0,...queue.find(song=>song.id==='1309915258')},...corpus];
const original=JSON.parse(await readFile(`${root}/queue-backup.json`,'utf8'));
for(const row of rows)row.durationMs=original.find(song=>song.resourceId===row.id).track.duration;
let next=0;
async function worker(){while(next<rows.length){const song=rows[next++];let url=new URL(song.cover);url.protocol='https:';url.search='param=1200y1200';let bytes=await readFile(`${root}/covers/${song.index}.img`).catch(()=>null);
 if(!bytes)for(let attempt=0;attempt<3;attempt++){try{const response=await fetch(url,{signal:AbortSignal.timeout(15000)});if(!response.ok)throw new Error(`HTTP ${response.status}`);bytes=Buffer.from(await response.arrayBuffer());await sharp(bytes).metadata();break;}catch(error){
  if(attempt===0){const detail=await(await fetch(`https://music.163.com/api/song/detail/?id=${song.id}&ids=%5B${song.id}%5D`,{signal:AbortSignal.timeout(15000)})).json();const cover=detail.songs?.[0]?.album?.picUrl;if(cover){url=new URL(cover);url.protocol='https:';url.search='param=1200y1200';song.providerMetadataRefreshed=true;}}
  if(attempt===2)throw new Error(`Cover ${song.index} ${song.title} ${url}: ${error}`);
 }}
 const file=`${song.index}.img`;await writeFile(`${root}/covers/${file}`,bytes);const info=await sharp(bytes).metadata();
 Object.assign(song,{file,sourceUrl:url.href,sha256:createHash('sha256').update(bytes).digest('hex'),bytes:bytes.length,width:info.width,height:info.height,format:info.format,localUrl:`http://127.0.0.2:9234/cover/${song.index}`});
 const provenance=await readFile(`${root}/covers/${song.index}.provenance.json`,'utf8').catch(()=>null);if(provenance)Object.assign(song,JSON.parse(provenance));
 if(song.index%10===0)console.log(`Frozen cover ${song.index}/100`);
}}
await Promise.all([worker(),worker(),worker()]);
await writeFile(`${root}/frozen-covers.json`,JSON.stringify(rows,null,2));console.log('Frozen 101 verified artwork inputs');

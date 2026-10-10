import {createServer} from 'node:http';
import {readFile} from 'node:fs/promises';
const folder=process.env.ISLE_BENCH_ROOT||'dist/performance/m3-release-c-2026-10-09';
const rows=JSON.parse(await readFile(`${folder}/frozen-covers.json`,'utf8'));
const images=new Map(await Promise.all(rows.map(async row=>[String(row.index),await readFile(`${folder}/covers/${row.file}`)])));
const pause=ms=>new Promise(done=>setTimeout(done,ms));
const server=createServer(async(req,res)=>{
 const id=/^\/cover\/(\d+)(?:\?.*)?$/.exec(req.url||'')?.[1],bytes=images.get(id);
 if(!bytes){res.writeHead(404);res.end();return;}
 res.writeHead(200,{'Content-Type':'image/jpeg','Content-Length':bytes.length,'Cache-Control':'no-store'});
 for(let offset=0;offset<bytes.length;offset+=65536){if(res.destroyed)return;res.write(bytes.subarray(offset,offset+65536));await pause(16);}
 res.end();
});
server.listen(9234,'127.0.0.2',()=>console.log('Fixed corpus HTTP replay on 127.0.0.2: same cache path in both builds, 64 KiB chunks, 16 ms/chunk'));

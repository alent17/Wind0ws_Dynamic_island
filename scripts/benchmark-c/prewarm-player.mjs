import {readFile,writeFile} from 'node:fs/promises';
import {Player} from './player.mjs';
const folder=process.env.ISLE_BENCH_ROOT||'dist/performance/m3-release-c-2026-10-09';
const corpus=JSON.parse(await readFile(`${folder}/corpus.json`,'utf8'));
const player=await Player.connect(),rows=[];
const pause=ms=>new Promise(done=>setTimeout(done,ms));
try {
 for(const song of corpus){
  await player.play(song.id);let state;
  for(let i=0;i<60;i++){state=await player.state();if(state.id===song.id)break;if(i===59)throw new Error(`Failed to select ${song.id}`);await pause(250);}
  await pause(1000);rows.push({index:song.index,id:song.id,title:state.title});
  if(song.index%10===0)console.log(`Player queue prewarm ${song.index}/100`);
 }
 await writeFile(`${folder}/player-prewarm.json`,JSON.stringify({scope:'Player preparation outside all formal Isle runs; fixed corpus IDs',tracks:rows},null,2));
}finally{player.close();}

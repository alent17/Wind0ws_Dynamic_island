import assert from 'node:assert/strict';
import {readFile,writeFile,stat} from 'node:fs/promises';
import {Player} from './player.mjs';
const folder=process.env.ISLE_BENCH_ROOT||'dist/performance/m3-release-c-2026-10-09',player=await Player.connect();
const pause=ms=>new Promise(done=>setTimeout(done,ms));
try {
 const backup=`${folder}/queue-backup.json`;
 if(!(await stat(backup).catch(()=>null))){
  const queue=await player.evaluate('return tool.getStore().playingList.curPlayingList;');
  await writeFile(backup,JSON.stringify(queue));
 }
 const original=JSON.parse(await readFile(backup,'utf8'));
 const corpus=JSON.parse(await readFile(`${folder}/corpus.json`,'utf8'));
 const ids=process.argv[2]==='restore'?original.map(x=>x.resourceId):['1309915258',...corpus.map(x=>x.id)];
 const list=ids.map((id,index)=>{const item=original.find(x=>x.resourceId===id);assert(item,`Missing queue item ${id}`);return restoreEntry(item,index);});
 function restoreEntry(item,index){return process.argv[2]==='restore'?item:{...item,displayOrder:index,randomOrder:index};}
 await player.evaluate(`await tool.getDispatch()({type:'playingList/replaceCurPlayingList',payload:{list:${JSON.stringify(list)}}});return true;`);
 for(let i=0;i<60;i++){const current=await player.evaluate('return tool.getStore().playingList.curPlayingList.map(x=>x.resourceId);');if(JSON.stringify(current)===JSON.stringify(ids))break;if(i===59)throw new Error('Queue readback mismatch');await pause(250);}
 const restore=process.argv[2]==='restore',old=JSON.parse(await readFile(`${folder}/player-before.json`,'utf8'));
 const mode=restore?old.mode:'playOrder';
 await player.evaluate(`await tool.getDispatch()({type:'playing/switchPlayingMode',payload:{playingMode:${JSON.stringify(mode)},triggerScene:'miniPlayer',HeartBeatFlage:false}});return true;`);
 // Selecting the same warmup ID again can retain its near-end position.
 // A different fixed corpus item first makes the warmup start deterministic.
 if(!restore){await player.play(corpus[0].id);await pause(250);}
 await player.play(restore?old.id:'1309915258');
 for(let i=0;i<60;i++){const value=await player.state();if(value.id===(restore?old.id:'1309915258')&&value.mode===mode)break;if(i===59)throw new Error('Starting song/mode mismatch');await pause(250);}
 console.log(restore?'Restored original queue and playback mode':'Prepared exact 101-song sequential queue; disconnecting player CDP before formal measurements');
}finally{player.close();}

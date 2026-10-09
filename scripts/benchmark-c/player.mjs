import {readFile} from 'node:fs/promises';
import {resolve} from 'node:path';

export class Player {
  constructor(socket,capture){this.socket=socket;this.capture=capture;this.sequence=0;this.pending=new Map();socket.onmessage=e=>{const value=JSON.parse(e.data),wait=this.pending.get(value.id);if(wait){this.pending.delete(value.id);clearTimeout(wait.timer);if(value.error)wait.reject(new Error(JSON.stringify(value.error)));else wait.resolve(value.result);}};}
  static async connect() {
    const text=await readFile(resolve('src-tauri/src/services/netease_cdp.rs'),'utf8');
    const capture=text.match(/const CLOUD_MUSIC_CAPTURE_TOOL: &str = r#"([\s\S]*?)"#;/)[1];
    const targets=await(await fetch('http://127.0.0.1:9223/json')).json();
    const target=targets.find(t=>t.type==='page'&&/app\.html|orpheus/.test(t.url));
    if(!target?.webSocketDebuggerUrl?.startsWith('ws://127.0.0.1:9223/'))throw new Error('No local NetEase page target');
    const socket=new WebSocket(target.webSocketDebuggerUrl);await new Promise((done,reject)=>{socket.onopen=done;socket.onerror=reject;});
    return new Player(socket,capture);
  }
  async evaluate(body) {
    const id=++this.sequence;
    const reply=new Promise((resolve,reject)=>{const timer=setTimeout(()=>{this.pending.delete(id);reject(new Error('NetEase command timeout'));},30000);this.pending.set(id,{resolve,reject,timer});});
    this.socket.send(JSON.stringify({id,method:'Runtime.evaluate',params:{expression:`(async()=>{const tool=(${this.capture})();${body}})()`,returnByValue:true,awaitPromise:true}}));
    const result=await reply;if(result.exceptionDetails)throw new Error(JSON.stringify(result.exceptionDetails));return result.result.value;
  }
  state(){return this.evaluate('const p=tool.getStore().playing;return {id:String(p.resourceTrackId),title:p.resourceName,mode:p.playingMode,state:p.playingState};');}
  play(id){return this.evaluate(`await tool.getDispatch()({type:'playing/playOneTrackInPlayingList',payload:{item:${JSON.stringify(String(id))},triggerScene:'playingList',switchType:'call',flag:0}});const p=tool.getStore().playing;return {id:String(p.resourceTrackId),title:p.resourceName,state:p.playingState};`);}
  pause(){return this.evaluate("await tool.getDispatch()({type:'playing/pause',payload:{triggerScene:'playingList'}});return true;");}
  close(){this.socket.close();}
}

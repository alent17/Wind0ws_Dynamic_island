import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {createInterface} from 'node:readline';
import {resolve} from 'node:path';
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
async function until(read,accept,label,milliseconds){const end=Date.now()+milliseconds;do{const value=await read();if(accept(value))return value;await sleep(10);}while(Date.now()<end);throw Error(label+' timeout');}
export class Helper {
  constructor(output, trace, command='pwsh.exe', args=['-NoProfile','-STA','-File',resolve('scripts/a5-static-input.ps1'),'-EnableNativeInput','-OutputDirectory',output]) {
    this.pending = new Map(); this.frames = []; this.nextId = 1;
    this.process = spawn(command,args, { windowsHide: true, stdio: ['pipe','pipe','pipe'] });
    this.process.stderr.on('data', x => trace.push({ source: 'helper-stderr', text: String(x) }));
    this.exitAcknowledged=new Promise(resolve=>this.process.once('close',(code,signal)=>resolve({code,signal,stdioClosed:true})));
    this.process.on('exit', (code, signal) => { this.exited = true; trace.push({ source:'helper-exit', code, signal }); });
    this.process.on('error', error => { this.exited=true; trace.push({source:'helper-process-error',error:String(error)}); });
    this.process.stdin.on('error', error => { this.transportError=error; trace.push({source:'helper-stdin-error',error:String(error)}); });
    createInterface({ input: this.process.stdout }).on('line', text => {
      trace.push({source:'helper-raw-line',text}); // preserve original Int64 tokens
      let value; try { value = JSON.parse(text); } catch { trace.push({ source:'helper-text', text }); return; }
      trace.push({ source:'helper', ...value });
      if (value.id && this.pending.has(value.id)) { const done = this.pending.get(value.id); this.pending.delete(value.id); done(value); }
      else this.frames.push(value);
    });
  }
  async next(predicate) { return until(() => this.frames.find(predicate), Boolean, 'helper lifecycle packet', 20000); }
  async request(action, args = {}) {
    assert(!this.exited, 'helper exited; no restart/kill');
    assert(!this.transportError, 'helper transport failed; ownership unconfirmed, do not replay');
    const id = this.nextId++;
    return new Promise((resolveResult, reject) => {
      const timer = setTimeout(() => {this.pending.delete(id);reject(Error(`helper ${action} timeout; ownership unknown, do not replay`));}, 20000);
      this.pending.set(id, value => { clearTimeout(timer); resolveResult(value); });
      this.process.stdin.write(JSON.stringify({ id, action, ...args }) + '\n', error => {
        if(error){clearTimeout(timer);this.pending.delete(id);reject(error);}
      });
    });
  }
}

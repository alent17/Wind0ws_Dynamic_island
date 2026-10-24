import { readFile, writeFile } from 'node:fs/promises';
import { resourceGate } from './a5-resource-gate.mjs';
const [resource, matrix, status, output] = process.argv.slice(2);
const load=async p=>JSON.parse((await readFile(p,'utf8')).replace(/^\uFEFF/,''));
const gate=resourceGate(await load(resource),await load(matrix),await load(status));
await writeFile(output,JSON.stringify(gate,null,2));
console.log(JSON.stringify({accepted:gate.accepted,errors:gate.errors,inside:gate.completeInsideSamples}));
if(!gate.accepted) process.exitCode=1;

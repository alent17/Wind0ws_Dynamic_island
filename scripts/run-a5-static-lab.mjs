import { dryPlan, SCOPE, BINARIES } from './a5-static-coordinator.mjs';
// No process, GUI, PInvoke, network or Playwright imports on the default path.
if(process.argv.includes('--cleanup-only')) {
  if(process.argv.includes('--live'))throw Error('Cleanup cannot enter live/test mode');
  const path=process.argv[process.argv.indexOf('--cleanup-only')+1];if(!path)throw Error('Cleanup control record required');
  const {requestCleanupOnly}=await import('./a5-static-cleanup-control.mjs');
  const result=await requestCleanupOnly(path);console.log(JSON.stringify(result,null,2));if(!result.ok)process.exitCode=1;
}
else if (!process.argv.includes('--live')) console.log(JSON.stringify(dryPlan(), null, 2));
else {
  const { readFile } = await import('node:fs/promises');
  const arg = name => process.argv[process.argv.indexOf(name) + 1];
  if (!process.argv.includes('--approval') || !process.argv.includes('--output')) throw Error('Refused before live imports: explicit future human approval record and fresh output required');
  const approval = JSON.parse(await readFile(arg('--approval'), 'utf8'));
  if (approval.approved !== true || approval.scope !== SCOPE || approval.maximumDowns !== 4
      || approval.exeSha256 !== BINARIES.exe || approval.dllSha256 !== BINARIES.dll || !approval.humanMessageReference)
    throw Error('Refused before live imports: approval scope/binaries/human reference missing');
  // The record is audit bookkeeping, never a substitute for actual user consent.
  if(process.argv.includes('--session-worker')) {
    const { runLive } = await import('./a5-static-live.mjs');
    await runLive(arg('--output'), approval);
  } else {
    const {superviseCoordinator}=await import('./a5-static-session-supervisor.mjs');
    const exit=await superviseCoordinator([...process.argv.slice(1),'--session-worker'],arg('--output'));
    if(!exit.normal)process.exitCode=1;
  }
}

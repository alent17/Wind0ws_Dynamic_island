import { readFile, writeFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';

export function checkMemoryBudget(report, limitBytes = 100_000_000) {
  if (!Number.isFinite(limitBytes) || limitBytes <= 0) throw new Error('Invalid memory budget');
  if (report.control || !Number.isInteger(report.rootPid) || report.rootPid <= 0) throw new Error('An explicit application RootPid is required');
  const samples = report.samples;
  if (!Array.isArray(samples) || samples.length < 2) throw new Error('At least two process-tree samples are required');
  for (const sample of samples) {
    if (!sample.processes?.some(process => process.pid === report.rootPid)) throw new Error('Root process missing from sample');
    for (const role of ['browser', 'renderer']) {
      if (!sample.processes.some(process => process.role === role)) throw new Error(`WebView2 ${role} missing; application UI is not ready for memory acceptance`);
    }
    if (sample.processes.some(process => !Number.isFinite(process.privateWorkingSetBytes) || process.privateWorkingSetBytes < 0 || !Number.isFinite(process.privateBytes) || process.privateBytes < 0)) throw new Error('Memory counters unavailable');
  }
  const stats = key => {
    const values = samples.map(sample => sample.processes.reduce((sum, process) => sum + process[key], 0)).sort((a,b) => a-b);
    return {meanBytes: values.reduce((a,b) => a+b, 0)/values.length, p95Bytes: values[Math.ceil(values.length*.95)-1], peakBytes: values.at(-1)};
  };
  const privateWorkingSet = stats('privateWorkingSetBytes');
  const privateCommit = stats('privateBytes');
  return {
    valid:true, rootPid:report.rootPid, sampleCount:samples.length, limitBytes,
    scope:'Application and all sampled descendants; decimal MB. Passing this scene does not qualify untested scenes.',
    privateWorkingSet, privateCommit,
    passed:privateWorkingSet.peakBytes < limitBytes,
    commitPassed:privateCommit.peakBytes < limitBytes,
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const input = process.argv[2];
    if (!input) throw new Error('Usage: node scripts/check-memory-budget.mjs <measure-native.json> [output.json]');
    const summary = checkMemoryBudget(JSON.parse((await readFile(input,'utf8')).replace(/^\uFEFF/,'')));
    if (process.argv[3]) await writeFile(process.argv[3],JSON.stringify(summary,null,2)+'\n');
    console.log(JSON.stringify(summary,null,2));
    if (!summary.passed) process.exitCode = 1;
  } catch (error) {
    // Never leave an older passing result at the requested output path.
    if (process.argv[3]) await writeFile(process.argv[3], JSON.stringify({valid:false,passed:false,error:error.message},null,2)+'\n');
    console.error(error.message); process.exitCode = 2;
  }
}

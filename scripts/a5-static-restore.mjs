import assert from 'node:assert/strict';

// Used by the real backend and fault tests. Studio's deferred persist must end
// before settings are restored, and disk rollback must follow process exit.
export const RESTORE_ORDER = Object.freeze(['closeLabWithForeground','closeStudioAndConfirmPersistedExit',
  'restoreSettingsAndSourceVisibility','closeOwnedAppAndConfirmExit','restorePersistentBytesAndStartup',
  'stopPlateAndRestoreNativeReadback']);
export async function restoreStaticSession(backend, report={ok:false,steps:[]}) {
  if(report.ok)return report;
  report.pending=false;
  delete report.error;delete report.failedStep;
  for(const name of RESTORE_ORDER) {
    if(report.steps.some(step=>step.name===name&&step.ok===true))continue;
    try {
      const proof=await backend[name]();
      assert(proof?.ok===true,`${name} did not confirm restoration`);
      report.steps.push({name,...proof});
    } catch(error) {
      report.pending=true;report.failedStep=name;report.error=String(error);return report;
    }
  }
  report.ok=true;return report;
}

// The live coordinator uses this same closure for initial cleanup and its
// cleanup-only endpoint. A resume cannot reach preparation/trial/down methods.
export function createCleanupResumer({backend,gate,persist,report={ok:false,steps:[]}}) {
  let running=false;
  return {report,async resume(release) {
    assert(!running,'cleanup already running');running=true;
    try {
      assert(release?.ok===true&&release.held===false,'release unknown/failed; no retry injection or cleanup');
      await gate(report);
      await restoreStaticSession(backend,report);
      await persist(report);return report;
    } catch(error) {
      report.ok=false;report.pending=true;report.error=String(error);
      try{await persist(report);}catch(failure){report.persistenceError=String(failure);}
      return report;
    } finally {running=false;}
  }};
}

import {recipientVerdict} from './a5-native-plan.mjs';
export async function executeDwellTrial(trial,io){
 const record={style:trial.style,point:trial.point,prescribedDwellMs:trial.prescribedDwellMs,status:'pending',actions:[]};
 const call=async(action,args={})=>{const r=await io.native(action,args);record.actions.push({action,...r});if(!r.ok){const error=new Error(JSON.stringify(r));error.releaseFailed=r.releaseOnError?.ok===false;throw error;}return r;};
 try{
  await io.configure(trial.style);
  await call('move',{x:-1870,y:500});await io.wait(300);
  const base=await call('pump'),candidate=await io.dom(trial.point);record.frozenCandidate=candidate;
  if(!candidate.valid){record.status='invalid-precondition';return record;}
  const plateBefore=base.bottomPlateReceipts.length,pageBefore=await io.pageClicks();
  const move=await call('move',trial.point);await io.wait(trial.prescribedDwellMs);
  const before=await io.dom(trial.point);record.beforeDownDom=before;
  if(!before.valid){record.status='invalid-precondition';return record;}
  const pre=await call('state');record.beforeDownNative=pre;
  if(pre.after.cursor.x!==trial.point.x||pre.after.cursor.y!==trial.point.y){record.status='invalid-cursor-drift';return record;}
  const down=await call('press-down');record.downInjected=true;
  const up=await call('press-up');record.upReleased=true;
  record.actualMoveEndToDownMs=(down.inputStartedQpc-move.inputEndedQpc)*1000/down.before.qpcFrequency;
  record.actualDownToUpMs=(up.inputStartedQpc-down.inputStartedQpc)*1000/down.before.qpcFrequency;
  await call('state');await call('pump');await io.wait(40);const late=await call('state');
  const events=await io.pageClicks();record.recipients=recipientVerdict({plateDelta:late.bottomPlateReceipts.length-plateBefore,pageClicks:events.length-pageBefore.length,latePointPid:late.after.pointPid});
  record.pageReceipts=events.slice(pageBefore.length);record.bottomReceipts=late.bottomPlateReceipts.slice(plateBefore);record.finalDom=await io.dom(trial.point);record.status=record.recipients.conflict?'conflict':record.recipients.uniqueRecipient?'observed':'unobserved';
 }catch(error){record.status=error.releaseFailed?'release-failed':'tool-error';record.error=String(error);}
 return record;
}

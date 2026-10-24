import assert from 'node:assert/strict';
export function trial(style, point, dwellMs) {
  assert([0,50,250].includes(dwellMs));
  assert(Number.isInteger(point.x)&&Number.isInteger(point.y));
  return {style,point:{...point},prescribedDwellMs:dwellMs,executed:false,
    sequence:[{action:'outside-reset',explicitDwellMs:300},{action:'state',label:'outside'},
      {action:'freeze-dom-point',reselectAllowed:false},{action:'move',...point},
      {action:'wait',ms:dwellMs},{action:'dom-and-native-state',label:'before-down'},
      {action:'press-down'},{action:'press-up'},{action:'state',label:'immediate-after'},
      {action:'pump'},{action:'wait',ms:40},{action:'state',label:'late-after'},
      {action:'collect-recipients',sources:['bottom-plate','trusted-page-events']}],
    invalidation:'Record cursor/rect/DPR/DOM target drift or missing precondition; do not change point, retry or omit failed input. Actual move→press dwell includes IPC/query costs, measure QPC instead of calling prescribed zero physically instantaneous.'};
}
export function plan(){return {mode:'dry-run',guiCreated:false,inputInjected:false,liveExecutionImplemented:false,
  coordinateStatus:'Proposed from saved snapshots; freeze once after later authorized readiness; do not silently reselect at press',
  trials:['edge-top','floating'].flatMap(style=>[0,50,250].map(d=>trial(style,{x:-960,y:style==='floating'?26:15},d))),
  borderReference:[trial('edge-top-original-border',{x:-960,y:2},0),trial('edge-top-original-border',{x:-960,y:2},250)],
  nativeRegion:'N/A: no product getter',runtimeMetadata:'N/A: not recorded; no retry of rejected launch',
  focus:{targets:['button','DIV[role=slider]','input','select','a[tabindex]'],read:['activeElement tag/role/label/tabIndex','focus-visible','outline','box-shadow','border','glyph styles','aria-valuenow','native foreground'],keys:['Tab','ArrowRight','ArrowLeft'],restore:'Actual original volume and foreground. Arrow keys only on existing volume slider; no End/Home 100% jump.'}};}
export function captureFocusedTarget(doc,readStyle){const e=doc.activeElement;if(!e)return null;const style=readStyle(e),r=e.getBoundingClientRect();return {tag:e.tagName,role:e.getAttribute('role'),label:e.getAttribute('aria-label'),tabIndex:e.tabIndex,focusVisible:e.matches(':focus-visible'),outline:style.outline,boxShadow:style.boxShadow,border:style.border,ariaValue:e.getAttribute('aria-valuenow'),rect:{x:r.x,y:r.y,width:r.width,height:r.height},glyphOutline:e.querySelector('.function-glyph')?readStyle(e.querySelector('.function-glyph')).outline:null};}
export function focusVerdict(raw){const styled=[raw.outline,raw.boxShadow,raw.glyphOutline].some(v=>v&&v!=='none'&&!/\bnone\b/.test(v));return {recognized:raw.tag==='BUTTON'||raw.role==='slider'||['INPUT','SELECT','A'].includes(raw.tag),focusVisible:raw.focusVisible??null,outline:raw.outline??null,stylingObserved:styled,accepted:raw.focusVisible===true&&styled&&raw.visualConfirmed===true,scope:'Captured active target, not button-only; native visual confirmation still required'};}
export function recipientVerdict({plateDelta,pageClicks,latePointPid}){assert(Number.isInteger(plateDelta)&&plateDelta>=0);assert(Number.isInteger(pageClicks)&&pageClicks>=0);const conflict=plateDelta>0&&pageClicks>0;return {plateDelta,pageClicks,deliveredTo:conflict?'ambiguous/conflict':plateDelta>0?'bottom-plate':pageClicks>0?'page':'unobserved',uniqueRecipient:!conflict&&(plateDelta>0||pageClicks>0),conflict,latePointPid,lateQueryOverridesRecipient:false};}

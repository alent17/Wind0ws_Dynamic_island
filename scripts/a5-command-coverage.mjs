// All operands use the same page performance clock. Node/CDP timestamps must
// not be substituted: their round-trip envelope can fabricate overlap.
export function summarizeCommandCoverage(transitions, commands) {
  const rows = transitions.map(transition => {
    const start = transition.firstStyleMutation?.performanceMs;
    const end = transition.lastStyleMutation?.performanceMs;
    const valid = Number.isFinite(start) && Number.isFinite(end) && end > start;
    const own = commands.filter(command => command.transitionIndex === transition.index);
    const overlapping = own.filter(command => valid && command.ok === true &&
      Number.isFinite(command.pageStartedPerformanceMs) && Number.isFinite(command.pageEndedPerformanceMs) &&
      command.pageEndedPerformanceMs > command.pageStartedPerformanceMs &&
      command.pageStartedPerformanceMs < end && command.pageEndedPerformanceMs > start);
    const media = overlapping.filter(command => command.command === 'control_media').length;
    const volume = overlapping.filter(command => command.command === 'set_system_volume').length;
    return { index: transition.index, valid, media, volume, covered: media > 0 && volume > 0 };
  });
  return { definition: 'Successful media AND volume bridge requests from each transition intersect its first-to-last style mutation span, using page performance.now. This does not prove native worker execution throughout the animation.',
    transitionCount: rows.length, coveredTransitions: rows.filter(row => row.covered).map(row => row.index),
    missingTransitions: rows.filter(row => !row.covered).map(row => row.index), rows,
    accepted: rows.length === 16 && rows.every((row, index) => row.index === index + 1 && row.covered) };
}

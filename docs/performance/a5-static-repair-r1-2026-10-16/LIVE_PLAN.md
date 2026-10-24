# A5 static repair R1: live execution plan (Task 13)

**State: OFFLINE READY — NOT APPROVED.** This document prepares one future controlled GUI session. It does not authorize or perform that session. The prior live permission was consumed by Batch 11; a new direct human approval must be recorded before any live run.

## Frozen candidate and scope

Use only the R1 `experimental-release` pair:

| Artifact | SHA-256 |
| --- | --- |
| `isle.exe` | `38102A0F66B93A4370155CCFD6F420BC8A3831646216B7076C67DB493F506E7A` |
| `isle_lib.dll` | `CBD7290142D7BA3140814F74387EF0D4F7AA356A1BB22D2764526EE1264B737D` |

The approval scope must be exactly `a5-static-lab-four-v3-candidate`, with `maximumDowns: 4`, both artifact hashes above, matching reviewed source hashes, and a new human-message reference. All 268 source files and the four packaged R1 binaries must still match the R1 manifests immediately before execution. Do not rebuild, patch, or substitute files.

The future approval file is `docs/performance/a5-static-repair-r1-2026-10-16/APPROVAL_TASK13.json`. It does not exist yet and must not be created with `approved: true` during offline preparation. The prior `PENDING_APPROVAL.json` remains false and unchanged.

## Preconditions and launch boundary

At the future run, verify no existing Isle instance, idle input/capture, and free localhost CDP port 9250 before app launch. After the single app launch, verify two monitors with the fixed secondary target at `monitorIndex: 1`, its non-primary status, and the newly created Studio foreground. Any failed or unknown prerequisite ends the attempt at that stage. Do not use Alt, focus-stealing, a browser launch, or another fallback.

Use the outer Node CLI only. It validates approval and starts the helper; the helper captures its initial state and private backups before the single permitted app launch attempt. Bind the candidate to the actual main module path, process ID, process start ticks, initial `get_settings` auto-start value, and pinned policy source hashes. Denial or identity mismatch stops the run. Do not launch or invoke the helper, session worker, EXE, or DLL directly. At most one app launch attempt is allowed, including after a zero-click startup failure.

The helper must install the Studio IPC fetch audit before creating a fresh Studio document. Record real `list_media_sessions`, `get_settings`, and `update_settings` request start/end/results. Apply only the prescribed player-order retention merge and `monitorIndex: 1`. Missing, overlapping, unknown, or incomplete IPC coverage, reload, or fallback makes the run pending/failed; do not repeat the launch.

The main window must be placed on the secondary display and hidden as prescribed. Studio remains at primary work-area origin plus 32 pixels with its original size, which must fit. Use only the approved no-activate plate and immutable lab document. Require the applied/native-ready receipt plus matching document identity and actual frame, DPI, style, Canvas backing, alpha/RGB, DPR, and viewport evidence. `nativeReady` and two animation frames alone do not prove a displayed paint.

## Fixed input sequence and stop rules

Before any cursor move or button-down, freeze the five-by-five opaque and transparent target coordinates and the start point at `(right + 32, bottom + 32)`; all must satisfy the documented secondary-work-area and margin rules. Reuse these coordinates throughout. No reselection, alternate point, or retry is permitted.

Run no more than four down/up trials in this exact order: inside, outside, inside, outside. Perform only the two prescribed moves for each trial, with no intentional dwell. Measure the actual QPC pipeline intervals. Before each input, verify the real Studio foreground, idle/button state, frame, DPI, styles, source hidden state, and root identity. Do not insert a status request between target move and mouse-up. Capture raw down/up/click tags, timestamps, and identities on both sides. Any missing, dual, split, duplicate, wrong-side, reordered, trusted, or coordinate-activation result stops all remaining clicks. After mouse-up, allow only the prescribed 250 ms observational drain; never retry to force a pass.

On failure, stop further clicks and do not add hover, DPI, drag, performance, or media tests. Release only a successfully owned held-down input through the allowed gate. A normal mouse-up does not justify an extra release. A failed release is sticky and must not be retried. If release, idle state, or ownership is unknown, retain the transport/process/evidence for review; do not mutate foreground, cursor, or windows and do not hard-kill.

## Cleanup and outcome proof

Cleanup must use the same session and ownership proofs: close the lab and confirm native disappearance; close Studio normally and confirm document destruction; restore only audited semantic settings and source visibility with immediate readbacks; send `WM_CLOSE` only to owned windows of the same instance and prove child exit plus CDP disconnect; restore the exact original config bytes and each startup value with per-value readbacks/acknowledgements; close the plate and restore original foreground/cursor only with fresh PID/TID/start-tick identity and age no greater than 30 minutes; then close the helper child normally. The outer supervisor records `coordinator-exit.json` from actual close evidence, not numeric PID disappearance.

Exit code zero or `normal: true` alone is not a pass: a precondition failure can still complete cleanup normally. A successful experiment requires `evidence.json` field `result.complete: true`, all four valid receipts, all required identity/input/native evidence, and successful restoration proofs. A failed zero-click attempt is reported as such. Preserve explicit pending/busy outcomes. If cleanup exceeds its execution budget or the pipe breaks, do not launch a duplicate; inspect durable evidence first. Do not use stale identities or stale foreground/cursor state for recovery.

Public delivery may include raw input evidence, receipts/status/document/native traces, frozen plan, screenshot if reached (otherwise mark N/A), cleanup readbacks, helper/coordinator exit proof, exact hashes, source/tool manifests, and offline analysis. Exclude private original settings/Run values, full Studio IPC audit, and cleanup token.

## Future command (documentation only)

Run this only after the publisher records fresh direct human approval in the approval file and confirms all prerequisites. This command has **not** been run during Task 13:

```powershell
Set-Location -LiteralPath '.'
node .\scripts\run-a5-static-lab.mjs --live --approval .\docs\performance\a5-static-repair-r1-2026-10-16\APPROVAL_TASK13.json --output .\dist\performance\a5-static-lab-v3-2026-10-16\run-01
```

The output path is reserved for the future live attempt and is absent at offline-readiness time. Do not use `--session-worker`, invoke the helper directly, or automatically retry.

If and only if the same live session has a reviewed, safe cleanup continuation, the documented cleanup-only entry point is:

```powershell
node .\scripts\run-a5-static-lab.mjs --cleanup-only .\dist\performance\a5-static-lab-v3-2026-10-16\run-01\cleanup-control.json
```

Use it only for the same proven session and closure. Do not resume after a known failed release, cold/reused HWND, pending/timeout, or unknown outcome until durable state is reviewed. Cleanup-only is not a second experiment or a fresh launch.

## Acceptance boundary

This plan prepares one static four-receipt GUI probe. It does not prove the original `Enqueued` cause, real GUI correctness, C1, or A5 performance acceptance. C1 and A5 remain unchecked; Tween 420/360/240, media behavior, C2/I1/UI, Spring, and other phases remain unchanged. Do not commit or push as part of this readiness batch.

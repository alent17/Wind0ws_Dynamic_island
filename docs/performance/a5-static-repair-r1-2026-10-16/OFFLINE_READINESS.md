# Task 13 offline readiness

**Status: ready for publisher review; live execution is not approved.**

The Task 13 text was copied into this directory. The matching R1 live plan is [LIVE_PLAN.md](LIVE_PLAN.md). No GUI session, app/helper/browser, player, input injection, or live environment inspection was performed.

Offline checks on 2026-10-16 found:

- All 268 source files in the accepted R1 manifest match the current workspace; all four R1 packaged binaries match their recorded hashes.
- The selected experimental EXE/DLL hashes are recorded in the live plan. The existing `PENDING_APPROVAL.json` remains `approved: false` and byte-identical to the previously reviewed copy.
- The ordinary CLI default dry-run exited 0 and produced a plan. It was invoked without `--live`, `--cleanup-only`, or `--session-worker`.
- The reserved live output directory `dist/performance/a5-static-lab-v3-2026-10-16/run-01` does not exist. Only `dist/performance/a5-static-lab-v3-2026-10-16/offline-readiness-01` contains this batch's dry-run and readiness evidence.
- No rebuild, business-code or lockfile edit, commit, or push was made. Repository HEAD remains `48530f7343e84129aef34113dbef4dd18f7431ef`; unrelated pre-existing worktree changes were preserved.

The prior live permission was consumed by Batch 11. This offline preparation does not grant a new live permission. The publisher must obtain a new direct human approval and create the separate Task 13 approval record before any live execution.

# M6 Settings Commit Implementation and Acceptance Status

Baseline: `2d915d5` (M5). M6 passed the settings correctness acceptance on 2026-10-12. The Release timing comparison showed a small absolute save-latency increase, recorded below.

## Implemented

- Full settings saves, partial patches, compatibility setters, startup state, floating-window position, and monitor selection now enter one serialized backend transaction. Patches merge into the latest state while holding the transaction owner, then normalize and validate before any publish.
- The transaction writes a sibling temporary file, flushes it, retains the previous valid snapshot as `settings.json.bak`, then replaces the primary file. Startup falls back to the backup if the primary is missing, malformed, or cannot deserialize.
- Memory state, revision, and events update only after persistence succeeds. Topmost and startup operations run before persistence; failures are returned and attempted side effects are compensated. Successful commits emit the legacy settings snapshot plus a monotonic `settings-committed` revision event.
- Studio no longer drops failed patches. It displays a localized retry action, keeps failed changes pending, and does not close the window when the final save fails.
- Tests were added for 100 concurrent setting/position commits, memory/disk/revision agreement, disk-write failure compensation, external-operation failure before persistence, backup recovery, and equal-payload direct-write versus atomic-write timing.

## Verification

- The failing unit-test EXE imported `comctl32!TaskDialogIndirect` but had no RT_MANIFEST resource. The exact M5 baseline test EXE also had no manifest and failed to launch with `0xc0000139` on this Windows 11 Insider build. This confirms a pre-existing test-harness build issue rather than an M6-only regression.
- `src-tauri/build.rs` now links Tauri's generated Windows resource library into the package test harness. The rebuilt EXE embeds the `Microsoft.Windows.Common-Controls` 6.0.0.0 dependency and starts successfully (`--list` exits 0).
- Windows tests exposed that `sync_all` on a read-only backup handle returns `ERROR_ACCESS_DENIED`; the backup is now reopened read/write before flushing.
- `cargo check --manifest-path src-tauri/Cargo.toml --tests`: passed before and after the test-harness fix.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib`: **61 passed, 0 failed, 1 ignored**. This includes the 100 concurrent settings/position writes, disk-failure compensation, memory/disk/revision agreement, backup recovery, and write-latency comparison. The first full run hit a transient existing HTTP-budget test failure; that test passed alone and the subsequent full run passed.
- Vitest: 95 tests passed across 20 files.
- `svelte-check`: 0 errors and 0 warnings.
- Vite production build: passed (3,868 modules transformed).
- Release-mode equal-payload write benchmark (100 writes each, same settings payload): direct write **61.874 ms**, atomic write with backup **761.578 ms** total (about **7.62 ms/write**, 12.3× the direct-write control). This is a small local-filesystem synthetic result; it is not a Studio interaction latency measurement.
- Native Release acceptance ran under the isolated identifier `com.isle-app.isle-m6-test`; the regular app identifier and settings were not used. Both M5 and M6 Studio windows were placed on `DISPLAY2` at physical position `(-1800, 100)` on the 1920×1080 secondary display at 100% scale.
- Failure injection made the isolated `settings.json` path a directory. Studio showed the save error, retained the pending appearance change, and `get_preferences` still returned the previous value. Requesting window close kept Studio open with the error visible. After restoring file access, Retry persisted the pending change and cleared the error.
- Restart recovery was tested by replacing the isolated primary JSON with invalid content while keeping a valid `.bak`; after restart, settings loaded from the backup. The test data was restored afterward.
- Matched Release Studio timing used the same setting, click-to-Apply path, backend-state confirmation, display, app identifier, and starting settings for four saves per build. M5 samples: **35, 31, 23, 22 ms** (median **27 ms**); M6 samples: **37, 41, 38, 41 ms** (median **39.5 ms**). M6 adds **12.5 ms** median, about **46%** relative to the small M5 number; the absolute median remains under 40 ms. One exploratory 61 ms sample used a different completion wait and is excluded. These are short local Release UI samples, not a broad latency distribution.
- `git diff --check`: passed.

## Acceptance conclusion

M6 correctness and native Studio acceptance passed, so M6 is marked `[x]` in `docs/OPTIMIZATION_PLAN.md`. The matched Release comparison shows a **12.5 ms** median save-latency increase; no save-latency budget was specified, and the absolute median remained below 40 ms. Keep this regression visible in future comparisons. The synthetic write benchmark remains separate from user-visible Studio latency. No conclusion is drawn about the unresolved ~25 MiB private-commit growth, GPU recovery, or the sixth-track timeout.

# C2 media probe: one passive run

Date: 2026-10-20 (Asia/Shanghai)

## Build

From `src-tauri`, using Rustup Cargo/rustc 1.94.1 and the existing isolated target directory:

```powershell
$env:RUSTC='<local-user-path>'
$env:CARGO_TARGET_DIR='.\src-tauri\target\c2-offline-2026-10-17'
& '<local-user-path>' build --offline --example c2-media-probe --features c2-media-probe
```

Exit code: `0` (`Finished dev profile`, 2m11s). No Release build was run.

- Worktree HEAD: `48530f7343e84129aef34113dbef4dd18f7431ef` (working tree contains pre-existing local changes).
- Probe EXE: `src-tauri/target/c2-offline-2026-10-17/debug/examples/c2-media-probe.exe`, 28,852,736 bytes, SHA-256 `6F5752C0AD298509817B8B9A44D2F44B4D46E819F3A7C9E2119480BE0C8ABBA0`.
- Source/input SHA-256: `Cargo.toml` `2ED9E40CFC140B4026EC91BFD011BD70121C42E2CF386B2D3804B67D890A6C9B`; `src/lib.rs` `F6F9CC5C645AE10FC2F9174C9C5B0FA2605AC8F614DD669B634B70D14486C2B4`; `examples/c2_media_probe.rs` `B1B4C1F0F5B58C13BB1E7B1071C6F4FEF2F335E365727B83F506D20D33AA9157`; `Cargo.lock` `C19A6B9079E51BF5475BC9DB55E81475988162B8AB2C1056B0F00F9F263C7D6F`.
- Product Release EXE/DLL remained unchanged: `6FF00C4C1D2304EDA70551F85A0C356FB98E3812A8FD3862505F69F8CF0B3036` / `2A5D20778EEBC7AC5F3A2F34C2B39F705200913B51F17626945CD8D4C2F51345`.

## Passive run

Launched exactly once from the absolute debug EXE path with hidden window style and redirected output:

```powershell
Start-Process -FilePath '.\src-tauri\target\c2-offline-2026-10-17\debug\examples\c2-media-probe.exe' -WindowStyle Hidden -RedirectStandardOutput '.\docs\performance\c2-media-probe-2026-10-20\host.stdout.txt' -RedirectStandardError '.\docs\performance\c2-media-probe-2026-10-20\host.stderr.txt' -PassThru
```

PID [omitted]; observed process image matched the built EXE. Process wall time `60.25s`; host exit code `0`; no forced termination; no Isle/probe process remained afterward. The host's bounded stdout summary reported `elapsed_ms=60001`:

```text
media_probe_summary elapsed_ms=60001 manager_ok=1 manager_error=0 refresh_manager_ready=1 refresh_event=0 refresh_fallback=29 snapshot_ok=30 snapshot_empty=0 snapshot_error=0 subscriptions_added=5 subscriptions_removed=5 subscription_remove_error=0 snapshot_mean_us=3235 snapshot_max_us=19042
```

Raw bounded output: [stdout](host.stdout.txt) (293 bytes); [stderr](host.stderr.txt) (empty, 0 bytes).

Only passive observation was performed. No players or existing Isle processes were started, controlled, or exited; no GUI or injected SMTC/input state was used. No media strings, app IDs, payloads, or registry values were logged.

## User-state before/after check

Read-only snapshots immediately before and after the run were compared in memory. `%APPDATA%\com.isle-app.isle\settings.json` existed both times with SHA-256 `3EDDEC967D655D920A2DE34B722E727FA91F4787436DBFDD45F4454939B59C0D`; `settings.json.bak` existed both times with SHA-256 `0B3192D5FD17C94B1CA0DAEE0697EE791772629C2071C51E42B67FF2C67633F0`. Both file hashes were unchanged.

At `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, `Isle` was present and `Wind0wsDynamicIsland` absent before and after. Raw values were compared in memory and unchanged; values were not printed or stored.

## Result and limits

One manager acquisition succeeded; 30 non-empty snapshots succeeded; there were no empty snapshots or snapshot errors. Five event tokens were added and five removal calls succeeded, with zero removal errors. The worker's stop-and-join path returned normally, and the process exited 0. This is evidence of token cleanup on this normal run only.

`refresh_event=0`, so no event-driven refresh was observed; 29 fallback refreshes occurred. No empty-session or idle behavior, one-time empty broadcast, exact 1/5-second cadence, controlled disconnection/reacquisition, or post-disconnection recovery was measured. There was no WebView, so frontend event receipt/rendering is unverified. The report does not establish CPU/power gains.

The 60-second interval is the observation window, not a hard wall-clock guarantee: synchronous WinRT `.get()` calls are not cancellable. In this run, stop/join completed normally at about 60 seconds. C2 remains open.


## Event coverage run (second passive run)

Preflight confirmed the prepared EXE still matched SHA-256 `6F5752C0AD298509817B8B9A44D2F44B4D46E819F3A7C9E2119480BE0C8ABBA0` and no prior probe process for this image was running. No source change or rebuild was made. The host was launched once, hidden and with output redirected:

```powershell
Start-Process -FilePath '.\src-tauri\target\c2-offline-2026-10-17\debug\examples\c2-media-probe.exe' -WindowStyle Hidden -RedirectStandardOutput '.\docs\performance\c2-media-probe-2026-10-20\host-event.stdout.txt' -RedirectStandardError '.\docs\performance\c2-media-probe-2026-10-20\host-event.stderr.txt' -PassThru
```

Start UTC `2026-10-20T08:55:56.6936731Z`, PID [omitted]. The coordinator was notified immediately after launch to let the user perform any player operation themselves. Wall time was `60.13s`; host exit code `0`, no forced termination, and PID [omitted] was absent after exit. Product Release EXE/DLL hashes remained `6FF00C4C1D2304EDA70551F85A0C356FB98E3812A8FD3862505F69F8CF0B3036` / `2A5D20778EEBC7AC5F3A2F34C2B39F705200913B51F17626945CD8D4C2F51345`.

Raw stdout summary: `media_probe_summary elapsed_ms=60002 manager_ok=1 manager_error=0 refresh_manager_ready=1 refresh_event=0 refresh_fallback=59 snapshot_ok=60 snapshot_empty=0 snapshot_error=0 subscriptions_added=8 subscriptions_removed=8 subscription_remove_error=0 snapshot_mean_us=7220 snapshot_max_us=29021`. Raw stdout is [host-event.stdout.txt](host-event.stdout.txt) (293 bytes); stderr is [host-event.stderr.txt](host-event.stderr.txt) (empty, 0 bytes).

`refresh_event=0`; event-triggered refresh was not observed in this run. The aggregate counters do not identify whether a specific user action occurred or prove individual event types. Sixty non-empty snapshots and 8 successful token removals with zero remove errors were observed; the listener stop/join returned normally. Empty-session/idle behavior, 1/5-second exact cadence, one-time empty broadcast, controlled disconnection/recovery and WebView delivery/rendering remain untested. No player or existing Isle process was controlled. `%APPDATA%\com.isle-app.isle\settings.json` and `.bak` hashes matched the previous snapshot and each remained unchanged; relevant HKCU Run values were compared in memory and unchanged (Isle present, legacy value absent), with no raw registry value logged. No CPU/power sampling; C2 remains open.

## Source audit and bounded diagnostic update (not run)

`refresh_event=0` alone does not identify why no event refresh was consumed. Before this update the five event handlers discarded the result of `SyncSender::try_send`, while `refresh_event` counted only messages consumed by the worker. Thus the earlier runs cannot distinguish no callback entry from queue-full loss, nor establish which event types corresponded to any user action. The coordinator received the user's “已操作” response during PID [omitted]'s window, but the operation type and exact time were not recorded; no details are inferred here.

The event path now has bounded, aggregate-only probe counters for entry to each of the five manager/session callback types, successful refresh enqueue, full queue and disconnected queue. Successful snapshots also count playback boolean changes and changes to a fingerprint of `source_display` (AppID/source identifier). That counter indicates only that the selected source identifier changed; it cannot show a switch between distinct COM session objects that share the same AppID. The fingerprint is held in memory only for equality/counting; neither media fields nor identifiers/fingerprints are written to output or persisted. The single final summary prints the counts. A one-time `media_probe_ready` stdout line is emitted after a successful session enumeration and completion of the current manager/session registration attempts, with elapsed milliseconds and successful token counts. It means enumeration/registration attempts completed, not that subscription readiness was achieved: zero tokens must be treated as no successful registrations. No event payloads are printed, and there is no per-event logging. These diagnostics are activated only by the existing `c2-media-probe` feature and observer; product listener runs have no probe observer. The media-properties token removal count was also corrected to count success only.

Subscription audit: handlers are retained by their manager/session subscription owners and successful registration tokens are removed during normal worker cleanup. In particular, the session-object reconciliation uses `IUnknown` identity rather than only the source/AppID string. `try_send` is non-blocking on the capacity-1 channel; a full channel loses that enqueue attempt, although an already queued refresh can coalesce multiple stimuli. The new counters separate callback entry and send outcome from a later consumed event refresh. They do not establish the semantics of an individual player action.

Host lifecycle audit: the locked local Tauri 2.10.3 source shows `Builder::build` creates the runtime and `App`, while `App::run_return` enters `runtime.run_return` and `Builder::run` calls `build(...).run(...)`. The probe calls `build` and does not run that loop. Its separate media worker calls `RoInitialize(RO_INIT_MULTITHREADED)` through `Apartment::enter`; the local Windows-only apartment tests assert MTA. Microsoft documents MTA as the model for background work without a message loop ([Processes, Threads, and Apartments](https://learn.microsoft.com/en-us/windows/win32/com/processes--threads--and-apartments)). This is evidence that the COM worker itself does not require a UI message loop merely by being MTA; it does not prove whether a specific SMTC provider or WinRT event-delivery path depends on the host's Tauri loop. There is no source evidence establishing that dependency, so the host was not changed and the event-loop explanation remains unresolved.


## Diagnostic host build (built, not run)

Before rebuilding, the previous debug host was copied to [the pre-diagnostics archive](archive/c2-media-probe-pre-diagnostics-6F5752C0AD298509.exe) and both its original path and archive copy matched SHA-256 `6F5752C0AD298509817B8B9A44D2F44B4D46E819F3A7C9E2119480BE0C8ABBA0`. This preserves both earlier run artifacts and does not alter their stdout/stderr.

Using Rust/Cargo 1.94.1, the existing isolated `CARGO_TARGET_DIR`, and offline dependencies, the exact command (working directory `src-tauri`) was:

```powershell
$env:RUSTC='<local-user-path>'
$env:CARGO_TARGET_DIR='.\src-tauri\target\c2-offline-2026-10-17'
& '<local-user-path>' build --offline --example c2-media-probe --features c2-media-probe
```

Exit code was `0` (`Finished dev profile`); the debug host is `src-tauri/target/c2-offline-2026-10-17/debug/examples/c2-media-probe.exe`, 28,863,488 bytes, SHA-256 `6BBD9B28888B1C4FAE59CD79E5B7D76C9E3BACD1C5DC7CEF323189790AD99CB5`. Source/config inputs recorded after the build: `src-tauri/Cargo.toml` `2ED9E40CFC140B4026EC91BFD011BD70121C42E2CF386B2D3804B67D890A6C9B`; `src-tauri/Cargo.lock` `C19A6B9079E51BF5475BC9DB55E81475988162B8AB2C1056B0F00F9F263C7D6F`; `src-tauri/src/lib.rs` `C89A6FAD0A40EBC27A605A441D8D6F2928D47268B1BE4DDD8C15020C757DFB98`; `src-tauri/examples/c2_media_probe.rs` `B1B4C1F0F5B58C13BB1E7B1071C6F4FEF2F335E365727B83F506D20D33AA9157`; `src-tauri/build.rs` `80774FD49F45D456E46CCFB7124C924D334636920920B16E2A4B01F9D953F046`; `src-tauri/tauri.conf.json` `5D98A643C9DFFC6DE0511CDEBA0B7008F9CD3BC42436518146AD2F3BC124636F`. No product Release build was run.

Prepared command for a later, separately authorized observation (not run in this batch):

```powershell
$env:RUSTC='<local-user-path>'
$env:CARGO_TARGET_DIR='.\src-tauri\target\c2-offline-2026-10-17'
& '.\src-tauri\target\c2-offline-2026-10-17\debug\examples\c2-media-probe.exe'
```

The expected one-time line format is `media_probe_ready elapsed_ms=<ms> manager_tokens=<n> session_tokens=<n> total_tokens=<n>`. The coordinator must inspect these successful-token counts; if total tokens are zero, do not describe subscriptions as ready or prompt an operation on the assumption that event handlers were registered. The example's nominal 60-second observation timer starts immediately after the listener task is started, but synchronous WinRT `.get()` calls are not cancellable and can extend wall time. No WebView exists, so frontend event delivery/rendering remains outside this probe. This diagnostic host is built and has not been run; the next observation must wait for a new user start signal. C2 remains open.

## Diagnostic event observation (third passive run)

The diagnostic EXE was re-hashed immediately before launch and matched `6BBD9B28888B1C4FAE59CD79E5B7D76C9E3BACD1C5DC7CEF323189790AD99CB5`; no process with this executable path was running. PID [omitted] was launched once, hidden, at `2026-10-20T09:14:53.3234109Z` (Windows process start `09:14:53.9833574Z`). New, non-conflicting raw logs are [host-diagnostic-2026-10-20.stdout.txt](host-diagnostic-2026-10-20.stdout.txt) and [host-diagnostic-2026-10-20.stderr.txt](host-diagnostic-2026-10-20.stderr.txt).

The stdout ready line was `media_probe_ready elapsed_ms=30 manager_tokens=2 session_tokens=6 total_tokens=8`. The line was read and sent to the coordinator at `2026-10-20T09:15:16Z`, with about 37 seconds left in the nominal observation window; the user was invited to act only after this positive token check. The user later replied “已操作”; the action type and exact time were not recorded and are not inferred. The 30 ms value is the observer's elapsed value, not a separately timestamped UTC callback.

Final stdout summary: `media_probe_summary elapsed_ms=60002 manager_ok=1 manager_error=0 refresh_manager_ready=1 refresh_event=8 refresh_fallback=55 callback_current=0 callback_sessions=0 callback_timeline=0 callback_playback=2 callback_media=6 refresh_enqueued=8 refresh_queue_full=0 refresh_queue_disconnected=0 playback_changes=2 selected_session_changes=0 snapshot_ok=64 snapshot_empty=0 snapshot_error=0 subscriptions_added=8 subscriptions_removed=8 subscription_remove_error=0 snapshot_mean_us=7610 snapshot_max_us=29682`. This run observed event callbacks for playback (2) and media properties (6), eight successful enqueues, and eight consumed event refreshes; there were no current-session/sessions/timeline callbacks and no full/disconnected send outcomes. Playback state changed twice across successful snapshots. The source identifier did not change; this says nothing about COM session-object changes under one AppID. All 64 successful snapshots were non-empty, and all 8 registered tokens were successfully removed. No CPU/power or WebView evidence was collected.

The first monitoring PowerShell script launched the probe, then failed when it tried to read stdout while `Start-Process` held the redirected file open. This was a monitor failure, not a probe failure. The probe PID remained alive; a separate supervisor verified the executable path and process start time, read the ready/summary lines, and observed natural process exit after about 60 seconds. It did not force termination. The child exit code was not retained by a handle and is unknown; it is not inferred from the summary or PID disappearance. Stderr was empty (0 bytes).

The failed monitor did not print its in-memory pre-run settings and Run snapshots before exiting, so this run has no auditable same-run before/after comparison. A read-only check after the run found `settings.json` SHA-256 `3EDDEC967D655D920A2DE34B722E727FA91F4787436DBFDD45F4454939B59C0D` and `.bak` SHA-256 `0B3192D5FD17C94B1CA0DAEE0697EE791772629C2071C51E42B67FF2C67633F0`, matching the previous runs' recorded hashes. Run value names after the run were `Isle` present and `Wind0wsDynamicIsland` absent, also matching earlier records; raw values were not printed. Because the current run's captured pre-state was lost with the failed monitor, these are consistency checks against prior evidence, not proof of unchanged state during this run.

C2 now has positive evidence that the diagnostic manager/session delegates can enter and enqueue events in this host, and that the worker consumes event refreshes and removes its registered tokens on this run. It does not identify which user action produced the callbacks, prove all five callback types, prove the production UI event loop is or is not needed, or validate frontend delivery/rendering. The nominal 60-second timer starts after listener startup; synchronous WinRT `.get()` can extend wall time. Empty-session/idle cadence, recovery, CPU/power, and frontend behavior remain untested. C2 remains open.

## Empty allowlist idle cadence run

### Scope and safety

The example now accepts `--no-selected-player`. Only this mode sets `AppState.settings.selected_player_ids = Some(Vec::new())` on its in-memory `AppState` before `.manage(...)`; the no-argument mode is unchanged. Production `selected_session` checks that empty allowlist and returns `Ok(None)` before reading session metadata or entering the artwork cache path. The probe still uses its isolated identifier and windowless `Builder::build`, without product setup/run, WebView, settings commands/services, cache directory access, or Run synchronization. EventBus state is in process memory. This tests an empty allowlist/no selected session; it does not mean Windows had no global SMTC sessions or test a real service disconnect.

### Build and run identity

The previous diagnostic EXE was preserved before rebuilding at [c2-media-probe-pre-empty-allowlist-6BBD9B28888B1C4F.exe](archive/c2-media-probe-pre-empty-allowlist-6BBD9B28888B1C4F.exe), SHA-256 `6BBD9B28888B1C4FAE59CD79E5B7D76C9E3BACD1C5DC7CEF323189790AD99CB5` (copy hash matched the original). Default library check and feature example check passed offline with Rust 1.94.1. Then one debug example build passed with exit code `0` using the existing isolated target and command `cargo build --offline --example c2-media-probe --features c2-media-probe`; no product Release was rebuilt.

The resulting EXE `src-tauri/target/c2-offline-2026-10-17/debug/examples/c2-media-probe.exe` is 28,875,776 bytes, SHA-256 `04B91E53325727039C5F16BFBD7C557358415C605027D6E5A07655D39DCE3A56`. Source/config hashes: `Cargo.toml` `2ED9E40CFC140B4026EC91BFD011BD70121C42E2CF386B2D3804B67D890A6C9B`; `Cargo.lock` `C19A6B9079E51BF5475BC9DB55E81475988162B8AB2C1056B0F00F9F263C7D6F`; `src/lib.rs` `F164F228FD82CEFD6A52E5CB882FA0B4C5904FB414BA395BE56A226D4F61BBC1`; `examples/c2_media_probe.rs` `F65D2E0DFF51D7F8CC0068E40DE2B8D0CFAFBCF12D7C7818331027A81D0C075B`; `build.rs` `80774FD49F45D456E46CCFB7124C924D334636920920B16E2A4B01F9D953F046`; `tauri.conf.json` `5D98A643C9DFFC6DE0511CDEBA0B7008F9CD3BC42436518146AD2F3BC124636F`.

The EXE hash and absence of a prior process with that image path were checked before one hidden launch. It ran as PID [omitted] with `--no-selected-player`; launch UTC was `2026-10-20T09:25:01.4636572Z`, process creation UTC `2026-10-20T09:25:01.5152072Z`. The one continuous PowerShell monitor retained the `Start-Process -PassThru` object/handle, read stdout through a `FileStream` opened with `FileShare.ReadWrite`, retained the settings hashes and full Run value snapshot only in memory, waited on that same process object, and captured its actual exit code. No shared-read failure occurred. The exact mode command was:

```powershell
Start-Process -FilePath '.\src-tauri\target\c2-offline-2026-10-17\debug\examples\c2-media-probe.exe' -ArgumentList '--no-selected-player' -WorkingDirectory '.' -WindowStyle Hidden -RedirectStandardOutput '.\docs\performance\c2-media-probe-2026-10-20\host-empty-allowlist-2026-10-20.stdout.txt' -RedirectStandardError '.\docs\performance\c2-media-probe-2026-10-20\host-empty-allowlist-2026-10-20.stderr.txt' -PassThru
```

The process exited naturally after `60.37s`, actual child exit code `0`; no forced termination. The program's observation summary reports `elapsed_ms=60004`. Raw bounded stdout is [host-empty-allowlist-2026-10-20.stdout.txt](host-empty-allowlist-2026-10-20.stdout.txt) (mode line, ready line, 13 poll lines, one summary); stderr is [host-empty-allowlist-2026-10-20.stderr.txt](host-empty-allowlist-2026-10-20.stderr.txt), 0 bytes.

### Poll timing and empty emission results

The host reported `media_probe_mode selection=empty_allowlist`, then `media_probe_ready elapsed_ms=25 manager_tokens=2 session_tokens=6 total_tokens=8`. All 13 bounded poll records said `snapshot=empty`. Sequence 1 was manager-ready at 26 ms and scheduled 1000 ms. Sequence 2 was fallback at 1028 ms and scheduled 5000 ms. Sequences 3–13 were fallback at 6029, 11029, 16030, 21031, 26031, 31033, 36034, 41037, 46038, 51039, and 56040 ms, each scheduling 5000 ms. Thus the first observed fallback gap was 1002 ms; subsequent observed fallback gaps were 5000–5003 ms, while the recorded next interval remained 5000 ms. No event refresh interrupted the cadence in this run. These elapsed deltas are observed between snapshots; `next_interval_ms` is the interval the code scheduled, not a guarantee about actual wake time.

Final summary: manager success/error `1/0`; manager-ready/event/fallback refreshes `1/0/12`; callback current/sessions/timeline/playback/media `0/0/0/0/0`; refresh enqueue/full/disconnected `0/0/0`; playback/source-identifier changes `0/0`; snapshots ok/empty/error `0/13/0`; subscription add/remove/remove-error `8/8/0`; poll records `13`; `empty_emit_calls=1`, `empty_emit_success=1`, `empty_emit_error=0`, `empty_payload_error=0`; snapshot mean/max `46/346 µs`. The successful return counts only the production EventBus emit call. There was no WebView, so it does not establish frontend receipt or rendering.

### Same-run state comparison and limits

In the same PowerShell process, settings hashes were equal before/after: main `3EDDEC967D655D920A2DE34B722E727FA91F4787436DBFDD45F4454939B59C0D`; backup `0B3192D5FD17C94B1CA0DAEE0697EE791772629C2071C51E42B67FF2C67633F0`. The entire current-user Run key value-name/kind/data snapshot compared equal in memory (`values_unchanged_in_memory=True`); raw values were not output or saved. No setting, cache, Run, or product release artifact was changed by this run.

This confirms the production listener's empty-allowlist behavior in the isolated host: one initial 1-second schedule, subsequent 5-second fallback schedules, 13 empty snapshots, and one empty-state EventBus call returning success. It is specifically an empty allowlist/no selected session. Windows-global no-session behavior, true SMTC disconnect/recovery, event interruptions during idle, WebView receipt/rendering, CPU/power and product integration remain untested. C2 remains open.

## Bounded listener metadata cache (implemented; not run)

The listener now owns one metadata entry alongside its existing MTA worker and current manager/session subscriptions. It retains title, artist, artwork, the selected `GlobalSystemMediaTransportControlsSession` interface, and a refresh timestamp. The direct `get_media_info` path still reads a full snapshot on each command call. Listener snapshots continue reading timeline, playback status, controls, shuffle, and repeat every refresh; only media properties and the thumbnail stream are eligible for reuse.

Metadata is read on initial selection, when the selected COM object identity changes, after `MediaPropertiesChanged` or manager current-session/sessions changes, or when the 5-second TTL threshold is reached. Five seconds is the cache-expiration threshold, not a hard wall-clock refresh bound: fallback polling is 1 second while playing and 2 seconds while paused, and the threshold is checked on a snapshot. A paused session therefore commonly refreshes around 6 seconds after the prior metadata read, plus synchronous WinRT query time; a slow query can extend that further. Playback and timeline snapshots are not delayed by this TTL. COM identity uses `IUnknown` equality from the locked `windows` 0.52 / `windows-core` 0.52 sources; that implementation queries `IUnknown` and compares its canonical pointer, so an AppID-only match is not used. A manager reacquisition and confirmed empty selection clear the actor cache. If selection changes and the new metadata read fails, the prior session's metadata has already been discarded. A failed refresh for the same session remains dirty/stale and is retried on the next refresh instead of being treated as valid.

The invalidation generation is incremented in the callback before `try_send`. A capacity-one queue returning `Full` therefore coalesces the refresh message without dropping the metadata dirty state. The worker samples the generation before snapshotting and only advances its seen generation after a successful metadata refresh; a callback arriving during the read leaves a newer generation pending for the next refresh. Probe summary gained only `metadata_reads` (metadata read attempts, including failed attempts) and `metadata_reuses` (snapshots using cached metadata).

Validation, Rust/Cargo 1.94.1 with the existing isolated target `src-tauri/target/c2-offline-2026-10-17`:

- `cargo check --offline --lib` — passed.
- `cargo check --offline --example c2-media-probe --features c2-media-probe` — passed.
- `cargo test --offline --lib metadata_refresh_policy_covers_identity_invalidation_and_five_second_ttl` — passed (1 test).
- `cargo test --offline --lib media_property_invalidation_survives_a_full_refresh_queue` — passed (1 test).
- `git diff --check` — passed.

These focused tests cover refresh policy boundaries and production callback ordering when the queue is full; they do not simulate a live WinRT session. At implementation time, no probe was rebuilt or run and no product Release was rebuilt. A subsequent one-time passive default-mode observation is recorded below. The expected reduction is structurally one metadata read per invalidation or 5-second TTL threshold rather than per 1-second playing snapshot; runtime counts below confirm reuse for that observed selected session. Cover-change behavior against a live provider, frontend delivery, CPU, and power remain unmeasured. C2 remains open.

## Metadata reuse passive observation (one default-mode run)

### Archive and build identity

Before rebuilding, the existing debug probe at `src-tauri/target/c2-offline-2026-10-17/debug/examples/c2-media-probe.exe` was SHA-256 `04B91E53325727039C5F16BFBD7C557358415C605027D6E5A07655D39DCE3A56`, size 28,875,776 bytes. It was copied to [the pre-metadata-reuse archive](archive/c2-media-probe-pre-metadata-reuse-04B91E5332572703.exe); the original and archive hashes matched exactly. The existing feature example was then built offline once with the same isolated target using `cargo build --offline --example c2-media-probe --features c2-media-probe` (exit 0, 41.48 seconds). The new debug probe is 28,887,552 bytes, SHA-256 `5D44DA0BBC34CA9F22D04CFC75DA197A98C17DAAFE92B459991E26135016C68B`. No product Release was rebuilt.

### Passive run and observed counts

The new default-mode probe ran once, without arguments, as PID [omitted], starting `2026-10-20T09:44:32.9297825Z`; it exited naturally after 60.32 seconds with the actual retained-process `ExitCode=0`. No timeout termination occurred. The supervisor retained the `Start-Process -PassThru` process handle and read the active stdout file through `FileShare.ReadWrite`. It captured the full current-user Run key snapshot (name, kind, and data) and product `settings.json` plus `.bak` hashes in memory before and after the run. Main settings hashes were `3EDDEC967D655D920A2DE34B722E727FA91F4787436DBFDD45F4454939B59C0D` both before/after; backup hashes were `0B3192D5FD17C94B1CA0DAEE0697EE791772629C2071C51E42B67FF2C67633F0` both before/after. The full Run key snapshot compared equal in memory; its values were not printed.

The host reported 63 successful selected-session snapshots and zero empty/error snapshots over `elapsed_ms=60001`. Metadata counters were `metadata_reads=14`, `metadata_reuses=49`: among these 63 successful snapshots, 49 reused cached metadata rather than entering the media-properties/thumbnail-stream read path. This is direct evidence of reuse for the selected session observed in this run, not a CPU or power measurement. Refresh counts were manager-ready/event/fallback `1/5/57`; callback counts current/sessions/timeline/playback/media `1/0/2/3/0`; queue enqueued/full/disconnected `5/1/0`; playback/source-identifier changes `2/1`; subscription add/remove/remove-error `8/8/0`; snapshot mean/max `2894/40960 µs`. All 16 bounded poll records were `snapshot=ok` with `next_interval_ms=1000`; the summary covers the full 60 seconds. No user player or product instance was started, controlled, or closed. Callback attribution to any user action is not inferred.

Raw bounded stdout is [host-metadata-reuse-2026-10-20.stdout.txt](host-metadata-reuse-2026-10-20.stdout.txt); stderr is [host-metadata-reuse-2026-10-20.stderr.txt](host-metadata-reuse-2026-10-20.stderr.txt), 0 bytes. This run did not exercise a `MediaPropertiesChanged` callback (`callback_media=0`), so it does not validate cover replacement under a media event. It also does not establish frontend receipt/rendering, CPU/power, or full product integration. The five seconds is a TTL threshold checked on snapshots, not a strict wall-clock refresh bound: fallback polls every second while playing and every two seconds while paused, so a paused refresh commonly lands around six seconds after the previous metadata read, plus synchronous WinRT query time. C2 remains open.

## 2026-10-24 dedicated SMTC event-session safety gate

The existing `src-tauri/examples/c2_media_probe.rs` is an isolated, windowless observer of the production listener/EventBus path. It can test an empty in-memory allowlist, but it has no SMTC publisher and cannot create or drive media/property/playback/timeline changes. The repo's `scripts/media-session.ps1` can snapshot sessions or call Play/Pause/Next on an already-existing `cloudmusic.exe` session; it is not a dedicated test session and would target an existing user player if used that way. No local SMTC test publisher/session launcher was found in `scripts/` or `src-tauri/examples/`.

Read-only CIM showed no `isle.exe` or `cloudmusic.exe` process and several Edge processes. Current SMTC session ownership could not be confirmed: the PowerShell 7 WinRT projection did not resolve `GlobalSystemMediaTransportControlsSessionManager`; that query returned no session data and performed no player action. Starting a new player/publisher could temporarily take over Windows' global media controls, so no test session or product app was started. Creation, real property/playback/timeline notifications, stop/disconnect, and resubscription remain untested; prior probe counts are not credited as this event-coverage run.

Minimum prerequisite for a safe next attempt: the user manually pauses their current NetEase playback and confirms a roughly 60-second window in which a dedicated silent test session may temporarily become the Windows current media-control session. Then a small one-shot test publisher can be paired with the existing probe, with no calls to the NetEase process and no product settings/device changes. Until the global-control handoff is explicitly clear, keep C2 open.

## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.

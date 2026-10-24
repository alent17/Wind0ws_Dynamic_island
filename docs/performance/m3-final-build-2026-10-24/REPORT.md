# M3 final offline production candidate

## Build

- Worktree: `.`
- HEAD at build: `48530f7343e84129aef34113dbef4dd18f7431ef`; the working tree was already dirty and contains the optimization changes recorded in `OPTIMIZATION_PLAN.md`.
- Rust/Cargo: `1.94.1` (`e408947bf`, `29ea6fb6a`)
- Frontend check: `pnpm run check`, passed with 0 errors and 0 warnings.
- Frontend production build after the U7 neutral button-fill slice: `pnpm run build`, passed; Vite 6.4.3 transformed 3,873 modules and emitted 28 files (24,151,473 bytes).
- Rust production command: `cargo build --offline --locked --release --features custom-protocol`, exit 0; the final incremental build after the U7 asset change completed in 3m47s. Fingerprint records `custom-protocol` and `default` features.
- Existing warning: `NeteaseTimelineSample.track_id` and `play_id` are unused.
- `git diff --check`: passed (Git printed existing LF-to-CRLF notices).

## Artifacts

| File | Bytes | SHA-256 |
|---|---:|---|
| `src-tauri/target/release/isle.exe` | 23,468,544 | `506DE69715876C60A72CE8DCF900FB5EDC7A7ED864F608DCAE3AED994215055A` |
| `src-tauri/target/release/isle_lib.dll` | 116,736 | `483D0C5DBDC7649021E92125EE3395FB58427B114F92303E559DB39129F41BC5` |

The immediately previous candidate was retained at `src-tauri/target/release/pre-m3-u7-final-2026-10-24/`: EXE SHA-256 `A5421BA26FF4A9EFD38378DC1BCE9594230458B06C03BCBA90B9E130A2ABF6F8`; DLL SHA-256 `040A8A2F6AFF86FB19CE036C38F3EEA173A1BA8830B46A84C3A5BFE36DBFEB95`. The pre-batch candidate remains in `pre-m3-final-2026-10-24/`.

## Frontend asset correspondence

The current `build/` tree has 28 files, totaling 24,151,473 bytes. A sorted manifest of relative path, byte count and each file's SHA-256 has aggregate SHA-256 `41654E2BFC82F4F1369EE8FB03FB30E303C5B2D70E464B7425FE3634A65746DD`. The newly generated `src-tauri/target/release/isle.d` references every one of the 28 current frontend files, and all were timestamped before the executable was produced. The Rust fingerprint confirms `custom-protocol`.

## Final fixture and migration checks

- App-level hide race Playwright test passed 1/1: re-expansion during the 360ms wait emits no delayed bounds command; a later ordinary hide still emits one.
- IslandSurface page-focus race and collapse/hide test passed 1/1 using actual toolbar/back actions and a focus-call probe.
- U7 neutral button style/proportion fixture passed 1/1 (58×58 transport, 28×28 menu glyph, 28×28 page-back).
- U5 normalizer Vitest passed 3/3; targeted Rust legacy migration test passed 1/1; changed Rust file passes rustfmt check.
- U6 legacy-field serde compatibility is already covered by the existing Rust regression; it was reviewed and not rerun.
- Final `pnpm run check` passed with 0 errors/warnings; `git diff --check` passed.

No application or media session was launched, installed or deployed. This is a build candidate only; runtime/native acceptance remains separate.


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.

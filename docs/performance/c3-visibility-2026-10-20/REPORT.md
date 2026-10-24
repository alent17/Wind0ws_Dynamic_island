# C3 visibility gate report — 2026-10-20

## Scope and behavior

This batch gates visual work in `FloatingWindow.svelte` with one `visualWorkVisible` value derived from `pageVisible && !isCaptureHidden`. It does not change the main island or spectrum consumer path.

- MV intent stays separate from SMTC playback. The video remains muted and mounted while capture-hidden, preserving its current position and the MV setting; it is paused, and its autoplay gate is false. The centralized play path also checks document visibility, capture visibility, current track/MV state, circular-cover mode, and SMTC playback. `loadeddata`, the 30-second preview loop, and same-track playback callbacks go through that gate. A play promise checks the latest desired state and video owner when it settles; destruction invalidates pending requests and pauses the element. This prevents a late completion from reviving a hidden or replaced MV while allowing a valid current playback to continue after a quick hide/show.
- The circular album's existing `.playing` class is now gated by `visualWorkVisible`, which pauses its current CSS rotation timeline while hidden and resumes it when visible.
- The progress interpolation timer exists only while visual progress is visible, SMTC is playing, and the window is not in compact-cover mode. It ticks immediately on activation and then every 250 ms. When hidden, paused, or compact, the timer is removed and `clockNow` is refreshed once; the next visible playback starts from `Date.now()`.
- No playback command is sent to SMTC, and countdown `endsAt` / completion behavior is unchanged. The existing `VisibleWindow` wrapper and native visibility checks are untouched.

## Validation

- Direct local `svelte-check` invocation — 0 errors, 0 warnings. The `npm run check` shim was unavailable in this checkout, so the installed checker entry point was invoked directly; no package was downloaded.
- Relevant existing Vitest files (`captureMode`, `countdown`, `spectrumStore`, `spectrumRender`) — 15 passed.
- Existing lifecycle Playwright fixture, including the added FloatingWindow tests — 6 passed. The tests use the real component with mocked Tauri IPC/events and controlled `HTMLMediaElement.play()` promises. They cover capture-hide pause, hidden media/loadeddata attempts, quick restore with an old promise settling later, paused-track non-resurrection, disposal with a pending play, 250 ms timer removal, and circular artwork CSS animation pause/resume.
- Existing lifecycle fixture production build — 3,854 modules transformed. Main `npm run build` — passed, Vite 6.4.1, 3,868 modules transformed.
- Impeccable detector ran once for `src/FloatingWindow.svelte`; it reported existing radius/type-ramp and easing/background advisories outside this change. No visual styles, copy, layout, or tokens were changed.
- `git diff --check` — passed.

## Limits

This validates component behavior under mocked capture events, not actual Windows capture-provider delivery, native HWND visibility transitions, video decode/power reduction, or CPU/GPU savings. The test stubs media play/pause to exercise lifecycle ordering; it does not measure real decoder work. C3 remains open for native WebView hidden/restore timing and controlled resource measurements.

## Follow-up: island marquee and weather polling

- `MarqueeTitle` now accepts an `active` prop defaulting to `true`, so existing callers retain their previous behavior. `IslandSurface` passes active only while the island is visible, expanded opacity is above `0.5`, and the selected page is music. Both the current media title and remembered title use the same gate. Inactive state cancels the WAAPI animation; returning to the visible music panel remeasures overflow and starts a fresh animation. Motion speed, pause timing, typography, and layout are unchanged.
- The existing App weather snapshot effect now also requires `!isHidden`. Hiding disposes the current interval and ignores any stale result; showing runs the effect once immediately and keeps the existing `idleApi` 30-second cache. No weather/system API split was introduced.
- Existing IslandFixture tests mount both current and remembered long titles. They check no running animation in compact mode, animation in the visible expanded music panel, cancellation on collapse and hide, remeasurement on restore, and cancellation after unmount. Lifecycle fixture coverage verifies hidden mode removes the 30-second polling interval, no invoke occurs while hidden, immediate restore uses the cache, and restore after the cache expires makes exactly one new `get_idle_snapshot` invoke.
- Validation for this follow-up: targeted Playwright cases passed (2 title cases; lifecycle suite 7/7), lifecycle fixture production build passed (3,854 modules), direct `svelte-check` passed with 0 errors and 0 warnings, and the main frontend production build passed (3,868 modules). The one requested UI detector run reported only existing style/layout advisories in unchanged IslandSurface CSS and returned exit code 1 for those findings; this batch changed no styles.
- The weather test uses capture-event mocks and Playwright's controlled clock; it validates invoke counts and reuse of the existing 30-second cache, not an actual Windows native hide/show path or network/weather freshness. C3 remains `[ ]`; native window behavior and CPU/GPU/resource savings remain unmeasured.

## Merged offline Release candidate

After both old product instances were reported exited, a fresh CIM query found no running `isle.exe` before the build; a post-build query also found none. The existing `src-tauri/target/c2-offline-2026-10-17/release/isle.exe` and `isle_lib.dll` were copied into `../c2-media-probe-2026-10-20/archive/` before rebuilding and each copy was checked against its source SHA-256. The archived pair is `isle-pre-c3-followup-542851A3757B406C.exe` (8,500,224 bytes, SHA-256 `542851A3757B406C611DCE1C52DDECD88FD8D91D5F50D06F8DA06CD6D5E16FAB`) and `isle-lib-pre-c3-followup-187C4B3AA9AD7026.dll` (116,736 bytes, SHA-256 `187C4B3AA9AD7026C370ECA41DD6B665A1D5EB4499E2078E68E883DC928CE03C`). Existing probe logs and earlier archives were not changed.

Using Rust/Cargo 1.94.1 and the existing isolated target, `cargo build --offline --locked --release` with default features succeeded in 4m42s. The new EXE is 8,515,072 bytes, SHA-256 `90A49423B24C569119D6DEC22BA6BE50F5132E280F148538222140C4881405BB`; the DLL is 116,736 bytes, SHA-256 `0B72FD68BBED4B58DF48CF0FBDBF1EED8987DB6E437631D21896F70458238465`. HEAD remained `48530f7343e84129aef34113dbef4dd18f7431ef`; the build came from the already-dirty shared worktree containing prior C1/A5/C2 changes and this C3 work, so it is not a clean-HEAD build. The final lockfile hashes are Cargo `C19A6B9079E51BF5475BC9DB55E81475988162B8AB2C1056B0F00F9F263C7D6F`, npm `F775FE6985E01B3892AB9A265C1C4DA732E97AED3E042FD592069375D026F208`, and pnpm `1D46CF1E46B93D6B36A32BB74F8BE4F9B181E24AA00790E6784685732AE00439`. The command was a direct Cargo build, not the Tauri bundler/installer. No product was launched, installed, or configured.

The same frontend production build emitted 28 files totaling 24,127,564 bytes. Resource identity includes `build/index.html` SHA-256 `D4062FA631D72D7E7D6D0730B6FE3B5DF12273787C47B5B31F52C6FD73177E46`, `build/assets/App-tlZ9z3Jt.js` SHA-256 `6177834C06E00BBC92BE8F641F4515741CBC2FE1CF3DC2351C33600EB578AF79`, `build/assets/idle-DJUqFxcY.js` SHA-256 `ABBF4F798A7F958AF097054A8B06B7468A41032E1F8176146FBA0C3582DC49D4`, and `build/assets/App-BW3uYUaM.css` SHA-256 `B9A9CF66B6C9CE3D4E7733ACADB1051EB5B27181584C1CE1C20EB133D9F0FC09`. This candidate has not received native startup or capture-hide validation; no CPU/GPU benefit is claimed.

## Bounded candidate startup check

On 2026-10-20 the first Computer Use launch request unexpectedly opened the installed `<local-path> (PID [omitted], SHA-256 `77446A616B9851E08E5C239152F310F926AC72B7185151512D777F6C748491A1`), not the candidate above. No window input was performed on that instance. After the user stopped Computer Use, a later process check found that PID already absent; it was not terminated by this task. One direct `.NET ProcessStartInfo` launch then started the exact candidate once: PID [omitted], start `2026-10-20T19:06:34.358991+08:00`, path `src-tauri/target/c2-offline-2026-10-17/release/isle.exe`, SHA-256 `90A49423B24C569119D6DEC22BA6BE50F5132E280F148538222140C4881405BB`; it was the only `isle.exe` at the final process check.

The initial read-only state was settings main/backup SHA-256 `3EDDEC967D655D920A2DE34B722E727FA91F4787436DBFDD45F4454939B59C0D` / `0B3192D5FD17C94B1CA0DAEE0697EE791772629C2071C51E42B67FF2C67633F0`, and a normalized HKCU Run name/kind/data digest `16D13802B63C8DEC6DAE583352B448D5A845A5C43114A79CBF24B1B198F277BE`. After the wrong-path app started, the main settings digest was `D52CC5DBB595D83A5B8A3D285B1B03CB7B75F2B3BAE5AB384A1C1DA257C437E0`; the backup stayed unchanged. Its Run digest was `25FB941E9D87C5AF0AF5167A9BD9D71F2EB149A891E0F5CC833260BD7E34130C`. Three seconds after the candidate launch, settings digests remained D52CC5… / 0B3192… and the Run digest was `B31B17D08D03DA819F3F0E3C3BA585D04E187B8D34D6C9745A69BA7D50F1EFFD`. The coordinator independently confirmed that the normalized `Isle` startup value matches the candidate's startup command. `autoStart=true`, `showSpectrum=true`, `floatingCircularAlbum=false`, `enableMvPlayback=true`, and all four capture-hide preferences remained unchanged. Run names/kinds remained the same; raw registry values were not displayed or saved. The settings hash difference is not attributed to formatting and no backup restore or manual registry change was made.

Computer Use was interrupted by the user's physical Escape before any state capture or input on the intended candidate. This establishes process identity only; launch UI, expand/collapse, floating window, temporary hide/restore, capture-provider behavior, and performance remain unverified. The candidate remains running for the user. No source changes or build occurred during this startup check.


## Publication note

Raw local logs, session captures, CSV/JSONL samples and executable artifacts referenced by this report remain in the local worktree and are not included in this GitHub upload.

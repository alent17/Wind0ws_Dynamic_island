# I1 / I2 interaction region and geometry plan

Source audit plus the first I2 geometry and I1 bounded-sender source slices. No cadence cap or quantization was added. Native IPC/click-boundary behavior is still unverified, so I1 and I2 remain open.

## Current path and confirmed gaps

- `IslandSurface` emits a region on every geometry effect and includes `settled`, but `App.applyIslandRegion` drops that flag and sends every changed JSON payload immediately. The signature is stored before `set_island_interaction_region` resolves, so a failed terminal update is suppressed if the same shape is emitted again. Rust's monotonic revision guard rejects stale arrivals; it does not bound serialization or IPC requests already sent.
- `navigationScaleForWorkArea` keeps its `.62` lower bound so the layout scale has a minimum; App's separate `.625` content-fit factor yields the actual Island content range `.3875–.625`. `navigationHostFor` also keeps `.62`, independently scaling the reserved native envelope to keep it large enough around the scaled content. The audited `surfaceOffsetFor` had incorrectly floored the visual gap at `.62` too, while anchor compensation used the true content scale, producing up to `5.115` CSS px drift. This slice removes only that gap floor; host and content-layout bounds remain intact.
- **Coordinate contract now documented at source helpers/call sites:** monitor work area and native window bounds use Win32 physical screen pixels; `HostGeometry`, island dimensions, region points/radii/offsets, and cursor points after native conversion use CSS-logical pixels. App derives content scale from physical work area and DPR, rounds the logical host to a physical window size, and positions within the physical work area. Rust subtracts the physical window origin from the physical cursor point and divides by that window's scale factor before testing the logical region. Native bounds rounding can move a fractional hidden placement by at most half a physical pixel; the two-pixel reveal is preserved within that rounding bound.
- I1 cadence reduction stays deferred until native acceptance confirms visual surface, region, host placement and physical cursor samples agree. Do not change click-through policy as part of the sender work.

## Existing coverage

- `src/lib/islandGeometry.test.ts` now covers basic host offsets, negative-origin placement, one-physical-pixel attached overlap, two-pixel hidden reveal, page fit, and final `.3875/.4/.55/.62/.625` content scales across all edges/styles and DPRs. It checks physical cursor-to-logical-region round trips and work-area placement bounds. It does not establish native click-through behavior or saved floating-window restore behavior.
- `src-tauri/src/commands/window.rs` unit tests cover region validation, rounded/polygon/extra-rect hit testing, stale revision rejection, and cursor scale-factor cache behavior. There is no test for frontend request coalescing, settled flushes, acknowledgement-after-success, or retry after IPC failure. Source/UI tests do not establish native click-through at the true window/DPI boundary.
- Existing geometry and revision machinery is useful and should be retained. I1/I2 do not require rebuilding the native sender or polygon hit test.

## Smallest safe next implementation slice

The first I2 source slice is complete: `islandContentScaleForWorkArea`, `scaleIslandGeometry`, `islandAnchorGap`, `interactionSurfaceOffsetFor`, and `physicalHostSizeFor` now express the production transformations, and App/IslandSurface use them. The `.62` host-envelope floor remains; visual scale/gap no longer use it.

The first I1 source slice is complete in `src/lib/islandRegionSender.ts`: it allows one in-flight command and one replaceable latest pending region, acknowledges only successful signatures, retries the newest terminal after failure with a 250 ms exponential backoff capped at 2 seconds, and stops timers/late completions after disposal. `App.svelte` assigns the next monotonic Rust revision only inside the actual send callback. `windowApi.setIslandInteractionRegion` returns the Tauri invoke promise without catching it, so errors propagate to the sender unchanged. Failure logging occurs once per uninterrupted failure burst. The payload keeps exact geometry; no 20 Hz cap, quantization, or click-through change was made. Since no frequency cap is active yet, each emitted shape still enters the latest-state sender; terminal same-signature emissions during an in-flight failure remain queued for retry, while a successful identical in-flight payload satisfies that exact terminal state.

Next, perform native acceptance on mixed-DPI monitors and all four edges; record command count/bytes, in-flight peak, visible controls, transparent corners, shoulder edges and desktop click-through before considering a frequency cap. Review any saved-bounds unit/migration issue separately; do not mix in A3 capture timing.

## Verification boundary

- Pure tests passed: final App scales `.3875/.4/.55/.62/.625`; all four edges and both styles; 100/125/150/200% DPI; negative monitor origins and taskbar-reduced work area; fractional geometry rounding; visual/region anchor agreement and physical↔logical cursor round trip within 1 logical px; two physical pixel reveal within half a physical pixel of integer-bound rounding. `pnpm exec vitest run src/lib/islandGeometry.test.ts`: 22/22; `pnpm run check`: 0 errors/warnings; `pnpm run build`: success (3868 modules).
- I1 sender and I2 geometry regression tests passed: `pnpm exec vitest run src/lib/islandRegionSender.test.ts src/lib/islandGeometry.test.ts` — 27/27; `pnpm run check` — 0 errors/warnings; `pnpm run build` — success (3869 modules). Sender tests cover one in-flight/latest pending, latest wins, A→B→A after an older acknowledgement, same-signature settled emission followed by failure/retry, success-only acknowledgement, bounded backoff, failure-burst log suppression, and disposal with timer or command pending.
- Native acceptance remains necessary and unperformed: on the actual Tauri window, at each edge and mixed-DPI monitor, sample visible control interiors, rounded transparent corners, shoulder boundaries and desktop points during normal/interrupted transitions. Confirm controls receive clicks, transparent areas pass clicks through, and the desktop does not get swallowed. Record DPI, bounds, sampled points, region revision, IPC count/bytes and in-flight peak. Unit/browser tests cannot substitute for this boundary; I1 and I2 remain unchecked.

## Evidence reviewed

- `src/App.svelte`: `applyIslandRegion`, expanded scale and host placement.
- `src/lib/components/island/IslandSurface.svelte`: region notification and `settled` production.
- `src/lib/islandGeometry.ts` / `src/lib/islandGeometry.test.ts`: scaling, offsets, placement helpers and current pure coverage.
- `src/lib/api/window.ts`: region IPC payload.
- `src/lib/islandRegionSender.ts` / `src/lib/islandRegionSender.test.ts`: serialized request, latest pending, ack/retry, disposal and deferred-promise coverage.
- `src-tauri/src/commands/window.rs`: normalization, revision gate and physical-cursor-to-window-local conversion.

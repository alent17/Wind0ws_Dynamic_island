# Migration audit: Tauri main repository and Native source

Date: 2026-10-05  
Tauri main baseline: `a87ba70f2dad0d433f23614d11cc7b6f95ed0282` (`main`)  
Native baseline: `a633e38720a3088c464a1f7fe4ff4e18f844cdc2` (`main`)

This audit compares the existing Tauri/Svelte source with the committed Native
source. The Native checkout also has local, uncommitted edits in
`crates/isle-app/src/floating_player.rs` and `crates/isle-app/src/main.rs`, plus
untracked media and screenshots. Those local files are not treated as migration
input and were left untouched.

## Audit scope

| Area | Decision | Evidence / notes |
|---|---|---|
| Tauri configuration | ✅ Keep | `src-tauri/tauri.conf.json` defines the Tauri 2 app and window lifecycle. The island window is configured as opaque per the current product direction. |
| Svelte frontend | ✅ Keep | `src/` contains the island, floating player, timer, Studio, stores, API wrappers, and UI components. |
| Island UI and settings | ✅ Keep | `src/App.svelte`, `src/FloatingWindow.svelte`, and `src/Studio.svelte` are the active UI. Do not replace them with Native UI. |
| Animation system | ✅ Keep | Island geometry and motion live in `src/lib/islandGeometry.ts` and `src/lib/islandMotion.ts`; window motion is in `src-tauri/src/commands/window.rs`. |
| State management | ✅ Keep, improve incrementally | Existing state and API layers live in Svelte components/stores and `src/lib/api/`; do not transplant Native UI state. |
| Rust backend | ✅ Keep | Tauri command, model, service, and state layers already exist under `src-tauri/src/`. |
| SMTC | ✅ Keep; compare targeted gaps | `src-tauri/src/services/media.rs` reads GSMTC metadata, timeline, controls, playback state, artwork, and capabilities. Native `crates/isle-app/src/media.rs` remains a reference for focused capability differences. |
| Netease controls | 🔄 Audit before port | Tauri already uses GSMTC and has Netease metadata/cover support in `services/media.rs`. Native `crates/isle-app/src/cdp.rs` and `netease.rs` add a local CDP route for player modes, not a wholesale replacement for play/skip controls. Real-client behavior still needs validation. |
| Other player controls | ✅ Keep | Tauri routes media actions through its existing GSMTC service and models. Native player categorization and capability contracts in `crates/isle-core/src/player.rs` and `player_extension.rs` can inform targeted changes. |
| System volume | ✅ Keep | Existing Tauri implementation is in `src-tauri/src/services/system_audio.rs` and `commands/audio.rs`. Native `system_audio.rs` is overlapping reference material. |
| Startup | ✅ Keep | Existing startup registry integration and commands are in `src-tauri/src/services/settings.rs` and `commands/settings.rs`. |
| Screenshot hiding | ✅ Keep | `src-tauri/src/lib.rs` monitors screenshot shortcuts and publishes capture state; settings are in `src-tauri/src/models/settings.rs`. |
| Window control | ✅ Keep | Tauri window APIs and Windows-specific interaction handling are in `src-tauri/src/commands/window.rs`. Native HWND/Direct2D lifecycle is not portable to this layer. |
| DPI / displays | ✅ Keep, verify on hardware | Tauri monitor enumeration and physical positioning are implemented in `commands/monitor.rs`, `commands/window.rs`, and startup positioning in `src-tauri/src/lib.rs`. Mixed-DPI behavior remains an acceptance test. |
| Persistence | 🔄 Compare semantics only | Tauri settings are modeled in `src-tauri/src/models/settings.rs` and persisted by `services/settings.rs`. Native `crates/isle-app/src/configuration.rs` and `crates/isle-core/src/settings.rs` use a different model; atomic-write ideas may be referenced without replacing the schema. |
| Native UI | 🗑 Do not migrate | `crates/isle-ui` and Native rendering/window modules serve a separate Win32 application. Keep all settings and island views in Svelte. |
| Duplicate dependencies | 🔄 Avoid importing wholesale | Both Rust sides use Windows crate 0.52; Native pins `=0.52.0`, while Tauri specifies `0.52`. Native's `tungstenite = 0.30.0` and Direct2D stack are not needed for the Tauri app unless a specific feature justifies them. |
| Duplicate Rust modules | 🔄 Port only proven gaps | Audio, media, settings, window, and display responsibilities already exist in Tauri. Compare one behavior at a time; do not copy modules wholesale. |
| Removable code | 🗑 None identified for immediate deletion | No broad cleanup is justified by this audit. Preserve current Svelte/Tauri layers. |

## Required questions

1. **What stays unchanged?** Tauri 2, Svelte UI, current island/settings surfaces,
   command/service layering, media session handling, and existing Windows system
   integrations remain the product baseline.
2. **What is newer or more stable in Native?** The committed Native source has a
   more explicit player extension/capability model and a local-CDP path for
   Netease playback-mode controls. This does not establish that those pieces are
   drop-in replacements or that real-client behavior is fully verified.
3. **Which Rust modules should move?** No whole module is approved for direct
   copying. First compare the Netease CDP mode operations and player capability
   model against the existing Tauri commands, then port only an isolated gap.
4. **Which overlapping implementation wins?** Tauri stays authoritative for
   SMTC, volume, startup, screenshot state, monitor selection, and windows unless
   a documented behavior gap is demonstrated.
5. **Which Native UI code must not move?** `crates/isle-ui`, Native settings
   rendering, Direct2D rendering, and Native HWND/window lifecycle code.
6. **Dependency conflicts?** Both use Windows crate 0.52, though Native pins the
   patch version. Native-only Tokio-free CDP/WebSocket and Direct2D dependencies
   should not be added without a concrete migration need. Tauri uses npm scripts;
   Native is a separate Cargo workspace.
7. **Configuration differences?** Tauri `AppPreferences` and Native settings
   have different structures and ownership. Preserve the Tauri settings schema;
   consider Native atomic-write/concurrency techniques separately.
8. **Highest risks?** Real Netease client compatibility, CDP endpoint lifecycle,
   settings-schema changes, and mixed-DPI/multiple-monitor positioning.
9. **Safest first migration?** A read-only behavior comparison of player
   capability classification and Netease mode handling. Do not add CDP until
   its lifecycle and real-client tests are clear.
10. **What needs retesting?** Netease play/pause/skip/mode changes, common SMTC
    players, artwork and timeline updates, screenshot/fullscreen hiding, startup
    behavior, volume synchronization, window placement, and mixed-DPI displays.

## Baseline issues found

- `src-tauri/tauri.conf.json` previously called `npm run web:dev` and
  `npm run web:build`, but `package.json` defines `dev` and `build`. These hooks
  must use the defined script names before Tauri launch/build can be validated.
- Frontend, Tauri, and Cargo versions were 1.0.11, 1.0.10, and 1.0.10. Tauri and
  Cargo metadata are being aligned to 1.0.11.
- The user directed that the UI must not be transparent. Island, floating-player,
  and timer windows therefore use opaque window surfaces; the original plan's
  transparent-window requirement is superseded.
- Static configuration does not prove runtime behavior. Startup/build, visual
  opacity, topmost/taskbar behavior, and DPI/multi-monitor acceptance remain
  unverified until run on Windows.

## First migration batch

No Native code has been migrated by this audit. The first candidate for a later,
separate change is Netease playback-mode control through a bounded local CDP
connection, after endpoint ownership and real-client tests are established.
SMTC, volume, startup, screenshot hiding, window placement, and display support
already have Tauri implementations and should only change for a demonstrated
gap.

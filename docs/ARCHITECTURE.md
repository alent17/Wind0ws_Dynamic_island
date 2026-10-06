# Architecture

This repository is the Tauri 2 + Svelte product. The Windows Native project is
a reference for backend behavior only; UI, layout, and animation stay in this
repository.

## Runtime layers

```text
Svelte windows
  App.svelte / IslandSurface
  Studio.svelte / FloatingWindow.svelte / TimerWindow.svelte
        │ typed invoke wrappers and Tauri events
        ▼
Tauri commands (`src-tauri/src/commands`)
        │
Rust services, models, and shared AppState
  services/  models/  state/  event_bus.rs
        │
Windows APIs (GSMTC, Core Audio, registry, Win32)
```

### Frontend

- `src/App.svelte` composes island behavior, monitor placement, and the media,
  audio, timer, idle, and capture integrations.
- `src/lib/IslandSurface.svelte` and its child components own island rendering
  and interaction affordances.
- `src/lib/islandGeometry.ts` contains geometry, placement, and work-area scale
  calculations; `src/lib/islandMotion.ts` defines motion timing and easing.
- `src/lib/islandStore.ts` owns the main island's expanded, hover, and selected
  page state. Use `transitionIsland` for those interaction changes.
- `src/lib/settingsStore.ts` owns the main window's normalized settings
  snapshot. It listens before loading the persisted snapshot so a newer settings
  event cannot be overwritten by an older initial read.
- `src/lib/api/` is the typed frontend boundary for Tauri commands. New invoke
  calls should be added there instead of embedded in UI components.
- `src/lib/mediaStore.ts` provides reference-counted media-event wiring for
  consumers that use that store. The island currently has additional media
  lifecycle logic in `App.svelte`; do not assume `mediaStore` is the only media
  source until that migration is complete.
- `src/utils/eventConstants.ts` names shared event channels, and
  `src/utils/eventManager.ts` provides listener cleanup and throttling.

### Backend

- `src-tauri/src/commands/` exposes Tauri invoke endpoints.
- `src-tauri/src/services/` contains Windows integrations and application
  services.
- `src-tauri/src/models/` defines serialized command/event data.
- `src-tauri/src/state/app_state.rs` owns state shared across Rust commands.
- `src-tauri/src/event_bus.rs` centralizes event names and broadcasts backend
  updates to webview windows.

The backend owns operating-system operations and persistence. Svelte owns
layout, presentation, and animation. Window geometry that must stay aligned
with the rendered island is computed in the frontend and applied through the
window API wrappers.

## Important event/data flows

| Flow | Backend/frontend boundary | Frontend consumer |
|---|---|---|
| Media snapshot and updates | `get_media_info`, `media-update` | `App.svelte`; `mediaStore` for subscribed consumers |
| Settings load and changes | `get_settings`, `update_settings`, `settings-updated` | `settingsStore` in the main window; Studio keeps an edit draft |
| Audio state and volume | audio commands and state responses | audio API wrappers and island volume panel |
| Timer window bridge | `timer-request-state`, `timer-state-changed`, `timer-action` | main island and timer window |
| Capture visibility | `capture-mode-changed` | main window visibility policy |
| Window layout | monitor/window commands | placement and geometry logic in `App.svelte` |

## Development constraints

- Keep the island face opaque; only the native host surface should be
  transparent to avoid a rectangular background.
- Do not move the Svelte UI or animation into the Native reference project.
- Keep invoke calls behind `src/lib/api/` wrappers.
- Keep event handlers and intervals paired with cleanup; asynchronous setup
  should use `src/lib/asyncCleanup.ts` where it fits.
- Preserve DPI-aware geometry in CSS/logical units and translate through the
  monitor APIs at the native boundary.

## Verification boundaries

`pnpm check` and `pnpm build` verify frontend diagnostics and bundling. They do
not prove Windows playback compatibility, capture behavior, mixed-DPI layout,
sleep recovery, release startup, or long-run CPU/memory behavior. Record those
as device acceptance work in the development plan rather than inferring them
from source inspection.

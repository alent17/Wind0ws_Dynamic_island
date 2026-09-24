---
name: Isle
description: The Windows media island with a dark control studio and white inspection stage.
colors:
  island-black: "#000000"
  island-white: "#ffffff"
  island-secondary: "rgba(255,255,255,.8)"
  clock-orange: "#f28b31"
  clock-minute: "#f7bd76"
  clock-caption: "#c9c1b8"
  studio-ink: "#f5f6f7"
  studio-surface: "#181a1e"
  studio-hairline: "#35373d"
  studio-accent: "#84a8ff"
typography:
  title:
    fontFamily: "MiSans"
    fontSize: "18px"
    fontWeight: 700
  body:
    fontFamily: "MiSans"
    fontSize: "14px"
  display:
    fontFamily: "MiSans"
    fontSize: "52px"
    fontWeight: 700
  clock-compact:
    fontFamily: "MiSans"
    fontSize: "33px"
    fontWeight: 700
  micro:
    fontSize: "11px"
  label:
    fontSize: "12px"
  caption:
    fontSize: "12px"
  data:
    fontSize: "13px"
  compact-title:
    fontSize: "13px"
rounded:
  island-compact: "14px"
  island-expanded: "45px"
  floating-shell: "5px"
  studio-panel: "16px"
components:
  island-compact:
    backgroundColor: "{colors.island-black}"
    textColor: "{colors.island-white}"
    rounded: "{rounded.island-compact}"
    width: "80–300px"
    height: "28px"
  island-hover:
    width: "90px"
    height: "30px"
  island-expanded:
    backgroundColor: "{colors.island-black}"
    textColor: "{colors.island-white}"
    rounded: "{rounded.island-expanded}"
    width: "300px"
    height: "160px"
  studio-panel:
    backgroundColor: "{colors.studio-surface}"
    textColor: "{colors.studio-ink}"
    rounded: "{rounded.studio-panel}"
    padding: "18px"
---

# Design System: Isle

## Overview

The production player surfaces use the project's original visual language. The main island is a very small black pill that grows into a 300 × 160 media controller. The separate floating player is a cover-first square stage with a compact black information strip and hover-revealed controls. Isle Studio uses the island's dark shell and bright active controls around a white inspection stage. It must not be replaced by the legacy settings page.

## Typography

MiSans is the only shipped typeface across Studio, the island, the floating player, the timer, and pixel-art mode. Regular, Medium, and Bold weights carry hierarchy. Legacy font preferences are read for compatibility but render as MiSans. Studio labels and metadata stay legible at compact desktop sizes.

## Dynamic Island

- Compact: 80–300 × 28 px; Hover adds 10 px up to 300 px; Expanded: 300 × 160 px.
- Compact content uses a 20 px cover and six-bar Canvas spectrum.
- Expanded content uses a 52 px cover, marquee title, artist, larger spectrum, source progress, three playback buttons, and the floating-player button.
- Width, height, opacity, and content changes use the original Svelte spring and drop/flip motion.
- The original black visual style, configurable expanded radius, auto-hide, reduced-motion, and debug-display behavior are retained.

## Floating Player

- The window is resizable from 200 × 200 px and restores its saved size and position.
- The cover stage fills the area above the 60 px song-information strip.
- Hover reveals the 25 px drag toolbar, source-specific progress layer, and centered playback overlay.
- Preserve high-resolution cover lookup, cover slide transitions, MV playback, pixel-art rendering, halftone overlay, lock, pin, close, and resize behavior.
- The shell uses the original 5 px radius and three-pixel black border.

## Isle Studio

Isle Studio shares the island's black and white control language. Its white stage remains the visual test surface so clipping, corners, and radius failures are visible. Inspector panels are charcoal, selected buttons are white, and the muted blue accent (`#84a8ff`) is reserved for live/editing state and keyboard focus. The native preview follows current media and weather; the browser preview uses labeled sample data. The island preview stays black even when the floating player has a custom fill color. Do not restore the archived Settings page or its HTML/build entry.

Studio uses an 8/12/16/20 px spacing rhythm: labels and controls form tight groups, related settings separate by 16 px, and inspector panels use 20 px padding on desktop. Controls distinguish rest, hover, selected, disabled, and keyboard focus with surface, border, and outline changes. On narrow windows, the Apply bar follows the controls so it cannot cover a slider.

## Compatibility

The persisted settings object contains both Studio-visible preferences and the legacy player fields. Missing fields are populated from defaults during deserialization, so reduced settings files from the interim architecture remain readable. Both the current preference commands and legacy settings commands address the same stored object.

## Comic Clock

The function-area clock uses MiSans Bold with tabular numbers, warm orange offset ink, and a white date strip. Use the smaller 33 px variant beside the weather forecast; keep surrounding player controls unchanged.

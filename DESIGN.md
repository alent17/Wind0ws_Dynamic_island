---
name: Isle
description: The restored original Windows media island with a white visual inspection studio.
colors:
  island-black: "#000000"
  island-white: "#ffffff"
  island-secondary: "rgba(255,255,255,.8)"
  studio-ink: "#111113"
  studio-surface: "#f5f5f7"
  studio-hairline: "#e5e5e8"
typography:
  title:
    fontFamily: "MiSans, Segoe UI, sans-serif"
    fontSize: "18px"
    fontWeight: 700
  body:
    fontFamily: "MiSans, Segoe UI, sans-serif"
    fontSize: "14px"
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

The production player surfaces use the project's original visual language. The main island is a very small black pill that grows into a 300 × 160 media controller. The separate floating player is a cover-first square stage with a compact black information strip and hover-revealed controls. Isle Studio remains a white inspection and settings surface and must not be replaced by the legacy settings page.

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

The current white Isle Studio is authoritative and stays visually unchanged. Its white stage, gray inspector panels, preview scenarios, behavior controls, and tools remain separate from the restored legacy player implementation. Do not restore the archived Settings page or its HTML/build entry.

## Compatibility

The persisted settings object contains both Studio-visible preferences and the legacy player fields. Missing fields are populated from defaults during deserialization, so reduced settings files from the interim architecture remain readable. Both the current preference commands and legacy settings commands address the same stored object.

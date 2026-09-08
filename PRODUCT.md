# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Windows users who keep music playing while working, gaming, or using full-screen apps and want glanceable playback status without opening the player.

## Product Purpose

Isle turns Windows media-session data into a compact, top-of-screen music surface. Success means playback information and controls remain readable, immediate, and unobtrusive across compact, expanded, hidden, studio-preview, and floating-player contexts.

## Positioning

Unlike a generic desktop mini-player, Isle combines the original compact top-edge island with a resizable, cover-first floating player and a separate visual test studio.

## Operating Context

The app runs as an always-available Tauri desktop utility, reads Windows media sessions, exposes playback controls, supports multiple monitors and full-screen detection, and can open a separate floating player.

## Capabilities and Constraints

- Preserve Windows media-session playback, spectrum, cover enrichment, cache, multi-monitor placement, always-on-top behavior, auto-start, and floating-window geometry.
- The main island has Compact, Hover, Expanded, and Hidden states.
- The old settings page remains replaced by Isle Studio, while the legacy player preference fields remain available as a compatibility layer.
- Preserve the original source-specific progress, cover enrichment, MV, pixel-art, halftone, theme, and floating-window behavior.
- Apple proprietary assets and bundled SF Pro fonts are not used.

## Brand Commitments

- Product name: Isle.
- The main surface uses the original 80 × 28 pill, 90 × 30 hover state, and 300 × 160 expanded state.
- Isle Studio is evaluated on a white canvas so clipping, square black corners, and radius failures remain visible.
- The floating player keeps the original cover-stage composition, hover toolbar, progress overlay, and compact five-pixel outer radius.

## Evidence on Hand

The repository contains the existing Svelte/Tauri application, source-player icons, Windows media integration, spectrum capture, cover handling, cache services, and the prior UI implementation. No testimonials or commercial performance claims are available and none should be invented.

## Product Principles

- Preserve the recognizable original player surfaces and their source-specific behavior.
- Keep Isle Studio visually independent from legacy player settings.
- Migrate persisted settings without discarding either current Studio fields or legacy playback options.

## Accessibility & Inclusion

Keyboard operation, visible focus, reduced-motion support, robust truncation for long multilingual titles, and sufficient contrast are required.

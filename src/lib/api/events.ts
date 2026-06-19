export const Events = {
  MEDIA_UPDATE: "media-update",
  AUDIO_SPECTRUM: "audio-spectrum",
  SETTINGS_CHANGED: "settings-changed",
  CORNER_RADIUS_CHANGED: "corner-radius-changed",
  FLOATING_WINDOW_CLOSED: "floating-window-closed",
} as const;

export const ThrottledEvents = {
  [Events.MEDIA_UPDATE]: { interval: 500 },
  [Events.AUDIO_SPECTRUM]: { interval: 50 },
} as const;

export type EventType = (typeof Events)[keyof typeof Events];

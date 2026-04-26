export const Events = {
  MEDIA_UPDATE: "media_update",
  AUDIO_SPECTRUM: "audio_spectrum",
  SETTINGS_CHANGED: "settings_changed",
  CORNER_RADIUS_CHANGED: "corner_radius_changed",
  FLOATING_WINDOW_CLOSED: "floating_window_closed",
} as const;

export const ThrottledEvents = {
  [Events.MEDIA_UPDATE]: { interval: 500 },
  [Events.AUDIO_SPECTRUM]: { interval: 50 },
} as const;

export type EventType = (typeof Events)[keyof typeof Events];

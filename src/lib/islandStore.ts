import { derived, writable } from "svelte/store";
import type { IslandMode, IslandPage } from "$lib/islandGeometry";

export interface IslandState {
  expanded: boolean;
  hovering: boolean;
  activePage: IslandPage;
  hasMediaSession: boolean;
}

export type IslandViewState =
  | { type: "idle" }
  | { type: "media" }
  | { type: "timer" }
  | { type: "volume" }
  | { type: "clock" }
  | { type: "weather" }
  | { type: "expanded"; page: IslandPage };

export type IslandEvent =
  | { type: "toggle" }
  | { type: "expand" }
  | { type: "collapse" }
  | { type: "hover-enter" }
  | { type: "hover-leave" }
  | { type: "media-session"; active: boolean }
  | { type: "select-page"; page: IslandPage };

const initialState: IslandState = {
  expanded: false,
  hovering: false,
  activePage: "music",
  hasMediaSession: false,
};

export const islandState = writable<IslandState>(initialState);

export const islandMode = derived(islandState, ({ expanded, hovering }): IslandMode =>
  expanded ? "expanded" : hovering ? "hover" : "compact",
);

export const islandView = derived(islandState, ({ expanded, activePage, hasMediaSession }): IslandViewState => {
  if (expanded) return { type: "expanded", page: activePage };
  if (activePage !== "music") return { type: activePage };
  return hasMediaSession ? { type: "media" } : { type: "idle" };
});

/** Apply one named interaction to the island's shared UI state. */
export function transitionIsland(event: IslandEvent): void {
  islandState.update((state) => {
    switch (event.type) {
      case "toggle":
        return { ...state, expanded: !state.expanded };
      case "expand":
        return state.expanded ? state : { ...state, expanded: true };
      case "collapse":
        return state.expanded ? { ...state, expanded: false } : state;
      case "hover-enter":
        return state.hovering ? state : { ...state, hovering: true };
      case "hover-leave":
        return state.hovering ? { ...state, hovering: false } : state;
      case "media-session":
        return state.hasMediaSession === event.active
          ? state
          : { ...state, hasMediaSession: event.active };
      case "select-page":
        return state.activePage === event.page ? state : { ...state, activePage: event.page };
    }
  });
}

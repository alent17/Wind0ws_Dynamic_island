import { derived, writable } from "svelte/store";
import type { IslandMode, IslandPage } from "$lib/islandGeometry";

export interface IslandState {
  expanded: boolean;
  hovering: boolean;
  activePage: IslandPage;
}

export type IslandEvent =
  | { type: "toggle" }
  | { type: "expand" }
  | { type: "collapse" }
  | { type: "hover-enter" }
  | { type: "hover-leave" }
  | { type: "select-page"; page: IslandPage };

const initialState: IslandState = {
  expanded: false,
  hovering: false,
  activePage: "music",
};

export const islandState = writable<IslandState>(initialState);

export const islandMode = derived(islandState, ({ expanded, hovering }): IslandMode =>
  expanded ? "expanded" : hovering ? "hover" : "compact",
);

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
      case "select-page":
        return state.activePage === event.page ? state : { ...state, activePage: event.page };
    }
  });
}

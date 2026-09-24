import { describe, expect, it } from "vitest";
import { moveSelectedPlayer } from "./playerOrder";

describe("player priority", () => {
  it("uses visible order across consecutive moves", () => {
    const selected = ["a", "b", "c"];
    const first = moveSelectedPlayer(["a", "b", "c", "off"], selected, "b", -1);
    expect(first).toEqual(["b", "a", "c", "off"]);
    expect(moveSelectedPlayer(first, selected, "b", 1)).toEqual(["a", "b", "c", "off"]);
  });

  it("keeps unchecked players outside the priority controls", () => {
    expect(moveSelectedPlayer(["a", "off", "b"], ["a", "b"], "a", 1))
      .toEqual(["b", "a", "off"]);
  });
});

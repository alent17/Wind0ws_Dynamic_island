import { describe, expect, it } from "vitest";
import { rollingDigitDelta } from "./rollingNumber";

describe("rolling digit movement", () => {
  it("takes the short forward path through zero", () => {
    expect(rollingDigitDelta(9, 0)).toBe(1);
  });

  it("takes the short backward path through zero", () => {
    expect(rollingDigitDelta(0, 9)).toBe(-1);
  });

  it("leaves unchanged digits still and uses a stable tie direction", () => {
    expect(rollingDigitDelta(4, 4)).toBe(0);
    expect(rollingDigitDelta(0, 5)).toBe(-5);
  });

  it("clamps invalid digit inputs before choosing a direction", () => {
    expect(rollingDigitDelta(-2, 2)).toBe(2);
    expect(rollingDigitDelta(8, 18)).toBe(1);
  });
});

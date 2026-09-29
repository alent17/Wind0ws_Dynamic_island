import { describe, expect, it, vi } from "vitest";
import { createAsyncCleanup } from "./asyncCleanup";

describe("async cleanup", () => {
  it("disposes registered callbacks once", () => {
    const cleanup = createAsyncCleanup();
    const dispose = vi.fn();
    cleanup.add(dispose);
    cleanup.dispose();
    cleanup.dispose();
    expect(dispose).toHaveBeenCalledTimes(1);
  });

  it("immediately disposes callbacks that arrive after teardown", () => {
    const cleanup = createAsyncCleanup();
    const dispose = vi.fn();
    cleanup.dispose();
    cleanup.add(dispose);
    expect(cleanup.disposed).toBe(true);
    expect(dispose).toHaveBeenCalledTimes(1);
  });
});

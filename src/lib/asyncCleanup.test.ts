import { expect, it, vi } from "vitest";
import { createAsyncCleanup } from "./asyncCleanup";

it("disposes existing and late listeners exactly once", () => {
  const registry = createAsyncCleanup();
  const existing = vi.fn();
  const late = vi.fn();
  registry.add(existing);
  registry.dispose();
  registry.add(late);
  registry.dispose();

  expect(existing).toHaveBeenCalledOnce();
  expect(late).toHaveBeenCalledOnce();
  expect(registry.disposed).toBe(true);
});

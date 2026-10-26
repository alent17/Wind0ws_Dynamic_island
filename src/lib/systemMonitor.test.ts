import { expect, it } from "vitest";
import { formatBytes, usagePercent } from "./systemMonitor";
import { normalizeWidgets } from "./widgetLayout";
it("formats network rates and rejects invalid sensor readings", () => {
  expect(formatBytes(0,true)).toBe("0 B/s");
  expect(formatBytes(1024**2*2.5,true)).toBe("2.5 MB/s");
  expect(formatBytes(NaN)).toBe("--");
  expect(usagePercent(null)).toBeNull();
  expect(usagePercent(NaN)).toBeNull();
  expect(usagePercent(103)).toBe(100);
  expect(normalizeWidgets({left:["system"],right:["system","clock"]})).toEqual({left:["system"],right:["clock"]});
});

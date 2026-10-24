import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

describe("weather snapshot API", () => {
  let idleApi: typeof import("./idle").idleApi;

  beforeEach(async () => {
    vi.useRealTimers();
    vi.resetModules();
    invokeMock.mockReset();
    ({ idleApi } = await import("./idle"));
  });

  it("keeps a one-entry cache keyed by location and separate from full system snapshots", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-20T00:00:00Z"));
    const weather = { weatherTemperature: 18, weatherCode: 2, weatherUpdatedAt: 1_800_000_000, weatherForecast: [] };
    invokeMock.mockImplementation(async (command: string) => command === "get_weather_snapshot" ? weather : {
      cpuPercent: 50, memoryPercent: 60, uploadBytesPerSecond: 7, downloadBytesPerSecond: 8,
      batteryPercent: 90, batteryCharging: true, weatherTemperature: 18, weatherCode: 2,
      weatherUpdatedAt: 1_800_000_000, weatherForecast: [],
    });

    expect(await idleApi.getWeatherSnapshot("city-a")).toEqual(weather);
    await idleApi.getWeatherSnapshot("city-a");
    expect(invokeMock.mock.calls.map(([command]) => command)).toEqual(["get_weather_snapshot"]);

    const systemSnapshot = await idleApi.getSnapshot("city-a");
    expect(systemSnapshot.cpuPercent).toBe(50);
    expect(systemSnapshot.batteryPercent).toBe(90);
    expect(invokeMock.mock.calls.map(([command]) => command)).toEqual(["get_weather_snapshot", "get_idle_snapshot"]);

    await idleApi.getWeatherSnapshot("city-b");
    await idleApi.getWeatherSnapshot("city-a");
    expect(invokeMock.mock.calls.map(([command]) => command)).toEqual([
      "get_weather_snapshot", "get_idle_snapshot", "get_weather_snapshot", "get_weather_snapshot",
    ]);

    vi.advanceTimersByTime(30_000);
    await idleApi.getWeatherSnapshot("city-a");
    expect(invokeMock).toHaveBeenCalledTimes(5);
  });

  it("single-flights simultaneous requests for a location across key changes", async () => {
    const resolvers: Array<(value: unknown) => void> = [];
    invokeMock.mockImplementation(() => new Promise(resolve => resolvers.push(resolve)));

    const cityA = idleApi.getWeatherSnapshot("city-a");
    const cityB = idleApi.getWeatherSnapshot("city-b");
    const cityAAgain = idleApi.getWeatherSnapshot("city-a");
    expect(cityAAgain).toBe(cityA);
    expect(invokeMock).toHaveBeenCalledTimes(2);

    const valueA = { weatherTemperature: 20, weatherCode: 1, weatherUpdatedAt: 1, weatherForecast: [] };
    const valueB = { weatherTemperature: 9, weatherCode: 61, weatherUpdatedAt: 2, weatherForecast: [] };
    resolvers[0](valueA);
    resolvers[1](valueB);
    await expect(cityA).resolves.toEqual(valueA);
    await expect(cityB).resolves.toEqual(valueB);
  });

  it("drops rejected pending entries so the same location can retry", async () => {
    const weather = { weatherTemperature: null, weatherCode: null, weatherUpdatedAt: null, weatherForecast: [] };
    invokeMock.mockRejectedValueOnce(new Error("offline")).mockResolvedValueOnce(weather);

    await expect(idleApi.getWeatherSnapshot("city-a")).rejects.toThrow("offline");
    await expect(idleApi.getWeatherSnapshot("city-a")).resolves.toEqual(weather);
    expect(invokeMock).toHaveBeenCalledTimes(2);
    expect(invokeMock.mock.calls.map(([command]) => command)).toEqual(["get_weather_snapshot", "get_weather_snapshot"]);
  });

  it("bounds distinct in-flight weather keys and recovers when one settles", async () => {
    const resolvers: Array<(value: unknown) => void> = [];
    invokeMock.mockImplementation(() => new Promise(resolve => resolvers.push(resolve)));
    const requests = ["a", "b", "c", "d"].map(key => idleApi.getWeatherSnapshot(key));
    await expect(idleApi.getWeatherSnapshot("e")).rejects.toThrow("Too many weather locations are pending");
    expect(invokeMock).toHaveBeenCalledTimes(4);

    const weather = { weatherTemperature: 16, weatherCode: 3, weatherUpdatedAt: 3, weatherForecast: [] };
    resolvers[0](weather);
    await expect(requests[0]).resolves.toEqual(weather);
    const retry = idleApi.getWeatherSnapshot("e");
    expect(invokeMock).toHaveBeenCalledTimes(5);
    resolvers[4](weather);
    await expect(retry).resolves.toEqual(weather);
  });
});

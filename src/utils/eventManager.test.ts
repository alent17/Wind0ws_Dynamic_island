import { afterEach, describe, expect, it, vi } from "vitest";
const bridge = vi.hoisted(() => ({ listen: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => bridge);
import { EventManager, onMediaUpdate, onThrottled } from "./eventManager";
import { Events } from "./eventConstants";
afterEach(() => { vi.useRealTimers(); bridge.listen.mockReset(); });
describe("EventManager teardown", () => {
  it("releases registration arriving after repeated destroy exactly once", async () => {
    let complete!: (value: () => void) => void;
    bridge.listen.mockImplementation(() => new Promise(resolve => complete = resolve));
    const manager = new EventManager(), handler = vi.fn(), unlisten = vi.fn();
    const pending = manager.on("late", handler, { debounce: 20 });
    manager.destroy(); manager.destroy();
    bridge.listen.mock.calls[0][1]({ payload: "late" });
    complete(unlisten);
    const cleanup = await pending; cleanup(); cleanup(); manager.destroy();
    expect(unlisten).toHaveBeenCalledTimes(1);
    expect(manager.getListenerCount()).toBe(0);
    expect(handler).not.toHaveBeenCalled();
  });
  it("cancels pending debounce and releases active listeners once", async () => {
    vi.useFakeTimers(); const unlisten = vi.fn(); bridge.listen.mockResolvedValue(unlisten);
    const manager = new EventManager(), handler = vi.fn();
    const cleanup = await manager.on("pending", handler, { debounce: 20 });
    bridge.listen.mock.calls[0][1]({ payload: 1 });
    manager.destroy(); cleanup(); vi.runAllTimers();
    expect(handler).not.toHaveBeenCalled(); expect(unlisten).toHaveBeenCalledTimes(1);
  });
});

describe("media update delivery", () => {
  it("delivers every closely-spaced media state and delta to independent consumers in order", async () => {
    vi.useFakeTimers(); vi.setSystemTime(1_000);
    const registrations:Array<{callback:(event:{payload:unknown})=>void;active:boolean;dispose:ReturnType<typeof vi.fn>}>=[];
    bridge.listen.mockImplementation((_event, callback) => {
      const row={callback,active:true,dispose:vi.fn()};
      row.dispose.mockImplementation(()=>{row.active=false;});
      registrations.push(row);
      return Promise.resolve(row.dispose);
    });
    const first:unknown[]=[]; const second:unknown[]=[];
    const releaseFirst=await onMediaUpdate(payload=>first.push(payload));
    const releaseSecond=await onMediaUpdate(payload=>second.push(payload));
    const updates=[
      {isPlaying:true,positionMs:0},
      {isPlaying:false,positionMs:0},
      {isPlaying:false,positionMs:2_000},
      {isPlaying:false,positionMs:2_000,albumArt:"cover-A"},
      {isPlaying:false,positionMs:2_000}, // omitted albumArt is a meaningful delta
      {isPlaying:false,positionMs:2_000,albumArt:""}, // explicit clear
    ];
    for(let i=0;i<updates.length;i++){
      if(i) await vi.advanceTimersByTimeAsync(100);
      for(const row of registrations) if(row.active) row.callback({payload:updates[i]});
    }
    expect(first).toEqual(updates);
    expect(second).toEqual(updates);
    releaseFirst(); releaseSecond();
    for(const row of registrations) if(row.active) row.callback({payload:{isPlaying:true}});
    expect(first).toHaveLength(updates.length);
    expect(second).toHaveLength(updates.length);
    expect(registrations.every(row=>row.dispose.mock.calls.length===1)).toBe(true);
  });

  it("isolates a throwing consumer without blocking another media listener", async () => {
    const callbacks:Array<(event:{payload:unknown})=>void>=[];
    bridge.listen.mockImplementation((_event,callback)=>{callbacks.push(callback);return Promise.resolve(vi.fn());});
    const errors=vi.spyOn(console,"error").mockImplementation(()=>{});
    const healthy=vi.fn();
    const releaseThrowing=await onMediaUpdate(()=>{throw new Error("consumer failed");});
    const releaseHealthy=await onMediaUpdate(healthy);
    callbacks.forEach(callback=>callback({payload:{isPlaying:false}}));
    expect(errors).toHaveBeenCalledTimes(1);
    expect(healthy).toHaveBeenCalledWith({isPlaying:false});
    releaseThrowing();releaseHealthy();errors.mockRestore();
  });

  it("keeps explicit throttling available for other low-priority events", async () => {
    vi.useFakeTimers(); vi.setSystemTime(1_000);
    const callback=vi.fn();let emit!:(event:{payload:number})=>void;
    bridge.listen.mockImplementation((_event,listener)=>{emit=listener;return Promise.resolve(vi.fn());});
    const release=await onThrottled(Events.AUDIO_SPECTRUM,callback,50);
    emit({payload:1});
    await vi.advanceTimersByTimeAsync(25);emit({payload:2});
    await vi.advanceTimersByTimeAsync(25);emit({payload:3});
    expect(callback.mock.calls.map(([value])=>value)).toEqual([1,3]);
    release();
  });
});

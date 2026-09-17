import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Events, ThrottledEvents, EventPriority } from "./eventConstants";

interface ListenerConfig {
  event: string;
  handler: (payload: any) => unknown;
  options?: {
    debounce?: number;
    throttle?: number;
    once?: boolean;
    priority?: number;
  };
}

interface ListenerEntry {
  id: number;
  unlisten: UnlistenFn;
  config: ListenerConfig;
}

class EventManager {
  private listeners: Map<string, ListenerEntry[]> = new Map();
  private pendingTimers: Map<string, ReturnType<typeof setTimeout>> = new Map();
  private lastEmitTime: Map<string, number> = new Map();
  private nextListenerId = 1;
  private isDestroyed = false;

  // 注册监听器
  async on(
    event: string,
    handler: (payload: any) => unknown,
    options?: ListenerConfig["options"]
  ): Promise<() => void> {
    if (this.isDestroyed) {
      console.warn(`[EventManager] 已销毁，无法监听事件: ${event}`);
      return () => {};
    }

    const config: ListenerConfig = { event, handler, options };
    const listenerId = this.nextListenerId++;

    // 包装处理器，添加防抖/节流。状态必须按监听器隔离，避免同一事件
    // 的两个消费者互相清掉对方的 debounce 或 throttle 状态。
    const wrappedHandler = this.wrapHandler(config, listenerId);

    try {
      const unlisten = await listen(event, (e: any) => {
        if (this.isDestroyed) return;
        wrappedHandler(e.payload);
      });

      const entry: ListenerEntry = { id: listenerId, unlisten, config };

      if (!this.listeners.has(event)) {
        this.listeners.set(event, []);
      }
      this.listeners.get(event)!.push(entry);

      // 返回清理函数
      return () => this.off(event, entry);
    } catch (error) {
      console.error(`[EventManager] 监听事件失败 ${event}:`, error);
      return () => {};
    }
  }

  // 注册一次性监听器
  async once(
    event: string,
    handler: (payload: any) => unknown
  ): Promise<() => void> {
    return this.on(event, handler, { once: true });
  }

  // 移除特定监听器
  off(event: string, entry: ListenerEntry): void {
    const eventListeners = this.listeners.get(event);
    if (!eventListeners) return;

    const index = eventListeners.indexOf(entry);
    if (index > -1) {
      eventListeners.splice(index, 1);
      entry.unlisten();
      this.clearListenerState(event, entry.id);
    }

    if (eventListeners.length === 0) {
      this.listeners.delete(event);
    }
  }

  // 移除事件的所有监听器
  offAll(event: string): void {
    const eventListeners = this.listeners.get(event);
    if (!eventListeners) return;

    eventListeners.forEach((entry) => {
      entry.unlisten();
      this.clearListenerState(event, entry.id);
    });
    this.listeners.delete(event);
  }

  // 销毁所有监听器
  destroy(): void {
    this.isDestroyed = true;

    this.listeners.forEach((eventListeners) => {
      eventListeners.forEach((entry) => entry.unlisten());
    });
    this.listeners.clear();

    this.pendingTimers.forEach((timer) => clearTimeout(timer));
    this.pendingTimers.clear();
    this.lastEmitTime.clear();
  }

  // 获取当前监听器数量
  getListenerCount(event?: string): number {
    if (event) {
      return this.listeners.get(event)?.length || 0;
    }

    let count = 0;
    this.listeners.forEach((entries) => {
      count += entries.length;
    });
    return count;
  }

  // 包装处理器，添加防抖/节流逻辑
  private wrapHandler(config: ListenerConfig, listenerId: number): (payload: any) => void {
    const { handler, options } = config;

    const invoke = (payload: any) => {
      try {
        const result = handler(payload);
        if (result && typeof (result as Promise<unknown>).catch === "function") {
          void (result as Promise<unknown>).catch((error) => {
            console.error(`[EventManager] 监听器异步处理失败 ${config.event}:`, error);
          });
        }
      } catch (error) {
        console.error(`[EventManager] 监听器处理失败 ${config.event}:`, error);
      }
    };

    if (!options) return invoke;

    // 一次性监听器
    if (options.once) {
      return (payload: any) => {
        invoke(payload);
        const entry = this.findEntryById(config.event, listenerId);
        if (entry) this.off(config.event, entry);
      };
    }

    // 防抖处理
    if (options.debounce && options.debounce > 0) {
      return (payload: any) => {
        const key = `debounce:${config.event}:${listenerId}`;
        this.clearTimer(key);

        const timer = setTimeout(() => {
          this.pendingTimers.delete(key);
          invoke(payload);
        }, options.debounce);

        this.pendingTimers.set(key, timer);
      };
    }

    // 节流处理
    if (options.throttle && options.throttle > 0) {
      return (payload: any) => {
        const now = Date.now();
        const key = `throttle:${config.event}:${listenerId}`;
        const lastTime = this.lastEmitTime.get(key) || 0;
        const interval = options.throttle as number;

        if (now - lastTime >= interval) {
          this.lastEmitTime.set(key, now);
          invoke(payload);
        }
      };
    }

    return handler;
  }

  private clearTimer(key: string): void {
    const timer = this.pendingTimers.get(key);
    if (timer) {
      clearTimeout(timer);
      this.pendingTimers.delete(key);
    }
  }

  private clearListenerState(event: string, listenerId: number): void {
    this.clearTimer(`debounce:${event}:${listenerId}`);
    this.lastEmitTime.delete(`throttle:${event}:${listenerId}`);
  }

  private findEntryById(event: string, listenerId: number): ListenerEntry | undefined {
    const eventListeners = this.listeners.get(event);
    if (!eventListeners) return undefined;

    return eventListeners.find((entry) => entry.id === listenerId);
  }
}

// 创建全局事件管理器实例
export const eventManager = new EventManager();

// 便捷函数：快速注册带自动节流的监听器
export async function onThrottled(
  event: string,
  handler: (payload: any) => unknown,
  customInterval?: number
): Promise<() => void> {
  const throttledConfig =
    ThrottledEvents[event as keyof typeof ThrottledEvents];
  const interval = customInterval || throttledConfig?.interval || 100;

  return eventManager.on(event, handler, { throttle: interval });
}

// 便捷函数：注册防抖监听器
export async function onDebounced(
  event: string,
  handler: (payload: any) => unknown,
  delay: number = 300
): Promise<() => void> {
  return eventManager.on(event, handler, { debounce: delay });
}

// 便捷函数：注册媒体更新监听器（自动节流）
export async function onMediaUpdate(
  handler: (payload: any) => unknown
): Promise<() => void> {
  return onThrottled(Events.MEDIA_UPDATE, handler, 500);
}

// 便捷函数：注册频谱监听器（自动节流）
export async function onAudioSpectrum(
  handler: (payload: any) => unknown
): Promise<() => void> {
  return onThrottled(Events.AUDIO_SPECTRUM, handler, 50);
}

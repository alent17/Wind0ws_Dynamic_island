import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Events, ThrottledEvents, EventPriority } from "./eventConstants";

interface ListenerConfig {
  event: string;
  handler: (payload: any) => void;
  options?: {
    debounce?: number;
    throttle?: number;
    once?: boolean;
    priority?: number;
  };
}

interface ListenerEntry {
  unlisten: UnlistenFn;
  config: ListenerConfig;
}

class EventManager {
  private listeners: Map<string, ListenerEntry[]> = new Map();
  private pendingTimers: Map<string, ReturnType<typeof setTimeout>> = new Map();
  private lastEmitTime: Map<string, number> = new Map();
  private isDestroyed = false;

  // 注册监听器
  async on(
    event: string,
    handler: (payload: any) => void,
    options?: ListenerConfig["options"]
  ): Promise<() => void> {
    if (this.isDestroyed) {
      console.warn(`[EventManager] 已销毁，无法监听事件: ${event}`);
      return () => {};
    }

    const config: ListenerConfig = { event, handler, options };

    // 包装处理器，添加防抖/节流
    const wrappedHandler = this.wrapHandler(config);

    try {
      const unlisten = await listen(event, (e: any) => {
        if (this.isDestroyed) return;
        wrappedHandler(e.payload);
      });

      const entry: ListenerEntry = { unlisten, config };

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
    handler: (payload: any) => void
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
    }

    if (eventListeners.length === 0) {
      this.listeners.delete(event);
    }
  }

  // 移除事件的所有监听器
  offAll(event: string): void {
    const eventListeners = this.listeners.get(event);
    if (!eventListeners) return;

    eventListeners.forEach((entry) => entry.unlisten());
    this.listeners.delete(event);

    // 清理相关定时器
    const timerKey = `debounce:${event}`;
    const throttleKey = `throttle:${event}`;
    this.clearTimer(timerKey);
    this.clearTimer(throttleKey);
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
  private wrapHandler(config: ListenerConfig): (payload: any) => void {
    const { handler, options } = config;

    if (!options) return handler;

    // 一次性监听器
    if (options.once) {
      return (payload: any) => {
        handler(payload);
        this.off(config.event, this.findEntry(config) as ListenerEntry);
      };
    }

    // 防抖处理
    if (options.debounce && options.debounce > 0) {
      return (payload: any) => {
        const key = `debounce:${config.event}`;
        this.clearTimer(key);

        const timer = setTimeout(() => {
          handler(payload);
        }, options.debounce);

        this.pendingTimers.set(key, timer);
      };
    }

    // 节流处理
    if (options.throttle && options.throttle > 0) {
      return (payload: any) => {
        const now = Date.now();
        const lastTime = this.lastEmitTime.get(config.event) || 0;
        const interval = options.throttle as number;

        if (now - lastTime >= interval) {
          this.lastEmitTime.set(config.event, now);
          handler(payload);
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

  private findEntry(config: ListenerConfig): ListenerEntry | undefined {
    const eventListeners = this.listeners.get(config.event);
    if (!eventListeners) return undefined;

    return eventListeners.find((entry) => entry.config === config);
  }
}

// 创建全局事件管理器实例
export const eventManager = new EventManager();

// 便捷函数：快速注册带自动节流的监听器
export async function onThrottled(
  event: string,
  handler: (payload: any) => void,
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
  handler: (payload: any) => void,
  delay: number = 300
): Promise<() => void> {
  return eventManager.on(event, handler, { debounce: delay });
}

// 便捷函数：注册媒体更新监听器（自动节流）
export async function onMediaUpdate(
  handler: (payload: any) => void
): Promise<() => void> {
  return onThrottled(Events.MEDIA_UPDATE, handler, 500);
}

// 便捷函数：注册频谱监听器（自动节流）
export async function onAudioSpectrum(
  handler: (payload: any) => void
): Promise<() => void> {
  return onThrottled(Events.AUDIO_SPECTRUM, handler, 50);
}

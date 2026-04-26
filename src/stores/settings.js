/**
 * Settings Store - 设置状态管理 (Svelte 5 Runes 语法 + 工厂模式)
 * 遵循 CLAUDE.md 规范：使用工厂函数 createSettingsStore()，隐藏原始 set/update 逻辑
 */

/**
 * 创建设置 Store 的工厂函数
 * @returns 设置状态和操作方法的封装对象
 */
function createSettingsStore() {
  // 内部私有状态
  let state = $state({
    autoStart: false,
    clickThrough: false,
    autoHide: false,
    alwaysOnTop: true,
    showTime: true,
    theme: 'dark', // dark, light, system
    language: 'zh-CN', // zh-CN, en-US
    cacheSize: 0,
    playerOrder: [] as string[]
  });

  // 返回封装后的公共 API
  return {
    // 只读状态访问
    get state() {
      return state;
    },
    
    // 设置操作
    setAutoStart(enable: boolean) { state.autoStart = enable; },
    setClickThrough(enable: boolean) { state.clickThrough = enable; },
    setAutoHide(enable: boolean) { state.autoHide = enable; },
    setAlwaysOnTop(enable: boolean) { state.alwaysOnTop = enable; },
    setShowTime(enable: boolean) { state.showTime = enable; },
    setTheme(theme: string) { state.theme = theme; },
    setLanguage(language: string) { state.language = language; },
    setCacheSize(size: number) { state.cacheSize = size; },
    clearCache() { state.cacheSize = 0; },
    setPlayerOrder(order: string[]) { state.playerOrder = order; },
    loadSettings(settings: any) { Object.assign(state, settings); }
  };
}

// 导出单例
export const settings = createSettingsStore();

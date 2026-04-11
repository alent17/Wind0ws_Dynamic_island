/**
 * Settings Store - 设置状态管理
 */
import { writable } from 'svelte/store';

export const settingsStore = writable({
  autoStart: false,
  clickThrough: false,
  autoHide: false,
  alwaysOnTop: true,
  showTime: true,
  theme: 'dark', // dark, light, system
  language: 'zh-CN', // zh-CN, en-US
  cacheSize: 0,
  playerOrder: []
});

// Actions
export const settingsActions = {
  setAutoStart: (enable) => {
    settingsStore.update(state => ({ ...state, autoStart: enable }));
  },
  
  setClickThrough: (enable) => {
    settingsStore.update(state => ({ ...state, clickThrough: enable }));
  },
  
  setAutoHide: (enable) => {
    settingsStore.update(state => ({ ...state, autoHide: enable }));
  },
  
  setAlwaysOnTop: (enable) => {
    settingsStore.update(state => ({ ...state, alwaysOnTop: enable }));
  },
  
  setShowTime: (enable) => {
    settingsStore.update(state => ({ ...state, showTime: enable }));
  },
  
  setTheme: (theme) => {
    settingsStore.update(state => ({ ...state, theme }));
  },
  
  setLanguage: (language) => {
    settingsStore.update(state => ({ ...state, language }));
  },
  
  setCacheSize: (size) => {
    settingsStore.update(state => ({ ...state, cacheSize: size }));
  },
  
  clearCache: () => {
    settingsStore.update(state => ({ ...state, cacheSize: 0 }));
  },
  
  setPlayerOrder: (order) => {
    settingsStore.update(state => ({ ...state, playerOrder: order }));
  },
  
  loadSettings: (settings) => {
    settingsStore.update(() => ({ ...settings }));
  }
};

export default settingsStore;

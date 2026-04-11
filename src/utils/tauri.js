/**
 * Tauri API 封装 - 统一的命令调用接口
 */
import { invoke } from '@tauri-apps/api/core';

/**
 * 窗口管理命令
 */
export const windowCommands = {
  /**
   * 切换悬浮窗显示/隐藏
   * @returns {Promise<void>}
   */
  toggleFloatingWindow: async () => {
    try {
      await invoke('toggle_floating_window');
    } catch (error) {
      console.error('Failed to toggle floating window:', error);
      throw error;
    }
  },

  /**
   * 设置点击穿透
   * @param {boolean} enable - 是否启用点击穿透
   * @returns {Promise<void>}
   */
  setClickThrough: async (enable) => {
    try {
      await invoke('set_click_through', { enable });
    } catch (error) {
      console.error('Failed to set click through:', error);
      throw error;
    }
  },

  /**
   * 设置始终置顶
   * @param {boolean} enable - 是否启用始终置顶
   * @returns {Promise<void>}
   */
  setAlwaysOnTop: async (enable) => {
    try {
      await invoke('set_always_on_top', { enable });
    } catch (error) {
      console.error('Failed to set always on top:', error);
      throw error;
    }
  },

  /**
   * 设置窗口大小
   * @param {number} width - 宽度
   * @param {number} height - 高度
   * @returns {Promise<void>}
   */
  setWindowSize: async (width, height) => {
    try {
      await invoke('set_window_size', { width, height });
    } catch (error) {
      console.error('Failed to set window size:', error);
      throw error;
    }
  },

  /**
   * 关闭窗口
   * @returns {Promise<void>}
   */
  closeWindow: async () => {
    try {
      await invoke('close_window');
    } catch (error) {
      console.error('Failed to close window:', error);
      throw error;
    }
  }
};

/**
 * 设置相关命令
 */
export const settingsCommands = {
  /**
   * 设置开机启动
   * @param {boolean} enable - 是否启用开机启动
   * @returns {Promise<void>}
   */
  setAutoStart: async (enable) => {
    try {
      await invoke('set_auto_start', { enable });
    } catch (error) {
      console.error('Failed to set auto start:', error);
      throw error;
    }
  },

  /**
   * 清除缓存
   * @returns {Promise<void>}
   */
  clearCache: async () => {
    try {
      await invoke('clear_cache');
    } catch (error) {
      console.error('Failed to clear cache:', error);
      throw error;
    }
  },

  /**
   * 获取缓存大小
   * @returns {Promise<number>}
   */
  getCacheSize: async () => {
    try {
      return await invoke('get_cache_size');
    } catch (error) {
      console.error('Failed to get cache size:', error);
      throw error;
    }
  }
};

/**
 * 播放器控制命令
 */
export const playerCommands = {
  /**
   * 播放/暂停
   * @returns {Promise<void>}
   */
  togglePlay: async () => {
    try {
      await invoke('toggle_play');
    } catch (error) {
      console.error('Failed to toggle play:', error);
      throw error;
    }
  },

  /**
   * 下一首
   * @returns {Promise<void>}
   */
  nextTrack: async () => {
    try {
      await invoke('next_track');
    } catch (error) {
      console.error('Failed to play next track:', error);
      throw error;
    }
  },

  /**
   * 上一首
   * @returns {Promise<void>}
   */
  previousTrack: async () => {
    try {
      await invoke('previous_track');
    } catch (error) {
      console.error('Failed to play previous track:', error);
      throw error;
    }
  },

  /**
   * 设置音量
   * @param {number} volume - 音量值 (0-100)
   * @returns {Promise<void>}
   */
  setVolume: async (volume) => {
    try {
      await invoke('set_volume', { volume });
    } catch (error) {
      console.error('Failed to set volume:', error);
      throw error;
    }
  }
};

/**
 * 系统相关命令
 */
export const systemCommands = {
  /**
   * 打开外部链接
   * @param {string} url - 要打开的 URL
   * @returns {Promise<void>}
   */
  openUrl: async (url) => {
    try {
      await invoke('open_url', { url });
    } catch (error) {
      console.error('Failed to open URL:', error);
      throw error;
    }
  },

  /**
   * 重启应用
   * @returns {Promise<void>}
   */
  restartApp: async () => {
    try {
      await invoke('restart_app');
    } catch (error) {
      console.error('Failed to restart app:', error);
      throw error;
    }
  },

  /**
   * 退出应用
   * @returns {Promise<void>}
   */
  quitApp: async () => {
    try {
      await invoke('quit_app');
    } catch (error) {
      console.error('Failed to quit app:', error);
      throw error;
    }
  }
};

/**
 * 错误处理包装器
 * @param {Function} fn - 要执行的函数
 * @param {string} errorMessage - 错误消息
 * @returns {Promise<any>}
 */
export async function withErrorHandling(fn, errorMessage) {
  try {
    return await fn();
  } catch (error) {
    console.error(`${errorMessage}:`, error);
    throw new Error(`${errorMessage}: ${error.message}`);
  }
}

export default {
  window: windowCommands,
  settings: settingsCommands,
  player: playerCommands,
  system: systemCommands,
  withErrorHandling
};

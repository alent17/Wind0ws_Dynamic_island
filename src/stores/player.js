/**
 * Player Store - 播放器状态管理
 */
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export const playerStore = writable({
  isPlaying: false,
  isPaused: false,
  isExpanded: false,
  currentSong: null,
  currentArtist: null,
  currentAlbum: null,
  progress: 0,
  duration: 0,
  volume: 0.7,
  isMuted: false,
  shuffle: false,
  repeat: 'off', // off, one, all
  playlist: [],
  currentIndex: 0
});

// 进度更新定时器
let progressInterval = null;

// Actions
export const playerActions = {
  play: () => {
    playerStore.update(state => ({ ...state, isPlaying: true, isPaused: false }));
    // 启动进度更新
    playerActions.startProgressUpdate();
  },
  
  pause: () => {
    playerStore.update(state => ({ ...state, isPlaying: false, isPaused: true }));
    // 停止进度更新
    playerActions.stopProgressUpdate();
  },
  
  toggle: () => {
    playerStore.update(state => ({
      ...state,
      isPlaying: !state.isPlaying,
      isPaused: state.isPlaying
    }));
  },
  
  expand: () => {
    playerStore.update(state => ({ ...state, isExpanded: true }));
  },
  
  collapse: () => {
    playerStore.update(state => ({ ...state, isExpanded: false }));
  },
  
  toggleExpand: () => {
    playerStore.update(state => ({ ...state, isExpanded: !state.isExpanded }));
  },
  
  setProgress: (progress) => {
    playerStore.update(state => ({ ...state, progress }));
  },
  
  setDuration: (duration) => {
    playerStore.update(state => ({ ...state, duration }));
  },
  
  setVolume: (volume) => {
    playerStore.update(state => ({ ...state, volume, isMuted: volume === 0 }));
  },
  
  toggleMute: () => {
    playerStore.update(state => ({ ...state, isMuted: !state.isMuted }));
  },
  
  toggleShuffle: () => {
    playerStore.update(state => ({ ...state, shuffle: !state.shuffle }));
  },
  
  setRepeat: (mode) => {
    playerStore.update(state => ({ ...state, repeat: mode }));
  },
  
  cycleRepeat: () => {
    playerStore.update(state => ({
      ...state,
      repeat: state.repeat === 'off' ? 'all' : state.repeat === 'all' ? 'one' : 'off'
    }));
  },
  
  setCurrentSong: (song) => {
    playerStore.update(state => ({
      ...state,
      currentSong: song.title,
      currentArtist: song.artist,
      currentAlbum: song.album,
      progress: 0
    }));
  },
  
  setPlaylist: (playlist) => {
    playerStore.update(state => ({ ...state, playlist, currentIndex: 0 }));
  },
  
  next: () => {
    playerStore.update(state => ({
      ...state,
      currentIndex: (state.currentIndex + 1) % state.playlist.length,
      progress: 0
    }));
  },
  
  previous: () => {
    playerStore.update(state => ({
      ...state,
      currentIndex: state.currentIndex === 0 ? state.playlist.length - 1 : state.currentIndex - 1,
      progress: 0
    }));
  },
  
  // 获取网易云缓存中的时长
  fetchDuration: async () => {
    try {
      const durationMs = await invoke('get_netease_duration');
      if (durationMs) {
        playerStore.update(state => ({ ...state, duration: durationMs / 1000 }));
        return durationMs / 1000;
      }
    } catch (e) {
      console.error('获取网易云时长失败:', e);
    }
    return null;
  },
  
  // 启动进度更新（每 100ms 前进一次）
  startProgressUpdate: () => {
    if (progressInterval) return;
    
    progressInterval = setInterval(() => {
      playerStore.update(state => {
        if (state.isPlaying && state.duration > 0) {
          const newProgress = state.progress + 0.1; // 100ms = 0.1s
          // 如果进度超过时长，停止播放
          if (newProgress >= state.duration) {
            clearInterval(progressInterval);
            progressInterval = null;
            return { ...state, progress: state.duration, isPlaying: false, isPaused: true };
          }
          return { ...state, progress: newProgress };
        }
        return state;
      });
    }, 100);
  },
  
  // 停止进度更新
  stopProgressUpdate: () => {
    if (progressInterval) {
      clearInterval(progressInterval);
      progressInterval = null;
    }
  },
  
  // 重置进度
  resetProgress: () => {
    playerStore.update(state => ({ ...state, progress: 0 }));
  }
};

// 清理函数（在组件卸载时调用）
export const cleanup = () => {
  if (progressInterval) {
    clearInterval(progressInterval);
    progressInterval = null;
  }
};

export default playerStore;

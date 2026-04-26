/**
 * Player Store - 播放器状态管理 (Svelte 5 Runes 语法)
 */
import { invoke } from '@tauri-apps/api/core';

// 使用 $state 创建全局响应式对象
export const playerState = $state({
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

// Actions 直接修改 state 属性即可，Svelte 5 会自动追踪依赖
export const playerActions = {
  play() {
    playerState.isPlaying = true;
    playerState.isPaused = false;
    this.startProgressUpdate();
  },
  
  pause() {
    playerState.isPlaying = false;
    playerState.isPaused = true;
    this.stopProgressUpdate();
  },
  
  toggle() {
    playerState.isPlaying = !playerState.isPlaying;
    playerState.isPaused = !playerState.isPlaying;
  },
  
  expand() { playerState.isExpanded = true; },
  collapse() { playerState.isExpanded = false; },
  toggleExpand() { playerState.isExpanded = !playerState.isExpanded; },
  
  setProgress(progress) { playerState.progress = progress; },
  setDuration(duration) { playerState.duration = duration; },
  setVolume(volume) { 
    playerState.volume = volume; 
    playerState.isMuted = volume === 0;
  },
  toggleMute() { playerState.isMuted = !playerState.isMuted; },
  toggleShuffle() { playerState.shuffle = !playerState.shuffle; },
  
  setRepeat(mode) { playerState.repeat = mode; },
  cycleRepeat() {
    playerState.repeat = playerState.repeat === 'off' 
      ? 'all' 
      : playerState.repeat === 'all' 
        ? 'one' 
        : 'off';
  },
  
  setCurrentSong(song) {
    playerState.currentSong = song?.title || null;
    playerState.currentArtist = song?.artist || null;
    playerState.currentAlbum = song?.album || null;
    playerState.progress = 0;
  },
  
  setPlaylist(playlist) {
    playerState.playlist = playlist;
    playerState.currentIndex = 0;
  },
  
  next() {
    if (playerState.playlist.length > 0) {
      playerState.currentIndex = (playerState.currentIndex + 1) % playerState.playlist.length;
      playerState.progress = 0;
    }
  },
  
  previous() {
    if (playerState.playlist.length > 0) {
      playerState.currentIndex = playerState.currentIndex === 0 
        ? playerState.playlist.length - 1 
        : playerState.currentIndex - 1;
      playerState.progress = 0;
    }
  },
  
  // 获取网易云缓存中的时长
  async fetchDuration() {
    try {
      const durationMs = await invoke('get_netease_duration_cmd');
      if (durationMs) {
        playerState.duration = durationMs / 1000;
        return durationMs / 1000;
      }
    } catch (e) {
      console.error('获取网易云时长失败:', e);
    }
    return null;
  },
  
  // 启动进度更新（每 100ms 前进一次）
  startProgressUpdate() {
    if (progressInterval) return;
    
    progressInterval = setInterval(() => {
      if (playerState.isPlaying && playerState.duration > 0) {
        const newProgress = playerState.progress + 0.1;
        if (newProgress >= playerState.duration) {
          clearInterval(progressInterval);
          progressInterval = null;
          playerState.progress = playerState.duration;
          playerState.isPlaying = false;
          playerState.isPaused = true;
        } else {
          playerState.progress = newProgress;
        }
      }
    }, 100);
  },
  
  // 停止进度更新
  stopProgressUpdate() {
    if (progressInterval) {
      clearInterval(progressInterval);
      progressInterval = null;
    }
  },
  
  // 重置进度
  resetProgress() { playerState.progress = 0; }
};

// 清理函数（在组件卸载时调用）
export const cleanup = () => {
  if (progressInterval) {
    clearInterval(progressInterval);
    progressInterval = null;
  }
};

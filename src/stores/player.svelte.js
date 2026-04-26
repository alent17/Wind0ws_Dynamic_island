/**
 * Player Store - 播放器状态管理 (Svelte 5 Runes 语法 + 工厂模式)
 * 遵循 CLAUDE.md 规范：使用工厂函数 createPlayerStore()，隐藏原始 set/update 逻辑
 */
import { getNeteaseDuration } from '$lib/api/media';

/**
 * 创建播放器 Store 的工厂函数
 * @returns 播放器状态和操作方法的封装对象
 */
function createPlayerStore() {
  // 内部私有状态
  let state = $state({
    isPlaying: false,
    isPaused: false,
    isExpanded: false,
    currentSong: null as string | null,
    currentArtist: null as string | null,
    currentAlbum: null as string | null,
    progress: 0,
    duration: 0,
    volume: 0.7,
    isMuted: false,
    shuffle: false,
    repeat: 'off' as 'off' | 'one' | 'all',
    playlist: [] as Array<{ title: string; artist: string; album?: string }>,
    currentIndex: 0
  });

  // 进度更新定时器
  let progressInterval: ReturnType<typeof setInterval> | null = null;

  // 启动进度更新（每 100ms 前进一次）
  function startProgressUpdate() {
    if (progressInterval) return;
    
    progressInterval = setInterval(() => {
      if (state.isPlaying && state.duration > 0) {
        const newProgress = state.progress + 0.1;
        if (newProgress >= state.duration) {
          clearInterval(progressInterval);
          progressInterval = null;
          state.progress = state.duration;
          state.isPlaying = false;
          state.isPaused = true;
        } else {
          state.progress = newProgress;
        }
      }
    }, 100);
  }
  
  // 停止进度更新
  function stopProgressUpdate() {
    if (progressInterval) {
      clearInterval(progressInterval);
      progressInterval = null;
    }
  }

  // 返回封装后的公共 API
  return {
    // 只读状态访问
    get state() {
      return state;
    },
    
    // 播放控制
    play() {
      state.isPlaying = true;
      state.isPaused = false;
      startProgressUpdate();
    },
    
    pause() {
      state.isPlaying = false;
      state.isPaused = true;
      stopProgressUpdate();
    },
    
    toggle() {
      state.isPlaying = !state.isPlaying;
      state.isPaused = !state.isPlaying;
      if (state.isPlaying) {
        startProgressUpdate();
      } else {
        stopProgressUpdate();
      }
    },
    
    // 展开/收起控制
    expand() { state.isExpanded = true; },
    collapse() { state.isExpanded = false; },
    toggleExpand() { state.isExpanded = !state.isExpanded; },
    
    // 进度控制
    setProgress(progress: number) { state.progress = progress; },
    setDuration(duration: number) { state.duration = duration; },
    
    // 音量控制
    setVolume(volume: number) { 
      state.volume = volume; 
      state.isMuted = volume === 0;
    },
    toggleMute() { state.isMuted = !state.isMuted; },
    toggleShuffle() { state.shuffle = !state.shuffle; },
    
    // 循环模式控制
    setRepeat(mode: 'off' | 'one' | 'all') { state.repeat = mode; },
    cycleRepeat() {
      state.repeat = state.repeat === 'off' 
        ? 'all' 
        : state.repeat === 'all' 
          ? 'one' 
          : 'off';
    },
    
    // 歌曲信息设置
    setCurrentSong(song: { title: string; artist: string; album?: string } | null) {
      if (song) {
        state.currentSong = song.title || null;
        state.currentArtist = song.artist || null;
        state.currentAlbum = song.album || null;
      } else {
        state.currentSong = null;
        state.currentArtist = null;
        state.currentAlbum = null;
      }
      state.progress = 0;
    },
    
    // 播放列表控制
    setPlaylist(playlist: Array<{ title: string; artist: string; album?: string }>) {
      state.playlist = playlist;
      state.currentIndex = 0;
    },
    
    next() {
      if (state.playlist.length > 0) {
        state.currentIndex = (state.currentIndex + 1) % state.playlist.length;
        state.progress = 0;
      }
    },
    
    previous() {
      if (state.playlist.length > 0) {
        state.currentIndex = state.currentIndex === 0 
          ? state.playlist.length - 1 
          : state.currentIndex - 1;
        state.progress = 0;
      }
    },
    
    // 获取网易云缓存中的时长（走 API 层）
    async fetchDuration() {
      try {
        const durationMs = await getNeteaseDuration();
        if (durationMs) {
          state.duration = durationMs / 1000;
          return durationMs / 1000;
        }
      } catch (e) {
        console.error('获取网易云时长失败:', e);
      }
      return null;
    },
    
    // 重置进度
    resetProgress() { state.progress = 0; },
    
    // 清理函数（组件卸载时调用）
    cleanup() {
      if (progressInterval) {
        clearInterval(progressInterval);
        progressInterval = null;
      }
    }
  };
}

// 导出单例
export const player = createPlayerStore();

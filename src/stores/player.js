/**
 * Player Store - 播放器状态管理
 */
import { writable } from 'svelte/store';

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

// Actions
export const playerActions = {
  play: () => {
    playerStore.update(state => ({ ...state, isPlaying: true, isPaused: false }));
  },
  
  pause: () => {
    playerStore.update(state => ({ ...state, isPlaying: false, isPaused: true }));
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
  }
};

export default playerStore;

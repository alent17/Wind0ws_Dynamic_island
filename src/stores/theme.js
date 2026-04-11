/**
 * Theme Store - 主题状态管理
 */
import { writable } from 'svelte/store';

// 从 design.md 中提取的 Spotify 设计系统主题
export const themeStore = writable({
  colors: {
    base: {
      black: '#000000',
      dark: '#121212',
      darkGray: '#181818',
      midGray: '#1f1f1f',
      card: '#252525',
      cardAlt: '#272727'
    },
    text: {
      base: '#ffffff',
      secondary: '#b3b3b3',
      subdued: '#cbcbcb',
      hint: '#7c7c7c',
      disabled: '#4d4d4d'
    },
    accent: {
      green: '#1ed760',
      greenDark: '#1db954',
      greenLight: '#1ed760'
    },
    semantic: {
      negative: '#f3727f',
      warning: '#ffa42b',
      announcement: '#539df5',
      positive: '#1ed760'
    },
    border: {
      gray: '#4d4d4d',
      light: '#7c7c7c',
      separator: '#b3b3b3',
      green: '#1db954'
    },
    shadow: {
      heavy: 'rgba(0, 0, 0, 0.5) 0px 8px 24px',
      medium: 'rgba(0, 0, 0, 0.3) 0px 8px 8px',
      light: 'rgba(0, 0, 0, 0.15) 0px 4px 4px'
    }
  },
  spacing: {
    xs: '4px',
    sm: '8px',
    md: '12px',
    lg: '16px',
    xl: '20px',
    xxl: '24px',
    xxxl: '32px'
  },
  radius: {
    pill: '9999px',
    circle: '50%',
    card: '8px',
    input: '500px',
    sm: '4px',
    md: '6px',
    lg: '10px',
    xl: '20px'
  },
  typography: {
    fontFamily: {
      title: "'SpotifyMixUITitle', 'CircularSp-Arab', sans-serif",
      ui: "'SpotifyMixUI', 'CircularSp', sans-serif",
      body: "'SpotifyMixUI', system-ui, sans-serif"
    },
    fontSize: {
      xs: '10px',
      sm: '12px',
      md: '14px',
      lg: '16px',
      xl: '18px',
      '2xl': '24px'
    },
    fontWeight: {
      normal: 400,
      medium: 600,
      bold: 700
    },
    letterSpacing: {
      tight: 'normal',
      normal: 'normal',
      wide: '1.4px',
      wider: '2px'
    }
  },
  transitions: {
    fast: '0.15s ease',
    base: '0.2s cubic-bezier(0.4, 0, 0.2, 1)',
    slow: '0.3s ease',
    bounce: '0.4s cubic-bezier(0.34, 1.56, 0.64, 1)'
  }
});

// Actions
export const themeActions = {
  setTheme: (themeName) => {
    // 未来可以扩展多主题支持
    console.log('Theme changed to:', themeName);
  },
  
  updateColor: (category, key, value) => {
    themeStore.update(state => ({
      ...state,
      colors: {
        ...state.colors,
        [category]: {
          ...state.colors[category],
          [key]: value
        }
      }
    }));
  },
  
  loadCustomTheme: (customTheme) => {
    themeStore.update(state => ({
      ...state,
      ...customTheme
    }));
  }
};

export default themeStore;

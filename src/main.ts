import './app.css';
import { mount } from 'svelte';
import App from './App.svelte';
import VisibleWindow from './lib/VisibleWindow.svelte';
import FloatingWindow from './FloatingWindow.svelte';
import TimerWindow from './TimerWindow.svelte';

// Keep browser-only actions out of the desktop application UI.
document.addEventListener('contextmenu', (event) => event.preventDefault());

// 获取目标挂载点
const targetElement = document.getElementById('app');

if (!targetElement) {
  throw new Error('找不到挂载点 #app');
}

// 解析 URL 参数
const urlParams = new URLSearchParams(window.location.search);
const windowType = urlParams.get('window');

let app;

// 路由分发逻辑
if (windowType === 'floating') {
  // 渲染独立悬浮窗
  app = mount(VisibleWindow, {
    props: { component: FloatingWindow },
    target: targetElement,
  });
} else if (windowType === 'timer') {
  app = mount(VisibleWindow, {
    props: { component: TimerWindow },
    target: targetElement,
  });
} else {
  // 默认渲染灵动岛主窗口
  app = mount(App, {
    target: targetElement,
  });
}

export default app;

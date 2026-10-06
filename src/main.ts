import './app.css';
import { mount } from 'svelte';

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

async function loadFloatingWindow() {
  return import('./FloatingWindow.svelte');
}

async function loadTimerWindow() {
  return import('./TimerWindow.svelte');
}

async function mountWindow() {
  // Load only the current window's component and styles. The compact island
  // does not need to retain the floating player and timer modules.
  if (windowType === 'floating' || windowType === 'timer') {
    // Keep each import in its own branch so Vite preloads that component's
    // CSS. A conditional import inside Promise.all can reuse the timer's
    // dependency list for the floating player in the production bundle.
    const contentModule = windowType === 'floating'
      ? loadFloatingWindow()
      : loadTimerWindow();
    const [{ default: VisibleWindow }, { default: Content }] = await Promise.all([
      import('./lib/components/island/VisibleWindow.svelte'),
      contentModule,
    ]);
    app = mount(VisibleWindow, { props: { component: Content }, target: targetElement! });
  } else {
    const { default: App } = await import('./App.svelte');
    app = mount(App, { target: targetElement! });
  }
}

void mountWindow().catch((error) => console.error('[Isle] Window initialization failed', error));
export { app as default };

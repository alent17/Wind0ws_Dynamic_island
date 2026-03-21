import './app.css'; // 关键：必须引入全局样式，否则 Tailwind 不生效，窗口会变全透明
import { mount } from 'svelte';
import Settings from './Settings.svelte';

// 关键：使用 Svelte 5 的 mount 语法
const app = mount(Settings, {
  target: document.getElementById('app')!,
});

export default app;

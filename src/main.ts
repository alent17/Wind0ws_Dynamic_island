import './app.css';
import { mount } from 'svelte';
import App from './App.svelte';

// 只加载主应用
const app = mount(App, {
  target: document.getElementById('app')!,
});

export default app;

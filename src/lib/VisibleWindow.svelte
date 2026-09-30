<script lang="ts">
  import { onMount, type Component } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  let { component: Content } = $props<{component: Component}>();
  let visible = $state(!("__TAURI_INTERNALS__" in window) && !document.hidden);
  onMount(() => {
    let disposed = false;
    let revision = 0;
    const cleanups: (() => void)[] = [];
    const refresh = async () => {
      const request = ++revision;
      let next = !document.hidden;
      if ('__TAURI_INTERNALS__' in window) {
        try { const shown = await getCurrentWindow().isVisible(); if (typeof shown === 'boolean') next &&= shown; } catch {}
      }
      if (!disposed && request === revision) visible = next;
    };
    const register = (dispose: () => void) => disposed ? dispose() : cleanups.push(dispose);
    document.addEventListener('visibilitychange', refresh);
    if ('__TAURI_INTERNALS__' in window) {
      void getCurrentWindow().listen<boolean>('isle-window-visible', ({payload}) => { revision++; visible = payload; }).then(register).catch(() => {});
      void getCurrentWindow().onFocusChanged(() => void refresh()).then(register).catch(() => {});
    }
    void refresh();
    return () => { disposed = true; revision++; document.removeEventListener('visibilitychange', refresh); cleanups.forEach(fn => fn()); };
  });
</script>
{#if visible}<Content />{/if}

<script lang="ts">
  let { kind, size = 24, colored = false }: { kind: string; size?: number; colored?: boolean } = $props();
</script>

<svg class:colored width={size} height={size} viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
  {#if kind === 'sun'}
    <circle class="sun" cx="12" cy="12" r="5"/><path class="sun" d="M11 1h2v3h-2zM11 20h2v3h-2zM1 11h3v2H1zM20 11h3v2h-3zM4.22 5.64l1.42-1.42 2.12 2.12-1.42 1.42zM16.24 17.66l1.42-1.42 2.12 2.12-1.42 1.42zM16.24 6.34l2.12-2.12 1.42 1.42-2.12 2.12zM4.22 18.36l2.12-2.12 1.42 1.42-2.12 2.12z"/>
  {:else if kind === 'cloud' || kind === 'rain' || kind === 'snow' || kind === 'fog' || kind === 'storm' || kind === 'partly' || kind === 'weather'}
    {#if kind === 'partly' || kind === 'weather'}<circle class="sun" cx="15.5" cy="7.5" r="4"/><path class="sun" d="M15 1h1v2h-1zM21 7h2v1h-2zM19.5 2.5l1 1-1.5 1.5-1-1z"/>{/if}
    <path class="cloud" d="M6.4 17a4.4 4.4 0 0 1-.3-8.79A6.4 6.4 0 0 1 18.3 9.6 3.7 3.7 0 0 1 18 17H6.4Z"/>
    {#if kind === 'rain'}<path class="rain" d="M7 18h2l-1 4H6zm5 0h2l-1 4h-2zm5 0h2l-1 4h-2z"/>{:else if kind === 'snow'}<path class="snow" d="M7 18h2v2h2v1H9v2H7v-2H5v-1h2zm8 0h2v2h2v1h-2v2h-2v-2h-2v-1h2z"/>{:else if kind === 'fog'}<path class="fog" d="M5 19h14v1.5H5zm2 3h10v1H7z"/>{:else if kind === 'storm'}<path class="storm" d="M12 17h4l-3 3h2l-5 4 1-4H9z"/>{/if}
  {:else if kind === 'timer'}
    <path fill-rule="evenodd" d="M9 1h6v2H9zm3 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18Zm.8 3.5h-1.6v4.1l-2.4 2.4 1.2 1.2 2.8-2.8V7.5Z" clip-rule="evenodd"/><path d="m18.2 4.2 1.4-1.4 2 2-1.4 1.4z"/>
  {:else if kind === 'clock'}
    <path fill-rule="evenodd" d="M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20Zm-.8 4h1.6v5.5l3.5 2.1-.8 1.4-4.3-2.6V6Z" clip-rule="evenodd"/>
  {:else if kind === 'calendar'}
    <path fill-rule="evenodd" d="M6 2h2v2h8V2h2v2h1a3 3 0 0 1 3 3v12a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3V7a3 3 0 0 1 3-3h1V2ZM4 9v10c0 .6.4 1 1 1h14c.6 0 1-.4 1-1V9H4Zm2 2h3v3H6zm5 0h3v3h-3zm5 0h2v3h-2zm-10 5h3v2H6zm5 0h3v2h-3zm5 0h2v2h-2Z" clip-rule="evenodd"/>
  {:else if kind === 'volume'}
    <path d="M3 8h4l5-4v16l-5-4H3a1 1 0 0 1-1-1V9a1 1 0 0 1 1-1Zm12.2.1a1 1 0 0 1 1.4 0 5.5 5.5 0 0 1 0 7.8 1 1 0 1 1-1.4-1.4 3.5 3.5 0 0 0 0-5 1 1 0 0 1 0-1.4Zm2.8-2.8a1 1 0 0 1 1.4 0 9.5 9.5 0 0 1 0 13.4 1 1 0 0 1-1.4-1.4 7.5 7.5 0 0 0 0-10.6 1 1 0 0 1 0-1.4Z"/>
  {:else if kind === 'files'}
    <path d="M3 4h6l2 2h10a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Zm0 5v9h18V9H3Z"/>
  {:else if kind === 'system'}
    <path d="M2 11h4.1l2.2-6.5a1 1 0 0 1 1.9 0l3.3 10 2-5a1 1 0 0 1 1.8-.1l1.3 2.6H22v2h-4a1 1 0 0 1-.9-.55l-.7-1.4-2.1 5.3a1 1 0 0 1-1.9-.05L9.2 8l-1.5 4.4a1 1 0 0 1-.95.6H2z"/>
  {:else if kind === 'floating'}<path d="M3 4h18a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Zm9 8v6h9v-6h-9Z" fill-rule="evenodd"/>
  {:else if kind === 'settings'}<path fill-rule="evenodd" d="M10 2h4l.6 2.3c.5.2 1 .4 1.5.6l2-1.2 2.8 2.8-1.2 2c.3.5.5 1 .6 1.5L22.5 11v4l-2.3.6c-.2.5-.4 1-.6 1.5l1.2 2-2.8 2.8-2-1.2c-.5.3-1 .5-1.5.6L14 23h-4l-.6-1.7c-.5-.2-1-.4-1.5-.6l-2 1.2-2.8-2.8 1.2-2c-.3-.5-.5-1-.6-1.5L1.5 15v-4l2.3-1c.2-.5.4-1 .6-1.5l-1.2-2L6 3.7l2 1.2c.5-.3 1-.5 1.5-.6L10 2Zm2 7a3 3 0 1 0 0 6 3 3 0 0 0 0-6Z" clip-rule="evenodd"/>
  {:else if kind === 'hide'}<path fill-rule="evenodd" d="M2 3.4 3.4 2l18.6 18.6-1.4 1.4-3.1-3.1A12 12 0 0 1 12 20C5 20 1 12 1 12a16 16 0 0 1 4-4.7L2 3.4ZM7.1 9.5A5 5 0 0 0 14.5 17l-1.7-1.7a2.7 2.7 0 0 1-3.9-3.9L7.1 9.5ZM12 4c7 0 11 8 11 8a16 16 0 0 1-3.4 4.1l-3-3A5 5 0 0 0 10.9 7.4L8.5 5A12 12 0 0 1 12 4Z" clip-rule="evenodd"/>{/if}
</svg>

<style>
  .colored .sun{fill:#ffc85a}.colored .cloud{fill:#c5d7e8}.colored .rain{fill:#002fa7}.colored .snow{fill:#a8ddff}.colored .fog{fill:#91a9bb}.colored .storm{fill:#ffc85a}
</style>

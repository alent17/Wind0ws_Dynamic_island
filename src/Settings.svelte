<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { 
    Home, 
    Search, 
    Library, 
    Settings as SettingsIcon, 
    X, 
    Minus,
    MonitorOff,
    AudioLines,
    Zap,
    Check
  } from "lucide-svelte";

  interface AppSettings {
    island_theme: string;
    auto_hide: boolean;
    show_spectrum: boolean;
    enable_animations: boolean;
    window_opacity: number;
    always_on_top: boolean;
  }

  let settings = $state<AppSettings>({
    island_theme: "original",
    auto_hide: true,
    show_spectrum: true,
    enable_animations: true,
    window_opacity: 255,
    always_on_top: true,
  });

  const appWindow = getCurrentWindow();
  let currentTab = $state("全部");

  onMount(async () => {
    try {
      const savedSettings = await invoke<AppSettings>("get_settings");
      settings = { ...settings, ...savedSettings };
    } catch (e) {
      console.error("无法读取设置", e);
    }
  });

  async function saveSettings(newSettings: Partial<AppSettings>) {
    settings = { ...settings, ...newSettings };
    try {
      await invoke("save_settings", { settings });
      if (newSettings.island_theme) {
        appWindow.emit("theme-changed", { islandTheme: settings.island_theme });
      }
    } catch (e) {
      console.error("保存失败", e);
    }
  }

  function closeWindow() { appWindow.close(); }
  function minimizeWindow() { appWindow.minimize(); }
</script>

<div class="flex h-screen bg-black text-white select-none overflow-hidden font-sans">
  
  <aside class="w-[72px] bg-black flex flex-col items-center py-6 gap-8 z-20">
    <div class="w-10 h-10 bg-[#181818] rounded-full flex items-center justify-center cursor-pointer hover:bg-[#282828] transition-colors" data-tauri-drag-region>
      <div class="w-2.5 h-6 bg-white rounded-full pointer-events-none"></div>
    </div>

    <nav class="flex flex-col gap-6 mt-4">
      <button class="text-[#b3b3b3] hover:text-white transition-colors"><Home size={24} strokeWidth={2} /></button>
      <button class="text-[#b3b3b3] hover:text-white transition-colors"><Search size={24} strokeWidth={2} /></button>
      <button class="text-[#b3b3b3] hover:text-white transition-colors"><Library size={24} strokeWidth={2} /></button>
    </nav>

    <div class="mt-auto">
      <button class="text-[#1ED760] transition-colors"><SettingsIcon size={24} strokeWidth={2.5} /></button>
    </div>
  </aside>

  <main class="flex-1 flex flex-col bg-[#121212] rounded-tl-lg overflow-hidden relative">
    
    <div class="h-16 flex items-center justify-between px-6 bg-[#121212]/95 backdrop-blur-md z-50 sticky top-0" data-tauri-drag-region>
      <div class="flex gap-2 pointer-events-auto">
        {#each ["全部", "外观", "行为"] as tab}
          <button 
            class="px-4 py-1.5 rounded-full text-[13px] font-semibold transition-all {currentTab === tab ? 'bg-white text-black' : 'bg-[#2a2a2a] text-white hover:bg-[#333]'}"
            onclick={() => currentTab = tab}
          >
            {tab}
          </button>
        {/each}
      </div>
      
      <div class="flex gap-4 pointer-events-auto">
        <button onclick={minimizeWindow} class="text-[#b3b3b3] hover:text-white transition-colors"><Minus size={20} /></button>
        <button onclick={closeWindow} class="text-[#b3b3b3] hover:text-[#ff5f56] transition-colors"><X size={20} /></button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto px-8 pb-16 custom-scrollbar">
      <div class="max-w-[1400px]"> {#if currentTab === "全部" || currentTab === "外观"}
        <section class="mt-6 mb-12">
          <div class="flex items-end justify-between mb-6">
            <h2 class="text-2xl font-bold tracking-tight hover:underline cursor-pointer">灵动岛外观主题</h2>
            <span class="text-[13px] font-bold text-[#b3b3b3] hover:text-white cursor-pointer tracking-widest">显示全部</span>
          </div>
          
          <div class="flex gap-6 flex-wrap">
            <button 
              class="flex flex-col gap-4 group w-[180px] p-4 bg-[#181818] rounded-xl hover:bg-[#282828] transition-all duration-300 text-left"
              onclick={() => saveSettings({ island_theme: "original" })}
            >
              <div class="w-full aspect-square rounded-full bg-[#121212] shadow-xl flex items-center justify-center relative transition-transform duration-300 group-hover:-translate-y-1 {settings.island_theme === 'original' ? 'ring-2 ring-[#1ED760] ring-offset-4 ring-offset-[#181818]' : ''}">
                <div class="w-16 h-4 bg-black rounded-full border border-white/5 shadow-inner"></div>
              </div>
              <div>
                <h3 class="font-bold text-[15px] truncate text-white">极简经典</h3>
                <p class="text-[13px] text-[#b3b3b3] mt-1">Apple 原始质感</p>
              </div>
            </button>

            <button 
              class="flex flex-col gap-4 group w-[180px] p-4 bg-[#181818] rounded-xl hover:bg-[#282828] transition-all duration-300 text-left"
              onclick={() => saveSettings({ island_theme: "ios26" })}
            >
              <div class="w-full aspect-square rounded-full bg-gradient-to-br from-[#2a2a2a] to-[#0a0a0a] shadow-[0_0_20px_rgba(255,255,255,0.03)] border border-white/5 flex items-center justify-center relative transition-transform duration-300 group-hover:-translate-y-1 {settings.island_theme === 'ios26' ? 'ring-2 ring-[#1ED760] ring-offset-4 ring-offset-[#181818]' : ''}">
                <div class="w-16 h-4 bg-black rounded-full shadow-[inset_0_2px_4px_rgba(255,255,255,0.15)]"></div>
                <div class="absolute top-0 w-full h-1/2 bg-gradient-to-b from-white/5 to-transparent rounded-t-full pointer-events-none"></div>
              </div>
              <div>
                <h3 class="font-bold text-[15px] truncate text-white">iOS 26</h3>
                <p class="text-[13px] text-[#b3b3b3] mt-1">液态玻璃与光影</p>
              </div>
            </button>
          </div>
        </section>
        {/if}

        {#if currentTab === "全部" || currentTab === "行为"}
        <section>
          <div class="flex items-end justify-between mb-6">
            <h2 class="text-2xl font-bold tracking-tight hover:underline cursor-pointer">交互与行为配置</h2>
            <span class="text-[13px] font-bold text-[#b3b3b3] hover:text-white cursor-pointer tracking-widest">显示全部</span>
          </div>

          <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
            
            <button 
              class="bg-[#181818] p-4 rounded-xl hover:bg-[#282828] transition-all duration-300 group text-left relative flex flex-col gap-4"
              onclick={() => saveSettings({ auto_hide: !settings.auto_hide })}
            >
              <div class="w-full aspect-square rounded-md shadow-lg flex items-center justify-center relative overflow-hidden transition-transform duration-300 {settings.auto_hide ? 'bg-gradient-to-br from-[#1ED760] to-[#117a37]' : 'bg-[#2a2a2a]'}">
                <MonitorOff size={48} strokeWidth={1.5} class="text-black/30" />
                
                <div class="absolute right-3 bottom-3 w-12 h-12 bg-[#1ED760] hover:bg-[#1fdf64] hover:scale-105 rounded-full flex items-center justify-center opacity-0 transform translate-y-3 group-hover:opacity-100 group-hover:translate-y-0 transition-all duration-300 shadow-xl z-10">
                  {#if settings.auto_hide}
                    <Check size={24} strokeWidth={3} class="text-black" />
                  {:else}
                    <div class="w-4 h-4 rounded-full border-2 border-black"></div>
                  {/if}
                </div>
              </div>
              <div>
                <h3 class="font-bold text-[15px] truncate text-white mb-1">全屏防遮挡</h3>
                <p class="text-[13px] text-[#b3b3b3] line-clamp-2 leading-relaxed">运行全屏应用或游戏时自动将灵动岛移出屏幕外。</p>
              </div>
            </button>

            <button 
              class="bg-[#181818] p-4 rounded-xl hover:bg-[#282828] transition-all duration-300 group text-left relative flex flex-col gap-4"
              onclick={() => saveSettings({ show_spectrum: !settings.show_spectrum })}
            >
              <div class="w-full aspect-square rounded-md shadow-lg flex items-center justify-center relative overflow-hidden transition-transform duration-300 {settings.show_spectrum ? 'bg-gradient-to-br from-[#ff2d55] to-[#a31232]' : 'bg-[#2a2a2a]'}">
                <AudioLines size={48} strokeWidth={1.5} class="text-black/30" />
                
                <div class="absolute right-3 bottom-3 w-12 h-12 bg-[#1ED760] hover:bg-[#1fdf64] hover:scale-105 rounded-full flex items-center justify-center opacity-0 transform translate-y-3 group-hover:opacity-100 group-hover:translate-y-0 transition-all duration-300 shadow-xl z-10">
                  {#if settings.show_spectrum}
                    <Check size={24} strokeWidth={3} class="text-black" />
                  {:else}
                    <div class="w-4 h-4 rounded-full border-2 border-black"></div>
                  {/if}
                </div>
              </div>
              <div>
                <h3 class="font-bold text-[15px] truncate text-white mb-1">律动频谱</h3>
                <p class="text-[13px] text-[#b3b3b3] line-clamp-2 leading-relaxed">音乐播放时在缩略状态下显示随音乐跳动的频谱。</p>
              </div>
            </button>

            <button 
              class="bg-[#181818] p-4 rounded-xl hover:bg-[#282828] transition-all duration-300 group text-left relative flex flex-col gap-4"
              onclick={() => saveSettings({ enable_animations: !settings.enable_animations })}
            >
              <div class="w-full aspect-square rounded-md shadow-lg flex items-center justify-center relative overflow-hidden transition-transform duration-300 {settings.enable_animations ? 'bg-gradient-to-br from-[#4f46e5] to-[#28237c]' : 'bg-[#2a2a2a]'}">
                <Zap size={48} strokeWidth={1.5} class="text-black/30" />
                
                <div class="absolute right-3 bottom-3 w-12 h-12 bg-[#1ED760] hover:bg-[#1fdf64] hover:scale-105 rounded-full flex items-center justify-center opacity-0 transform translate-y-3 group-hover:opacity-100 group-hover:translate-y-0 transition-all duration-300 shadow-xl z-10">
                  {#if settings.enable_animations}
                    <Check size={24} strokeWidth={3} class="text-black" />
                  {:else}
                    <div class="w-4 h-4 rounded-full border-2 border-black"></div>
                  {/if}
                </div>
              </div>
              <div>
                <h3 class="font-bold text-[15px] truncate text-white mb-1">物理动画引擎</h3>
                <p class="text-[13px] text-[#b3b3b3] line-clamp-2 leading-relaxed">启用由 Svelte Spring 驱动的弹性窗口伸缩与抖动。</p>
              </div>
            </button>

          </div>
        </section>
        {/if}

      </div>
    </div>
  </main>
</div>

<style>
  /* 极致定制化的滚动条，贴合暗黑主题 */
  .custom-scrollbar::-webkit-scrollbar {
    width: 14px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background-color: rgba(255, 255, 255, 0.2);
    border: 4px solid #121212;
    border-radius: 10px;
    background-clip: padding-box;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background-color: rgba(255, 255, 255, 0.4);
  }
</style>
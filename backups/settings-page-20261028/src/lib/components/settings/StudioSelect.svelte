<script lang="ts">
  import { tick } from "svelte";
  import { Check, ChevronDown } from "lucide-svelte";

  type Option = { value: string; label: string };

  let { label, value, options, disabled = false, onSelect } = $props<{
    label: string;
    value: string;
    options: Option[];
    disabled?: boolean;
    onSelect: (value: string) => void;
  }>();

  let open = $state(false);
  let root: HTMLDivElement;
  let trigger: HTMLButtonElement;
  const menuId = `studio-select-${Math.random().toString(36).slice(2)}`;
  let selectedLabel = $derived(options.find((option: Option) => option.value === value)?.label ?? options[0]?.label ?? "");

  $effect(() => {
    if (!open) return;
    const dismiss = (event: PointerEvent) => {
      if (!root?.contains(event.target as Node)) open = false;
    };
    window.addEventListener("pointerdown", dismiss);
    return () => window.removeEventListener("pointerdown", dismiss);
  });

  async function focusOption(index: number) {
    open = true;
    await tick();
    const buttons = root.querySelectorAll<HTMLButtonElement>("[role=option]");
    buttons[Math.max(0, Math.min(index, buttons.length - 1))]?.focus();
  }

  function choose(nextValue: string) {
    onSelect(nextValue);
    open = false;
    trigger.focus();
  }

  function handleTriggerKeydown(event: KeyboardEvent) {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
    event.preventDefault();
    const selectedIndex = Math.max(0, options.findIndex((option: Option) => option.value === value));
    void focusOption(selectedIndex);
  }

  function handleMenuKeydown(event: KeyboardEvent) {
    const items = Array.from(root.querySelectorAll<HTMLButtonElement>("[role=option]"));
    const current = items.indexOf(document.activeElement as HTMLButtonElement);
    if (event.key === "Escape") {
      event.preventDefault();
      open = false;
      trigger.focus();
    } else if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
      event.preventDefault();
      const next = event.key === "Home" ? 0 : event.key === "End" ? items.length - 1
        : (current + (event.key === "ArrowDown" ? 1 : -1) + items.length) % items.length;
      items[next]?.focus();
    }
  }
</script>

<div class="studio-select" class:open bind:this={root} onfocusout={(event) => {
  if (!root.contains(event.relatedTarget as Node | null)) open = false;
}}>
  <button
    bind:this={trigger}
    type="button"
    class="select-trigger"
    aria-label={label}
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-controls={menuId}
    {disabled}
    onclick={() => open = !open}
    onkeydown={handleTriggerKeydown}
  >
    <span>{selectedLabel}</span><ChevronDown size={14} aria-hidden="true" />
  </button>
  {#if open}
    <div id={menuId} class="select-menu" role="listbox" aria-label={label} tabindex="-1" onkeydown={handleMenuKeydown}>
      {#each options as option (option.value)}
        <button type="button" role="option" aria-selected={option.value === value} class:active={option.value === value} title={option.label} onclick={() => choose(option.value)}>
          <span>{option.label}</span>{#if option.value === value}<Check size={14} aria-hidden="true" />{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .studio-select{position:relative;min-width:0;font-family:var(--app-font)}
  .select-trigger{display:flex;align-items:center;justify-content:space-between;gap:8px;width:100%;height:30px;padding:0 9px;border:1px solid #484b53;border-radius:8px;color:#f2f3f5;background:#22252a;font:500 12px/1 var(--app-font);text-align:left;cursor:pointer}
  .select-trigger:hover:not(:disabled),.open .select-trigger{border-color:#717684;background:#2b2e34}
  .select-trigger:focus-visible{outline:2px solid var(--studio-accent,#84a8ff);outline-offset:2px}
  .select-trigger:disabled{opacity:.45;cursor:not-allowed}
  .select-trigger span,.select-menu button span{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
  .select-trigger :global(svg){flex:none;transition:transform 150ms ease}
  .open .select-trigger :global(svg){transform:rotate(180deg)}
  .select-menu{position:absolute;top:calc(100% + 5px);right:0;z-index:40;width:max(100%,168px);max-width:calc(100vw - 24px);max-height:190px;padding:4px;overflow-y:auto;border:1px solid #41454f;border-radius:10px;background:#17191d;box-shadow:0 14px 32px rgba(0,0,0,.48)}
  .select-menu button{display:flex;align-items:center;justify-content:space-between;gap:8px;width:100%;min-height:29px;padding:5px 8px;border:0;border-radius:7px;color:#d6d8df;background:transparent;font:500 11px/1.25 var(--app-font);text-align:left;cursor:pointer}
  .select-menu button:hover,.select-menu button:focus-visible{color:#fff;background:#30343d;outline:none}
  .select-menu button.active{color:#fff;background:#303a4d}
  .select-menu button :global(svg){flex:none;color:#a7c5ff}
  .select-menu::-webkit-scrollbar{width:5px}.select-menu::-webkit-scrollbar-thumb{border-radius:999px;background:#545963}
  @media(prefers-reduced-motion:reduce){.select-trigger :global(svg){transition:none}}
</style>

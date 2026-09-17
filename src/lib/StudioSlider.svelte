<script lang="ts">
  let {
    label = "",
    hint = "",
    min = 0,
    max = 100,
    step = 1,
    value = $bindable(0),
    unit = "",
    showValue = true,
    hideLabel = false,
    previewTarget = null,
    onPreviewStart,
    onPreviewMove,
    onPreviewEnd,
  } = $props<{
    label?: string;
    hint?: string;
    min?: number;
    max?: number;
    step?: number;
    value?: number;
    unit?: string;
    showValue?: boolean;
    hideLabel?: boolean;
    previewTarget?: string | null;
    onPreviewStart?: ((value: number) => void) | undefined;
    onPreviewMove?: ((value: number) => void) | undefined;
    onPreviewEnd?: ((value: number) => void) | undefined;
  }>();

  const displayValue = $derived.by(() => `${value}${unit}`);

  function numericValue(event: Event) {
    return Number((event.currentTarget as HTMLInputElement).value);
  }

  function handleInput(event: Event) {
    const nextValue = numericValue(event);
    value = nextValue;
    onPreviewMove?.(nextValue);
  }

  function handlePointerDown() {
    onPreviewStart?.(value);
  }

  function handleChange(event: Event) {
    onPreviewEnd?.(numericValue(event));
  }
</script>

<div class="studio-slider" class:is-highlighted={previewTarget !== null} role="group" onmouseenter={() => onPreviewStart?.(value)} onfocusin={() => onPreviewStart?.(value)}>
  <div class="range-label" class:hidden={hideLabel}>
    <div class="title-group">
      {#if label}<span class="label">{label}</span>{/if}
      {#if hint}<small>{hint}</small>{/if}
    </div>

    {#if showValue}
      <output>{displayValue}</output>
    {/if}
  </div>

  <input
    type="range"
    {min}
    {max}
    {step}
    value={value}
    oninput={handleInput}
    onpointerdown={handlePointerDown}
    onchange={handleChange}
    aria-label={label || undefined}
  />
</div>

<style>
  .studio-slider {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .range-label {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 10px;
    min-height: 17px;
    margin: 14px 0 8px;
    font-size: 11px;
  }

  .range-label.hidden {
    display: none;
  }

  .title-group {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .label {
    font-size: 11px;
    font-weight: 600;
    color: #141416;
    line-height: 1.2;
  }

  .title-group small {
    color: #7b7b84;
    font-size: 9px;
    line-height: 1.35;
  }

  output {
    flex: none;
    font-size: 10px;
    font-weight: 400;
    color: #66666e;
    font-variant-numeric: tabular-nums;
  }

  input[type="range"] {
    width: 100%;
    margin: 0;
    accent-color: #111113;
    cursor: pointer;
    color-scheme: light;
  }

  input[type="range"]:focus-visible {
    outline: 2px solid #111113;
    outline-offset: 3px;
    border-radius: 4px;
  }
</style>

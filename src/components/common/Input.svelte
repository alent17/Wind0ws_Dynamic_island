<script lang="ts">
  import type { Component } from "svelte";

  let {
    type = "text",
    value = "",
    placeholder = "",
    label = "",
    error = "",
    disabled = false,
    variant = "pill",
    icon = null,
    ariaLabel = "",
    oninput: onInput,
    onchange: onChange,
    onfocus: onFocus,
    onblur: onBlur,
  }: {
    type?: string;
    value?: string;
    placeholder?: string;
    label?: string;
    error?: string;
    disabled?: boolean;
    variant?: string;
    icon?: Component | null;
    ariaLabel?: string;
    oninput?: (detail: { value: string }) => void;
    onchange?: (e: Event) => void;
    onfocus?: (e: FocusEvent) => void;
    onblur?: (e: FocusEvent) => void;
  } = $props();

  function handleChange(event: Event) {
    const target = event.target as HTMLInputElement;
    value = target.value;
    onInput?.({ value: target.value });
    onChange?.(event);
  }

  function handleFocus(event: FocusEvent) {
    onFocus?.(event);
  }

  function handleBlur(event: FocusEvent) {
    onBlur?.(event);
  }

  let className = $derived(
    [
      "input-wrapper",
      `input-${variant}`,
      error ? "input-error" : "",
      disabled ? "input-disabled" : "",
    ]
      .filter(Boolean)
      .join(" "),
  );
</script>

<div class={className}>
  {#if label}
    <label class="input-label">
      {label}
    </label>
  {/if}

  <div class="input-container">
      {#if icon}
      <span class="input-icon">
        <icon />
      </span>
    {/if}

    <input
      {type}
      {value}
      {placeholder}
      {disabled}
      aria-label={ariaLabel}
      class="input-field"
      oninput={handleChange}
      onfocus={handleFocus}
      onblur={handleBlur}
    />

    {#if error}
      <span class="input-error-text">{error}</span>
    {/if}
  </div>
</div>

<style>
  @import "../../styles/variables.css";

  .input-wrapper {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .input-label {
    font-family: var(--font-family-ui);
    font-size: var(--text-sm);
    font-weight: var(--font-bold);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
  }

  .input-container {
    position: relative;
    display: flex;
    align-items: center;
  }

  .input-field {
    width: 100%;
    font-family: var(--font-family-ui);
    font-size: var(--text-md);
    background: var(--base-mid-gray);
    color: var(--text-base);
    border: none;
    transition: all var(--transition-base);
  }

  .input-field:focus {
    outline: none;
    box-shadow: var(--inset-border);
  }

  .input-field::placeholder {
    color: var(--text-hint);
  }

  .input-field:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* ========== Variants ========== */
  .input-pill .input-field {
    border-radius: var(--radius-input);
    padding: var(--spacing-md) var(--spacing-lg);
    padding-left: var(--spacing-xl);
  }

  .input-pill.has-icon .input-field {
    padding-left: calc(var(--spacing-xl) + 24px);
  }

  .input-rounded .input-field {
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
  }

  .input-square .input-field {
    border-radius: var(--radius-sm);
    padding: var(--spacing-md);
  }

  /* ========== Icon ========== */
  .input-icon {
    position: absolute;
    left: var(--spacing-md);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    color: var(--text-secondary);
    pointer-events: none;
  }

  .input-pill.has-icon .input-icon {
    left: var(--spacing-lg);
  }

  /* ========== Error State ========== */
  .input-error .input-field {
    border: 1px solid var(--text-negative);
  }

  .input-error-text {
    position: absolute;
    bottom: calc(-100% - var(--spacing-xs));
    right: 0;
    font-size: var(--text-sm);
    color: var(--text-negative);
  }

  /* ========== Disabled ========== */
  .input-disabled .input-field {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>

<script lang="ts">
  let {
    checked = false,
    disabled = false,
    size = "md",
    ariaLabel = "",
    onChange,
  }: {
    checked?: boolean;
    disabled?: boolean;
    size?: string;
    ariaLabel?: string;
    onChange?: (detail: { checked: boolean }) => void;
  } = $props();

  function toggle() {
    if (disabled) return;
    checked = !checked;
    onChange?.({ checked });
  }

  let className = $derived(
    [
      "toggle",
      checked ? "toggle-active" : "",
      disabled ? "toggle-disabled" : "",
      `toggle-${size}`,
    ]
      .filter(Boolean)
      .join(" "),
  );
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={ariaLabel}
  {disabled}
  class={className}
  onclick={toggle}
>
  <span class="toggle-knob"></span>
</button>

<style>
  @import "../../styles/variables.css";

  .toggle {
    position: relative;
    border: none;
    cursor: pointer;
    transition: all var(--transition-base);
    background: var(--base-mid-gray);
    box-shadow: var(--inset-border);
    flex-shrink: 0;
  }

  .toggle:hover:not(.toggle-disabled) {
    background: var(--base-card);
  }

  .toggle-active {
    background: var(--accent-green);
    box-shadow: none;
  }

  .toggle-knob {
    position: absolute;
    top: 50%;
    left: 3px;
    transform: translateY(-50%);
    width: calc(100% - 6px);
    max-width: 25px;
    height: calc(100% - 6px);
    background: white;
    border-radius: var(--radius-circle);
    box-shadow: 0 3px 8px rgba(0, 0, 0, 0.3);
    transition: transform var(--transition-base);
  }

  .toggle-active .toggle-knob {
    transform: translateY(-50%) translateX(20px);
  }

  /* ========== Sizes ========== */
  .toggle-sm {
    width: 40px;
    height: 24px;
    border-radius: 12px;
  }

  .toggle-sm .toggle-knob {
    width: 18px;
    height: 18px;
  }

  .toggle-md {
    width: 51px;
    height: 31px;
    border-radius: 16px;
  }

  .toggle-md .toggle-knob {
    width: 25px;
    height: 25px;
  }

  .toggle-lg {
    width: 60px;
    height: 36px;
    border-radius: 18px;
  }

  .toggle-lg .toggle-knob {
    width: 30px;
    height: 30px;
  }

  /* ========== Disabled ========== */
  .toggle-disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>

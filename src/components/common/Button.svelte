<script>
  export let variant = 'primary'; // primary, secondary, outlined, danger, circular
  export let size = 'md'; // sm, md, lg
  export let disabled = false;
  export let href = null;
  export let type = 'button';
  export let ariaLabel = '';
  
  let className = $derived([
    'btn',
    `btn-${variant}`,
    `btn-${size}`,
    disabled ? 'btn-disabled' : ''
  ].filter(Boolean).join(' '));
</script>

{#if href}
  <a
    {href}
    class={className}
    {ariaLabel}
    role="button"
  >
    <slot />
  </a>
{:else}
  <button
    {type}
    {disabled}
    class={className}
    {ariaLabel}
  >
    <slot />
  </button>
{/if}

<style>
  @import '../../styles/variables.css';
  
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-sm);
    font-family: var(--font-family-ui);
    font-weight: var(--font-bold);
    text-transform: uppercase;
    letter-spacing: var(--tracking-wide);
    border: none;
    cursor: pointer;
    transition: all var(--transition-base);
    text-decoration: none;
  }
  
  .btn:not(.btn-circular):not(.btn-disabled):hover {
    transform: scale(1.02);
  }
  
  .btn:not(.btn-circular):not(.btn-disabled):active {
    transform: scale(0.98);
  }
  
  /* ========== Variants ========== */
  .btn-primary {
    background: var(--accent-green);
    color: var(--base-dark);
    border-radius: var(--radius-pill);
  }
  
  .btn-primary:hover:not(.btn-disabled) {
    background: var(--accent-green-dark);
  }
  
  .btn-secondary {
    background: var(--base-mid-gray);
    color: var(--text-base);
    border-radius: var(--radius-pill);
  }
  
  .btn-secondary:hover:not(.btn-disabled) {
    background: var(--base-card);
  }
  
  .btn-outlined {
    background: transparent;
    color: var(--text-base);
    border: 1px solid var(--border-light);
    border-radius: var(--radius-pill);
    padding-left: calc(var(--spacing-lg) + 8px);
  }
  
  .btn-outlined:hover:not(.btn-disabled) {
    border-color: var(--text-base);
    background: rgba(255, 255, 255, 0.05);
  }
  
  .btn-danger {
    background: var(--text-negative);
    color: white;
    border-radius: var(--radius-pill);
  }
  
  .btn-danger:hover:not(.btn-disabled) {
    opacity: 0.9;
  }
  
  .btn-circular {
    background: var(--base-mid-gray);
    color: var(--text-base);
    border-radius: var(--radius-circle);
    padding: var(--spacing-md);
  }
  
  .btn-circular:hover:not(.btn-disabled) {
    background: var(--base-card);
  }
  
  .btn-circular.btn-primary {
    background: var(--accent-green);
    color: var(--base-dark);
  }
  
  /* ========== Sizes ========== */
  .btn-sm {
    padding: var(--spacing-xs) var(--spacing-md);
    font-size: var(--text-sm);
    letter-spacing: var(--tracking-normal);
  }
  
  .btn-md {
    padding: var(--spacing-sm) var(--spacing-lg);
    font-size: var(--text-md);
  }
  
  .btn-lg {
    padding: var(--spacing-md) var(--spacing-xl);
    font-size: var(--text-lg);
    letter-spacing: var(--tracking-wider);
  }
  
  .btn-circular.btn-sm {
    padding: var(--spacing-xs);
    width: 32px;
    height: 32px;
  }
  
  .btn-circular.btn-md {
    padding: var(--spacing-sm);
    width: 40px;
    height: 40px;
  }
  
  .btn-circular.btn-lg {
    padding: var(--spacing-md);
    width: 48px;
    height: 48px;
  }
  
  /* ========== Disabled ========== */
  .btn-disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none !important;
  }
</style>

<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    variant = "default",
    padding = "md",
    hoverable = false,
    href = null,
    class: className = "",
    children,
  }: {
    variant?: string;
    padding?: string;
    hoverable?: boolean;
    href?: string | null;
    class?: string;
    children?: Snippet;
  } = $props();

  let resolvedClass = $derived(
    [
      "card",
      `card-${variant}`,
      `card-padding-${padding}`,
      hoverable ? "card-hoverable" : "",
      className,
    ]
      .filter(Boolean)
      .join(" "),
  );
</script>

{#if href}
  <a {href} class={resolvedClass}>
    {@render children!()}
  </a>
{:else}
  <div class={resolvedClass}>
    {@render children!()}
  </div>
{/if}

<style>
  @import "../../styles/variables.css";

  .card {
    background: var(--base-dark-gray);
    border-radius: var(--radius-card);
    transition: all var(--transition-base);
  }

  .card-default {
    box-shadow: var(--shadow-medium);
  }

  .card-elevated {
    box-shadow: var(--shadow-heavy);
  }

  .card-interactive {
    background: var(--base-mid-gray);
    cursor: pointer;
  }

  .card-interactive:hover {
    background: var(--base-card);
    transform: translateY(-2px);
  }

  /* ========== Padding Variants ========== */
  .card-padding-none {
    padding: 0;
  }

  .card-padding-sm {
    padding: var(--spacing-sm);
  }

  .card-padding-md {
    padding: var(--spacing-md);
  }

  .card-padding-lg {
    padding: var(--spacing-lg);
  }

  /* ========== Hoverable ========== */
  .card-hoverable:hover {
    background: var(--base-mid-gray);
  }
</style>

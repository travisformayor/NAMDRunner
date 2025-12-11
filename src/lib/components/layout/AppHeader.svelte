<script lang="ts">
  import { breadcrumbs, theme, uiStore } from '../../stores/ui';
  import ConnectionDropdown from './ConnectionDropdown.svelte';

  function toggleTheme() {
    const newTheme = $theme === 'light' ? 'dark' : 'light';
    uiStore.setTheme(newTheme);
    localStorage.setItem('namd-theme', newTheme);
  }
</script>

<header class="app-header">
  <div class="header-left">
    <nav class="breadcrumbs" aria-label="Breadcrumb">
      <ol class="breadcrumb-list">
        {#each $breadcrumbs as crumb, index}
          <li class="breadcrumb-item">
            {#if crumb.onClick}
              <button
                class="breadcrumb-link"
                on:click={crumb.onClick}
                type="button"
              >
                {crumb.label}
              </button>
            {:else}
              <span class="breadcrumb-current">{crumb.label}</span>
            {/if}
            {#if index < $breadcrumbs.length - 1}
              <span class="breadcrumb-separator">›</span>
            {/if}
          </li>
        {/each}
      </ol>
    </nav>
  </div>

  <div class="header-right">
    <button
      class="theme-toggle"
      on:click={toggleTheme}
      title="Toggle {$theme === 'light' ? 'dark' : 'light'} theme"
      aria-label="Toggle theme"
    >
      {#if $theme === 'light'}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
        </svg>
      {:else}
        ☀️
      {/if}
    </button>

    <ConnectionDropdown />
  </div>
</header>

<style>
  .app-header {
    height: var(--namd-header-height);
    background-color: var(--namd-bg-primary);
    border-bottom: 1px solid var(--namd-border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--namd-spacing-lg);
    flex-shrink: 0;
  }

  .header-left {
    flex: 1;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-md);
  }

  .breadcrumbs {
    display: flex;
    align-items: center;
  }

  .breadcrumb-list {
    display: flex;
    align-items: center;
    list-style: none;
    margin: 0;
    padding: 0;
    gap: var(--namd-spacing-xs);
  }

  .breadcrumb-item {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-xs);
  }

  .breadcrumb-link {
    background: none;
    border: none;
    color: var(--namd-text-secondary);
    cursor: pointer;
    font-size: var(--namd-font-size-base);
    font-weight: var(--namd-font-weight-medium);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
    transition: all var(--namd-transition-fast);
  }

  .breadcrumb-link:hover {
    background-color: var(--namd-accent);
    color: var(--namd-text-primary);
  }

  .breadcrumb-current {
    color: var(--namd-text-primary);
    font-size: var(--namd-font-size-base);
    font-weight: var(--namd-font-weight-medium);
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
  }

  .breadcrumb-separator {
    color: var(--namd-text-secondary);
    font-size: var(--namd-font-size-base);
    margin: 0 var(--namd-spacing-xs);
  }

  .theme-toggle {
    background: none;
    border: none;
    font-size: 1.25rem;
    cursor: pointer;
    padding: var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
    transition: background-color var(--namd-transition-fast);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .theme-toggle:hover {
    background-color: var(--namd-accent);
  }

</style>
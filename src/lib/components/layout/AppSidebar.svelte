<script lang="ts">
  import { currentView, uiStore } from '../../stores/ui';
  import { isConnected } from '../../stores/session';
  import { jobCounts } from '../../stores/jobs';

  interface NavItem {
    id: string;
    label: string;
    icon: string;
    view: 'jobs' | 'create' | 'settings';
    disabled?: boolean;
    badge?: string | number;
  }

  $: navItems = [
    {
      id: 'jobs',
      label: 'Jobs',
      icon: 'briefcase',
      view: 'jobs' as const,
      badge: $jobCounts.total > 0 ? $jobCounts.total : undefined
    },
    {
      id: 'create',
      label: 'Create Job',
      icon: 'plus',
      view: 'create' as const,
      disabled: !$isConnected
    }
  ] as NavItem[];

  function handleNavClick(view: 'jobs' | 'create') {
    if (navItems.find(item => item.view === view)?.disabled) return;
    uiStore.setView(view);
  }

  function renderIcon(iconName: string) {
    switch (iconName) {
      case 'briefcase':
        return `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="2" y="7" width="20" height="14" rx="2" ry="2"></rect>
                  <path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"></path>
                </svg>`;
      case 'plus':
        return `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="12" y1="5" x2="12" y2="19"></line>
                  <line x1="5" y1="12" x2="19" y2="12"></line>
                </svg>`;
      default:
        return '';
    }
  }
</script>

<nav class="sidebar">
  <div class="sidebar-header">
    <div class="logo">
      <span class="logo-text">🧬 NAMDRunner</span>
    </div>
  </div>

  <div class="nav-items">
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={$currentView === item.view}
        class:disabled={item.disabled}
        disabled={item.disabled}
        on:click={() => handleNavClick(item.view)}
        title={item.disabled ? (item.id === 'create' ? 'Connect to cluster first' : 'Coming soon') : item.label}
      >
        <span class="nav-icon">{@html renderIcon(item.icon)}</span>
        <span class="nav-label">{item.label}</span>
        {#if item.badge}
          <span class="nav-badge">{item.badge}</span>
        {/if}
      </button>
    {/each}
  </div>

</nav>

<style>
  .sidebar {
    width: 192px; /* w-48 = 12rem = 192px */
    background-color: var(--namd-sidebar-bg);
    border-right: 1px solid var(--namd-sidebar-border);
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .sidebar-header {
    padding: 16px;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: var(--namd-spacing-sm);
  }

  .logo-text {
    font-size: 1.25rem; /* text-xl */
    font-weight: var(--namd-font-weight-medium);
    color: var(--namd-sidebar-fg);
  }

  .nav-items {
    flex: 1;
    padding: 0 8px; /* px-2 */
    display: flex;
    flex-direction: column;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 12px; /* mr-3 equivalent */
    padding: 8px 12px;
    background: none;
    border: none;
    border-radius: var(--namd-border-radius-sm);
    color: var(--namd-sidebar-fg);
    cursor: pointer;
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-normal);
    text-align: left;
    transition: all 0.15s ease;
    width: 100%;
    position: relative;
    margin-bottom: 4px; /* mb-1 */
    justify-content: flex-start;
  }

  .nav-item:hover:not(.disabled) {
    background-color: var(--namd-sidebar-hover);
  }

  .nav-item.active {
    background-color: var(--namd-sidebar-active);
    color: var(--namd-sidebar-active-fg);
  }

  .nav-item.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .nav-icon {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .nav-label {
    flex: 1;
  }

  .nav-badge {
    background-color: var(--namd-primary);
    color: var(--namd-primary-fg);
    border-radius: 9999px;
    padding: 0.125rem 0.375rem;
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-medium);
    min-width: 1.25rem;
    text-align: center;
    line-height: 1;
  }

</style>
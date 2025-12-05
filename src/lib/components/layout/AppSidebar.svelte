<script lang="ts">
  import { currentView, uiStore } from '../../stores/ui';
  import { isConnected } from '../../stores/session';
  import { jobCounts } from '../../stores/jobs';

  interface NavItem {
    id: string;
    label: string;
    icon: string;
    view: 'jobs' | 'create' | 'templates' | 'settings';
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
    },
    {
      id: 'templates',
      label: 'Templates',
      icon: 'template',
      view: 'templates' as const
    },
    {
      id: 'settings',
      label: 'Settings',
      icon: 'settings',
      view: 'settings' as const
    }
  ] as NavItem[];

  function handleNavClick(view: NavItem['view']) {
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
      case 'template':
        return `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                  <line x1="9" y1="9" x2="15" y2="9"></line>
                  <line x1="9" y1="13" x2="15" y2="13"></line>
                  <line x1="9" y1="17" x2="13" y2="17"></line>
                </svg>`;
      case 'settings':
        return `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
                  <path d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
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
        title={item.disabled && item.id === 'create' ? 'Connect to cluster first' : item.label}
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
    font-size: var(--namd-font-size-base);
    font-weight: var(--namd-font-weight-normal);
    text-align: left;
    transition: all var(--namd-transition-fast);
    width: 100%;
    position: relative;
    margin-bottom: 4px;
    justify-content: flex-start;
  }

  .nav-item:hover:not(.disabled):not(.active) {
    background-color: var(--namd-sidebar-hover);
    color: var(--namd-sidebar-hover-fg);
  }

  .nav-item.active {
    background-color: var(--namd-sidebar-active);
    color: var(--namd-sidebar-active-fg);
  }

  .nav-item.active:hover {
    background-color: var(--namd-sidebar-active-hover);
    color: var(--namd-sidebar-active-hover-fg);
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
    padding: var(--namd-spacing-xs) 0.375rem;
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-medium);
    min-width: 1.25rem;
    text-align: center;
    line-height: 1;
  }

</style>
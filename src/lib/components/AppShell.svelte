<script lang="ts">
  import { onMount } from 'svelte';
  import AppSidebar from './layout/AppSidebar.svelte';
  import AppHeader from './layout/AppHeader.svelte';
  import SSHConsolePanel from './layout/SSHConsolePanel.svelte';
  import JobsPage from './pages/JobsPage.svelte';
  import JobDetailPage from './pages/JobDetailPage.svelte';
  import CreateJobPage from './pages/CreateJobPage.svelte';
  import { currentView, selectedJobId, consoleOpen, uiStore } from '../stores/ui';
  import { jobsStore } from '../stores/jobs';
  import { sessionActions } from '../stores/session';

  onMount(() => {
    // Initialize theme from localStorage or system preference
    const savedTheme = localStorage.getItem('namd-theme');
    const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    const theme = savedTheme || systemTheme;
    uiStore.setTheme(theme as 'light' | 'dark');

    // Listen for system theme changes
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleThemeChange = (e: MediaQueryListEvent) => {
      if (!localStorage.getItem('namd-theme')) {
        uiStore.setTheme(e.matches ? 'dark' : 'light');
      }
    };
    mediaQuery.addEventListener('change', handleThemeChange);

    // Add mock connected state for UI testing - wait for components to mount
    setTimeout(() => {
      sessionActions.mockConnected();
    }, 500);

    return () => {
      mediaQuery.removeEventListener('change', handleThemeChange);
    };
  });

</script>

<div class="app-shell">
  <!-- Sidebar -->
  <AppSidebar />

  <!-- Main Content Area -->
  <div class="main-content">
    <!-- Header -->
    <AppHeader />

    <!-- Content -->
    <div class="content-area">
      {#if $currentView === 'jobs' && $selectedJobId}
        <JobDetailPage />
      {:else if $currentView === 'create'}
        <CreateJobPage />
      {:else}
        <JobsPage />
      {/if}
    </div>

    <!-- SSH Console -->
    <SSHConsolePanel />
  </div>
</div>

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    width: 100vw;
    background-color: var(--namd-bg-primary);
    color: var(--namd-text-primary);
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .content-area {
    flex: 1;
    overflow: auto;
    background-color: var(--namd-bg-secondary);
  }
</style>
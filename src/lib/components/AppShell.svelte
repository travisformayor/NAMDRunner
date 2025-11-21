<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import AppSidebar from './layout/AppSidebar.svelte';
  import AppHeader from './layout/AppHeader.svelte';
  import LogsPanel from './layout/LogsPanel.svelte';
  import ToastContainer from './ui/ToastContainer.svelte';
  import JobsPage from './pages/JobsPage.svelte';
  import JobDetailPage from './pages/JobDetailPage.svelte';
  import CreateJobPage from './pages/CreateJobPage.svelte';
  import TemplatesPage from './pages/TemplatesPage.svelte';
  import TemplateEditorPage from './pages/TemplateEditorPage.svelte';
  import SettingsPage from './pages/SettingsPage.svelte';
  import { currentView, selectedJobId, uiStore } from '../stores/ui';
  import { clusterConfig } from '../stores/clusterConfig';
  import { templateStore } from '../stores/templateStore';
  import { jobsStore } from '../stores/jobs';
  import type { ApiResult, AppInitializationData } from '../types/api';

  onMount(() => {
    // Initialize app data from backend using centralized command
    (async () => {
      const result = await invoke<ApiResult<AppInitializationData>>('initialize_app');

      if (result.success && result.data) {
        clusterConfig.set(result.data.capabilities);
        templateStore.setTemplates(result.data.templates);
        jobsStore.setJobs(result.data.jobs);
      }
    })();

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
      {#if $currentView === 'settings'}
        <SettingsPage />
      {:else if $currentView === 'jobs' && $selectedJobId}
        <JobDetailPage />
      {:else if $currentView === 'create'}
        <CreateJobPage />
      {:else if $currentView === 'templates'}
        <TemplatesPage />
      {:else if $currentView === 'template-edit'}
        <TemplateEditorPage />
      {:else}
        <JobsPage />
      {/if}
    </div>

    <!-- Logs Panel -->
    <LogsPanel />
  </div>

  <!-- Toast Notifications -->
  <ToastContainer />
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
<script lang="ts">
  import { onMount } from 'svelte';
  import AppShell from '../lib/components/AppShell.svelte';
  import { sessionActions } from '../lib/stores/session';
  import '../lib/styles/app.css';

  onMount(async () => {
    // Check connection status on app start
    await sessionActions.checkStatus();
  });
</script>

<AppShell />
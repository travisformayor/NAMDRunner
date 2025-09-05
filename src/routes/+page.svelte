<script lang="ts">
  import { onMount } from 'svelte';
  import ConnectionStatus from '../lib/components/ConnectionStatus.svelte';
  import ConnectionDialog from '../lib/components/ConnectionDialog.svelte';
  import { sessionActions, isConnected } from '../lib/stores/session';

  let showConnectionDialog = false;

  onMount(async () => {
    // Check connection status on app start
    await sessionActions.checkStatus();
  });

  function handleConnect() {
    showConnectionDialog = true;
  }

  function handleDisconnect() {
    sessionActions.disconnect();
  }

  function closeConnectionDialog() {
    showConnectionDialog = false;
  }
</script>

<main class="container">
  <header class="app-header">
    <h1>NAMDRunner</h1>
    <p class="subtitle">SLURM NAMD Simulation Manager</p>
  </header>

  <div class="main-content">
    <section class="connection-section">
      <h2>Cluster Connection</h2>
      <ConnectionStatus
        onConnect={handleConnect}
        onDisconnect={handleDisconnect}
      />
    </section>

    {#if $isConnected}
      <section class="jobs-section">
        <h2>Jobs</h2>
        <div class="placeholder-content">
          <p>Job management interface will appear here once connected.</p>
          <p>📋 Create and submit NAMD simulations</p>
          <p>📊 Track job status and progress</p>
          <p>📁 Manage input and output files</p>
        </div>
      </section>
    {:else}
      <section class="welcome-section">
        <div class="welcome-content">
          <h3>Welcome to NAMDRunner</h3>
          <p>Connect to your SLURM cluster to begin managing NAMD molecular dynamics simulations.</p>
          
          <div class="features">
            <div class="feature">
              <div class="feature-icon">🔗</div>
              <div class="feature-text">
                <h4>Secure SSH Connection</h4>
                <p>Password-based authentication to SLURM clusters</p>
              </div>
            </div>
            
            <div class="feature">
              <div class="feature-icon">⚡</div>
              <div class="feature-text">
                <h4>NAMD Integration</h4>
                <p>Automated job creation and submission for NAMD simulations</p>
              </div>
            </div>
            
            <div class="feature">
              <div class="feature-icon">📊</div>
              <div class="feature-text">
                <h4>Status Tracking</h4>
                <p>Real-time monitoring of job progress and completion</p>
              </div>
            </div>
          </div>
        </div>
      </section>
    {/if}
  </div>

  <ConnectionDialog
    isOpen={showConnectionDialog}
    onClose={closeConnectionDialog}
  />
</main>

<style>
  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background-color: #f8fafc;
    color: #1f2937;
    margin: 0;
    padding: 0;
  }

  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
    min-height: 100vh;
  }

  .app-header {
    text-align: center;
    margin-bottom: 40px;
    padding: 20px 0;
  }

  .app-header h1 {
    font-size: 2.5rem;
    font-weight: 700;
    color: #1f2937;
    margin: 0 0 8px 0;
  }

  .subtitle {
    font-size: 1.1rem;
    color: #6b7280;
    margin: 0;
  }

  .main-content {
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .connection-section,
  .jobs-section,
  .welcome-section {
    background: white;
    border-radius: 12px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    overflow: hidden;
  }

  .connection-section h2,
  .jobs-section h2 {
    background: #f8fafc;
    margin: 0;
    padding: 16px 20px;
    border-bottom: 1px solid #e5e7eb;
    font-size: 1.25rem;
    font-weight: 600;
    color: #374151;
  }

  .connection-section {
    padding: 0;
  }

  .connection-section :global(.connection-status) {
    margin: 20px;
    box-shadow: none;
    border: none;
    background: transparent;
  }

  .jobs-section {
    padding: 0;
  }

  .placeholder-content {
    padding: 40px 20px;
    text-align: center;
    color: #6b7280;
  }

  .placeholder-content p {
    margin: 8px 0;
    font-size: 1rem;
  }

  .welcome-section {
    padding: 40px;
  }

  .welcome-content {
    max-width: 800px;
    margin: 0 auto;
    text-align: center;
  }

  .welcome-content h3 {
    font-size: 1.875rem;
    font-weight: 700;
    color: #1f2937;
    margin: 0 0 16px 0;
  }

  .welcome-content > p {
    font-size: 1.125rem;
    color: #6b7280;
    margin: 0 0 40px 0;
    line-height: 1.7;
  }

  .features {
    display: grid;
    gap: 32px;
    margin-top: 40px;
  }

  .feature {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    text-align: left;
  }

  .feature-icon {
    font-size: 2rem;
    flex-shrink: 0;
    margin-top: 4px;
  }

  .feature-text h4 {
    font-size: 1.125rem;
    font-weight: 600;
    color: #1f2937;
    margin: 0 0 8px 0;
  }

  .feature-text p {
    color: #6b7280;
    margin: 0;
    line-height: 1.6;
  }

  @media (min-width: 768px) {
    .features {
      grid-template-columns: 1fr 1fr 1fr;
    }

    .feature {
      flex-direction: column;
      text-align: center;
      align-items: center;
    }

    .feature-icon {
      margin-top: 0;
    }
  }

  @media (prefers-color-scheme: dark) {
    :global(body) {
      background-color: #0f172a;
      color: #f1f5f9;
    }

    .app-header h1 {
      color: #f1f5f9;
    }

    .subtitle {
      color: #94a3b8;
    }

    .connection-section,
    .jobs-section,
    .welcome-section {
      background: #1e293b;
      border: 1px solid #334155;
    }

    .connection-section h2,
    .jobs-section h2 {
      background: #0f172a;
      color: #e2e8f0;
      border-bottom-color: #334155;
    }

    .placeholder-content {
      color: #94a3b8;
    }

    .welcome-content h3 {
      color: #f1f5f9;
    }

    .welcome-content > p {
      color: #94a3b8;
    }

    .feature-text h4 {
      color: #f1f5f9;
    }

    .feature-text p {
      color: #94a3b8;
    }
  }
</style>

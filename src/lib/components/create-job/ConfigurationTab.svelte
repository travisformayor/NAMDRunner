<script lang="ts">
  import type { NAMDConfig } from '../../types/api';

  export let job_name: string;
  export let namdConfig: NAMDConfig;
  export let errors: Record<string, string>;

  // Helper function to toggle execution mode
  function toggleExecutionMode() {
    namdConfig.execution_mode = namdConfig.execution_mode === 'minimize' ? 'run' : 'minimize';
  }

  // Manage cell basis vectors when PME is toggled
  $: {
    if (namdConfig.pme_enabled) {
      // Initialize cell basis vectors if they don't exist
      if (!namdConfig.cell_basis_vector1) {
        namdConfig.cell_basis_vector1 = { x: 0, y: 0, z: 0 };
      }
      if (!namdConfig.cell_basis_vector2) {
        namdConfig.cell_basis_vector2 = { x: 0, y: 0, z: 0 };
      }
      if (!namdConfig.cell_basis_vector3) {
        namdConfig.cell_basis_vector3 = { x: 0, y: 0, z: 0 };
      }
    } else {
      // Clear cell basis vectors when PME is disabled
      delete namdConfig.cell_basis_vector1;
      delete namdConfig.cell_basis_vector2;
      delete namdConfig.cell_basis_vector3;
    }
  }
</script>

<div class="namd-tab-panel">
  <!-- Basic Job Information -->
  <div class="namd-section">
    <div class="namd-section-header">
      <h3 class="namd-section-title">Job Information</h3>
    </div>
    <div class="config-grid">
      <div class="field-group">
        <label class="namd-label" for="jobName">Job Name *</label>
        <input
          class="namd-input"
          id="jobName"
          type="text"
          bind:value={job_name}
          placeholder="my-simulation"
          class:error={errors.job_name}
        />
        {#if errors.job_name}
          <span class="error-text">{errors.job_name}</span>
        {/if}
      </div>

      <div class="field-group">
        <label class="namd-label" for="outputName">Output Basename *</label>
        <input
          class="namd-input"
          id="outputName"
          type="text"
          bind:value={namdConfig.outputname}
          placeholder="output"
          class:error={errors.outputname}
        />
        {#if errors.outputname}
          <span class="error-text">{errors.outputname}</span>
        {/if}
      </div>
    </div>
  </div>

  <!-- Simulation Mode and Parameters -->
  <div class="namd-section">
    <div class="namd-section-header">
      <h3 class="namd-section-title">Simulation Parameters</h3>
    </div>
    <div class="config-grid">
      <div class="field-group">
        <label class="namd-label">Execution Mode *</label>
        <div class="radio-group">
          <label class="radio-label">
            <input
              type="radio"
              value="minimize"
              bind:group={namdConfig.execution_mode}
            />
            Minimize
          </label>
          <label class="radio-label">
            <input
              type="radio"
              value="run"
              bind:group={namdConfig.execution_mode}
            />
            Run (MD)
          </label>
        </div>
      </div>

      <div class="field-group">
        <label class="namd-label" for="simulationSteps">
          {namdConfig.execution_mode === 'minimize' ? 'Minimization' : 'Simulation'} Steps *
        </label>
        <input
          class="namd-input"
          id="simulationSteps"
          type="number"
          bind:value={namdConfig.steps}
          min="1"
          class:error={errors.steps}
        />
        {#if errors.steps}
          <span class="error-text">{errors.steps}</span>
        {/if}
      </div>

      <div class="field-group">
        <label class="namd-label" for="temperature">Temperature (K) *</label>
        <input
          class="namd-input"
          id="temperature"
          type="number"
          bind:value={namdConfig.temperature}
          min="1"
          max="1000"
          class:error={errors.temperature}
        />
        {#if errors.temperature}
          <span class="error-text">{errors.temperature}</span>
        {/if}
      </div>

      <div class="field-group">
        <label class="namd-label" for="timestep">Timestep (fs) *</label>
        <input
          class="namd-input"
          id="timestep"
          type="number"
          bind:value={namdConfig.timestep}
          min="0.1"
          max="4.0"
          step="0.1"
          class:error={errors.timestep}
        />
        {#if errors.timestep}
          <span class="error-text">{errors.timestep}</span>
        {/if}
      </div>

      <div class="field-group">
        <label class="namd-label" for="langevinDamping">Langevin Damping *</label>
        <input
          class="namd-input"
          id="langevinDamping"
          type="number"
          bind:value={namdConfig.langevin_damping}
          min="0.1"
          step="0.1"
          class:error={errors.langevin_damping}
        />
        {#if errors.langevin_damping}
          <span class="error-text">{errors.langevin_damping}</span>
        {/if}
      </div>
    </div>
  </div>

  <!-- Periodic Boundary Conditions -->
  <div class="namd-section">
    <div class="namd-section-header">
      <h3 class="namd-section-title">Periodic Boundary Conditions</h3>
    </div>

    <div class="checkbox-group">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={namdConfig.pme_enabled} />
        Enable PME (Particle Mesh Ewald)
      </label>
      <span class="field-hint">Required for explicit solvent simulations</span>
    </div>

    {#if namdConfig.pme_enabled}
      <p class="field-hint" style="margin-bottom: 1rem;">
        Cell basis vectors define the periodic boundary box. For solvated systems, these are typically found in the CRYST1 record of your PDB file or from the solvation process.
      </p>
      <div class="config-grid cell-basis-grid">
        <div class="field-group">
          <label class="namd-label">Cell Basis Vector 1 (Å) *</label>
          <div class="vector-inputs">
            <input
              class="namd-input vector-input"
              type="number"
              placeholder="X"
              bind:value={namdConfig.cell_basis_vector1!.x}
              step="0.1"
              required
            />
            <input
              class="namd-input vector-input"
              type="number"
              placeholder="Y"
              bind:value={namdConfig.cell_basis_vector1!.y}
              step="0.1"
              required
            />
            <input
              class="namd-input vector-input"
              type="number"
              placeholder="Z"
              bind:value={namdConfig.cell_basis_vector1!.z}
              step="0.1"
              required
            />
          </div>
        </div>

        <div class="field-group">
          <label class="namd-label">Cell Basis Vector 2 (Å) *</label>
          <div class="vector-inputs">
            <input
              class="namd-input vector-input"
              type="number"
              placeholder="X"
              bind:value={namdConfig.cell_basis_vector2!.x}
              step="0.1"
              required
            />
            <input
              class="namd-input vector-input"
              type="number"
              placeholder="Y"
              bind:value={namdConfig.cell_basis_vector2!.y}
              step="0.1"
              required
            />
            <input
              class="namd-input vector-input"
              type="number"
              placeholder="Z"
              bind:value={namdConfig.cell_basis_vector2!.z}
              step="0.1"
              required
            />
          </div>
        </div>

        <div class="field-group">
          <label class="namd-label">Cell Basis Vector 3 (Å) *</label>
          <div class="vector-inputs">
            <input
              class="namd-input vector-input"
              type="number"
              placeholder="X"
              bind:value={namdConfig.cell_basis_vector3!.x}
              step="0.1"
              required
            />
            <input
              class="namd-input vector-input"
              type="number"
              placeholder="Y"
              bind:value={namdConfig.cell_basis_vector3!.y}
              step="0.1"
              required
            />
            <input
              class="namd-input vector-input"
              type="number"
              placeholder="Z"
              bind:value={namdConfig.cell_basis_vector3!.z}
              step="0.1"
              required
            />
          </div>
        </div>
      </div>
    {/if}

    <div class="checkbox-group">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={namdConfig.npt_enabled} />
        Enable NPT (Constant Pressure)
      </label>
      <span class="field-hint">Langevin piston for pressure control</span>
    </div>
  </div>

  <!-- Output Frequencies -->
  <div class="namd-section">
    <div class="namd-section-header">
      <h3 class="namd-section-title">Output Frequencies</h3>
      <span class="section-hint">In timesteps</span>
    </div>
    <div class="config-grid">
      <div class="field-group">
        <label class="namd-label" for="xstFreq">XST Frequency *</label>
        <input
          class="namd-input"
          id="xstFreq"
          type="number"
          bind:value={namdConfig.xst_freq}
          min="1"
        />
      </div>

      <div class="field-group">
        <label class="namd-label" for="outputEnergies">Energy Output *</label>
        <input
          class="namd-input"
          id="outputEnergies"
          type="number"
          bind:value={namdConfig.output_energies_freq}
          min="1"
        />
      </div>

      <div class="field-group">
        <label class="namd-label" for="dcdFreq">DCD Frequency *</label>
        <input
          class="namd-input"
          id="dcdFreq"
          type="number"
          bind:value={namdConfig.dcd_freq}
          min="1"
        />
      </div>

      <div class="field-group">
        <label class="namd-label" for="restartFreq">Restart Frequency *</label>
        <input
          class="namd-input"
          id="restartFreq"
          type="number"
          bind:value={namdConfig.restart_freq}
          min="1"
        />
      </div>

      <div class="field-group">
        <label class="namd-label" for="outputPressure">Pressure Output *</label>
        <input
          class="namd-input"
          id="outputPressure"
          type="number"
          bind:value={namdConfig.output_pressure_freq}
          min="1"
        />
      </div>
    </div>
  </div>
</div>

<style>
  .namd-tab-panel {
    padding: 1.5rem;
  }

  .namd-section {
    background: rgba(16, 185, 129, 0.05);
    border-radius: 8px;
    padding: 1.5rem;
    border: 1px solid rgba(16, 185, 129, 0.1);
    margin-bottom: 1.5rem;
  }

  .namd-section-header {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  .namd-section-title {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--namd-success);
  }

  .section-hint {
    font-size: 0.875rem;
    color: var(--namd-text-secondary);
  }

  .config-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5rem;
  }

  .cell-basis-grid {
    margin-top: 1rem;
  }

  .field-group {
    display: flex;
    flex-direction: column;
  }

  .namd-label {
    display: block;
    margin-bottom: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--namd-text-primary);
  }

  .namd-input {
    padding: 0.5rem 0.75rem;
    background: var(--namd-bg-primary);
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius-sm);
    color: var(--namd-text-primary);
    font-size: 0.875rem;
    transition: all 0.15s ease;
  }

  .namd-input:hover {
    border-color: var(--namd-text-muted);
  }

  .namd-input:focus {
    outline: none;
    border-color: var(--namd-success);
    box-shadow: 0 0 0 3px rgba(16, 185, 129, 0.1);
  }

  .namd-input.error {
    border-color: var(--namd-error);
  }

  .vector-inputs {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.5rem;
  }

  .vector-input {
    padding: 0.375rem 0.5rem;
  }

  .radio-group {
    display: flex;
    gap: 1.5rem;
    padding: 0.5rem 0;
  }

  .radio-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    color: var(--namd-text-primary);
    font-size: 0.875rem;
  }

  .radio-label input[type="radio"] {
    cursor: pointer;
  }

  .checkbox-group {
    margin: 1rem 0;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    color: var(--namd-text-primary);
    font-size: 0.875rem;
  }

  .checkbox-label input[type="checkbox"] {
    cursor: pointer;
  }

  .field-hint {
    font-size: 0.75rem;
    color: var(--namd-text-secondary);
    margin-left: 1.5rem;
    display: block;
    margin-top: 0.25rem;
  }

  .error-text {
    color: var(--namd-error);
    font-size: 0.75rem;
    margin-top: 0.25rem;
  }
</style>
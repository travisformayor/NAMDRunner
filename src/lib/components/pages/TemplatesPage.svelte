<script lang="ts">
  import { onMount } from 'svelte';
  import { templates, templatesLoading, templatesError, templateStore } from '$lib/stores/templateStore';
  import type { TemplateSummary } from '$lib/types/template';
  import { uiStore } from '$lib/stores/ui';
  import ConfirmDialog from '../ui/ConfirmDialog.svelte';

  let showDeleteConfirm = false;
  let deleteTargetId: string | null = null;
  let deleteTargetName: string | null = null;

  onMount(async () => {
    await templateStore.loadTemplates();
  });

  function handleCreateNew() {
    // Navigate to template editor in create mode
    uiStore.editTemplate(null, 'create');
  }

  function handleEdit(template: TemplateSummary) {
    // Navigate to template editor in edit mode
    uiStore.editTemplate(template.id, 'edit');
  }

  async function handleDuplicate(template: TemplateSummary) {
    // Load full template, create copy with new ID
    const fullTemplate = await templateStore.loadTemplate(template.id);

    if (fullTemplate) {
      const duplicatedTemplate = {
        ...fullTemplate,
        id: `${template.id}_copy_${Date.now()}`,
        name: `${template.name} (Copy)`,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        is_builtin: false
      };

      const success = await templateStore.createTemplate(duplicatedTemplate);
      if (success) {
        await templateStore.loadTemplates(); // Refresh list
      }
    }
  }

  function confirmDelete(template: TemplateSummary) {
    deleteTargetId = template.id;
    deleteTargetName = template.name;
    showDeleteConfirm = true;
  }

  async function handleDeleteConfirm() {
    if (!deleteTargetId) return;

    await templateStore.deleteTemplate(deleteTargetId);

    showDeleteConfirm = false;
    deleteTargetId = null;
    deleteTargetName = null;
  }

  function handleDeleteCancel() {
    showDeleteConfirm = false;
    deleteTargetId = null;
    deleteTargetName = null;
  }
</script>

<div class="templates-page">
  <div class="page-header">
    <h1>Simulation Templates</h1>
    <button class="namd-button namd-button--primary" on:click={handleCreateNew}>
      <span class="icon">+</span>
      Create Template
    </button>
  </div>

  {#if $templatesLoading}
    <div class="loading">Loading templates...</div>
  {:else if $templatesError}
    <div class="error">
      <strong>Error:</strong> {$templatesError}
    </div>
  {:else if $templates.length === 0}
    <div class="empty-state">
      <p>No templates available.</p>
      <p>Create a new template to get started.</p>
    </div>
  {:else}
    <!-- All Templates in Single List -->
    <div class="template-grid">
      {#each $templates as template}
        {@const isBuiltIn = template.is_builtin}
        <div class="template-card" class:built-in={isBuiltIn}>
          <div class="template-header">
            <h3>{template.name}</h3>
            <span class="badge" class:badge-builtin={isBuiltIn} class:badge-custom={!isBuiltIn}>
              {isBuiltIn ? 'Built-in' : 'Custom'}
            </span>
          </div>
          <p class="template-description">{template.description}</p>
          <div class="template-actions">
            <button class="namd-button namd-button--secondary namd-button--sm" on:click={() => handleEdit(template)}>
              Edit
            </button>
            <button class="namd-button namd-button--secondary namd-button--sm" on:click={() => handleDuplicate(template)}>
              Duplicate
            </button>
            <button class="namd-button namd-button--destructive namd-button--sm" on:click={() => confirmDelete(template)}>
              Delete
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
  isOpen={showDeleteConfirm}
  title="Delete Template?"
  message="Are you sure you want to delete '{deleteTargetName}'? This action cannot be undone."
  confirmText="Delete"
  cancelText="Cancel"
  confirmStyle="destructive"
  onConfirm={handleDeleteConfirm}
  onCancel={handleDeleteCancel}
/>

<style>
  .templates-page {
    padding: 2rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }

  .page-header h1 {
    margin: 0;
    font-size: var(--namd-font-size-2xl);
    color: var(--namd-text-primary);
  }

  .template-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1.5rem;
  }

  .template-card {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    padding: var(--namd-spacing-lg);
    background: var(--namd-bg-secondary);
    box-shadow: var(--namd-shadow-sm);
    transition: box-shadow 0.2s;
  }

  .template-card:hover {
    box-shadow: var(--namd-shadow-md);
  }

  .template-header {
    display: flex;
    justify-content: space-between;
    align-items: start;
    margin-bottom: 1rem;
  }

  .template-header h3 {
    margin: 0;
    font-size: var(--namd-font-size-xl);
    color: var(--namd-text-primary);
  }

  .badge {
    padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
    border-radius: var(--namd-border-radius-sm);
    font-size: var(--namd-font-size-xs);
    font-weight: var(--namd-font-weight-semibold);
    text-transform: uppercase;
  }

  .badge-builtin {
    background: var(--namd-info-bg);
    color: var(--namd-info-fg);
  }

  .badge-custom {
    background: var(--namd-success-bg);
    color: var(--namd-success-fg);
  }

  .template-description {
    color: var(--namd-text-secondary);
    margin-bottom: var(--namd-spacing-lg);
    flex-grow: 1;
  }

  .template-actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-top: auto;
  }

  .empty-state {
    text-align: center;
    padding: var(--namd-spacing-2xl);
    color: var(--namd-text-secondary);
  }

  .loading, .error {
    padding: var(--namd-spacing-xl);
    text-align: center;
  }

  .error {
    color: var(--namd-error);
  }

  .icon {
    margin-right: var(--namd-spacing-xs);
  }
</style>

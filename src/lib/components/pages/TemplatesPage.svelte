<script lang="ts">
  import { onMount } from 'svelte';
  import { templates, templatesLoading, templatesError, loadTemplates, deleteTemplate, loadTemplate, createTemplate } from '$lib/stores/templateStore';
  import type { TemplateSummary } from '$lib/types/template';
  import { uiStore } from '$lib/stores/ui';
  import ConfirmDialog from '../ui/ConfirmDialog.svelte';

  let showDeleteConfirm = false;
  let deleteTargetId: string | null = null;
  let deleteTargetName: string | null = null;

  onMount(async () => {
    await loadTemplates();
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
    const fullTemplate = await loadTemplate(template.id);

    if (fullTemplate) {
      const duplicatedTemplate = {
        ...fullTemplate,
        id: `${template.id}_copy_${Date.now()}`,
        name: `${template.name} (Copy)`,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        is_builtin: false
      };

      const success = await createTemplate(duplicatedTemplate);
      if (success) {
        await loadTemplates(); // Refresh list
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

    const success = await deleteTemplate(deleteTargetId);

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
    <button class="btn btn-primary" on:click={handleCreateNew}>
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
            <button class="btn btn-secondary btn-sm" on:click={() => handleEdit(template)}>
              Edit
            </button>
            <button class="btn btn-secondary btn-sm" on:click={() => handleDuplicate(template)}>
              Duplicate
            </button>
            <button class="btn btn-danger btn-sm" on:click={() => confirmDelete(template)}>
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
    font-size: 2rem;
  }

  .template-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1.5rem;
  }

  .template-card {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-color, #ddd);
    border-radius: 8px;
    padding: 1.5rem;
    background: var(--card-bg, white);
    transition: box-shadow 0.2s;
  }

  .template-card:hover {
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  }

  .template-card.built-in {
    border-color: var(--primary-light, #e3f2fd);
  }

  .template-header {
    display: flex;
    justify-content: space-between;
    align-items: start;
    margin-bottom: 1rem;
  }

  .template-header h3 {
    margin: 0;
    font-size: 1.25rem;
  }

  .badge {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  .badge-builtin {
    background: var(--primary-light, #e3f2fd);
    color: var(--primary, #1976d2);
  }

  .badge-custom {
    background: var(--success-light, #e8f5e9);
    color: var(--success, #2e7d32);
  }

  .template-description {
    color: var(--text-secondary, #666);
    margin-bottom: 1.5rem;
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
    padding: 3rem;
    color: var(--text-secondary, #666);
  }

  .loading, .error {
    padding: 2rem;
    text-align: center;
  }

  .error {
    color: var(--error, #d32f2f);
  }

  /* Button Styles */
  .btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: background 0.2s;
  }

  .btn-primary {
    background: var(--primary, #1976d2);
    color: white;
  }

  .btn-primary:hover {
    background: var(--primary-dark, #1565c0);
  }

  .btn-secondary {
    background: var(--secondary, #f5f5f5);
    color: var(--text-primary, #333);
  }

  .btn-secondary:hover {
    background: var(--secondary-dark, #e0e0e0);
  }

  .btn-danger {
    background: var(--error, #d32f2f);
    color: white;
  }

  .btn-danger:hover {
    background: var(--error-dark, #c62828);
  }

  .btn-sm {
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
  }

  .icon {
    margin-right: 0.5rem;
  }
</style>

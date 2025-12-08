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
        updated_at: new Date().toISOString()
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

  async function handleImport() {
    await templateStore.importTemplate();
  }

  async function handleExport(templateId: string) {
    await templateStore.exportTemplate(templateId);
  }
</script>

<div class="templates-page namd-page">
  <div class="page-header namd-page-header">
    <h1>Simulation Templates</h1>
    <div class="header-actions">
      <button class="namd-button namd-button--secondary" on:click={handleImport}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
          <polyline points="7,10 12,15 17,10"/>
          <line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        Import Template
      </button>
      <button class="namd-button namd-button--primary" on:click={handleCreateNew}>
        <span class="icon">+</span>
        Create Template
      </button>
    </div>
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
        <div class="template-card">
          <div class="template-header">
            <h3>{template.name}</h3>
          </div>
          <p class="template-description">{template.description}</p>
          <div class="template-actions">
            <button class="namd-button namd-button--secondary namd-button--sm" on:click={() => handleEdit(template)}>
              Edit
            </button>
            <button class="namd-button namd-button--secondary namd-button--sm" on:click={() => handleDuplicate(template)}>
              Duplicate
            </button>
            <button class="namd-button namd-button--secondary namd-button--sm" on:click={() => handleExport(template.id)} title="Export template to JSON file">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                <polyline points="17,8 12,3 7,8"/>
                <line x1="12" y1="3" x2="12" y2="15"/>
              </svg>
              Export
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
    padding: var(--namd-spacing-xl);
    max-width: var(--namd-max-width-content);
    margin: 0 auto;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--namd-spacing-xl);
  }

  .page-header h1 {
    margin: 0;
    font-size: var(--namd-font-size-2xl);
    color: var(--namd-text-primary);
  }

  .header-actions {
    display: flex;
    gap: var(--namd-spacing-sm);
  }

  .template-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--namd-spacing-lg);
  }

  .template-card {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--namd-border);
    border-radius: var(--namd-border-radius);
    padding: var(--namd-spacing-lg);
    background: var(--namd-bg-primary);
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
    margin-bottom: var(--namd-spacing-md);
  }

  .template-header h3 {
    margin: 0;
    font-size: var(--namd-font-size-xl);
    color: var(--namd-text-primary);
  }

  .template-description {
    color: var(--namd-text-secondary);
    margin-bottom: var(--namd-spacing-lg);
    flex-grow: 1;
  }

  .template-actions {
    display: flex;
    gap: var(--namd-spacing-sm);
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

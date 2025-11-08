<script lang="ts">
  import { selectedTemplateId, templateEditorMode, uiStore } from '$lib/stores/ui';
  import { loadTemplate } from '$lib/stores/templateStore';
  import TemplateEditor from '../templates/TemplateEditor.svelte';
  import type { Template } from '$lib/types/template';
  import { onMount } from 'svelte';

  let template: Template | null = null;
  let loading = true;
  let error: string | null = null;

  onMount(async () => {
    if ($selectedTemplateId && $templateEditorMode === 'edit') {
      try {
        template = await loadTemplate($selectedTemplateId);
        if (!template) {
          error = `Template not found: ${$selectedTemplateId}`;
        }
      } catch (e) {
        error = `Failed to load template: ${e}`;
      }
    }
    loading = false;
  });

  function handleSaved() {
    // Navigate back to templates list
    uiStore.setView('templates');
  }

  function handleCancel() {
    // Navigate back to templates list
    uiStore.setView('templates');
  }
</script>

<div class="template-editor-page">
  {#if loading}
    <div class="loading">Loading template...</div>
  {:else if error}
    <div class="error-message">{error}</div>
    <button class="btn btn-secondary" on:click={handleCancel}>Back to Templates</button>
  {:else}
    <TemplateEditor
      template={template}
      mode={$templateEditorMode}
      on:saved={handleSaved}
      on:cancel={handleCancel}
    />
  {/if}
</div>

<style>
  .template-editor-page {
    padding: 2rem;
    max-width: 1200px;
    margin: 0 auto;
    height: 100%;
    overflow-y: auto;
  }

  .loading {
    text-align: center;
    padding: var(--namd-spacing-xl);
    font-size: var(--namd-font-size-base);
    color: var(--namd-text-secondary);
  }

  .error-message {
    background: var(--namd-error-bg);
    border: 1px solid var(--namd-error-border);
    color: var(--namd-error-fg);
    padding: var(--namd-spacing-md);
    border-radius: var(--namd-border-radius);
    margin-bottom: var(--namd-spacing-lg);
  }

  .btn {
    padding: var(--namd-spacing-sm) var(--namd-spacing-md);
    border: none;
    border-radius: var(--namd-border-radius-sm);
    cursor: pointer;
    font-size: var(--namd-font-size-sm);
    font-weight: var(--namd-font-weight-medium);
    transition: all 0.15s ease;
  }

  .btn-secondary {
    background: var(--namd-secondary);
    color: var(--namd-secondary-fg);
  }

  .btn-secondary:hover {
    background: var(--namd-secondary-hover);
  }
</style>

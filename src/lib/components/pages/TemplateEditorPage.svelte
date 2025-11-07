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
    padding: 2rem;
    font-size: 1rem;
    color: var(--text-secondary, #666);
  }

  .error-message {
    background: var(--error-light, #ffebee);
    border: 1px solid var(--error, #d32f2f);
    color: var(--error-dark, #c62828);
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1.5rem;
  }

  .btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: background 0.2s;
  }

  .btn-secondary {
    background: var(--secondary, #f5f5f5);
    color: var(--text-primary, #333);
  }

  .btn-secondary:hover {
    background: var(--secondary-dark, #e0e0e0);
  }
</style>

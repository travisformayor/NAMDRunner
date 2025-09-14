# Task Management System

This directory contains all task planning and tracking for the NAMDRunner Tauri/Svelte/Rust migration.

## Structure

- **roadmap.md** - High-level phases and milestones
- **backlog/** - Pending tasks organized by category
  - core.md - Core functionality tasks
  - ui.md - Frontend tasks
  - backend.md - Rust backend tasks
  - testing.md - Testing infrastructure
  - deployment.md - Build & packaging tasks
- **active/** - Currently in-progress tasks (one at a time)
- **completed/** - Archived completed tasks
- **templates/** - Standard templates for tasks

## Workflow

### Starting a Task
1. Select task from backlog or roadmap
2. Create task file in `active/` using template
3. Fill out objective, context, and plan
4. Get approval before starting implementation

### Working on a Task
1. Update task file with discoveries and progress
2. Check off completed steps
3. Document any blockers or changes

### Completing a Task  
1. Verify all success criteria met
2. Update any affected documentation
3. Move task file to `completed/` with date prefix
4. Select next task from roadmap

## Task File Naming

- Active: `tasks/active/task-brief-description.md`
- Completed: `tasks/completed/task-brief-description.md`

## Priority Guidelines

1. **Critical Path**: Tasks blocking other work
2. **Core Features**: Essential functionality 
3. **User Experience**: UI and workflow improvements
4. **Nice to Have**: Optimizations and enhancements
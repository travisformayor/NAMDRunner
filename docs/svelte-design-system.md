# Design System for Svelte Implementation

## Overview

This document analyzes the React mockup implementation in `docs/design_mockup/` to guide the Svelte component architecture.

## Visual Reference

See actual UI implementation in mockup screenshots:
`docs/design_mockup/mockup_screenshots/`

These screenshots show the design system in action across different states and views.

## Design System Analysis from React Mockup

### Color Palette

The mockup uses a sophisticated design system with CSS custom properties that support both light and dark themes.

#### Light Theme Colors
```css
/* Core Colors */
--background: #ffffff;
--foreground: oklch(0.145 0 0);  /* ~#1f2937 dark gray */
--card: #ffffff;
--card-foreground: oklch(0.145 0 0);

/* Interactive Elements */
--primary: #030213;  /* Very dark blue/black */
--primary-foreground: oklch(1 0 0);  /* White */
--secondary: oklch(0.95 0.0058 264.53);  /* Light gray */
--secondary-foreground: #030213;

/* UI States */
--muted: #ececf0;  /* Light gray background */
--muted-foreground: #717182;  /* Medium gray text */
--accent: #e9ebef;  /* Slightly darker than muted */
--accent-foreground: #030213;
--destructive: #d4183d;  /* Red */
--destructive-foreground: #ffffff;

/* Borders and Inputs */
--border: rgba(0, 0, 0, 0.1);  /* Light gray with opacity */
--input: transparent;
--input-background: #f3f3f5;  /* Light gray input bg */

/* Sidebar Specific */
--sidebar: oklch(0.985 0 0);  /* Off-white */
--sidebar-foreground: oklch(0.145 0 0);
--sidebar-primary: #030213;
--sidebar-primary-foreground: oklch(0.985 0 0);
--sidebar-accent: oklch(0.97 0 0);
--sidebar-border: oklch(0.922 0 0);
```

#### Dark Theme Colors
```css
--background: oklch(0.145 0 0);  /* Dark background */
--foreground: oklch(0.985 0 0);  /* Light text */
--card: oklch(0.145 0 0);
--card-foreground: oklch(0.985 0 0);

--primary: oklch(0.985 0 0);  /* White in dark mode */
--primary-foreground: oklch(0.205 0 0);

--secondary: oklch(0.269 0 0);  /* Dark gray */
--muted: oklch(0.269 0 0);
--muted-foreground: oklch(0.708 0 0);  /* Light gray text */

--sidebar: oklch(0.205 0 0);  /* Darker sidebar */
--sidebar-primary: oklch(0.488 0.243 264.376);  /* Blue accent */
```

#### Status Colors (From Tailwind-based system)
```css
/* Success/Green */
--color-green-100: oklch(.962 .044 156.743);
--color-green-600: oklch(.627 .194 149.214);
--color-green-800: oklch(.448 .119 151.328);

/* Warning/Yellow */
--color-yellow-100: oklch(.973 .071 103.193);
--color-yellow-600: oklch(.681 .162 75.834);
--color-yellow-800: oklch(.476 .114 61.907);

/* Error/Red */
--color-red-100: oklch(.936 .032 17.717);
--color-red-600: oklch(.577 .245 27.325);
--color-red-800: oklch(.444 .177 26.899);

/* Info/Blue */
--color-blue-100: oklch(.932 .032 255.585);
--color-blue-600: oklch(.546 .245 262.881);
--color-blue-800: oklch(.424 .199 265.638);

/* Neutral/Gray */
--color-gray-100: oklch(.967 .003 264.542);
--color-gray-600: oklch(.446 .03 256.802);
--color-gray-800: oklch(.278 .033 256.848);
```

### Typography

#### Font Stacks
```css
--font-sans: ui-sans-serif, system-ui, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji";
--font-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
```

#### Font Sizes
```css
--font-size: 14px;  /* Base font size */
--text-xs: .75rem;
--text-sm: .875rem;
--text-base: 1rem;
--text-lg: 1.125rem;
--text-xl: 1.25rem;
--text-2xl: 1.5rem;
```

#### Font Weights
```css
--font-weight-normal: 400;
--font-weight-medium: 500;
--font-weight-semibold: 600;
```

#### Typography Usage Patterns
From the mockup's base layer:
- **h1**: 1.5rem (--text-2xl), medium weight
- **h2**: 1.25rem (--text-xl), medium weight
- **h3**: 1.125rem (--text-lg), medium weight
- **h4**: 1rem (--text-base), medium weight
- **p**: 1rem (--text-base), normal weight
- **label**: 1rem (--text-base), medium weight
- **button**: 1rem (--text-base), medium weight
- **input**: 1rem (--text-base), normal weight

### Spacing and Layout

#### Border Radius
```css
--radius: 0.625rem;  /* Base radius - 10px */
--radius-sm: calc(var(--radius) - 4px);  /* 6px */
--radius-md: calc(var(--radius) - 2px);  /* 8px */
--radius-lg: var(--radius);  /* 10px */
--radius-xl: calc(var(--radius) + 4px);  /* 14px */
```

#### Container Sizes
```css
--container-sm: 24rem;
--container-lg: 32rem;
```

#### Spacing Base
```css
--spacing: .25rem;  /* 4px base unit */
```

### Component-Specific Design Patterns

#### Status Badges
From `JobStatusBadge.tsx`, status badges use:
- **CREATED**: Gray background and text
- **PENDING**: Yellow/amber background and text
- **RUNNING**: Blue background and text
- **COMPLETED**: Green background and text
- **FAILED**: Red background and text
- **CANCELLED**: Dark gray background and text

#### Job Table
- **Sortable headers**: Clickable with chevron icons
- **Row hover**: Light muted background
- **Font families**: Monospace for job IDs, runtime values
- **Status integration**: Inline status badges

#### Connection Status
- **Dot indicators**: Small colored circles for status
- **Popover pattern**: Dropdown with form fields
- **State-based styling**: Different colors for each connection state

## Svelte CSS Custom Properties Implementation

### Root Variables Setup
```css
/* src/app.css */
:root {
  /* Theme Colors */
  --namd-bg-primary: #ffffff;
  --namd-bg-secondary: #f8fafc;
  --namd-text-primary: #1f2937;
  --namd-text-secondary: #6b7280;
  --namd-text-muted: #9ca3af;

  /* Interactive Elements */
  --namd-primary: #030213;
  --namd-primary-fg: #ffffff;
  --namd-secondary: #f3f4f6;
  --namd-secondary-fg: #1f2937;

  /* Status Colors */
  --namd-success: #10b981;
  --namd-success-bg: #ecfdf5;
  --namd-warning: #f59e0b;
  --namd-warning-bg: #fffbeb;
  --namd-error: #ef4444;
  --namd-error-bg: #fef2f2;
  --namd-info: #3b82f6;
  --namd-info-bg: #eff6ff;

  /* Layout */
  --namd-border: rgba(0, 0, 0, 0.1);
  --namd-border-radius: 0.625rem;
  --namd-sidebar-width: 12rem;

  /* Typography */
  --namd-font-sans: ui-sans-serif, system-ui, sans-serif;
  --namd-font-mono: ui-monospace, 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
  --namd-font-size-base: 0.875rem;
  --namd-font-weight-normal: 400;
  --namd-font-weight-medium: 500;

  /* Spacing */
  --namd-spacing-xs: 0.25rem;
  --namd-spacing-sm: 0.5rem;
  --namd-spacing-md: 1rem;
  --namd-spacing-lg: 1.5rem;
  --namd-spacing-xl: 2rem;
}

/* Dark theme */
@media (prefers-color-scheme: dark) {
  :root {
    --namd-bg-primary: #0f172a;
    --namd-bg-secondary: #1e293b;
    --namd-text-primary: #f1f5f9;
    --namd-text-secondary: #cbd5e1;
    --namd-text-muted: #64748b;

    --namd-primary: #f1f5f9;
    --namd-primary-fg: #1e293b;
    --namd-secondary: #334155;
    --namd-secondary-fg: #f1f5f9;

    --namd-border: rgba(255, 255, 255, 0.1);
  }
}

/* Optional explicit dark theme class */
.dark {
  /* Same dark theme properties */
}
```

### Component-Level CSS Patterns

#### Button Components
```css
.namd-button {
  font-family: var(--namd-font-sans);
  font-size: var(--namd-font-size-base);
  font-weight: var(--namd-font-weight-medium);
  border-radius: var(--namd-border-radius);
  padding: var(--namd-spacing-sm) var(--namd-spacing-md);
  transition: all 0.15s ease;
}

.namd-button--primary {
  background-color: var(--namd-primary);
  color: var(--namd-primary-fg);
}

.namd-button--secondary {
  background-color: var(--namd-secondary);
  color: var(--namd-secondary-fg);
  border: 1px solid var(--namd-border);
}
```

#### Status Badge Pattern
```css
.namd-status-badge {
  display: inline-flex;
  align-items: center;
  padding: var(--namd-spacing-xs) var(--namd-spacing-sm);
  border-radius: calc(var(--namd-border-radius) * 0.5);
  font-size: 0.75rem;
  font-weight: var(--namd-font-weight-medium);
  text-transform: uppercase;
  letter-spacing: 0.025em;
}

.namd-status-badge--running {
  background-color: var(--namd-info-bg);
  color: var(--namd-info);
}

.namd-status-badge--completed {
  background-color: var(--namd-success-bg);
  color: var(--namd-success);
}

.namd-status-badge--failed {
  background-color: var(--namd-error-bg);
  color: var(--namd-error);
}
```

#### Layout Patterns
```css
.namd-sidebar {
  width: var(--namd-sidebar-width);
  background-color: var(--namd-bg-secondary);
  border-right: 1px solid var(--namd-border);
}

.namd-main-content {
  flex: 1;
  background-color: var(--namd-bg-primary);
  color: var(--namd-text-primary);
}

.namd-card {
  background-color: var(--namd-bg-primary);
  border: 1px solid var(--namd-border);
  border-radius: var(--namd-border-radius);
  padding: var(--namd-spacing-lg);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}
```

## Svelte Implementation Strategy

### 1. CSS Custom Properties in `src/app.css`
Create the root CSS custom properties following our naming convention (`--namd-*`).

### 2. Component-Scoped Styles
Each Svelte component uses scoped CSS that references the custom properties:

```svelte
<!-- StatusBadge.svelte -->
<script>
  export let status;
</script>

<span class="status-badge status-badge--{status.toLowerCase()}">
  {status}
</span>

<style>
  .status-badge {
    /* Base styles using custom properties */
  }

  .status-badge--running {
    background-color: var(--namd-info-bg);
    color: var(--namd-info);
  }
</style>
```

### 3. Theme Switching Support
Use Svelte stores for theme management:

```typescript
// stores/theme.ts
import { writable } from 'svelte/store';

export const theme = writable<'light' | 'dark'>('light');

export function toggleTheme() {
  theme.update(current => current === 'light' ? 'dark' : 'light');
}
```

### 4. Utility Classes
Create utility classes for common patterns:

```css
/* Utilities */
.namd-text-mono { font-family: var(--namd-font-mono); }
.namd-text-muted { color: var(--namd-text-muted); }
.namd-bg-muted { background-color: var(--namd-bg-secondary); }
.namd-border-b { border-bottom: 1px solid var(--namd-border); }

/* Layout utilities */
.namd-flex { display: flex; }
.namd-flex-col { flex-direction: column; }
.namd-items-center { align-items: center; }
.namd-justify-between { justify-content: space-between; }
.namd-gap-sm { gap: var(--namd-spacing-sm); }
.namd-gap-md { gap: var(--namd-spacing-md); }
```

This design system maintains consistency with the mockup's visual design while being optimized for Svelte's component model and CSS scoping capabilities.
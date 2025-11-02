# NAMD Configuration Templates - Design Document

> **Status**: Design Phase - Not Yet Implemented  
> **Target Phase**: Phase 8+  
> **Last Updated**: 2025-11-01  
> **Related**: See [MultiJob_And_Templates.md](MultiJob_And_Templates.md) for job chaining integration

## Table of Contents

### Part 1: Core Template System
1. [Executive Summary](#1-executive-summary)
2. [The Template Problem](#2-the-template-problem)
3. [Template-Driven Dynamic Forms](#3-template-driven-dynamic-forms)
4. [Field Types and Validation](#4-field-types-and-validation)
5. [Template Definitions](#5-template-definitions)

### Part 2: Implementation
6. [Data Model](#6-data-model)
7. [Backend Architecture](#7-backend-architecture)
8. [Frontend Integration](#8-frontend-integration)
9. [Template Storage](#9-template-storage)

### Part 3: NAMD-Specific Design
10. [NAMD Config Generation](#10-namd-config-generation)
11. [Tutorial-Derived Templates](#11-tutorial-derived-templates)
12. [Validation Rules](#12-validation-rules)

### Part 4: Advanced Features
13. [User-Defined Templates](#13-user-defined-templates)
14. [Template Versioning](#14-template-versioning)
15. [Template Library](#15-template-library)

### Part 5: Reference
16. [Integration Points](#16-integration-points)
17. [Success Criteria](#17-success-criteria)
18. [Open Questions](#18-open-questions)
19. [Appendices](#19-appendices)

---

## 1. Executive Summary

### What This Document Covers

This document defines the **NAMD Configuration Templates System** - a dynamic form generation system that enables users to create NAMD simulations with validated, simulation-type-specific parameter sets.

### The Core Problem

Different NAMD simulation types (vacuum optimization, explicit solvent NPT, implicit solvent, etc.) require completely different parameter combinations. Providing all parameters as a flat form leads to:
- Invalid configurations (e.g., PME=yes with vacuum box dimensions)
- Confusion for new users who don't know which parameters go together
- No guidance on appropriate values for different simulation types
- Inability to validate interdependent parameters

### The Solution

**Template-Driven Dynamic Forms**: Backend defines the UI structure and validation rules, frontend renders forms dynamically based on template definitions.

**Key Benefits**:
- Type-safe parameter combinations
- Context-specific help text and validation
- Conditional field visibility
- Backend-controlled UI without frontend code changes
- Support for multiple simulation types without UI updates

### Relationship to Job Chaining

Templates work standalone for single jobs but also enable multi-stage workflows when combined with job chaining (see [MultiJob_And_Templates.md](MultiJob_And_Templates.md)). This document focuses solely on the template system itself.

### Prerequisites

**Must complete first**:
- Phase 6.7: Template Type 2 NAMD config fixes (cellBasisVector, extrabonds, configurable PME/NPT) ✅ COMPLETE
- Review existing `cluster_config.rs` patterns for dynamic configuration
- Understand current NAMD config generation in `slurm/script_generator.rs`

---

## 2. The Template Problem

### 2.1 Observation from NAMD Tutorial

**Tutorial Source**: `examples/origamiTutorial/origamiprotocols_0_markdown.md`

Different simulation types require **completely different parameter combinations**:

#### Vacuum Optimization (Step 2)
```tcl
PME                 no
cellBasisVector1    1000 0 0     # Large box
langevinDamping     0.1          # Low damping
fullElectFrequency  3
margin              30
```

#### Explicit Solvent NPT (Step 3)
```tcl
PME                 yes
PMEGridSpacing      1.5
cellBasisVector1    124 0.0 0.0  # Actual system size
langevinDamping     5            # High damping
langevinPiston      on           # Pressure control
fullElectFrequency  2
```

**Key Insight**: These aren't just different values - they're fundamentally different **sets of relevant parameters**.

### 2.2 Why Current Hardcoded Forms Don't Scale

**Current Approach** (Phase 6.7):
```svelte
<!-- ConfigurationTab.svelte - hardcoded fields -->
<label>Temperature (K)</label>
<input type="number" bind:value={namdConfig.temperature} />

<label>Timestep (fs)</label>
<input type="number" bind:value={namdConfig.timestep} />

<label>PME Enabled</label>
<input type="checkbox" bind:checked={namdConfig.pme_enabled} />

<!-- ... 20+ more hardcoded fields ... -->
```

**Problems**:
1. Adding new parameters requires editing Svelte component
2. Different templates need different field subsets
3. Can't conditionally hide/show fields (e.g., margin only when PME=no)
4. Can't provide template-specific help text
5. Can't validate interdependent parameters (e.g., PME requires cellBasisVector)
6. Every new simulation type = edit frontend code

**Example Pain Point**: User enables PME but provides 1000Å box dimensions (appropriate for vacuum, invalid for periodic boundaries). Current system can't prevent this.

### 2.3 The Scale Problem

**Simulation types users may want**:
- Vacuum structural optimization
- Explicit solvent equilibration (NPT)
- Explicit solvent production (NVT)  
- Implicit solvent simulations
- Constant volume (NVE) simulations
- Steered molecular dynamics
- Umbrella sampling
- Replica exchange
- Different force fields (CHARMM, AMBER, OPLS)

Each type needs:
- Different parameter subsets
- Different default values
- Different validation rules
- Different help text
- Different field groupings

**Hardcoding all this in frontend = unmaintainable.**

---

## 3. Template-Driven Dynamic Forms

### 3.1 Core Concept

**Backend defines UI structure, frontend renders dynamically.**

### 3.2 Architecture Overview

```
Backend (Rust)                     Frontend (Svelte)
─────────────────                  ──────────────────
Template Definitions    ─────►     Dynamic Form Renderer
  ├─ Field Types                     ├─ DynamicField Components
  ├─ Validation Rules                ├─ Conditional Visibility
  ├─ Default Values                  ├─ Real-time Validation
  └─ Help Text                       └─ Error Display

Template Registry       ─────►     Template Selector
  ├─ Vacuum Opt                      (Dropdown: "Explicit Solvent NPT")
  ├─ Explicit Solvent NPT
  └─ Implicit Solvent

Config Generator        ◄─────     Form Values (JSON)
  └─ template_values → .namd file
```

### 3.3 Data Flow

1. **User selects template** from dropdown
2. **Backend sends template definition** (field types, defaults, validation)
3. **Frontend renders form dynamically** based on definition
4. **User fills form**, frontend validates in real-time
5. **User submits**, backend validates and generates NAMD config
6. **Job created** with template_id + template_values stored

### 3.4 Example: Template Definition (Rust)

```rust
pub struct NAMDTemplate {
    pub id: String,                           // "explicit_solvent_npt_v1"
    pub name: String,                         // "Explicit Solvent NPT"
    pub description: String,                  // "Constant pressure/temperature..."
    pub use_case: String,                     // "Equilibration in aqueous solution"
    pub form_sections: Vec<FormSection>,      // Grouped UI fields
    pub namd_config_template: String,         // Template string for .namd generation
}

pub struct FormSection {
    pub title: String,                        // "Periodic Boundaries"
    pub description: Option<String>,          // Help text for section
    pub fields: Vec<FieldDefinition>,         // Fields in this section
}

pub struct FieldDefinition {
    pub key: String,                          // "use_pme" (maps to template variable)
    pub label: String,                        // "Use PME Electrostatics"
    pub field_type: FieldType,                // Boolean | Number | Text | etc.
    pub default_value: Value,                 // json!(true)
    pub required: bool,                       // true
    pub help_text: Option<String>,            // "PME computes long-range..."
    pub visible_when: Option<VisibilityCondition>,  // Show only if PME=true
    pub validation: Option<FieldValidation>,  // Min/max, patterns, etc.
}
```

### 3.5 Example: Frontend Rendering (Svelte)

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import DynamicField from './DynamicField.svelte';
  
  export let templateId: string;
  
  let template: NAMDTemplate;
  let formValues: Record<string, any> = {};
  
  onMount(async () => {
    // Fetch template definition from backend
    template = await invoke('get_template', { templateId });
    
    // Initialize form with defaults
    template.form_sections.forEach(section => {
      section.fields.forEach(field => {
        formValues[field.key] = field.default_value;
      });
    });
  });
  
  function shouldShowField(field: FieldDefinition): boolean {
    if (!field.visible_when) return true;
    
    const { field: dependentField, operator, value } = field.visible_when;
    const actualValue = formValues[dependentField];
    
    return evaluateCondition(operator, actualValue, value);
  }
</script>

<form>
  {#each template.form_sections as section}
    <fieldset>
      <legend>{section.title}</legend>
      {#if section.description}
        <p class="section-help">{section.description}</p>
      {/if}
      
      {#each section.fields as field}
        {#if shouldShowField(field)}
          <DynamicField 
            {field} 
            bind:value={formValues[field.key]}
            on:change={() => validateForm()}
          />
        {/if}
      {/each}
    </fieldset>
  {/each}
  
  <button on:click={submitForm}>Create Job</button>
</form>
```

### 3.6 Key Advantages

1. **Backend Controls UI**: Add new parameters without touching frontend
2. **Type Safety**: Field types enforce correct input widgets
3. **Conditional Logic**: Show/hide fields based on other values
4. **Validation**: Both frontend (UX) and backend (security) validation
5. **Reusability**: Same rendering logic for all templates
6. **Maintainability**: Template changes = backend data changes only

---

## 4. Field Types and Validation

### 4.1 Field Type System

```rust
pub enum FieldType {
    Boolean,
    
    Number {
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
        unit: Option<String>,  // "K", "fs", "Å", "kcal/mol/Ų"
    },
    
    Text {
        pattern: Option<String>,        // Regex for validation
        max_length: Option<usize>,
        placeholder: Option<String>,
    },
    
    Select {
        options: Vec<SelectOption>,     // Dropdown choices
        allow_custom: bool,             // Allow user to type custom value
    },
    
    Dimensions {                        // Special: X, Y, Z coordinate inputs
        min: Option<f64>,
        max: Option<f64>,
        unit: String,                   // "Å"
    },
    
    FileUpload {
        accepted_extensions: Vec<String>,  // [".exb", ".extra"]
        file_type: NAMDFileType,           // For backend processing
        optional: bool,
    },
}

pub struct SelectOption {
    pub value: String,               // "minimize"
    pub label: String,               // "Energy Minimization"
    pub description: Option<String>, // "Find nearest energy minimum..."
}
```

### 4.2 Field Type Rendering

**Frontend Component** (`DynamicField.svelte`):

```svelte
<script lang="ts">
  export let field: FieldDefinition;
  export let value: any;
  
  $: errorMessage = validateField(field, value);
</script>

<div class="field" class:has-error={errorMessage}>
  <label for={field.key}>
    {field.label}
    {#if field.required}<span class="required">*</span>{/if}
  </label>
  
  {#if field.help_text}
    <p class="help-text">{field.help_text}</p>
  {/if}
  
  {#if field.field_type === 'Boolean'}
    <input type="checkbox" id={field.key} bind:checked={value} />
    
  {:else if field.field_type.Number}
    <input 
      type="number" 
      id={field.key}
      bind:value
      min={field.field_type.min}
      max={field.field_type.max}
      step={field.field_type.step}
    />
    {#if field.field_type.unit}
      <span class="unit">{field.field_type.unit}</span>
    {/if}
    
  {:else if field.field_type.Dimensions}
    <div class="dimensions">
      <input type="number" bind:value={value.x} placeholder="X" />
      <input type="number" bind:value={value.y} placeholder="Y" />
      <input type="number" bind:value={value.z} placeholder="Z" />
      <span class="unit">{field.field_type.unit}</span>
    </div>
    
  {:else if field.field_type.Select}
    <select id={field.key} bind:value>
      {#each field.field_type.options as option}
        <option value={option.value} title={option.description}>
          {option.label}
        </option>
      {/each}
    </select>
  {/if}
  
  {#if errorMessage}
    <p class="error">{errorMessage}</p>
  {/if}
</div>
```

### 4.3 Conditional Visibility System

```rust
pub struct VisibilityCondition {
    pub field: String,              // "use_pme"
    pub operator: ConditionOperator,
    pub value: Value,               // json!(true)
}

pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,       // For text fields
    IsEmpty,
    IsNotEmpty,
}
```

**Example Use Cases**:

1. **Show `PMEGridSpacing` only when PME enabled**:
```rust
FieldDefinition {
    key: "pme_grid_spacing",
    label: "PME Grid Spacing",
    field_type: FieldType::Number { 
        min: Some(0.5), 
        max: Some(3.0), 
        step: Some(0.1),
        unit: Some("Å".to_string()),
    },
    visible_when: Some(VisibilityCondition {
        field: "use_pme".to_string(),
        operator: ConditionOperator::Equals,
        value: json!(true),
    }),
}
```

2. **Show `margin` only when PME disabled** (vacuum simulations):
```rust
FieldDefinition {
    key: "margin",
    label: "Pairlist Margin",
    field_type: FieldType::Number {
        min: Some(0.0),
        max: Some(50.0),
        step: Some(5.0),
        unit: None,
    },
    visible_when: Some(VisibilityCondition {
        field: "use_pme".to_string(),
        operator: ConditionOperator::Equals,
        value: json!(false),
    }),
}
```

### 4.4 Validation Rules System

```rust
pub struct FieldValidation {
    pub rules: Vec<ValidationRule>,
    pub custom_validator: Option<String>,  // Function name for complex validation
}

pub enum ValidationRule {
    Range { 
        min: f64, 
        max: f64, 
        message: String 
    },
    
    MinValue { 
        min: f64, 
        message: String 
    },
    
    MaxValue { 
        max: f64, 
        message: String 
    },
    
    Pattern { 
        regex: String, 
        message: String 
    },
    
    RequiredIf { 
        field: String, 
        equals: Value, 
        message: String 
    },
    
    OneOf { 
        values: Vec<Value>, 
        message: String 
    },
    
    DependsOn {
        field: String,
        message: String,
    },
}
```

**Validation Examples**:

#### Example 1: Temperature Range
```rust
FieldDefinition {
    key: "temperature",
    label: "Temperature",
    field_type: FieldType::Number {
        min: None,
        max: None,
        step: Some(10.0),
        unit: Some("K".to_string()),
    },
    validation: Some(FieldValidation {
        rules: vec![
            ValidationRule::Range {
                min: 200.0,
                max: 400.0,
                message: "Temperature must be in biological range (200-400 K)".to_string(),
            },
        ],
        custom_validator: None,
    }),
}
```

#### Example 2: PME Requires Cell Basis Vectors
```rust
FieldDefinition {
    key: "cell_basis_vector_1",
    label: "Cell Basis Vector 1",
    field_type: FieldType::Dimensions {
        min: Some(10.0),
        max: Some(1000.0),
        unit: "Å".to_string(),
    },
    validation: Some(FieldValidation {
        rules: vec![
            ValidationRule::RequiredIf {
                field: "use_pme".to_string(),
                equals: json!(true),
                message: "Cell basis vectors required when PME is enabled".to_string(),
            },
        ],
        custom_validator: None,
    }),
}
```

#### Example 3: Timestep Must Match Execution Mode
```rust
// Custom validator function
pub fn validate_timestep_for_mode(
    timestep: f64,
    execution_mode: &str,
) -> Result<(), String> {
    match execution_mode {
        "minimize" => {
            // Minimization doesn't use timestep
            Ok(())
        }
        "run" => {
            if timestep < 0.5 || timestep > 4.0 {
                Err("Timestep for MD runs should be 0.5-4.0 fs".to_string())
            } else {
                Ok(())
            }
        }
        _ => Err("Unknown execution mode".to_string()),
    }
}
```

### 4.5 Frontend Validation Execution

```typescript
function validateField(field: FieldDefinition, value: any): string | null {
  if (!field.validation) return null;
  
  for (const rule of field.validation.rules) {
    let isValid = true;
    
    switch (rule.type) {
      case 'Range':
        isValid = value >= rule.min && value <= rule.max;
        break;
        
      case 'RequiredIf':
        const dependentValue = formValues[rule.field];
        if (dependentValue === rule.equals && !value) {
          isValid = false;
        }
        break;
        
      case 'Pattern':
        const regex = new RegExp(rule.regex);
        isValid = regex.test(value);
        break;
        
      // ... other rules
    }
    
    if (!isValid) {
      return rule.message;
    }
  }
  
  return null;  // All rules passed
}
```

### 4.6 Backend Validation (Authoritative)

```rust
pub fn validate_template_values(
    template_id: &str,
    field_values: &HashMap<String, Value>,
) -> ValidationResult {
    let template = get_template(template_id)?;
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    
    for field in template.get_all_fields() {
        let value = &field_values[&field.key];
        
        if let Some(validation) = &field.validation {
            for rule in &validation.rules {
                if !check_rule(rule, value, field_values) {
                    issues.push(ValidationIssue {
                        field: field.key.clone(),
                        message: rule.message.clone(),
                        severity: Severity::Error,
                    });
                }
            }
            
            // Custom validation functions
            if let Some(validator_name) = &validation.custom_validator {
                if let Err(e) = call_custom_validator(validator_name, value, field_values) {
                    issues.push(ValidationIssue {
                        field: field.key.clone(),
                        message: e,
                        severity: Severity::Error,
                    });
                }
            }
        }
    }
    
    ValidationResult {
        is_valid: issues.iter().all(|i| i.severity != Severity::Error),
        issues,
        warnings,
        suggestions: generate_suggestions(template, field_values),
    }
}
```

### 4.7 Custom Validators

**Problem**: Some validations are too complex for simple rules.

**Example**: Validate that cellBasisVector dimensions match uploaded PDB file

**Proposed Solution**:
```rust
pub enum ValidationRule {
    // ... existing rules ...

    CustomValidator {
        function_name: String,
        message: String,
    },
}

// Registry of custom validator functions
type CustomValidatorFn = fn(&serde_json::Value, &HashMap<String, serde_json::Value>) -> Option<String>;

lazy_static! {
    static ref CUSTOM_VALIDATORS: HashMap<String, CustomValidatorFn> = {
        let mut validators = HashMap::new();
        validators.insert("validate_box_vs_pdb".to_string(), validate_box_vs_pdb as CustomValidatorFn);
        validators.insert("validate_pme_grid_dimensions".to_string(), validate_pme_grid_dimensions as CustomValidatorFn);
        validators
    };
}

fn validate_box_vs_pdb(
    value: &serde_json::Value,
    all_values: &HashMap<String, serde_json::Value>
) -> Option<String> {
    // Load PDB, check CRYST1 record
    // Compare with cellBasisVector values
    // Return error message if mismatch
    None  // or Some("error message")
}
```

**Usage in Template**:
```rust
FieldDefinition {
    key: "cell_basis_vector_x",
    validation: Some(FieldValidation {
        rules: vec![
            ValidationRule::CustomValidator {
                function_name: "validate_box_vs_pdb".to_string(),
                message: "Box dimensions don't match PDB file".to_string(),
            },
        ],
    }),
}
```

**Open Question**: Is this overengineering? Start with simple rules only?

---

## 5. Template Definitions

### 5.1 Template Storage Format

Templates are stored as JSON files in `src-tauri/templates/`:

```
src-tauri/templates/
├── explicit_solvent_npt_v1.json
├── vacuum_optimization_v1.json
└── implicit_solvent_v1.json
```

**Example Template JSON**:
```json
{
  "id": "explicit_solvent_npt_v1",
  "name": "Explicit Solvent NPT",
  "description": "Constant pressure/temperature simulation in explicit solvent",
  "use_case": "Equilibration and production runs in aqueous solution with full electrostatics",
  "form_sections": [
    {
      "title": "Basic Parameters",
      "fields": [
        {
          "key": "temperature",
          "label": "Temperature",
          "field_type": {
            "Number": {
              "min": 200.0,
              "max": 400.0,
              "step": 10.0,
              "unit": "K"
            }
          },
          "default_value": 300.0,
          "required": true,
          "help_text": "Simulation temperature (typical: 298-310 K for biological systems)"
        }
      ]
    }
  ],
  "namd_config_template": "...template string with {{variable}} placeholders..."
}
```

### 5.2 Template Registry

```rust
// src-tauri/src/templates/registry.rs

pub struct TemplateRegistry {
    templates: HashMap<String, NAMDTemplate>,
}

impl TemplateRegistry {
    pub fn new() -> Result<Self> {
        let mut templates = HashMap::new();
        
        // Load built-in templates from JSON files
        for entry in fs::read_dir("templates")? {
            let path = entry?.path();
            if path.extension() == Some("json") {
                let template: NAMDTemplate = serde_json::from_reader(
                    File::open(path)?
                )?;
                templates.insert(template.id.clone(), template);
            }
        }
        
        Ok(TemplateRegistry { templates })
    }
    
    pub fn get(&self, id: &str) -> Option<&NAMDTemplate> {
        self.templates.get(id)
    }
    
    pub fn list_all(&self) -> Vec<TemplateSummary> {
        self.templates.values()
            .map(|t| TemplateSummary {
                id: t.id.clone(),
                name: t.name.clone(),
                description: t.description.clone(),
                use_case: t.use_case.clone(),
            })
            .collect()
    }
}
```

### 5.3 Template Lifecycle

```
1. App Startup
   └─> Load templates from disk
       └─> Validate JSON schema
           └─> Build template registry

2. User Opens Create Job
   └─> Fetch template list
       └─> Display template selector dropdown

3. User Selects Template
   └─> Fetch full template definition
       └─> Render dynamic form
           └─> Initialize with defaults

4. User Fills Form
   └─> Real-time frontend validation
       └─> Show/hide conditional fields

5. User Submits
   └─> Backend validates template values
       └─> Generate NAMD config from template
           └─> Create job with template_id + template_values
```

---

## 6. Data Model

### 6.1 Template-Related Database Schema

**Add to JobInfo** (`src-tauri/src/types/core.rs`):
```rust
pub struct JobInfo {
    // ... existing fields ...
    
    // NEW: Template tracking
    pub template_id: String,                    // "explicit_solvent_npt_v1"
    pub template_values: HashMap<String, Value>, // User's filled form values
}
```

**Database Schema** (SQLite):
```sql
ALTER TABLE jobs ADD COLUMN template_id TEXT NOT NULL DEFAULT 'legacy';
ALTER TABLE jobs ADD COLUMN template_values TEXT;  -- JSON serialized HashMap
```

**Migration Strategy**:
- Existing jobs get `template_id = "legacy"`
- Legacy jobs use old hardcoded `NAMDConfig` struct
- New jobs use template system
- Both systems coexist during transition

### 6.2 Template Value Storage

**JSON Format** (stored in `jobs.template_values`):
```json
{
  "temperature": 310.0,
  "timestep": 2.0,
  "use_pme": true,
  "pme_grid_spacing": 1.5,
  "cell_basis_vector_1": {"x": 124.0, "y": 0.0, "z": 0.0},
  "cell_basis_vector_2": {"x": 0.0, "y": 114.0, "z": 0.0},
  "cell_basis_vector_3": {"x": 0.0, "y": 0.0, "z": 323.0},
  "langevin_damping": 5.0,
  "execution_mode": "run",
  "steps": 2400000
}
```

**Advantages**:
- Schema-less: Add new template fields without migration
- Template evolution: Templates can change without breaking old jobs
- Audit trail: Know exactly what user specified
- Reproducibility: Can recreate exact NAMD config

---

## 7. Backend Architecture

### 7.1 Module Structure

```
src-tauri/src/templates/
├── mod.rs              # Module exports
├── registry.rs         # Template loading and management
├── types.rs            # NAMDTemplate, FieldDefinition, etc.
├── validation.rs       # Template value validation
├── generator.rs        # NAMD config generation from template
└── builtins/          # Built-in template definitions
    ├── explicit_solvent_npt.rs
    └── vacuum_optimization.rs
```

### 7.2 IPC Commands

```rust
// src-tauri/src/commands/templates.rs

#[tauri::command]
pub async fn list_templates() -> Result<Vec<TemplateSummary>> {
    Ok(TEMPLATE_REGISTRY.list_all())
}

#[tauri::command]
pub async fn get_template(template_id: String) -> Result<NAMDTemplate> {
    TEMPLATE_REGISTRY.get(&template_id)
        .ok_or_else(|| anyhow!("Template not found: {}", template_id))
}

#[tauri::command]
pub async fn validate_template_values(
    template_id: String,
    values: HashMap<String, Value>,
) -> Result<ValidationResult> {
    templates::validation::validate_template_values(&template_id, &values)
}

#[tauri::command]
pub async fn generate_namd_config(
    template_id: String,
    values: HashMap<String, Value>,
) -> Result<String> {
    let template = TEMPLATE_REGISTRY.get(&template_id)?;
    templates::generator::generate_config(template, &values)
}
```

### 7.3 Config Generation

```rust
// src-tauri/src/templates/generator.rs

pub fn generate_config(
    template: &NAMDTemplate,
    values: &HashMap<String, Value>,
) -> Result<String> {
    let mut config = template.namd_config_template.clone();
    
    // Replace all {{variable}} placeholders
    for (key, value) in values {
        let placeholder = format!("{{{{{}}}}}", key);
        let replacement = format_value_for_namd(value)?;
        config = config.replace(&placeholder, &replacement);
    }
    
    // Validate no unreplaced placeholders remain
    if config.contains("{{") {
        return Err(anyhow!("Template has unreplaced placeholders"));
    }
    
    Ok(config)
}

fn format_value_for_namd(value: &Value) -> Result<String> {
    match value {
        Value::Bool(b) => Ok(if *b { "yes" } else { "no" }),
        Value::Number(n) => Ok(n.to_string()),
        Value::String(s) => Ok(s.clone()),
        Value::Object(obj) if obj.contains_key("x") => {
            // Dimensions: {x: 124, y: 0, z: 0} → "124 0 0"
            Ok(format!("{} {} {}", 
                obj["x"].as_f64().unwrap(),
                obj["y"].as_f64().unwrap(),
                obj["z"].as_f64().unwrap()
            ))
        }
        _ => Err(anyhow!("Unsupported value type for NAMD config")),
    }
}
```

---

## 8. Frontend Integration

### 8.1 Template Selector Component

```svelte
<!-- src/lib/components/create-job/TemplateSelector.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { createEventDispatcher, onMount } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  let templates: TemplateSummary[] = [];
  let selectedTemplateId: string | null = null;
  
  onMount(async () => {
    templates = await invoke('list_templates');
    
    // Select first template by default
    if (templates.length > 0) {
      selectedTemplateId = templates[0].id;
      loadTemplate(selectedTemplateId);
    }
  });
  
  async function loadTemplate(templateId: string) {
    const template = await invoke('get_template', { templateId });
    dispatch('templateSelected', template);
  }
</script>

<div class="template-selector">
  <label>Simulation Type</label>
  <select bind:value={selectedTemplateId} on:change={() => loadTemplate(selectedTemplateId)}>
    {#each templates as template}
      <option value={template.id}>
        {template.name}
      </option>
    {/each}
  </select>
  
  {#if selectedTemplateId}
    {@const template = templates.find(t => t.id === selectedTemplateId)}
    <div class="template-info">
      <h4>{template.use_case}</h4>
      <p>{template.description}</p>
    </div>
  {/if}
</div>
```

### 8.2 Dynamic Form Component

```svelte
<!-- src/lib/components/create-job/DynamicConfigForm.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import DynamicField from './DynamicField.svelte';
  
  export let template: NAMDTemplate;
  export let onSubmit: (values: Record<string, any>) => void;
  
  let formValues: Record<string, any> = {};
  let validationErrors: Record<string, string> = {};
  
  // Initialize form with default values
  $: if (template) {
    formValues = {};
    template.form_sections.forEach(section => {
      section.fields.forEach(field => {
        formValues[field.key] = field.default_value;
      });
    });
  }
  
  function shouldShowField(field: FieldDefinition): boolean {
    if (!field.visible_when) return true;
    
    const { field: depField, operator, value } = field.visible_when;
    const actualValue = formValues[depField];
    
    switch (operator) {
      case 'Equals': return actualValue === value;
      case 'NotEquals': return actualValue !== value;
      // ... other operators
      default: return true;
    }
  }
  
  async function handleSubmit() {
    // Backend validation
    const result = await invoke('validate_template_values', {
      templateId: template.id,
      values: formValues,
    });
    
    if (result.is_valid) {
      onSubmit(formValues);
    } else {
      // Show errors
      validationErrors = {};
      result.issues.forEach(issue => {
        validationErrors[issue.field] = issue.message;
      });
    }
  }
</script>

<form on:submit|preventDefault={handleSubmit}>
  {#each template.form_sections as section}
    <fieldset>
      <legend>{section.title}</legend>
      
      {#each section.fields as field}
        {#if shouldShowField(field)}
          <DynamicField
            {field}
            bind:value={formValues[field.key]}
            error={validationErrors[field.key]}
          />
        {/if}
      {/each}
    </fieldset>
  {/each}
  
  <button type="submit">Create Job</button>
</form>
```

### 8.3 Complete Component Architecture

**Template Store Pattern** (follows ClusterConfig pattern):

```typescript
// src/lib/stores/templateConfig.ts
import { writable, derived } from 'svelte/store';
import type { TemplateCapabilities, NAMDTemplate } from '../types/template';

const templateConfigStore = writable<TemplateCapabilities | null>(null);

export async function initTemplateConfig(): Promise<void> {
  const result = await CoreClientFactory.getClient().getTemplateCapabilities();
  if (result.success && result.data) {
    templateConfigStore.set(result.data);
  }
}

export const availableTemplates = derived(
  templateConfigStore,
  $config => $config?.templates ?? []
);

export const getTemplate = derived(
  templateConfigStore,
  $config => (templateId: string) =>
    $config?.templates.find(t => t.id === templateId)
);
```

**DynamicField Component** (field type rendering):

```svelte
<!-- src/lib/components/create-job/DynamicField.svelte -->
<script lang="ts">
  import type { FieldDefinition } from '../../types/template';

  export let definition: FieldDefinition;
  export let value: any;

  $: fieldType = definition.field_type;
</script>

<div class="field-group">
  <label for={definition.key}>
    {definition.label}
    {#if definition.required}*{/if}
  </label>

  {#if fieldType.type === 'Boolean'}
    <input
      type="checkbox"
      id={definition.key}
      bind:checked={value}
    />
  {:else if fieldType.type === 'Number'}
    <input
      type="number"
      id={definition.key}
      bind:value
      min={fieldType.min}
      max={fieldType.max}
      step={fieldType.step}
    />
    {#if fieldType.unit}
      <span class="unit">{fieldType.unit}</span>
    {/if}
  {:else if fieldType.type === 'Select'}
    <select id={definition.key} bind:value>
      {#each fieldType.options as option}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
  {:else if fieldType.type === 'Dimensions'}
    <div class="dimensions-grid">
      <input type="number" bind:value={value.x} placeholder="X (Å)" />
      <input type="number" bind:value={value.y} placeholder="Y (Å)" />
      <input type="number" bind:value={value.z} placeholder="Z (Å)" />
    </div>
  {/if}

  {#if definition.help_text}
    <span class="help-text">{definition.help_text}</span>
  {/if}
</div>
```

**Integration into CreateJobPage**:

```svelte
<!-- Modify src/lib/components/pages/CreateJobPage.svelte -->
<script>
  import DynamicConfigForm from '../create-job/DynamicConfigForm.svelte';

  let selectedTemplateId = 'explicit_solvent_npt_v1';
  let namdFieldValues = {};

  // When form submitted, merge template values with CreateJobParams
  function buildCreateJobParams() {
    return {
      job_name,
      template_id: selectedTemplateId,
      namd_config: buildNAMDConfigFromFields(namdFieldValues),
      slurm_config,
      input_files,
    };
  }
</script>

<!-- Configuration Tab -->
<DynamicConfigForm
  bind:selectedTemplateId
  bind:fieldValues={namdFieldValues}
/>
```

**Component Reuse**:
- Fresh job creation
- Job continuation (pre-filled with parent values)
- User template creation (save current form as template)

---

## 9. Template Storage

### 9.1 JSON Template Files

Templates stored in `src-tauri/templates/` as JSON:

```json
{
  "id": "explicit_solvent_npt_v1",
  "name": "Explicit Solvent NPT",
  "description": "Constant pressure/temperature equilibration in explicit solvent",
  "use_case": "Multi-stage equilibration with PME electrostatics and pressure control",
  
  "form_sections": [
    {
      "title": "Execution Parameters",
      "fields": [
        {
          "key": "execution_mode",
          "label": "Execution Mode",
          "field_type": {
            "Select": {
              "options": [
                {"value": "minimize", "label": "Energy Minimization"},
                {"value": "run", "label": "Molecular Dynamics"}
              ],
              "allow_custom": false
            }
          },
          "default_value": "run",
          "required": true,
          "help_text": "minimize = find energy minimum, run = time evolution"
        },
        {
          "key": "steps",
          "label": "Simulation Steps",
          "field_type": {
            "Number": {"min": 1000, "max": 10000000, "step": 1000, "unit": null}
          },
          "default_value": 2400000,
          "required": true,
          "visible_when": {"field": "execution_mode", "operator": "Equals", "value": "run"}
        }
      ]
    },
    {
      "title": "Periodic Boundaries",
      "fields": [
        {
          "key": "use_pme",
          "label": "Use PME Electrostatics",
          "field_type": "Boolean",
          "default_value": true,
          "required": true,
          "help_text": "PME computes long-range electrostatics with periodic boundaries"
        },
        {
          "key": "pme_grid_spacing",
          "label": "PME Grid Spacing",
          "field_type": {
            "Number": {"min": 0.5, "max": 3.0, "step": 0.1, "unit": "Å"}
          },
          "default_value": 1.5,
          "required": true,
          "visible_when": {"field": "use_pme", "operator": "Equals", "value": true}
        },
        {
          "key": "cell_basis_vector_1",
          "label": "Cell Basis Vector 1",
          "field_type": {
            "Dimensions": {"min": 10.0, "max": 1000.0, "unit": "Å"}
          },
          "default_value": {"x": 124.0, "y": 0.0, "z": 0.0},
          "required": true,
          "validation": {
            "rules": [
              {
                "RequiredIf": {
                  "field": "use_pme",
                  "equals": true,
                  "message": "Cell basis vectors required when PME enabled"
                }
              }
            ]
          }
        }
      ]
    }
  ],
  
  "namd_config_template": "# NAMD Configuration - {{template_name}}\n\n# Execution\n{{#if execution_mode == 'minimize'}}minimize {{steps}}{{else}}run {{steps}}{{/if}}\n\n# PME\nPME {{use_pme}}\n{{#if use_pme}}PMEGridSpacing {{pme_grid_spacing}}{{/if}}\n\n# Cell Dimensions\ncellBasisVector1 {{cell_basis_vector_1}}\n..."
}
```

### 9.2 Loading Strategy

```rust
lazy_static! {
    static ref TEMPLATE_REGISTRY: TemplateRegistry = {
        TemplateRegistry::new().expect("Failed to load templates")
    };
}

impl TemplateRegistry {
    pub fn new() -> Result<Self> {
        let mut templates = HashMap::new();
        
        // Load from JSON files in templates/
        let template_dir = PathBuf::from("templates");
        for entry in fs::read_dir(template_dir)? {
            let path = entry?.path();
            if path.extension() == Some(OsStr::new("json")) {
                let template: NAMDTemplate = serde_json::from_reader(
                    BufReader::new(File::open(&path)?)
                )?;
                
                // Validate template structure
                validate_template_schema(&template)?;
                
                templates.insert(template.id.clone(), template);
            }
        }
        
        Ok(TemplateRegistry { templates })
    }
}
```

---

## 10. NAMD Config Generation

### 10.1 Template String System

**Approach**: Use Handlebars-style template syntax with `{{variable}}` placeholders.

**Example Template**:
```tcl
# NAMD Configuration
# Generated from template: {{template_id}}

# Execution Mode
{{#if execution_mode == "minimize"}}
minimize {{steps}}
{{else}}
run {{steps}}
timestep {{timestep}}
{{/if}}

# Temperature Control
langevin on
langevinTemp {{temperature}}
langevinDamping {{langevin_damping}}

# Periodic Boundaries
PME {{use_pme}}
{{#if use_pme}}
PMEGridSpacing {{pme_grid_spacing}}
cellBasisVector1 {{cell_basis_vector_1}}
cellBasisVector2 {{cell_basis_vector_2}}
cellBasisVector3 {{cell_basis_vector_3}}
{{else}}
margin {{margin}}
{{/if}}

# Pressure Control (NPT)
{{#if use_npt}}
langevinPiston on
langevinPistonTarget {{pressure_target}}
langevinPistonPeriod {{pressure_period}}
langevinPistonDecay {{pressure_decay}}
langevinPistonTemp {{temperature}}
{{/if}}

# Extrabonds Restraints
{{#if extrabonds_file}}
extraBondsFile {{extrabonds_file}}
{{/if}}
```

### 10.2 Generation Implementation

```rust
pub fn generate_namd_config(
    template: &NAMDTemplate,
    values: &HashMap<String, Value>,
) -> Result<String> {
    use handlebars::Handlebars;
    
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("namd", &template.namd_config_template)?;
    
    // Convert template values to Handlebars-compatible format
    let data = prepare_template_data(values)?;
    
    let config = handlebars.render("namd", &data)?;
    
    Ok(config)
}

fn prepare_template_data(values: &HashMap<String, Value>) -> Result<serde_json::Value> {
    let mut data = serde_json::Map::new();
    
    for (key, value) in values {
        match value {
            Value::Bool(b) => {
                // Convert bool to NAMD yes/no
                data.insert(key.clone(), json!(if *b { "yes" } else { "no" }));
            }
            Value::Object(obj) if obj.contains_key("x") => {
                // Dimensions: format as "x y z"
                let formatted = format!("{} {} {}",
                    obj["x"].as_f64().unwrap(),
                    obj["y"].as_f64().unwrap(),
                    obj["z"].as_f64().unwrap()
                );
                data.insert(key.clone(), json!(formatted));
            }
            _ => {
                data.insert(key.clone(), value.clone());
            }
        }
    }
    
    Ok(Value::Object(data))
}
```

---

## 11. Tutorial-Derived Templates

### 11.1 Explicit Solvent NPT Template

**Based on**: `examples/origamiTutorial/step3/equil_min.namd` and `equil_k0.5.namd`

**Key Parameters**:
- PME: yes (periodic boundaries required)
- NPT ensemble: langevinPiston on
- High damping: langevinDamping 5
- System-specific box dimensions
- Optional extrabonds for restraints
- Two execution modes: minimize (first stage) or run (subsequent stages)

**Template ID**: `explicit_solvent_npt_v1`

**Form Sections**:
1. Execution Parameters (mode, steps, timestep)
2. Temperature Control (temperature, damping)
3. Periodic Boundaries (PME, grid spacing, cell vectors)
4. Pressure Control (NPT settings)
5. Restraints (extrabonds file - optional)
6. Output Frequencies (DCD, restart, energies)

### 11.2 Vacuum Optimization Template (Future)

**Based on**: `examples/origamiTutorial/step2/hextube.namd`

**Key Parameters**:
- PME: no (non-periodic)
- Large box: 1000Å × 1000Å × 1000Å
- Low damping: langevinDamping 0.1
- Large margin: 30Å
- Extrabonds for structure stabilization

**Template ID**: `vacuum_optimization_v1`

**Current Status**: Not implemented - users run Step 2 locally before cluster submission

---

## 12. Validation Rules

### 12.1 Physical Constraints

```rust
// Temperature must be in reasonable biological range
ValidationRule::Range {
    min: 200.0,
    max: 400.0,
    message: "Temperature should be 200-400 K for biological systems".to_string(),
}

// Timestep must be stable for MD integration
ValidationRule::Range {
    min: 0.5,
    max: 4.0,
    message: "Timestep should be 0.5-4.0 fs for stability".to_string(),
}

// PME grid spacing affects accuracy
ValidationRule::Range {
    min: 0.5,
    max: 3.0,
    message: "PME grid spacing should be 0.5-3.0 Å".to_string(),
}
```

### 12.2 Interdependent Validation

```rust
// PME requires cell basis vectors
ValidationRule::RequiredIf {
    field: "use_pme".to_string(),
    equals: json!(true),
    message: "Cell basis vectors must be provided when PME is enabled".to_string(),
}

// NPT requires temperature to be set
ValidationRule::RequiredIf {
    field: "use_npt".to_string(),
    equals: json!(true),
    message: "Temperature must be specified for NPT ensemble".to_string(),
}

// Minimization doesn't use timestep
// (handled by conditional visibility - don't show timestep field when mode=minimize)
```

---

## 13. User-Defined Templates

### 13.1 Template Creation UI (Future)

Allow users to create custom templates through UI:

1. **Template Editor Page**:
   - Template metadata (name, description, use_case)
   - Field definition builder (add/remove fields)
   - Field type selector
   - Validation rule builder
   - NAMD config template editor with syntax highlighting

2. **Template Preview**:
   - Live preview of generated form
   - Test with sample values
   - Generate sample NAMD config

3. **Template Storage**:
   - User templates stored in local database
   - Export/import as JSON
   - Share templates with other users

### 13.2 Template Library (Future)

**Community Template Sharing**:
- Upload templates to central repository
- Browse/search community templates
- Rating and review system
- Version control for templates
- Template categories (protein folding, DNA origami, drug discovery, etc.)

---

## 14. Template Versioning

### 14.1 Version Strategy

Templates include version in ID: `explicit_solvent_npt_v1`

**Version Changes**:
- `v1 → v2`: Breaking changes (field structure changes)
- Keep old versions for backwards compatibility
- Jobs store `template_id` with version, can always regenerate config

**Migration Strategy**:
- Old jobs use old template versions
- New jobs use latest versions
- Template registry maintains all versions
- UI shows only latest by default, but users can select older versions

---

## 15. Template Library

### 15.1 Built-in Templates

**Phase 8 Initial Set**:
1. `explicit_solvent_npt_v1` - Multi-stage equilibration (DNA origami)
2. `implicit_solvent_v1` - Generalized Born implicit solvent (future)
3. `constant_volume_v1` - NVE ensemble simulations (future)

**Future Additions**:
- Different force fields (AMBER, OPLS)
- Advanced sampling methods (umbrella sampling, steered MD)
- Specific biomolecule types (protein, DNA, lipids)

### 15.2 Template Discovery

```rust
#[tauri::command]
pub async fn search_templates(query: String, category: Option<String>) -> Result<Vec<TemplateSummary>> {
    let registry = &TEMPLATE_REGISTRY;
    
    let results = registry.list_all()
        .into_iter()
        .filter(|t| {
            let matches_query = t.name.contains(&query) || t.description.contains(&query);
            let matches_category = category.as_ref()
                .map(|c| t.category == *c)
                .unwrap_or(true);
            matches_query && matches_category
        })
        .collect();
    
    Ok(results)
}
```

---

## 16. Integration Points

### 16.1 Integration with Current Job Creation

**Phase 6 Job Creation** (uses hardcoded NAMDConfig):
```rust
pub struct JobInfo {
    pub namd_config: NAMDConfig,  // Hardcoded struct
    // ...
}
```

**Phase 8 Template-Based Creation**:
```rust
pub struct JobInfo {
    pub namd_config: NAMDConfig,     // Deprecated - kept for backwards compat
    pub template_id: String,         // NEW
    pub template_values: HashMap<String, Value>,  // NEW
    // ...
}
```

**Config Generation**:
```rust
// OLD: Use hardcoded struct
let config = SlurmScriptGenerator::generate_namd_config(&job)?;

// NEW: Use template system
let config = if job.template_id == "legacy" {
    SlurmScriptGenerator::generate_namd_config(&job)?
} else {
    let template = TEMPLATE_REGISTRY.get(&job.template_id)?;
    templates::generator::generate_config(template, &job.template_values)?
};
```

### 16.2 Integration with Job Chaining

**See**: [MultiJob_And_Templates.md](MultiJob_And_Templates.md) for how templates enable job continuation.

**Key Point**: When creating continuation jobs, the child inherits `template_id` from parent but can modify `template_values` (e.g., change restraint file, adjust simulation length).

---

## 17. Success Criteria

### 17.1 Functional Success

- [x] Phase 6.7: Basic template system (cellBasisVector, extrabonds, PME/NPT toggles)
- [ ] Template definitions load from JSON files
- [ ] Dynamic forms render based on template structure
- [ ] Conditional field visibility works correctly
- [ ] Frontend and backend validation both functional
- [ ] NAMD configs generate correctly from template values
- [ ] Jobs created with template system work end-to-end

### 17.2 Template Coverage Success

- [ ] Explicit Solvent NPT template supports DNA origami workflows
- [ ] All tutorial scenarios can be configured through template
- [ ] Template validation prevents common configuration errors

### 17.3 User Experience Success

- [ ] Scientists without NAMD expertise can create valid configs
- [ ] Template help text provides adequate guidance
- [ ] Validation messages are clear and actionable
- [ ] Form responds quickly (< 100ms for field updates)

---

## 18. Open Questions

### 18.1 Template String Syntax

**Decision Needed**: Use Handlebars, Tera, or custom template engine?

**Considerations**:
- Handlebars: Industry standard, well-documented
- Tera: More Rust-native, similar syntax
- Custom: Full control, but reinventing wheel

**Recommendation**: Start with Handlebars, evaluate if limitations encountered.

### 18.2 User Template Storage

**Decision Needed**: Where to store user-created templates?

**Options**:
1. Local database table
2. Filesystem (user_templates/ directory)
3. Both (DB for metadata, filesystem for template content)

**Recommendation**: Option 3 - DB for searchability, filesystem for editing.

### 18.3 Template Validation

**Decision Needed**: How strict should template schema validation be?

**Considerations**:
- Too strict: Hard to create templates
- Too loose: Risk of broken templates
- Balance: Validate structure, warn on unusual patterns

---

## 19. Appendices

### Appendix A: Tutorial File Reference

**Primary Source**: `examples/origamiTutorial/origamiprotocols_0_markdown.md`

**Key Files**:
- `step3/equil_min.namd` - Minimization stage template source
- `step3/equil_k0.5.namd` - Restart mechanism example
- `step3/equil_k0.1.namd` - Progressive restraint reduction pattern

### Appendix B: Related Documentation

- [MultiJob_And_Templates.md](MultiJob_And_Templates.md) - Job chaining integration
- [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md) - Current system architecture
- [docs/API.md](../../docs/API.md) - IPC command interfaces

### Appendix C: Implementation Checklist

**Phase 8 Milestone 8.1: Template System Foundation**
- [ ] Create template data structures (types.rs)
- [ ] Implement template registry (registry.rs)
- [ ] Add template IPC commands
- [ ] Create DynamicField component
- [ ] Create DynamicConfigForm component
- [ ] Implement frontend validation
- [ ] Implement backend validation
- [ ] Create config generator

**Phase 8 Milestone 8.2: Template Definitions**
- [ ] Design explicit_solvent_npt_v1 template
- [ ] Write template JSON file
- [ ] Test all field types render correctly
- [ ] Test conditional visibility
- [ ] Test validation rules
- [ ] Verify NAMD config generation

**Phase 8 Milestone 8.3: Integration**
- [ ] Add template_id and template_values to JobInfo
- [ ] Migrate database schema
- [ ] Update job creation workflow
- [ ] Update NAMD config generation in script_generator
- [ ] Test end-to-end job creation with template
- [ ] Verify backwards compatibility with legacy jobs

---

**Document Status**: Complete - Ready for Phase 8 Implementation  
**Last Updated**: 2025-11-01  
**Next Steps**: Begin Phase 8 implementation following this design

// Template type definitions matching Rust backend

export interface TemplateSummary {
  id: string;
  name: string;
  description: string;
}

export interface Template {
  id: string;
  name: string;
  description: string;
  namd_config_template: string;
  variables: Record<string, VariableDefinition>;
  created_at: string;
  updated_at: string;
}

export interface VariableDefinition {
  key: string;
  label: string;
  var_type: VariableType;
  help_text: string | null;
}

export type VariableType =
  | { Number: { min: number; max: number; default: number } }
  | { Text: { default: string } }
  | { Boolean: { default: boolean } }
  | { FileUpload: { extensions: string[] } };

// Helper function to get the variant name
export function getVariableTypeName(varType: VariableType): 'Number' | 'Text' | 'Boolean' | 'FileUpload' {
  if ('Number' in varType) return 'Number';
  if ('Text' in varType) return 'Text';
  if ('Boolean' in varType) return 'Boolean';
  if ('FileUpload' in varType) return 'FileUpload';
  throw new Error('Unknown variable type');
}


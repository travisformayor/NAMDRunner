import { describe, it, expect } from 'vitest';
import { extractVariablesFromTemplate, generateLabel, getSampleValue } from './template-utils';
import type { VariableDefinition } from '$lib/types/template';

describe('extractVariablesFromTemplate', () => {
  it('finds variables in order of first occurrence', () => {
    const template = `
structure {{structure_file}}
coordinates {{coordinates_file}}
temperature {{temperature}}
timestep {{timestep}}
    `;

    const result = extractVariablesFromTemplate(template);

    expect(result).toEqual(['structure_file', 'coordinates_file', 'temperature', 'timestep']);
  });

  it('handles duplicate variables (only tracks first occurrence)', () => {
    const template = `
temperature {{temperature}}
structure {{structure_file}}
temperature again {{temperature}}
    `;

    const result = extractVariablesFromTemplate(template);

    expect(result).toEqual(['temperature', 'structure_file']);
    expect(result).toHaveLength(2); // No duplicate
  });

  it('returns empty array for template with no variables', () => {
    const template = 'PME yes\ntimestep 2.0\nrun 1000';

    const result = extractVariablesFromTemplate(template);

    expect(result).toEqual([]);
  });

  it('handles variables at different positions correctly', () => {
    const template = 'end {{last}} middle {{middle}} start {{first}}';

    const result = extractVariablesFromTemplate(template);

    // Order based on position in string, not alphabetical
    expect(result).toEqual(['last', 'middle', 'first']);
  });

  it('ignores malformed placeholders', () => {
    const template = 'valid {{valid_var}} {single} {{spaced var}} {{-invalid}}';

    const result = extractVariablesFromTemplate(template);

    expect(result).toEqual(['valid_var']); // Only valid identifier
  });
});

describe('generateLabel', () => {
  it('converts snake_case to Title Case', () => {
    expect(generateLabel('structure_file')).toBe('Structure File');
    expect(generateLabel('pme_grid_spacing')).toBe('Pme Grid Spacing');
    expect(generateLabel('langevin_damping')).toBe('Langevin Damping');
  });

  it('handles single word', () => {
    expect(generateLabel('temperature')).toBe('Temperature');
    expect(generateLabel('steps')).toBe('Steps');
  });

  it('handles already capitalized', () => {
    expect(generateLabel('PME')).toBe('PME');
  });

  it('handles multiple underscores', () => {
    expect(generateLabel('output_energies_freq')).toBe('Output Energies Freq');
  });

  it('handles empty string', () => {
    expect(generateLabel('')).toBe('');
  });
});

describe('getSampleValue', () => {
  it('uses Number default value', () => {
    const varDef: VariableDefinition = {
      key: 'temperature',
      label: 'Temperature',
      var_type: { Number: { min: 200, max: 400, default: 300 } },
      required: true,
      help_text: null
    };

    expect(getSampleValue(varDef)).toBe('300');
  });

  it('uses Text default value', () => {
    const varDef: VariableDefinition = {
      key: 'output_name',
      label: 'Output Name',
      var_type: { Text: { default: 'simulation' } },
      required: true,
      help_text: null
    };

    expect(getSampleValue(varDef)).toBe('simulation');
  });

  it('converts Boolean true to "yes"', () => {
    const varDef: VariableDefinition = {
      key: 'pme_enabled',
      label: 'PME Enabled',
      var_type: { Boolean: { default: true } },
      required: true,
      help_text: null
    };

    expect(getSampleValue(varDef)).toBe('yes');
  });

  it('converts Boolean false to "no"', () => {
    const varDef: VariableDefinition = {
      key: 'pme_enabled',
      label: 'PME Enabled',
      var_type: { Boolean: { default: false } },
      required: true,
      help_text: null
    };

    expect(getSampleValue(varDef)).toBe('no');
  });

  it('generates filename with variable name + first extension', () => {
    const varDef: VariableDefinition = {
      key: 'structure_file',
      label: 'Structure File',
      var_type: { FileUpload: { extensions: ['.psf', '.pdb'] } },
      required: true,
      help_text: null
    };

    expect(getSampleValue(varDef)).toBe('structure_file.psf');
  });

  it('uses first extension for FileUpload', () => {
    const varDef: VariableDefinition = {
      key: 'parameters',
      label: 'Parameters',
      var_type: { FileUpload: { extensions: ['.prm', '.par', '.str'] } },
      required: true,
      help_text: null
    };

    expect(getSampleValue(varDef)).toBe('parameters.prm');
  });

  it('handles FileUpload with no extensions (fallback)', () => {
    const varDef: VariableDefinition = {
      key: 'data_file',
      label: 'Data File',
      var_type: { FileUpload: { extensions: [] } },
      required: true,
      help_text: null
    };

    expect(getSampleValue(varDef)).toBe('data_file.dat');
  });
});

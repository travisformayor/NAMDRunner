import { describe, it, expect } from 'vitest';
import { extractVariablesFromTemplate, generateLabel } from './template-utils';

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

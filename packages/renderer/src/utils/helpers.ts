import { ProgramNode } from '@codama/nodes';

function normalizedProgramName(value?: string | null): string {
    return value?.toLowerCase().replace(/[^a-z0-9]/g, '') ?? '';
}

export function partition<T>(
    arr: readonly T[],
    predicate: (value: T, index: number, array: readonly T[]) => boolean,
): [T[], T[]] {
    return arr.reduce<[T[], T[]]>(
        (out, v, i, a) => {
            out[predicate(v, i, a) ? 0 : 1].push(v);
            return out;
        },
        [[], []],
    );
}

/**
 * Helper function to check if a program is token-2022
 * Checks program node name, original program name, and package name for consistency
 */
export function isToken2022Program(program?: ProgramNode | null, originalName?: string, packageName?: string): boolean {
    return [program?.name, originalName, packageName].some(value => normalizedProgramName(value) === 'token2022');
}

/**
 * Helper function to check if a program is the classic SPL Token Program.
 * Checks program node name, original program name, and package name for consistency.
 */
export function isTokenProgram(program?: ProgramNode | null, originalName?: string, packageName?: string): boolean {
    return [program?.name, originalName, packageName].some(value => normalizedProgramName(value) === 'tokenprogram');
}

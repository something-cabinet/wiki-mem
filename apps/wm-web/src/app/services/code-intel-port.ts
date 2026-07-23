import { InjectionToken } from '@angular/core';
import { Observable } from 'rxjs';

export interface CodeIntelSymbol {
  name: string;
  kind: string;
  file: string;
  line: number;
  column: number;
  snippet: string;
  parent_name?: string;
  language: string;
}

export interface CodeIntelDep {
  target: string;
  kind: string;
  line: number;
}

export interface CodeIntelDepSet {
  file: string;
  deps: CodeIntelDep[];
}

export interface CodeIntelPort {
  searchSymbols(params: {
    name?: string;
    kind?: string;
    language?: string;
    file?: string;
    max_results?: number;
  }): Observable<{ symbols: CodeIntelSymbol[] }>;
  getDeps(params: { file?: string; reverse?: boolean }): Observable<{ dependencies: CodeIntelDepSet[] }>;
  getFile(params: { path: string }): Observable<{ content: string; language: string }>;
}

export const CODE_INTEL_PORT = new InjectionToken<CodeIntelPort>('CodeIntelPort');

/** Map of symbol kinds to badge color classes (Tailwind v4) */
const SYMBOL_KIND_COLORS: Record<string, string> = {
  function: 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300',
  method: 'bg-sky-100 text-sky-700 dark:bg-sky-900/30 dark:text-sky-300',
  class: 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300',
  interface: 'bg-teal-100 text-teal-700 dark:bg-teal-900/30 dark:text-teal-300',
  struct: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300',
  enum: 'bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300',
  trait: 'bg-pink-100 text-pink-700 dark:bg-pink-900/30 dark:text-pink-300',
  impl: 'bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-300',
  type: 'bg-violet-100 text-violet-700 dark:bg-violet-900/30 dark:text-violet-300',
  const: 'bg-rose-100 text-rose-700 dark:bg-rose-900/30 dark:text-rose-300',
  module: 'bg-gray-100 text-gray-700 dark:bg-gray-800/60 dark:text-gray-300',
  macro: 'bg-lime-100 text-lime-700 dark:bg-lime-900/30 dark:text-lime-300',
};

/**
 * Returns a Tailwind utility class string for a symbol-kind-colored badge.
 * Falls back to a generic muted style for unknown kinds.
 */
export function symbolKindBadgeClass(kind: string): string {
  return SYMBOL_KIND_COLORS[kind.toLowerCase()] ?? 'bg-muted/60 text-muted-foreground';
}

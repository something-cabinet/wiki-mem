import { Injectable, OnDestroy } from '@angular/core';
import { Subject, Observable } from 'rxjs';

export const PAGE_TYPES: { key: string; label: string }[] = [
  { key: 'concept', label: 'Concept' },
  { key: 'spec', label: 'Spec' },
  { key: 'task', label: 'Task' },
  { key: 'memory', label: 'Memory' },
  { key: 'pattern', label: 'Pattern' },
  { key: 'decision', label: 'Decision' },
  { key: 'howto', label: 'How-to' },
  { key: 'reference', label: 'Reference' },
];

/**
 * Singleton service that reads --page-type-{type} and --edge-type-{type}
 * CSS custom properties from the document. Caches parsed values and
 * observes <html>.classList for .dark theme toggles.
 */
@Injectable({ providedIn: 'root' })
export class GraphColorService implements OnDestroy {
  private readonly themeChangeSubject = new Subject<void>();
  readonly themeChanged$: Observable<void> = this.themeChangeSubject.asObservable();

  private observer: MutationObserver | null = null;

  /** Cache for CSS string results (with alpha applied) */
  private colorCache = new Map<string, string>();

  /** Cache for parsed oklch [l, c, h] components */
  private rgbCache = new Map<string, [number, number, number]>();

  constructor() {
    this.setupThemeObserver();
  }

  // ── Public API ────────────────────────────────────────

  /**
   * Returns a CSS color string from `--page-type-{type}` with 0.85 alpha.
   */
  nodeColor(type: string): string {
    const cacheKey = `node:${type}`;
    const cached = this.colorCache.get(cacheKey);
    if (cached !== undefined) return cached;

    const val = this.readCSSVar(`--page-type-${type}`);
    if (!val) {
      return this.resolveFallback(cacheKey, 'node');
    }
    const result = this.withAlpha(val, 0.85);
    this.colorCache.set(cacheKey, result);
    return result;
  }

  /**
   * Returns oklch [l, c, h] components from `--page-type-{type}` for WebGL.
   * l ranges 0–1, c is chroma, h is hue in degrees.
   */
  nodeColorRGB(type: string): [number, number, number] {
    const cacheKey = `node-rgb:${type}`;
    const cached = this.rgbCache.get(cacheKey);
    if (cached !== undefined) return cached;

    const val = this.readCSSVar(`--page-type-${type}`);
    const parsed = this.parseOklch(val);
    if (parsed) {
      this.rgbCache.set(cacheKey, parsed);
      return parsed;
    }
    return this.resolveFallbackRGB(cacheKey);
  }

  /**
   * Returns a CSS color string from `--edge-type-{type}` with 0.6 alpha.
   */
  edgeColor(type: string): string {
    const cacheKey = `edge:${type}`;
    const cached = this.colorCache.get(cacheKey);
    if (cached !== undefined) return cached;

    const val = this.readCSSVar(`--edge-type-${type}`);
    if (!val) {
      return this.resolveFallback(cacheKey, 'edge');
    }
    const result = this.withAlpha(val, 0.6);
    this.colorCache.set(cacheKey, result);
    return result;
  }

  /**
   * Returns all 8 page types with their resolved CSS color strings.
   */
  allPageTypes(): { key: string; label: string; color: string }[] {
    return PAGE_TYPES.map((pt) => ({
      key: pt.key,
      label: pt.label,
      color: this.nodeColor(pt.key),
    }));
  }

  // ── Lifecycle ─────────────────────────────────────────

  ngOnDestroy(): void {
    this.observer?.disconnect();
    this.themeChangeSubject.complete();
  }

  // ── Internal ──────────────────────────────────────────

  private setupThemeObserver(): void {
    this.observer = new MutationObserver(() => {
      this.colorCache.clear();
      this.rgbCache.clear();
      this.themeChangeSubject.next();
    });
    this.observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class'],
    });
  }

  private readCSSVar(name: string): string {
    return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  }

  /**
   * Given a raw oklch(l c h) or oklch(l c h / a) string, return it with
   * the given alpha value.
   */
  private withAlpha(val: string, alpha: number): string {
    if (val.includes('/')) {
      return val.replace(/\/\s*[\d.]+/, `/ ${alpha}`);
    }
    // Remove trailing )
    const inner = val.slice(6, -1).trim();
    return `oklch(${inner} / ${alpha})`;
  }

  /**
   * Parse oklch(l c h) or oklch(l c h / a) into [l, c, h] number components.
   * Returns null if parsing fails.
   */
  private parseOklch(val: string): [number, number, number] | null {
    if (!val.startsWith('oklch(')) return null;
    const inner = val.slice(6, val.lastIndexOf(')')).trim();
    // Strip alpha portion if present: oklch(l c h / a) → oklch(l c h)
    const noAlpha = inner.includes('/') ? inner.slice(0, inner.indexOf('/')).trim() : inner;
    const parts = noAlpha.split(/\s+/);
    if (parts.length < 3) return null;
    const l = parseFloat(parts[0]);
    const c = parseFloat(parts[1]);
    const h = parseFloat(parts[2]);
    if (isNaN(l) || isNaN(c) || isNaN(h)) return null;
    return [l, c, h];
  }

  /**
   * Resolve a fallback CSS color for node/edge when the type-specific
   * variable is not defined. Caches and returns the result.
   */
  private resolveFallback(cacheKey: string, kind: 'node' | 'edge'): string {
    const fallbackVar = kind === 'node' ? '--muted-foreground' : '--border';
    const fallbackVal = this.readCSSVar(fallbackVar);
    const result = fallbackVal || 'oklch(0.5 0.05 0)';
    this.colorCache.set(cacheKey, result);
    return result;
  }

  /**
   * Resolve fallback oklch components. Caches and returns the result.
   */
  private resolveFallbackRGB(cacheKey: string): [number, number, number] {
    const fallbackVal = this.readCSSVar('--muted-foreground');
    const parsed = this.parseOklch(fallbackVal);
    const result: [number, number, number] = parsed ?? [0.5, 0.05, 0];
    this.rgbCache.set(cacheKey, result);
    return result;
  }
}

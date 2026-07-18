import { Component, computed, input } from '@angular/core';
import type { ClassValue } from 'clsx';

@Component({
  selector: 'wm-spinner',
  standalone: true,
  template: `
    <span
      class="inline-block rounded-full animate-spin"
      [class]="computedClass()"
      role="status"
      aria-live="polite"
    >
      <span class="sr-only">Loading...</span>
    </span>
  `,
})
export class WmSpinner {
  public readonly class = input<ClassValue>('');
  public readonly size = input<'sm' | 'md' | 'lg'>('sm');

  protected computedClass = computed(() => [
    this.size() === 'sm' && 'w-4 h-4 border-2',
    this.size() === 'md' && 'w-5 h-5 border-[2.5px]',
    this.size() === 'lg' && 'w-6 h-6 border-3',
    'border-border border-t-primary',
    this.class(),
  ]);
}

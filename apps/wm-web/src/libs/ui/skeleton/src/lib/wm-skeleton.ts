import { Component, computed, input } from '@angular/core';
import type { ClassValue } from 'clsx';

/**
 * Lightweight loading-skeleton placeholder. Renders a pulsing muted block;
 * compose several of them (or size via `class`) to mimic the shape of the
 * content being fetched.
 */
@Component({
  selector: 'wm-skeleton',
  standalone: true,
  template: `
    <div
      class="rounded-md animate-pulse bg-muted"
      [class]="computedClass()"
      aria-hidden="true"
    ></div>
  `,
})
export class WmSkeleton {
  public readonly class = input<ClassValue>('');

  protected computedClass = computed(() => [this.class()]);
}

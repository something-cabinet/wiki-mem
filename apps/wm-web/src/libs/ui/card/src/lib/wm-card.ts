import { Component, computed, input } from '@angular/core';
import type { ClassValue } from 'clsx';

@Component({
  selector: 'div[wmCard]',
  standalone: true,
  host: { '[class]': 'computedClass()' },
  template: '<ng-content />',
})
export class WmCard {
  public readonly class = input<ClassValue>('');

  protected computedClass = computed(() => [
    'rounded-xl border border-border bg-card text-card-foreground shadow-sm p-5',
    this.class(),
  ]);
}

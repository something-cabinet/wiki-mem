import { Directive, computed, input } from '@angular/core';
import type { ClassValue } from 'clsx';

@Directive({
  selector: 'input[wmInput]',
  exportAs: 'wmInput',
  host: { '[class]': 'computedClass()' },
})
export class WmInput {
  public readonly class = input<ClassValue>('');

  protected computedClass = computed(() => [
    'flex h-10 w-full rounded-lg border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:border-transparent disabled:cursor-not-allowed disabled:opacity-50 disabled:bg-muted/50 transition-shadow',
    this.class(),
  ]);
}

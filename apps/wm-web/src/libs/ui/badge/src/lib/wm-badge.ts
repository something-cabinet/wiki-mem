import { Directive, computed, input } from '@angular/core';
import type { ClassValue } from 'clsx';

@Directive({
  selector: '[wmBadge]',
  exportAs: 'wmBadge',
  host: { '[class]': 'computedClass()' },
})
export class WmBadge {
  public readonly variant = input<string>('default');
  public readonly class = input<ClassValue>('');

  protected computedClass = computed(() => [
    'inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1',
    this.variant() === 'default' && 'bg-primary/10 text-primary hover:bg-primary/20',
    this.variant() === 'secondary' && 'bg-muted text-muted-foreground hover:bg-muted/80',
    this.variant() === 'outline' && 'border border-input text-foreground hover:bg-accent hover:text-accent-foreground',
    this.variant() === 'success' && 'bg-success/10 text-success hover:bg-success/20',
    this.class(),
  ]);
}

import type { ClassValue } from 'clsx';
import { Directive, computed, input } from '@angular/core';
import { BrnButton } from '@spartan-ng/brain/button';

@Directive({
  selector: 'button[wmBtn]',
  exportAs: 'wmBtn',
  hostDirectives: [{ directive: BrnButton, inputs: ['disabled'] }],
  host: { '[class]': 'computedClass()' },
})
export class WmButton {
  public readonly variant = input<'default' | 'outline' | 'ghost' | 'destructive'>('default');
  public readonly size = input<'default' | 'sm' | 'lg' | 'icon'>('default');
  public readonly class = input<ClassValue>('');

  protected computedClass = computed(() => [
    'inline-flex items-center justify-center whitespace-nowrap rounded-lg text-sm font-medium transition-colors duration-150 outline-none select-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 disabled:shadow-none',
    this.variant() === 'default' && 'bg-primary text-primary-foreground hover:bg-primary/90 shadow-sm hover:shadow',
    this.variant() === 'outline' && 'border border-input bg-background hover:bg-accent hover:text-accent-foreground shadow-sm',
    this.variant() === 'ghost' && 'text-foreground hover:bg-accent hover:text-accent-foreground',
    this.variant() === 'destructive' && 'bg-destructive text-destructive-foreground hover:bg-destructive/90 shadow-sm hover:shadow',
    this.size() === 'default' && 'h-10 px-4 py-2',
    this.size() === 'sm' && 'h-8 px-3 text-xs',
    this.size() === 'lg' && 'h-12 px-6',
    this.size() === 'icon' && 'h-10 w-10',
    this.class(),
  ]);
}

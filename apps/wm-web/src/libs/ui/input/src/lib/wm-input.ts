import { Directive, computed, input, ElementRef, inject } from '@angular/core';
import type { ClassValue } from 'clsx';

@Directive({
  selector: 'input[wmInput], textarea[wmInput]',
  exportAs: 'wmInput',
  host: { '[class]': 'computedClass()' },
})
export class WmInput {
  public readonly class = input<ClassValue>('');
  private element: ElementRef<HTMLInputElement | HTMLTextAreaElement> = inject(ElementRef);

  protected computedClass = computed(() => {
    const base = 'w-full rounded-lg border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:border-transparent disabled:cursor-not-allowed disabled:opacity-50 disabled:bg-muted/50 transition-shadow';
    const sizing = this.element.nativeElement.tagName === 'TEXTAREA'
      ? 'min-h-[80px] resize-y'
      : 'flex h-10';
    return [base, sizing, this.class()];
  });
}

import { Component, input, output } from '@angular/core';

@Component({
  selector: 'wm-select',
  standalone: true,
  template: `
    <div class="relative">
      <select
        [value]="value()"
        (change)="valueChange.emit($any($event.target).value)"
        class="flex h-10 w-full rounded-lg border border-input bg-background px-3 py-2 pr-8 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:border-transparent disabled:cursor-not-allowed disabled:opacity-50 disabled:bg-muted/50 appearance-none cursor-pointer transition-colors duration-150 hover:border-foreground/20"
      >
        <ng-content />
      </select>
      <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2">
        <svg class="h-4 w-4 text-muted-foreground transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
      </div>
    </div>
  `,
})
export class WmSelect {
  public readonly value = input<string>('');
  public readonly valueChange = output<string>();
}

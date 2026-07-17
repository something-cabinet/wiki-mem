import { Component, input, output, signal } from '@angular/core';

@Component({
  selector: 'wm-accordion',
  standalone: true,
  template: `
    <div class="border-b border-border">
      <button
        (click)="toggle()"
        [attr.aria-expanded]="expanded()"
        class="flex w-full items-center justify-between py-3 text-sm font-medium text-left text-foreground hover:bg-muted/50 px-2 rounded-lg transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1"
      >
        <ng-content select="[slot=header]" />
        <svg
          class="h-4 w-4 text-muted-foreground transition-transform duration-300 ease-out"
          [class.rotate-180]="expanded()"
          fill="none" viewBox="0 0 24 24" stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
        </svg>
      </button>
      <div
        class="grid transition-[grid-template-rows] duration-300 ease-out"
        [style.grid-template-rows]="expanded() ? '1fr' : '0fr'"
      >
        <div class="overflow-hidden">
          <div class="pb-3 px-2 text-sm text-muted-foreground">
            <ng-content />
          </div>
        </div>
      </div>
    </div>
  `,
})
export class WmAccordion {
  public readonly expanded = input(false);
  public readonly expandedChange = output<boolean>();

  toggle() {
    this.expandedChange.emit(!this.expanded());
  }
}

import { Component, input, output } from '@angular/core';
import { BrnDialog, BrnDialogOverlay, BrnDialogContent, BrnDialogImports } from '@spartan-ng/brain/dialog';

@Component({
  selector: 'wm-dialog',
  standalone: true,
  imports: [BrnDialogImports],
  template: `
    <brn-dialog [state]="isOpen() ? 'open' : 'closed'" (stateChanged)="onStateChanged($event)">
      @if (isOpen()) {
        <brn-dialog-overlay (click)="close.emit()" class="fixed inset-0 bg-black/60 backdrop-blur-[2px] z-40 transition-all duration-200 ease-out" />
      }
      <div
        *brnDialogContent
        class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-[calc(100%-2rem)] max-w-lg max-h-[90vh] overflow-y-auto rounded-xl border border-border/50 bg-card p-6 shadow-2xl transition-all duration-200 ease-out data-[state=closed]:opacity-0 data-[state=closed]:scale-95 data-[state=open]:opacity-100 data-[state=open]:scale-100"
      >
        <button
          (click)="close.emit()"
          class="absolute top-3 right-3 rounded-full p-1 text-muted-foreground opacity-70 hover:opacity-100 hover:bg-muted transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          aria-label="Close dialog"
        >
          <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
        <ng-content />
      </div>
    </brn-dialog>
  `,
})
export class WmDialog {
  public readonly isOpen = input(false);
  public readonly close = output<void>();

  onStateChanged(state: string) {
    if (state === 'closed') {
      this.close.emit();
    }
  }
}

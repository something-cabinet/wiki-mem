import { Component, inject, input } from '@angular/core';
import { Location } from '@angular/common';
import { Router } from '@angular/router';
import { HlmButton } from '@ui/button';
import { NgIcon } from '@ng-icons/core';
import { lucideChevronLeft } from '@ng-icons/lucide';
import { provideIcons } from '@ng-icons/core';

@Component({
  selector: 'app-back-button',
  standalone: true,
  imports: [HlmButton, NgIcon],
  providers: [provideIcons({ lucideChevronLeft })],
  template: `
    <button hlmBtn variant="ghost" size="sm" (click)="goBack()" class="-ml-2">
      <ng-icon name="lucideChevronLeft" size="16" />
      Back
    </button>
  `,
})
export class BackButtonComponent {
  private location = inject(Location);
  private router = inject(Router);
  /** Fallback route when there's no browser history to go back to */
  fallback = input<string>();

  goBack() {
    if (window.history.length > 1) {
      this.location.back();
    } else if (this.fallback()) {
      this.router.navigate([this.fallback()]);
    }
  }
}

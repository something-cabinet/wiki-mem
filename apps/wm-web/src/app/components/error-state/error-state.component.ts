import { Component, ChangeDetectionStrategy, input, output } from '@angular/core';
import { NgIcon, provideIcons } from '@ng-icons/core';
import { lucideRefreshCw } from '@ng-icons/lucide';
import { HlmButton } from '@ui/button';
import { HlmAlert, HlmAlertTitle, HlmAlertDescription } from '@ui/alert';

/**
 * Shared inline error state: a single destructive alert with a retry button.
 * Convention matches the Settings view error block so every view reports
 * failures the same way (alert + retry, no duplicate toast).
 */
@Component({
  selector: 'wm-error-state',
  standalone: true,
  imports: [HlmAlert, HlmAlertTitle, HlmAlertDescription, HlmButton, NgIcon],
  providers: [provideIcons({ lucideRefreshCw })],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div role="alert" hlmAlert variant="destructive" class="max-w-sm">
      <p hlmAlertTitle>{{ title() }}</p>
      <p hlmAlertDescription>{{ message() }}</p>
      <button hlmBtn variant="outline" size="sm" (click)="retry.emit()" class="mt-3">
        <ng-icon name="lucideRefreshCw" size="14" />
        Retry
      </button>
    </div>
  `,
})
export class WmErrorState {
  /** Alert heading, e.g. "Connection Error" or "Failed to load graph". */
  title = input<string>('Connection Error');
  /** Human-readable failure detail. */
  message = input.required<string>();
  /** Emitted when the user clicks Retry. */
  retry = output<void>();
}

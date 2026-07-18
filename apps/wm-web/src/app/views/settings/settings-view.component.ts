import { Component, OnInit, ChangeDetectionStrategy, DestroyRef } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { NgIcon, provideIcons } from '@ng-icons/core';
import { lucideRefreshCw, lucideSun, lucideMoon } from '@ng-icons/lucide';
import { WmButton } from '@ui/button';
import { WmCard } from '@ui/card';
import { WmBadge } from '@ui/badge';
import { WmSpinner } from '@ui/spinner';
import { ApiService, InitialState } from '../../services/api.service';

@Component({
  selector: 'app-settings-view',
  standalone: true,
  imports: [NgIcon, WmButton, WmCard, WmBadge, WmSpinner],
  providers: [provideIcons({ lucideRefreshCw, lucideSun, lucideMoon })],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="flex flex-col h-full">
      <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
        <h1 class="text-xl sm:text-2xl font-bold">Settings</h1>
        <button
          wmBtn
          variant="outline"
          (click)="refresh()"
          title="Refresh engine status"
          class="gap-1.5"
        >
           <ng-icon name="lucideRefreshCw" size="16" />
          Refresh
        </button>
      </header>
      <div class="flex-1 p-6 max-w-2xl mx-auto overflow-y-auto">
      @if (state) {
        <div wmCard class="p-5">
          <h2 class="font-semibold mb-4 text-sm uppercase tracking-wider text-muted-foreground">Engine Status</h2>
          <dl class="space-y-3 text-sm">
            <div class="flex justify-between items-center py-1 border-b border-border">
              <dt class="text-muted-foreground">Graph Nodes</dt>
              <dd><span wmBadge variant="secondary">{{ state.graph_node_count }}</span></dd>
            </div>
            <div class="flex justify-between items-center py-1 border-b border-border">
              <dt class="text-muted-foreground">Graph Edges</dt>
              <dd><span wmBadge variant="secondary">{{ state.graph_edge_count }}</span></dd>
            </div>
            <div class="flex justify-between items-center py-1 border-b border-border">
              <dt class="text-muted-foreground">Session Memory</dt>
              <dd><span wmBadge variant="secondary">{{ state.session_memory_count }}</span></dd>
            </div>
            <div class="flex justify-between items-center py-1 border-b border-border">
              <dt class="text-muted-foreground">Uptime</dt>
              <dd><span wmBadge variant="secondary" class="font-mono">{{ formatUptime(state.uptime_secs) }}</span></dd>
            </div>
            <div class="flex justify-between items-center py-1">
              <dt class="text-muted-foreground">Stale</dt>
              <dd>
                @if (state.stale) {
                  <span wmBadge class="bg-destructive/10 text-destructive">Yes</span>
                } @else {
                  <span wmBadge variant="success">No</span>
                }
              </dd>
            </div>
          </dl>
        </div>
        <div wmCard class="p-5 mt-4">
          <h2 class="font-semibold mb-4 text-sm uppercase tracking-wider text-muted-foreground">Appearance</h2>
          <div class="flex items-center justify-between">
            <span class="text-sm">Dark Mode</span>
            <button
              wmBtn
              variant="outline"
              size="sm"
              (click)="toggleDarkMode()"
              class="gap-2 min-w-[130px] justify-center"
              [attr.aria-label]="isDarkMode ? 'Switch to light mode' : 'Switch to dark mode'"
            >
              <ng-icon [name]="isDarkMode ? 'lucideSun' : 'lucideMoon'" size="16" />
              <span>{{ isDarkMode ? 'Light Mode' : 'Dark Mode' }}</span>
            </button>
          </div>
        </div>
      } @else if (error) {
        <div class="p-4 bg-card border border-destructive/30 rounded-xl text-destructive text-sm shadow-sm max-w-sm">
          <p class="font-medium">Connection Error</p>
          <p class="text-muted-foreground mt-1">{{ error }}</p>
          <button wmBtn variant="outline" size="sm" (click)="refresh()" class="mt-3">
            <ng-icon name="lucideRefreshCw" size="14" />
            Retry
          </button>
        </div>
      } @else {
        <div class="flex items-center gap-2 text-muted-foreground">
          <wm-spinner size="sm" />
          Loading...
        </div>
      }
      </div>
    </div>
  `,
})
export class SettingsViewComponent implements OnInit {
  state: InitialState | null = null;
  error = '';
  isDarkMode = localStorage.getItem('wm-dark-mode') === 'true';

  constructor(private api: ApiService, private destroyRef: DestroyRef) {}

  ngOnInit() {
    this.applyDarkMode();
    this.refresh();
  }

  toggleDarkMode() {
    this.isDarkMode = !this.isDarkMode;
    this.applyDarkMode();
    localStorage.setItem('wm-dark-mode', String(this.isDarkMode));
  }

  private applyDarkMode() {
    document.documentElement.classList.toggle('dark', this.isDarkMode);
  }

  refresh() {
    this.error = '';
    this.api.getInitial().pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        if (res.success) this.state = res;
      },
      error: () => {
        this.error = 'Failed to load settings. Check that the server is running.';
        this.state = null;
      },
    });
  }

  formatUptime(seconds: number): string {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    return `${h}h ${m}m ${s}s`;
  }
}

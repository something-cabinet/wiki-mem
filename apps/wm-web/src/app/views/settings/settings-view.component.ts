import { Component, OnInit, ChangeDetectionStrategy, DestroyRef, Inject } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { NgIcon, provideIcons } from '@ng-icons/core';
import { lucideRefreshCw, lucideAlertTriangle, lucideCheckCircle, lucideRotateCw } from '@ng-icons/lucide';
import { toast } from 'ngx-sonner';
import { HlmButton } from '@ui/button';
import { HlmCard } from '@ui/card';
import { HlmBadge } from '@ui/badge';
import { WmSpinner } from '@ui/spinner';
import { HlmSwitch } from '@ui/switch';
import { HlmAlert, HlmAlertTitle, HlmAlertDescription } from '@ui/alert';
import { EnginePort, ENGINE_PORT, InitialState } from '../../services/engine-port';
import { ThemeService } from '../../services/theme.service';

@Component({
  selector: 'app-settings-view',
  standalone: true,
  imports: [NgIcon, HlmButton, HlmCard, HlmBadge, WmSpinner,     HlmSwitch, HlmAlert, HlmAlertTitle, HlmAlertDescription],
  providers: [provideIcons({ lucideRefreshCw, lucideAlertTriangle, lucideCheckCircle, lucideRotateCw })],
  changeDetection: ChangeDetectionStrategy.Default,
  template: `
    <div class="flex flex-col h-full">
      <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
        <h1 class="text-xl sm:text-2xl font-semibold">Settings</h1>
        <button
          hlmBtn
          variant="outline"
          (click)="refresh()"
          title="Refresh engine status"
          class="gap-1.5"
        >
           <ng-icon name="lucideRefreshCw" size="16" />
          Refresh
        </button>
      </header>
      <div class="flex-1 overflow-y-auto">
      <div class="p-6 max-w-4xl mx-auto w-full">
      @if (state) {
        <div hlmCard class="p-5">
          <h2 class="font-semibold mb-4 text-sm uppercase tracking-wider text-muted-foreground">Engine Status</h2>
          <dl class="space-y-3 text-sm">
            <div class="flex justify-between items-center py-1 border-b border-border">
              <dt class="text-muted-foreground">Graph Nodes</dt>
              <dd><span hlmBadge variant="secondary" class="font-mono tabular-nums">{{ state.graph_node_count }}</span></dd>
            </div>
            <div class="flex justify-between items-center py-1 border-b border-border">
              <dt class="text-muted-foreground">Graph Edges</dt>
              <dd><span hlmBadge variant="secondary" class="font-mono tabular-nums">{{ state.graph_edge_count }}</span></dd>
            </div>
            <div class="flex justify-between items-center py-1 border-b border-border">
              <dt class="text-muted-foreground">Session Memory</dt>
              <dd><span hlmBadge variant="secondary" class="font-mono tabular-nums">{{ state.session_memory_count }}</span></dd>
            </div>
            <div class="flex justify-between items-center py-1 border-b border-border">
              <dt class="text-muted-foreground">Uptime</dt>
              <dd><span hlmBadge variant="secondary" class="font-mono">{{ formatUptime(state.uptime_secs) }}</span></dd>
            </div>
            <div class="flex justify-between items-center py-1">
              <dt class="text-muted-foreground">Index Status</dt>
              <dd class="flex items-center gap-2">
                @if (state.stale) {
                  <span hlmBadge variant="outline" class="text-destructive border-destructive/30">
                    <ng-icon name="lucideAlertTriangle" size="12" class="mr-1" />
                    Stale
                  </span>
                  <button
                    hlmBtn
                    variant="default"
                    size="sm"
                    (click)="rebuildIndex()"
                    [disabled]="rebuilding"
                    class="gap-1"
                  >
                    <ng-icon name="lucideRotateCw" size="14" [class.animate-spin]="rebuilding" />
                    {{ rebuilding ? 'Rebuilding...' : 'Rebuild' }}
                  </button>
                } @else {
                  <span hlmBadge variant="secondary">
                    <ng-icon name="lucideCheckCircle" size="12" class="mr-1" />
                    Fresh
                  </span>
                }
              </dd>
            </div>
          </dl>
        </div>
        <div hlmCard class="p-5 mt-4">
          <h2 class="font-semibold mb-4 text-sm uppercase tracking-wider text-muted-foreground">Appearance</h2>
          <div class="flex items-center justify-between">
            <label class="flex items-center gap-2 text-sm cursor-pointer">
              <span>Dark Mode</span>
              <hlm-switch [checked]="theme.isDark()" (checkedChange)="theme.toggle()" aria-label="Toggle dark mode" />
            </label>
          </div>

        </div>
      } @else if (error) {
        <div hlmAlert variant="destructive" class="max-w-sm">
          <p hlmAlertTitle>Connection Error</p>
          <p hlmAlertDescription>{{ error }}</p>
          <button hlmBtn variant="outline" size="sm" (click)="refresh()" class="mt-3">
            <ng-icon name="lucideRefreshCw" size="14" />
            Retry
          </button>
        </div>
      } @else {
        <div class="flex items-center justify-center gap-2 text-muted-foreground py-16">
          <wm-spinner size="sm" />
          <span class="text-sm">Loading settings...</span>
        </div>
      }
      </div>
      </div>
    </div>
  `,
})
export class SettingsViewComponent implements OnInit {
  state: InitialState | null = null;
  error = '';
  rebuilding = false;

  constructor(
    @Inject(ENGINE_PORT) private api: EnginePort,
    private destroyRef: DestroyRef,
    protected theme: ThemeService,
  ) {}

  ngOnInit() {
    this.refresh();
  }

  refresh() {
    this.error = '';
    this.api.getInitial().pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        this.state = res;
      },
      error: () => {
        this.error = 'Failed to load settings. Check that the server is running.';
        this.state = null;
      },
    });
  }

  rebuildIndex() {
    this.rebuilding = true;
    this.api.rebuildIndex().pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        this.rebuilding = false;
        if (res.success) {
          toast.success(`Index rebuilt — ${res.nodes} nodes`);
          this.refresh();
        }
      },
      error: () => {
        this.rebuilding = false;
        toast.error('Failed to rebuild index');
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

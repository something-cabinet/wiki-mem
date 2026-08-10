import { Component, OnInit, ChangeDetectionStrategy, DestroyRef, Inject, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { NgIcon, provideIcons } from '@ng-icons/core';
import { lucideBrain } from '@ng-icons/lucide';
import { EnginePort, ENGINE_PORT, MemoryEntry } from '../../services/engine-port';
import { HlmButton } from '@ui/button';
import { HlmCard } from '@ui/card';
import { HlmBadge } from '@ui/badge';
import { WmSkeleton } from '@ui/skeleton';
import { WmErrorState } from '../../components/error-state/error-state.component';
import { HlmSelect, HlmSelectTrigger, HlmSelectValue, HlmSelectContent, HlmSelectPortal, HlmSelectItem } from '@ui/select';

@Component({
  selector: 'app-memory-view',
  standalone: true,
  imports: [
    HlmButton,
    HlmCard,
    HlmBadge,
    WmSkeleton,
    WmErrorState,
    HlmSelect,
    HlmSelectTrigger,
    HlmSelectValue,
    HlmSelectContent,
    HlmSelectPortal,
    HlmSelectItem,
    NgIcon,
  ],
  providers: [provideIcons({ lucideBrain })],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="flex flex-col h-full wm-page-enter">
      <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
        <h1 class="text-xl sm:text-2xl font-semibold">Memory</h1>
        <div class="flex items-center gap-2 flex-wrap">
          <div hlmSelect [value]="selectedLayer()" (valueChange)="selectedLayer.set($event ?? ''); loadMemory()" class="w-44 shrink-0">
            <hlm-select-trigger class="w-full">
              <hlm-select-value placeholder="Layer" />
            </hlm-select-trigger>
            <hlm-select-content *hlmSelectPortal>
              <hlm-select-item value="">All Memory</hlm-select-item>
            </hlm-select-content>
          </div>
          <div hlmSelect [value]="selectedStatus()" (valueChange)="selectedStatus.set($event ?? ''); loadMemory()" class="w-44 shrink-0">
            <hlm-select-trigger class="w-full">
              <hlm-select-value placeholder="Status" />
            </hlm-select-trigger>
            <hlm-select-content *hlmSelectPortal>
              <hlm-select-item value="">All Statuses</hlm-select-item>
              <hlm-select-item value="active">Active</hlm-select-item>
              <hlm-select-item value="stale">Stale</hlm-select-item>
              <hlm-select-item value="archived">Archived</hlm-select-item>
            </hlm-select-content>
          </div>
        </div>
      </header>
      <div class="flex-1 overflow-y-auto">
        <div class="p-6 max-w-4xl mx-auto w-full">
          @if (loading()) {
            <div role="status" aria-live="polite" aria-busy="true" class="space-y-2">
              @for (row of skeletonRows; track row) {
                <wm-skeleton class="h-24 w-full" />
              }
              <span class="sr-only">Loading memory entries...</span>
            </div>
          }
          @if (error()) {
            <wm-error-state [message]="error()" (retry)="loadMemory()" />
          }
          @if (entries().length > 0) {
            <div class="space-y-2" role="list">
              @for (e of entries(); track e.id) {
                <div hlmCard class="p-4 transition-shadow hover:shadow-md hover:border-foreground/20 cursor-pointer" role="listitem">
                  <div class="flex items-center justify-between">
                    <span class="font-medium truncate">{{ e.title || e.id }}</span>
                    <span class="text-xs text-muted-foreground font-mono">{{ e.created_at.substring(0, 10) }}</span>
                  </div>
                  @if (e.tags.length > 0) {
                    <div class="flex flex-wrap gap-1.5 mt-2">
                      @for (tag of e.tags; track tag) {
                        <span hlmBadge variant="secondary">{{ tag }}</span>
                      }
                    </div>
                  }
                  <div class="mt-2">
                    @if (expanded()[e.id]) {
                      <p class="text-sm text-muted-foreground leading-relaxed">{{ e.content }}</p>
                    } @else {
                      <p class="text-sm text-muted-foreground leading-relaxed line-clamp-3">{{ e.content }}</p>
                    }
                    @if (e.content.length > 240) {
                      <button
                        hlmBtn
                        variant="link"
                        size="xs"
                        (click)="toggleExpanded(e.id)"
                        [attr.aria-expanded]="expanded()[e.id]"
                        class="mt-1.5"
                      >
                        {{ expanded()[e.id] ? 'Show less' : 'Show more' }}
                      </button>
                    }
                  </div>
                </div>
              }
            </div>
          }
          @if (!loading() && !error() && entries().length === 0) {
            <div class="flex flex-col items-center justify-center py-16 text-muted-foreground">
              <ng-icon name="lucideBrain" size="32" class="text-muted-foreground/30" />
              <p class="text-lg font-medium mt-4">No memory entries</p>
              <p class="text-xs text-muted-foreground/60 mt-1">Memory entries appear here once they are added to the wiki.</p>
            </div>
          }
        </div>
      </div>
    </div>
  `,
})
export class MemoryViewComponent implements OnInit {
  selectedLayer = signal('project');
  selectedStatus = signal('');
  entries = signal<MemoryEntry[]>([]);
  loading = signal(true);
  error = signal('');
  expanded = signal<Record<string, boolean>>({});
  /** Static row indices used to shape the loading skeleton. */
  skeletonRows = [0, 1, 2, 3];

  private destroyRef = inject(DestroyRef);

  constructor(@Inject(ENGINE_PORT) private api: EnginePort) {}

  ngOnInit() {
    this.loadMemory();
  }

  toggleExpanded(id: string) {
    this.expanded.update((m) => ({ ...m, [id]: !m[id] }));
  }

  loadMemory() {
    this.loading.set(true);
    this.error.set('');
    this.api.listMemory(this.selectedLayer(), this.selectedStatus()).pipe(
      takeUntilDestroyed(this.destroyRef),
    ).subscribe({
      next: (res) => {
        this.entries.set(res.entries || []);
        this.loading.set(false);
      },
      error: () => {
        this.error.set('Failed to load memory entries');
        this.loading.set(false);
      },
    });
  }
}

import { Component, ChangeDetectionStrategy, DestroyRef } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { NgIcon, provideIcons } from '@ng-icons/core';
import { lucideSearch } from '@ng-icons/lucide';
import { HlmButton } from '@ui/button';
import { HlmInput } from '@ui/input';
import { HlmBadge } from '@ui/badge';
import { WmSpinner } from '@ui/spinner';
import { HlmAlert, HlmAlertDescription } from '@ui/alert';
import { HlmCard } from '@ui/card';
import { HlmTabs, HlmTabsList, HlmTabsTrigger } from '@ui/tabs';
import { HlmTooltipImports } from '@ui/tooltip';
import { ApiService, SearchResult } from '../../services/api.service';
import { pageTypeBadgeClass } from '@ui/graph';

@Component({
  selector: 'app-search-view',
  standalone: true,
  imports: [FormsModule, RouterLink, HlmButton, HlmInput, HlmBadge, WmSpinner, NgIcon, HlmAlert, HlmAlertDescription, HlmCard, HlmTabs, HlmTabsList, HlmTabsTrigger, HlmTooltipImports],
  providers: [provideIcons({ lucideSearch })],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="flex flex-col h-full">
      <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
        <h1 class="text-xl sm:text-2xl font-semibold">Search</h1>
        @if (!loading && results.length > 0) {
          <span hlmBadge variant="secondary">{{ results.length }} result{{ results.length === 1 ? '' : 's' }}</span>
        }
      </header>
      <div class="flex-1 overflow-y-auto">
      <div class="p-6 max-w-4xl mx-auto w-full">
      <div class="flex flex-col gap-3 mb-4">
        <div class="flex gap-2">
          <div class="relative flex-1">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <ng-icon name="lucideSearch" size="16" class="text-muted-foreground" />
              @if (debouncing) {
                <span class="ml-2 w-1.5 h-1.5 rounded-full bg-primary animate-pulse" aria-label="Typing..."></span>
              }
            </div>
            <input
              hlmInput
              #searchInput
              [(ngModel)]="query"
              (input)="onSearchInput()"
              (keydown.enter)="doSearch()"
              placeholder="Search pages, tasks, memory..."
              aria-label="Search query"
              class="pl-9"
            />
          </div>
          <button
            hlmBtn
            variant="default"
            (click)="doSearch()"
          >
            Search
          </button>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-xs text-muted-foreground font-medium">TYPE</span>
          <div hlmTabs [tab]="searchType" (tabActivated)="searchType = $event; doSearch()">
            <div hlmTabsList class="h-8">
              @for (t of typeOptions; track t.value) {
                <button hlmTabsTrigger [hlmTabsTrigger]="t.value">{{ t.label }}</button>
              }
            </div>
          </div>
        </div>
      </div>
      @if (loading) {
        <div role="status" aria-live="polite" class="flex items-center gap-2 text-muted-foreground">
          <wm-spinner size="sm" />
          Searching...
        </div>
      }
      @if (error) {
        <div role="alert" hlmAlert variant="destructive" class="p-3 text-sm">
          <p hlmAlertDescription>{{ error }}</p>
        </div>
      }
      @if (!loading && !error && results.length > 0) {
        <div role="list" aria-label="Search results" class="space-y-2">
          @for (r of results; track r.id) {
            <a hlmCard size="sm" role="listitem" [routerLink]="['/pages', r.id]"
               class="block px-4 py-3 hover:bg-accent/50 transition-colors no-underline cursor-pointer">
              <div class="flex items-center justify-between">
                <span class="font-medium text-primary truncate">{{ (r.id.split('#')[0].split(':').pop()) || r.id }}</span>
              </div>
              <div class="flex items-center gap-2">
                <span hlmBadge variant="secondary" [class]="pageTypeBadgeClass(r.type)" class="font-medium">{{ r.type }}</span>
                @if (r.page_type) {
                  <span hlmBadge variant="outline" [class]="pageTypeBadgeClass(r.page_type)" class="font-medium">{{ r.page_type }}</span>
                }
                <span class="ml-auto text-xs text-muted-foreground font-mono cursor-help underline decoration-dotted underline-offset-2"
                      [hlmTooltip]="r.score_breakdown ? scoreTip : 'score ' + r.score.toFixed(2)"
                      position="left"
                      tabindex="0">score {{ r.score.toFixed(2) }}</span>
              </div>
              <p class="text-sm text-muted-foreground leading-relaxed line-clamp-2">{{ r.snippet }}</p>
              <ng-template #scoreTip>
                <div class="w-48" role="table" aria-label="Score breakdown">
                  <div class="px-1 pb-1 text-[10px] font-semibold uppercase tracking-wider opacity-60">Score breakdown</div>
                  @for (row of breakdownRows(r); track row.label) {
                    <div class="flex items-baseline justify-between gap-6 px-1 py-0.5">
                      <span class="opacity-70">{{ row.label }}</span>
                      <span class="font-mono tabular-nums">{{ row.value.toFixed(3) }}</span>
                    </div>
                  }
                  <div class="mx-1 my-1.5 border-t border-current opacity-20"></div>
                  <div class="flex items-baseline justify-between gap-6 px-1 pt-0.5 font-semibold">
                    <span>Final</span>
                    <span class="font-mono tabular-nums">{{ r.score.toFixed(3) }}</span>
                  </div>
                </div>
              </ng-template>
            </a>
          }
        </div>
      }
      @if (!loading && !error && !query && results.length === 0) {
        <div class="text-center py-16">
          <ng-icon name="lucideSearch" size="36" class="text-muted-foreground/30 mx-auto mb-4" />
          <p class="text-lg font-medium text-muted-foreground">Search across pages, tasks, and memory</p>
          <p class="text-sm text-muted-foreground/60 mt-1">Type a query above and press <kbd class="px-1.5 py-0.5 bg-muted rounded text-xs font-mono">Enter</kbd> to search</p>
        </div>
      }
      @if (!loading && !error && query && results.length === 0) {
        <div class="flex flex-col items-center justify-center py-16 text-muted-foreground">
          <ng-icon name="lucideSearch" size="32" class="text-muted-foreground/30" />
          <p class="text-lg font-medium mt-4">No results found</p>
          <p class="text-sm text-muted-foreground/60 mt-1">Try a different search term or adjust the search type.</p>
        </div>
      }
      </div>
      </div>
    </div>
  `,
})

export class SearchViewComponent {
  /** Bound to template — Angular templates can't call imported functions directly */
  pageTypeBadgeClass = pageTypeBadgeClass;
  query = '';
  searchType = 'all';
  results: SearchResult[] = [];
  loading = false;
  error = '';
  debouncing = false;
  private searchTimeout: ReturnType<typeof setTimeout> | null = null;
  typeOptions = [
    { value: 'all', label: 'All' },
    { value: 'page', label: 'Pages' },
    { value: 'task', label: 'Tasks' },
    { value: 'memory', label: 'Memory' },
  ];

  breakdownRows(r: SearchResult): { label: string; value: number }[] {
    const b = r.score_breakdown!;
    return [
      { label: 'BM25', value: b.bm25 },
      { label: 'RRF', value: b.rrf },
      { label: 'Semantic', value: b.semantic },
      { label: 'Title', value: b.title_density },
      { label: 'Exact title', value: b.exact_title },
      { label: 'Tags', value: b.tag_overlap },
      { label: 'Exact ID', value: b.exact_id },
      { label: 'Recency', value: b.recency },
    ];
  }

  constructor(private api: ApiService, private destroyRef: DestroyRef) {}

  onSearchInput() {
    if (this.searchTimeout) {
      clearTimeout(this.searchTimeout);
    }
    this.error = '';
    this.debouncing = true;
    this.searchTimeout = setTimeout(() => {
      this.debouncing = false;
      this.doSearch();
    }, 300);
  }

  doSearch() {
    if (!this.query.trim()) return;
    this.loading = true;
    this.error = '';
    this.api.searchQuery(this.query, this.searchType).pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        if (res.success) {
          this.results = res.results || [];
        } else {
          this.error = res.error || 'Search failed';
        }
        this.loading = false;
      },
      error: () => {
        this.error = 'Search failed. Check that the server is running.';
        this.loading = false;
      },
    });
  }
}


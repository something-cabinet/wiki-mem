import { Component, ChangeDetectionStrategy, DestroyRef } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { NgIcon, provideIcons } from '@ng-icons/core';
import { lucideSearch } from '@ng-icons/lucide';
import { WmButton } from '@ui/button';
import { WmInput } from '@ui/input';
import { WmBadge } from '@ui/badge';
import { WmSpinner } from '@ui/spinner';
import { ApiService, SearchResult } from '../../services/api.service';

@Component({
  selector: 'app-search-view',
  standalone: true,
  imports: [FormsModule, RouterLink, WmButton, WmInput, WmBadge, WmSpinner, NgIcon],
  providers: [provideIcons({ lucideSearch })],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="flex flex-col h-full">
      <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
        <h1 class="text-xl sm:text-2xl font-bold">Search</h1>
        @if (!loading && results.length > 0) {
          <span class="text-xs font-medium px-2.5 py-1 rounded-full bg-muted/30 text-muted-foreground">
            {{ results.length }} result{{ results.length === 1 ? '' : 's' }}
          </span>
        }
      </header>
      <div class="flex-1 p-6 max-w-4xl mx-auto overflow-y-auto">
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
              wmInput
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
            wmBtn
            variant="default"
            (click)="doSearch()"
          >
            Search
          </button>
        </div>
        <div class="flex items-center gap-2 flex-wrap">
          <span class="text-xs text-muted-foreground uppercase tracking-wider font-medium">Type</span>
          @for (t of typeOptions; track t.value) {
            <button
              wmBtn
              size="sm"
              [variant]="searchType === t.value ? 'default' : 'outline'"
              (click)="searchType = t.value; doSearch()"
            >
              {{ t.label }}
            </button>
          }
        </div>
      </div>
      @if (loading) {
        <div role="status" aria-live="polite" class="flex items-center gap-2 text-muted-foreground">
          <wm-spinner size="sm" />
          Searching...
        </div>
      }
      @if (error) {
        <div role="alert" class="p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-sm">
          {{ error }}
        </div>
      }
      @if (!loading && !error && results.length > 0) {
        <div role="list" aria-label="Search results" class="space-y-2">
          @for (r of results; track r.id) {
            <a
              role="listitem"
              [routerLink]="['/pages', r.id]"
              class="block rounded-xl border border-border bg-card text-card-foreground shadow-sm p-5 hover:bg-accent/50 transition-colors no-underline cursor-pointer"
            >
              <div class="flex items-center justify-between">
                <span class="font-medium text-primary">{{ r.id }}</span>
                <span class="text-xs text-muted-foreground font-mono">score {{ r.score.toFixed(2) }}</span>
              </div>
              <div class="flex gap-2 mt-1.5">
                <span wmBadge variant="secondary" class="font-medium">{{ r.type }}</span>
                <span wmBadge variant="secondary" class="font-medium">{{ r.page_type }}</span>
              </div>
              <p class="text-sm text-muted-foreground mt-2 leading-relaxed line-clamp-2">{{ r.snippet }}</p>
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
        <div class="p-8 text-center text-muted-foreground">
          <p class="text-lg mb-1">No results found</p>
          <p class="text-sm">Try a different search term or adjust the search type.</p>
        </div>
      }
      </div>
    </div>
  `,
})

export class SearchViewComponent {
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
    this.api.search(this.query, this.searchType).pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
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

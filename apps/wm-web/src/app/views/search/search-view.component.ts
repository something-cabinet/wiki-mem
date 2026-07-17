import { Component, ChangeDetectionStrategy } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { WmButton } from '@ui/button';
import { WmInput } from '@ui/input';
import { WmBadge } from '@ui/badge';
import { WmCard } from '@ui/card';
import { ApiService, SearchResult } from '../../services/api.service';

@Component({
  selector: 'app-search-view',
  standalone: true,
  imports: [FormsModule, RouterLink, WmButton, WmInput, WmBadge, WmCard],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="p-6 max-w-4xl mx-auto">
      <div class="flex items-center justify-between mb-4">
        <h1 class="text-xl sm:text-2xl font-bold">Search</h1>
        @if (!loading && results.length > 0) {
          <span class="text-xs font-medium px-2.5 py-1 rounded-full bg-slate-100 text-slate-600">
            {{ results.length }} result{{ results.length === 1 ? '' : 's' }}
          </span>
        }
      </div>
      <div class="flex flex-col gap-3 mb-4">
        <div class="flex gap-2">
          <div class="relative flex-1">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4 text-gray-400">
                <path stroke-linecap="round" stroke-linejoin="round" d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z" />
              </svg>
            </div>
            <input
              wmInput
              #searchInput
              [(ngModel)]="query"
              (keyup.enter)="doSearch()"
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
        <div class="flex items-center gap-2">
          <span class="text-xs text-gray-400 uppercase tracking-wider font-medium">Type</span>
          @for (t of typeOptions; track t.value) {
            <button
              wmBadge
              [variant]="searchType === t.value ? 'default' : 'secondary'"
              (click)="searchType = t.value; doSearch()"
              class="cursor-pointer"
            >
              {{ t.label }}
            </button>
          }
        </div>
      </div>
      @if (loading) {
        <div role="status" aria-live="polite" class="flex items-center gap-2 text-gray-500">
          <span class="inline-block w-4 h-4 border-2 border-gray-300 border-t-blue-600 rounded-full animate-spin"></span>
          Searching...
        </div>
      }
      @if (error) {
        <div role="alert" class="p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
          {{ error }}
        </div>
      }
      @if (!loading && !error && results.length > 0) {
        <div role="list" aria-label="Search results" class="space-y-2">
          @for (r of results; track r.id) {
            <div
              wmCard
              role="listitem"
              class="cursor-pointer"
              [routerLink]="['/pages', r.id]"
            >
              <div class="flex items-center justify-between">
                <span class="font-medium text-blue-700">{{ r.id }}</span>
                <span class="text-xs text-gray-400 font-mono">score {{ r.score.toFixed(2) }}</span>
              </div>
              <div class="flex gap-2 mt-1.5">
                <span wmBadge variant="secondary" class="font-medium">{{ r.type }}</span>
                <span wmBadge variant="secondary" class="font-medium">{{ r.page_type }}</span>
              </div>
              <p class="text-sm text-gray-600 mt-2 leading-relaxed line-clamp-2">{{ r.snippet }}</p>
            </div>
          }
        </div>
      }
      @if (!loading && !error && query && results.length === 0) {
        <div class="p-8 text-center text-gray-400">
          <p class="text-lg mb-1">No results found</p>
          <p class="text-sm">Try a different search term or adjust the search type.</p>
        </div>
      }
    </div>
  `,
})
export class SearchViewComponent {
  query = '';
  searchType = 'all';
  results: SearchResult[] = [];
  loading = false;
  error = '';
  typeOptions = [
    { value: 'all', label: 'All' },
    { value: 'page', label: 'Pages' },
    { value: 'task', label: 'Tasks' },
    { value: 'memory', label: 'Memory' },
  ];

  constructor(private api: ApiService) {}

  doSearch() {
    if (!this.query.trim()) return;
    this.loading = true;
    this.error = '';
    this.api.search(this.query, this.searchType).subscribe({
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

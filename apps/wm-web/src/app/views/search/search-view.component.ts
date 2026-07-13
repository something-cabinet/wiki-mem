import { Component } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { RouterLink } from '@angular/router';
import { ApiService, SearchResult } from '../../services/api.service';

@Component({
  selector: 'app-search-view',
  standalone: true,
  imports: [FormsModule, RouterLink],
  template: `
    <div class="p-6 max-w-4xl mx-auto">
      <h1 class="text-2xl font-bold mb-4">Search</h1>
      <div class="flex gap-2 mb-4">
        <input
          #searchInput
          [(ngModel)]="query"
          (keyup.enter)="doSearch()"
          placeholder="Search pages, tasks, memory..."
          aria-label="Search query"
          class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <select
          [(ngModel)]="searchType"
          aria-label="Search type"
          class="px-3 py-2 border border-gray-300 rounded-lg"
        >
          <option value="all">All</option>
          <option value="page">Pages</option>
          <option value="task">Tasks</option>
          <option value="memory">Memory</option>
        </select>
        <button
          (click)="doSearch()"
          class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:ring-2 focus:ring-blue-500"
        >
          Search
        </button>
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
            <a
              [routerLink]="['/pages', r.id]"
              role="listitem"
              class="block p-3 bg-white rounded-lg shadow-sm border border-gray-200 hover:border-blue-400 hover:shadow-md transition-all cursor-pointer"
            >
              <div class="flex items-center justify-between">
                <span class="font-medium text-blue-700">{{ r.id }}</span>
                <span class="text-xs text-gray-400">score: {{ r.score.toFixed(2) }}</span>
              </div>
              <div class="flex gap-2 mt-1">
                <span class="text-xs px-2 py-0.5 rounded bg-gray-100">{{ r.type }}</span>
                <span class="text-xs px-2 py-0.5 rounded bg-gray-100">{{ r.page_type }}</span>
              </div>
              <p class="text-sm text-gray-600 mt-1 line-clamp-2">{{ r.snippet }}</p>
            </a>
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

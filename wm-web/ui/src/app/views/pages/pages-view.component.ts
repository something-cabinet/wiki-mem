import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { DatePipe } from '@angular/common';
import { ApiService, Page } from '../../services/api.service';

@Component({
  selector: 'app-pages-view',
  standalone: true,
  imports: [RouterLink, DatePipe],
  template: `
    <div class="p-6 max-w-4xl mx-auto">
      @if (selectedPage) {
        <button (click)="selectedPage = null" class="text-blue-600 mb-4 block">&larr; Back to pages</button>
        <h1 class="text-2xl font-bold mb-2">{{ selectedPage.title || selectedPage.id }}</h1>
        <div class="flex gap-2 mb-4">
          <span class="text-xs px-2 py-0.5 rounded bg-gray-100">{{ selectedPage.type }}</span>
          <span class="text-xs px-2 py-0.5 rounded bg-gray-100">{{ selectedPage.status }}</span>
        </div>
        @if (pageContent) {
          <pre class="p-4 bg-gray-50 rounded-lg border text-sm overflow-auto max-h-96">{{ pageContent }}</pre>
        }
      } @else {
        <h1 class="text-2xl font-bold mb-4">Pages</h1>
        @if (loading) {
          <p class="text-gray-500">Loading pages...</p>
        }
        @if (error) {
          <p class="text-red-500">{{ error }}</p>
        }
        <div class="grid gap-2">
          @for (p of pages; track p.id) {
            <div
              (click)="loadPage(p.id)"
              class="p-3 bg-white rounded-lg shadow-sm border border-gray-200 cursor-pointer hover:border-blue-400 transition-colors"
            >
              <div class="flex items-center justify-between">
                <span class="font-medium">{{ p.title || p.id }}</span>
                <span class="text-xs text-gray-400">{{ p.type }}</span>
              </div>
              <p class="text-xs text-gray-500 mt-1">{{ p.id }}</p>
            </div>
          }
        </div>
      }
    </div>
  `,
})
export class PagesViewComponent implements OnInit {
  pages: Page[] = [];
  selectedPage: Page | null = null;
  pageContent = '';
  loading = true;
  error = '';

  constructor(
    private api: ApiService,
    private route: ActivatedRoute,
  ) {}

  ngOnInit() {
    const pageId = this.route.snapshot.paramMap.get('id');
    if (pageId) {
      this.loadPage(pageId);
    } else {
      this.api.listPages().subscribe({
        next: (res) => {
          this.pages = res.pages || [];
          this.loading = false;
        },
        error: () => {
          this.error = 'Failed to load pages';
          this.loading = false;
        },
      });
    }
  }

  loadPage(id: string) {
    this.api.getPage(id).subscribe({
      next: (res) => {
        if (res.success) {
          this.selectedPage = res.page;
          this.pageContent = res.content || '';
        } else {
          this.error = res.error || 'Page not found';
        }
        this.loading = false;
      },
      error: () => {
        this.error = 'Failed to load page';
        this.loading = false;
      },
    });
  }
}

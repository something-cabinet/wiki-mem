import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, RouterLink } from '@angular/router';
import { DatePipe } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ApiService, Page } from '../../services/api.service';

@Component({
  selector: 'app-pages-view',
  standalone: true,
  imports: [RouterLink, DatePipe, FormsModule],
  template: `
    <div class="p-6 max-w-4xl mx-auto">
      @if (selectedPage) {
        <button (click)="selectedPage = null" class="text-blue-600 mb-4 block flex items-center gap-1 hover:text-blue-800 transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4"><path stroke-linecap="round" stroke-linejoin="round" d="M15.75 19.5 8.25 12l7.5-7.5" /></svg>
          Back to pages
        </button>
        <h1 class="text-2xl font-bold mb-2">{{ selectedPage.title || selectedPage.id }}</h1>
        <div class="flex gap-2 mb-4">
          <span class="text-xs px-2 py-0.5 rounded-full font-medium" [class]="typeBadgeClass(selectedPage.type)">{{ selectedPage.type }}</span>
          <span class="text-xs px-2 py-0.5 rounded-full bg-gray-100 text-gray-600 font-medium">{{ selectedPage.status }}</span>
        </div>
        @if (pageContent) {
          <div class="relative">
            <div class="absolute top-0 right-0 px-2 py-1 text-[10px] text-gray-400 font-mono uppercase tracking-wider">Raw Content</div>
            <pre class="p-4 pt-6 bg-slate-50 rounded-lg border border-gray-200 text-sm overflow-auto max-h-96 font-mono text-slate-700 leading-relaxed">{{ pageContent }}</pre>
          </div>
        }
      } @else {
        <div class="flex items-center justify-between mb-4">
          <h1 class="text-2xl font-bold">Pages</h1>
          <button
            (click)="showCreateModal = true"
            class="flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4"><path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" /></svg>
            Create Page
          </button>
        </div>
        @if (showCreateModal) {
          <div class="fixed inset-0 bg-black/40 flex items-center justify-center z-50" (click)="showCreateModal = false">
            <div class="bg-white rounded-xl shadow-xl p-6 w-full max-w-md mx-4" (click)="$event.stopPropagation()">
              <h2 class="text-lg font-bold mb-4">Create Page</h2>
              <div class="space-y-3">
                <div>
                  <label class="block text-xs font-medium text-gray-500 uppercase tracking-wider mb-1">Path / ID</label>
                  <input [(ngModel)]="newPagePath" placeholder="e.g. projects/my-page" class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm" />
                </div>
                <div>
                  <label class="block text-xs font-medium text-gray-500 uppercase tracking-wider mb-1">Title</label>
                  <input [(ngModel)]="newPageTitle" placeholder="Page title" class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm" />
                </div>
                <div>
                  <label class="block text-xs font-medium text-gray-500 uppercase tracking-wider mb-1">Type</label>
                  <select [(ngModel)]="newPageType" class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm">
                    <option value="">Default</option>
                    <option value="task">Task</option>
                    <option value="concept">Concept</option>
                    <option value="project">Project</option>
                    <option value="note">Note</option>
                  </select>
                </div>
              </div>
              <div class="flex justify-end gap-2 mt-5">
                <button (click)="showCreateModal = false" class="px-3 py-1.5 text-sm text-gray-600 hover:bg-gray-100 rounded-lg transition-colors">Cancel</button>
                <button (click)="createPage()" class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-medium transition-colors">Create</button>
              </div>
            </div>
          </div>
        }
        @if (loading) {
          <div class="flex items-center gap-2 text-gray-500">
            <span class="inline-block w-4 h-4 border-2 border-gray-300 border-t-blue-600 rounded-full animate-spin"></span>
            Loading pages...
          </div>
        }
        @if (error) {
          <p class="text-red-500 text-sm">{{ error }}</p>
        }
        <div class="grid gap-2">
          @for (p of pages; track p.id) {
            <div
              (click)="loadPage(p.id)"
              class="p-3 bg-white rounded-lg shadow-sm border border-gray-200 cursor-pointer hover:shadow-md hover:border-blue-300 transition-all"
            >
              <div class="flex items-center justify-between">
                <span class="font-medium">{{ p.title || p.id }}</span>
                <span class="text-xs px-2 py-0.5 rounded-full font-medium" [class]="typeBadgeClass(p.type)">{{ p.type }}</span>
              </div>
              <p class="text-xs text-gray-500 mt-1 font-mono">{{ p.id }}</p>
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
  showCreateModal = false;
  newPagePath = '';
  newPageTitle = '';
  newPageType = '';

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

  typeBadgeClass(type: string): string {
    const map: Record<string, string> = {
      task: 'bg-blue-50 text-blue-700',
      concept: 'bg-emerald-50 text-emerald-700',
      project: 'bg-violet-50 text-violet-700',
      note: 'bg-amber-50 text-amber-700',
      page: 'bg-slate-50 text-slate-700',
    };
    return map[type] || 'bg-gray-100 text-gray-600';
  }

  createPage() {
    if (!this.newPagePath.trim() || !this.newPageTitle.trim()) return;
    this.api.createPage(this.newPagePath, this.newPageTitle, '', this.newPageType || undefined).subscribe({
      next: (res) => {
        if (res.success) {
          this.showCreateModal = false;
          this.newPagePath = '';
          this.newPageTitle = '';
          this.newPageType = '';
          this.api.listPages().subscribe((r) => {
            this.pages = r.pages || [];
          });
        } else {
          this.error = res.error || 'Failed to create page';
        }
      },
      error: () => {
        this.error = 'Failed to create page';
      },
    });
  }
}

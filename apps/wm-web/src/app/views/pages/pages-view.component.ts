import { Component, OnInit, ChangeDetectionStrategy } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { WmButton } from '@ui/button';
import { WmInput } from '@ui/input';
import { WmDialog } from '@ui/dialog';
import { ApiService, Page } from '../../services/api.service';

@Component({
  selector: 'app-pages-view',
  standalone: true,
  imports: [FormsModule, WmButton, WmInput, WmDialog],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="p-6 max-w-4xl mx-auto">
      @if (selectedPage) {
        <button (click)="selectedPage = null" class="text-blue-600 mb-4 block flex items-center gap-1 hover:text-blue-800 transition-colors">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4"><path stroke-linecap="round" stroke-linejoin="round" d="M15.75 19.5 8.25 12l7.5-7.5" /></svg>
          Back to pages
        </button>
        <h1 class="text-xl sm:text-2xl font-bold mb-2">{{ selectedPage.title || selectedPage.id }}</h1>
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
          <h1 class="text-xl sm:text-2xl font-bold">Pages</h1>
          <button
            wmBtn
            variant="default"
            (click)="showCreateModal = true"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4"><path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" /></svg>
            Create Page
          </button>
        </div>
        <wm-dialog [isOpen]="showCreateModal" (close)="showCreateModal = false">
          <h2 class="text-lg font-bold mb-4">Create Page</h2>
          <div class="space-y-3">
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Path / ID</label>
              <input wmInput [(ngModel)]="newPagePath" placeholder="e.g. projects/my-page" />
            </div>
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Title</label>
              <input wmInput [(ngModel)]="newPageTitle" placeholder="Page title" />
            </div>
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Type</label>
              <select [(ngModel)]="newPageType" class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-ring">
                <option value="">Default</option>
                <option value="task">Task</option>
                <option value="concept">Concept</option>
                <option value="project">Project</option>
                <option value="note">Note</option>
              </select>
            </div>
          </div>
          <div class="flex justify-end gap-2 mt-5">
            <button wmBtn variant="ghost" (click)="showCreateModal = false">Cancel</button>
            <button wmBtn variant="default" (click)="createPage()">Create</button>
          </div>
        </wm-dialog>
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
              class="p-4 bg-card rounded-xl border border-border shadow-sm cursor-pointer hover:shadow-md hover:border-foreground/20 transition-all"
            >
              <div class="flex items-center justify-between">
                <span class="font-medium text-foreground">{{ p.title || p.id }}</span>
                <span class="text-xs px-2 py-0.5 rounded-full font-medium" [class]="typeBadgeClass(p.type)">{{ p.type }}</span>
              </div>
              <p class="text-xs text-muted-foreground mt-1 font-mono">{{ p.id }}</p>
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

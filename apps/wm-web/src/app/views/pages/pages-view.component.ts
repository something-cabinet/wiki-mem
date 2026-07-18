import { Component, OnInit, DestroyRef, ChangeDetectionStrategy } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { NgIcon, provideIcons } from '@ng-icons/core';
import { lucideChevronLeft, lucidePlus, lucidePencil, lucideTrash2 } from '@ng-icons/lucide';
import { WmButton } from '@ui/button';
import { WmInput } from '@ui/input';
import { WmDialog } from '@ui/dialog';
import { WmSelect } from '@ui/select';
import { WmSpinner } from '@ui/spinner';
import { ApiService, Page } from '../../services/api.service';

@Component({
  selector: 'app-pages-view',
  standalone: true,
  imports: [FormsModule, WmButton, WmInput, WmDialog, WmSelect, WmSpinner, NgIcon],
  providers: [provideIcons({ lucideChevronLeft, lucidePlus, lucidePencil, lucideTrash2 })],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="flex flex-col h-full">
      @if (selectedPage) {
        <div class="flex-1 p-6 max-w-4xl mx-auto overflow-y-auto">
          <div class="flex items-center gap-2 mb-4">
            <button wmBtn variant="ghost" size="sm" (click)="selectedPage = null" class="-ml-2">
              <ng-icon name="lucideChevronLeft" size="16" />
              Back to pages
            </button>
            <button wmBtn variant="outline" size="sm" (click)="openEdit()" class="ml-auto">
              <ng-icon name="lucidePencil" size="14" />
              Edit
            </button>
            <button wmBtn variant="destructive" size="sm" (click)="openDeleteConfirm()">
              <ng-icon name="lucideTrash2" size="14" />
              Delete
            </button>
          </div>
          <h1 class="text-xl sm:text-2xl font-bold mb-2">{{ selectedPage.title || selectedPage.id }}</h1>
          <div class="flex gap-2 mb-4">
            <span class="text-xs px-2 py-0.5 rounded-full font-medium" [class]="typeBadgeClass(selectedPage.type)">{{ selectedPage.type }}</span>
            <span class="text-xs px-2 py-0.5 rounded-full bg-muted/50 text-muted-foreground font-medium">{{ selectedPage.status }}</span>
          </div>
          @if (pageContent) {
            <div class="relative">
              <div class="absolute top-0 right-0 px-2 py-1 text-[10px] text-muted-foreground font-mono uppercase tracking-wider">Raw Content</div>
              <pre class="p-4 pt-6 bg-muted/30 rounded-lg border border-border text-sm overflow-auto max-h-96 font-mono text-muted-foreground leading-relaxed">{{ pageContent }}</pre>
            </div>
          }
        </div>
      } @else {
        <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
          <h1 class="text-xl sm:text-2xl font-bold">Pages</h1>
          <button
            wmBtn
            variant="default"
            (click)="showCreateModal = true"
          >
            <ng-icon name="lucidePlus" size="16" />
            Create Page
          </button>
        </header>
        <div class="flex-1 p-6 max-w-4xl mx-auto overflow-y-auto">
        <wm-dialog [isOpen]="showCreateModal" (close)="showCreateModal = false; formSubmitted = false">
          <h2 class="text-lg font-bold mb-4">Create Page</h2>
          <div class="space-y-3">
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Path / ID</label>
              <input wmInput [(ngModel)]="newPagePath" placeholder="e.g. projects/my-page" required />
              @if (formSubmitted && !newPagePath.trim()) {
                <p class="text-xs text-destructive mt-1">Path is required</p>
              }
            </div>
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Title</label>
              <input wmInput [(ngModel)]="newPageTitle" placeholder="Page title" required />
              @if (formSubmitted && !newPageTitle.trim()) {
                <p class="text-xs text-destructive mt-1">Title is required</p>
              }
            </div>
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Content</label>
              <textarea wmInput [(ngModel)]="newPageContent" placeholder="Page body content" rows="4"></textarea>
            </div>
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Type</label>
              <wm-select [value]="newPageType" (valueChange)="newPageType = $event">
                <option value="">Default</option>
                <option value="task">Task</option>
                <option value="concept">Concept</option>
                <option value="project">Project</option>
                <option value="note">Note</option>
              </wm-select>
            </div>
          </div>
          <div class="flex justify-end gap-2 mt-5">
            <button wmBtn variant="ghost" (click)="showCreateModal = false">Cancel</button>
            <button wmBtn variant="default" (click)="createPage()">Create</button>
          </div>
        </wm-dialog>
        @if (loading) {
          <div class="flex items-center gap-2 text-muted-foreground p-6">
            <wm-spinner size="sm" />
            <span class="text-sm">Loading pages...</span>
          </div>
        }
        @if (error) {
          <p class="text-destructive text-sm">{{ error }}</p>
        }
        <wm-dialog [isOpen]="showEditModal" (close)="showEditModal = false">
          <h2 class="text-lg font-bold mb-4">Edit Page</h2>
          <div class="space-y-3">
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Path / ID</label>
              <input wmInput [(ngModel)]="editPath" />
            </div>
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Title</label>
              <input wmInput [(ngModel)]="editTitle" />
            </div>
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Content</label>
              <textarea wmInput [(ngModel)]="editContent" rows="4"></textarea>
            </div>
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Type</label>
              <wm-select [value]="editType" (valueChange)="editType = $event">
                <option value="">Default</option>
                <option value="task">Task</option>
                <option value="concept">Concept</option>
                <option value="project">Project</option>
                <option value="note">Note</option>
              </wm-select>
            </div>
          </div>
          @if (editError) {
            <p class="text-destructive text-sm mt-3">{{ editError }}</p>
          }
          <div class="flex justify-end gap-2 mt-5">
            <button wmBtn variant="ghost" (click)="showEditModal = false" [disabled]="editLoading">Cancel</button>
            <button wmBtn variant="default" (click)="saveEdit()" [disabled]="editLoading">
              @if (editLoading) {
                <wm-spinner size="sm" class="mr-1" />
              }
              Save
            </button>
          </div>
        </wm-dialog>

        <wm-dialog [isOpen]="showDeleteConfirm" (close)="showDeleteConfirm = false">
          <h2 class="text-lg font-bold mb-4">Delete Page</h2>
          <p class="text-sm text-muted-foreground">Are you sure you want to delete this page?</p>
          @if (deleteError) {
            <p class="text-destructive text-sm mt-3">{{ deleteError }}</p>
          }
          <div class="flex justify-end gap-2 mt-5">
            <button wmBtn variant="ghost" (click)="showDeleteConfirm = false" [disabled]="deleteLoading">Cancel</button>
            <button wmBtn variant="destructive" (click)="confirmDelete()" [disabled]="deleteLoading">
              @if (deleteLoading) {
                <wm-spinner size="sm" class="mr-1" />
              }
              Delete
            </button>
          </div>
        </wm-dialog>

        <div class="grid gap-2">
          @for (p of pages; track p.id) {
            <div
              (click)="loadPage(p.id)"
              (keydown.enter)="loadPage(p.id)"
              (keydown.space)="loadPage(p.id); $event.preventDefault()"
              tabindex="0"
              role="button"
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
  formSubmitted = false;
  newPagePath = '';
  newPageTitle = '';
  newPageType = '';
  newPageContent = '';
  showEditModal = false;
  editPath = '';
  editTitle = '';
  editContent = '';
  editType = '';
  editLoading = false;
  editError = '';
  showDeleteConfirm = false;
  deleteLoading = false;
  deleteError = '';

  constructor(
    private api: ApiService,
    private route: ActivatedRoute,
    private destroyRef: DestroyRef,
  ) {}

  ngOnInit() {
    const pageId = this.route.snapshot.paramMap.get('id');
    if (pageId) {
      this.loadPage(pageId);
    } else {
      this.api.listPages().pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
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
    this.api.getPage(id).pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
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
      task: 'bg-primary/10 text-primary',
      concept: 'bg-success/10 text-success',
      project: 'bg-accent/30 text-accent-foreground',
      note: 'bg-secondary/30 text-secondary-foreground',
      page: 'bg-muted/30 text-muted-foreground',
    };
    return map[type] || 'bg-muted/50 text-muted-foreground';
  }

  createPage() {
    this.formSubmitted = true;
    if (!this.newPagePath.trim() || !this.newPageTitle.trim()) return;
    this.api.createPage(this.newPagePath, this.newPageTitle, this.newPageContent, this.newPageType || undefined).pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        if (res.success) {
          this.showCreateModal = false;
          this.formSubmitted = false;
          this.newPagePath = '';
          this.newPageTitle = '';
          this.newPageType = '';
          this.newPageContent = '';
          this.api.listPages().pipe(takeUntilDestroyed(this.destroyRef)).subscribe((r) => {
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

  openEdit() {
    if (!this.selectedPage) return;
    this.editPath = this.selectedPage.id;
    this.editTitle = this.selectedPage.title;
    this.editType = this.selectedPage.type;
    this.editContent = this.pageContent;
    this.editError = '';
    this.editLoading = false;
    this.showEditModal = true;
  }

  saveEdit() {
    if (!this.selectedPage) return;
    this.editLoading = true;
    this.editError = '';
    this.api.updatePage(this.selectedPage.id, {
      title: this.editTitle,
      content: this.editContent,
      type: this.editType,
    }).pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        this.editLoading = false;
        if (res.success) {
          this.showEditModal = false;
          this.loadPage(this.selectedPage!.id);
        } else {
          this.editError = res.error || 'Failed to update page';
        }
      },
      error: () => {
        this.editLoading = false;
        this.editError = 'Failed to update page';
      },
    });
  }

  openDeleteConfirm() {
    this.showDeleteConfirm = true;
    this.deleteError = '';
    this.deleteLoading = false;
  }

  confirmDelete() {
    if (!this.selectedPage) return;
    this.deleteLoading = true;
    this.deleteError = '';
    this.api.deletePage(this.selectedPage.id).pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        this.deleteLoading = false;
        if (res.success) {
          this.showDeleteConfirm = false;
          this.selectedPage = null;
          this.pageContent = '';
          this.api.listPages().pipe(takeUntilDestroyed(this.destroyRef)).subscribe((r) => {
            this.pages = r.pages || [];
          });
        } else {
          this.deleteError = res.error || 'Failed to delete page';
        }
      },
      error: () => {
        this.deleteLoading = false;
        this.deleteError = 'Failed to delete page';
      },
    });
  }
}

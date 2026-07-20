import { Component, OnInit, DestroyRef, ChangeDetectionStrategy } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { NgIcon, provideIcons } from '@ng-icons/core';
import { lucideChevronLeft, lucidePlus, lucidePencil, lucideTrash2 } from '@ng-icons/lucide';
import { HlmBadge } from '@ui/badge';
import { HlmButton } from '@ui/button';
import { HlmInput } from '@ui/input';
import { WmSpinner } from '@ui/spinner';
import { HlmAlert, HlmAlertTitle, HlmAlertDescription } from '@ui/alert';
import { HlmCard } from '@ui/card';
import { BrnDialogImports } from '@spartan-ng/brain/dialog';
import { HlmDialogOverlay, HlmDialogHeader, HlmDialogTitle, HlmDialogFooter } from '@ui/dialog';
import { BrnSelectImports } from '@spartan-ng/brain/select';
import { HlmSelectTrigger, HlmSelectValue, HlmSelectContent, HlmSelectItem } from '@ui/select';
import { ApiService, Page } from '../../services/api.service';

@Component({
  selector: 'app-pages-view',
  standalone: true,
  imports: [
    FormsModule,
    HlmButton,
    HlmInput,
    HlmBadge,
    WmSpinner,
    HlmAlert,
    HlmAlertTitle,
    HlmAlertDescription,
    HlmCard,
    BrnDialogImports,
    HlmDialogOverlay,
    HlmDialogHeader,
    HlmDialogTitle,
    HlmDialogFooter,
    BrnSelectImports,
    HlmSelectTrigger,
    HlmSelectValue,
    HlmSelectContent,
    HlmSelectItem,
    NgIcon,
  ],
  providers: [provideIcons({ lucideChevronLeft, lucidePlus, lucidePencil, lucideTrash2 })],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="flex flex-col h-full">
      @if (selectedPage) {
        <div class="flex-1 p-6 max-w-4xl mx-auto overflow-y-auto">
          <div class="flex items-center gap-2 mb-4">
            <button hlmBtn variant="ghost" size="sm" (click)="selectedPage = null" class="-ml-2">
              <ng-icon name="lucideChevronLeft" size="16" />
              Back to pages
            </button>
            <button hlmBtn variant="outline" size="sm" (click)="openEdit()" class="ml-auto">
              <ng-icon name="lucidePencil" size="14" />
              Edit
            </button>
            <button hlmBtn variant="destructive" size="sm" (click)="openDeleteConfirm()">
              <ng-icon name="lucideTrash2" size="14" />
              Delete
            </button>
          </div>
          <h1 class="text-xl sm:text-2xl font-semibold mb-2">{{ selectedPage.title || selectedPage.id }}</h1>
          <div class="flex gap-2 mb-4">
            <span hlmBadge [variant]="typeBadgeVariant(selectedPage.type)" class="font-medium">{{ selectedPage.type }}</span>
            <span class="text-xs px-2 py-0.5 rounded-full bg-muted/50 text-muted-foreground font-medium">{{ selectedPage.status }}</span>
          </div>
          @if (pageContent) {
            <div class="relative">
              <div class="absolute top-0 right-0 px-2 py-1 text-[10px] text-muted-foreground font-mono uppercase tracking-wider">Content</div>
              <pre class="p-4 pt-6 bg-muted/30 rounded-lg border border-border text-sm overflow-auto max-h-96 font-mono text-muted-foreground leading-relaxed">{{ pageContent }}</pre>
            </div>
          }
        </div>
      } @else {
        <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
          <h1 class="text-xl sm:text-2xl font-semibold">Pages</h1>
          <button
            hlmBtn
            variant="default"
            (click)="showCreateModal = true"
          >
            <ng-icon name="lucidePlus" size="16" />
            Create Page
          </button>
        </header>
        <div class="flex-1 p-6 max-w-4xl mx-auto overflow-y-auto">
        <brn-dialog [state]="showCreateModal ? 'open' : 'closed'" (stateChanged)="showCreateModal = $event === 'open'; formSubmitted = $event === 'open' ? formSubmitted : false">
          @if (showCreateModal) {
            <div brnDialogOverlay hlmDialogOverlay (click)="showCreateModal = false"></div>
          }
          <div *brnDialogContent="let ctx" hlmDialogContent>
            <div hlmDialogHeader>
              <h3 hlmDialogTitle>Create Page</h3>
            </div>
            <div class="space-y-3">
              <div>
                <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Path / ID</label>
                <input hlmInput [(ngModel)]="newPagePath" placeholder="e.g. projects/my-page" required />
                @if (formSubmitted && !newPagePath.trim()) {
                  <p class="text-xs text-destructive mt-1">Path is required</p>
                }
              </div>
              <div>
                <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Title</label>
                <input hlmInput [(ngModel)]="newPageTitle" placeholder="Page title" required />
                @if (formSubmitted && !newPageTitle.trim()) {
                  <p class="text-xs text-destructive mt-1">Title is required</p>
                }
              </div>
              <div>
                <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Content</label>
                <textarea hlmInput [(ngModel)]="newPageContent" placeholder="Page body content" rows="4"></textarea>
              </div>
              <div>
                <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Type</label>
                <div brnSelect [value]="newPageType" (valueChange)="newPageType = $event ?? ''" class="w-full">
                  <hlm-select-trigger>
                    <hlm-select-value />
                  </hlm-select-trigger>
                  <hlm-select-content>
                    <hlm-select-item value="">Default</hlm-select-item>
                    <hlm-select-item value="task">Task</hlm-select-item>
                    <hlm-select-item value="concept">Concept</hlm-select-item>
                    <hlm-select-item value="project">Project</hlm-select-item>
                    <hlm-select-item value="note">Note</hlm-select-item>
                  </hlm-select-content>
                </div>
              </div>
            </div>
            <div hlmDialogFooter class="flex justify-end gap-2">
              <button hlmBtn variant="ghost" (click)="showCreateModal = false">Cancel</button>
              <button hlmBtn variant="default" (click)="createPage()">Create</button>
            </div>
          </div>
        </brn-dialog>
        @if (loading) {
          <div class="flex items-center justify-center gap-2 text-muted-foreground py-16">
            <wm-spinner size="sm" />
            <span class="text-sm">Loading pages...</span>
          </div>
        }
        @if (error) {
          <div hlmAlert variant="destructive">
            <span hlmAlertTitle>Error</span>
            <p hlmAlertDescription>{{ error }}</p>
          </div>
        }
        <brn-dialog [state]="showEditModal ? 'open' : 'closed'" (stateChanged)="showEditModal = $event === 'open'">
          @if (showEditModal) {
            <div brnDialogOverlay hlmDialogOverlay (click)="showEditModal = false"></div>
          }
          <div *brnDialogContent="let ctx" hlmDialogContent>
            <div hlmDialogHeader>
              <h3 hlmDialogTitle>Edit Page</h3>
            </div>
            <div class="space-y-3">
              <div>
                <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Path / ID</label>
                <input hlmInput [(ngModel)]="editPath" />
              </div>
              <div>
                <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Title</label>
                <input hlmInput [(ngModel)]="editTitle" />
              </div>
              <div>
                <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Content</label>
                <textarea hlmInput [(ngModel)]="editContent" rows="4"></textarea>
              </div>
              <div>
                <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Type</label>
                <div brnSelect [value]="editType" (valueChange)="editType = $event ?? ''" class="w-full">
                  <hlm-select-trigger>
                    <hlm-select-value />
                  </hlm-select-trigger>
                  <hlm-select-content>
                    <hlm-select-item value="">Default</hlm-select-item>
                    <hlm-select-item value="task">Task</hlm-select-item>
                    <hlm-select-item value="concept">Concept</hlm-select-item>
                    <hlm-select-item value="project">Project</hlm-select-item>
                    <hlm-select-item value="note">Note</hlm-select-item>
                  </hlm-select-content>
                </div>
              </div>
            </div>
            @if (editError) {
              <div hlmAlert variant="destructive">
                <span hlmAlertTitle>Error</span>
                <p hlmAlertDescription>{{ editError }}</p>
              </div>
            }
            <div hlmDialogFooter class="flex justify-end gap-2">
              <button hlmBtn variant="ghost" (click)="showEditModal = false" [disabled]="editLoading">Cancel</button>
              <button hlmBtn variant="default" (click)="saveEdit()" [disabled]="editLoading">
                @if (editLoading) {
                  <wm-spinner size="sm" class="mr-1" />
                }
                Save
              </button>
            </div>
          </div>
        </brn-dialog>

        <brn-dialog [state]="showDeleteConfirm ? 'open' : 'closed'" (stateChanged)="showDeleteConfirm = $event === 'open'">
          @if (showDeleteConfirm) {
            <div brnDialogOverlay hlmDialogOverlay (click)="showDeleteConfirm = false"></div>
          }
          <div *brnDialogContent="let ctx" hlmDialogContent>
            <div hlmDialogHeader>
              <h3 hlmDialogTitle>Delete Page</h3>
            </div>
            <p class="text-sm text-muted-foreground">Are you sure you want to delete this page?</p>
            @if (deleteError) {
              <div hlmAlert variant="destructive">
                <span hlmAlertTitle>Error</span>
                <p hlmAlertDescription>{{ deleteError }}</p>
              </div>
            }
            <div hlmDialogFooter class="flex justify-end gap-2">
              <button hlmBtn variant="ghost" (click)="showDeleteConfirm = false" [disabled]="deleteLoading">Cancel</button>
              <button hlmBtn variant="destructive" (click)="confirmDelete()" [disabled]="deleteLoading">
                @if (deleteLoading) {
                  <wm-spinner size="sm" class="mr-1" />
                }
                Delete
              </button>
            </div>
          </div>
        </brn-dialog>

        <div class="grid gap-2">
          @for (p of pages; track p.id) {
            <button
              hlmCard
              type="button"
              (click)="loadPage(p.id)"
              class="w-full text-left p-4 cursor-pointer hover:shadow-md hover:border-foreground/20 transition-all"
            >
              <div class="flex items-center justify-between">
                <span class="font-medium text-foreground">{{ p.title || p.id }}</span>
                <span hlmBadge [variant]="typeBadgeVariant(p.type)" class="font-medium">{{ p.type }}</span>
              </div>
              <p class="text-xs text-muted-foreground mt-1 font-mono">{{ p.id }}</p>
            </button>
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
          this.selectedPage = { id, title: res.title, type: res.type, status: res.status };
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

  typeBadgeVariant(type: string): 'default' | 'outline' | 'secondary' {
    const map: Record<string, 'default' | 'outline' | 'secondary'> = {
      task: 'default',
      concept: 'secondary',
      project: 'secondary',
      note: 'outline',
      page: 'secondary',
    };
    return map[type] ?? 'secondary';
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









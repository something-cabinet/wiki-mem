import { Component, OnInit, DestroyRef, ChangeDetectionStrategy, inject } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { NgIcon, provideIcons } from '@ng-icons/core';
import { lucidePlus, lucidePencil, lucideTrash2, lucideFileText } from '@ng-icons/lucide';
import { toast } from 'ngx-sonner';
import { BackButtonComponent } from '../../components/back-button/back-button.component';
import { PageDialogsComponent } from '../../components/page-dialogs/page-dialogs.component';
import { HlmBadge } from '@ui/badge';
import { HlmButton } from '@ui/button';
import { HlmInput } from '@ui/input';
import { WmSpinner } from '@ui/spinner';
import { HlmAlert, HlmAlertTitle, HlmAlertDescription } from '@ui/alert';
import { HlmCard } from '@ui/card';
import { BrnDialogImports } from '@spartan-ng/brain/dialog';
import { HlmDialogOverlay, HlmDialogHeader, HlmDialogTitle, HlmDialogFooter } from '@ui/dialog';
import { HlmSelect, HlmSelectTrigger, HlmSelectValue, HlmSelectContent, HlmSelectItem, HlmSelectPortal } from '@ui/select';
import { ApiService, Page } from '../../services/api.service';
import { pageTypeBadgeClass } from '@ui/graph';

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
    HlmSelect,
    HlmSelectTrigger,
    HlmSelectValue,
    HlmSelectContent,
    HlmSelectPortal,
    HlmSelectItem,
    NgIcon,
    BackButtonComponent,
    PageDialogsComponent,
  ],
  providers: [provideIcons({ lucidePlus, lucidePencil, lucideTrash2, lucideFileText })],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="flex flex-col h-full">
      @if (selectedPage) {
        <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
          <div class="flex items-center gap-2">
            <app-back-button />
            <h1 class="text-xl sm:text-2xl font-semibold truncate max-w-sm">{{ selectedPage.title || selectedPage.id }}</h1>
          </div>
          <div class="flex items-center gap-2">
            <span hlmBadge variant="secondary" [class]="pageTypeBadgeClass(selectedPage.type)" class="font-medium">{{ selectedPage.type }}</span>
            <span hlmBadge variant="outline">{{ selectedPage.status }}</span>
            <button hlmBtn variant="outline" size="sm" (click)="openEdit()">
              <ng-icon name="lucidePencil" size="14" />
              Edit
            </button>
            <button hlmBtn variant="destructive" size="sm" (click)="openDeleteConfirm()">
              <ng-icon name="lucideTrash2" size="14" />
              Delete
            </button>
          </div>
        </header>
        <div class="flex-1 p-6 max-w-4xl mx-auto overflow-y-auto">
          @if (pageContent) {
            <div class="relative">
              <div class="absolute top-0 right-0 px-2 py-1 text-[10px] text-muted-foreground font-mono uppercase tracking-wider">Content</div>
              <pre class="p-4 pt-6 bg-muted/30 rounded-lg border border-border text-sm overflow-auto max-h-96 font-mono text-muted-foreground leading-relaxed whitespace-pre-wrap break-words">{{ pageContent }}</pre>
            </div>
          }
        </div>
      } @else {
        <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
          <h1 class="text-xl sm:text-2xl font-semibold">Pages</h1>
          <button hlmBtn variant="default" (click)="showCreateModal = true">
            <ng-icon name="lucidePlus" size="16" />
            Create Page
          </button>
        </header>
        <div class="flex-1 overflow-y-auto">
        <div class="p-6 max-w-4xl mx-auto w-full">
        <brn-dialog [state]="showCreateModal ? 'open' : 'closed'" (stateChanged)="showCreateModal = $event === 'open'; formSubmitted = $event === 'open' ? formSubmitted : false">
          @if (showCreateModal) {
            <div brnDialogOverlay hlmDialogOverlay (click)="showCreateModal = false"></div>
          }
          <hlm-dialog-content *brnDialogContent>
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
                <div hlmSelect [value]="newPageType" (valueChange)="newPageType = $event ?? ''" class="w-full">
                  <hlm-select-trigger>
                    <hlm-select-value />
                  </hlm-select-trigger>
                  <hlm-select-content *hlmSelectPortal>
                    <hlm-select-item value="">Default</hlm-select-item>
                    <hlm-select-item value="task">Task</hlm-select-item>
                    <hlm-select-item value="concept">Concept</hlm-select-item>
                    <hlm-select-item value="spec">Spec</hlm-select-item>
                    <hlm-select-item value="pattern">Pattern</hlm-select-item>
                    <hlm-select-item value="decision">Decision</hlm-select-item>
                    <hlm-select-item value="howto">How-to</hlm-select-item>
                    <hlm-select-item value="reference">Reference</hlm-select-item>
                    <hlm-select-item value="memory">Memory</hlm-select-item>
                  </hlm-select-content>
                </div>
              </div>
            </div>
            <div hlmDialogFooter class="flex justify-end gap-2">
              <button hlmBtn variant="ghost" (click)="showCreateModal = false">Cancel</button>
              <button hlmBtn variant="default" (click)="createPage()">Create</button>
            </div>
          </hlm-dialog-content>
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
        @if (pages.length === 0 && !loading) {
          <div class="flex flex-col items-center justify-center py-16 text-muted-foreground">
            <ng-icon name="lucideFileText" size="32" class="text-muted-foreground/30" />
            <p class="text-lg font-medium mt-4">No pages yet</p>
            <p class="text-xs text-muted-foreground/60 mt-1">Create a page to start building your wiki.</p>
          </div>
        }
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
                <span hlmBadge variant="secondary" [class]="pageTypeBadgeClass(p.type)" class="font-medium">{{ p.type }}</span>
              </div>
              <p class="text-xs text-muted-foreground mt-1 font-mono">{{ p.id }}</p>
            </button>
          }
          </div>
          </div>
        </div>
      }
      <app-page-dialogs
        [data]="dialogData"
        [showEdit]="showEditModal"
        [showDelete]="showDeleteConfirm"
        [deleteError]="deleteError"
        (showEditChange)="showEditModal = $event"
        (showDeleteChange)="showDeleteConfirm = $event"
        (save)="onDialogSave($event)"
        (confirmDelete)="confirmDelete()"
      />
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
  editLoading = false;
  editError = '';
  showDeleteConfirm = false;
  deleteLoading = false;
  deleteError = '';

  get dialogData() {
    return this.selectedPage ? { id: this.selectedPage.id, title: this.selectedPage.title, type: this.selectedPage.type, content: this.pageContent } : null;
  }

  onDialogSave(data: { id: string; title: string; content: string; type: string }) {
    this.editLoading = true;
    this.editError = '';
    this.api.updatePage(data.id, {
      title: data.title,
      content: data.content,
      type: data.type,
    }).pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        this.editLoading = false;
        if (res.success) {
          this.showEditModal = false;
          toast.success('Page updated');
          this.loadPage(data.id);
        } else {
          this.editError = res.error || 'Failed to update page';
          toast.error(res.error || 'Failed to update page');
        }
      },
      error: () => {
        this.editLoading = false;
        this.editError = 'Failed to update page';
        toast.error('Failed to update page');
      },
    });
  }

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
      task: 'secondary',
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
          toast.success('Page created');
          this.api.listPages().pipe(takeUntilDestroyed(this.destroyRef)).subscribe((r) => {
            this.pages = r.pages || [];
          });
        } else {
          this.error = res.error || 'Failed to create page';
          toast.error(res.error || 'Failed to create page');
        }
      },
      error: () => {
        this.error = 'Failed to create page';
        toast.error('Failed to create page');
      },
    });
  }

  openEdit() {
    this.showEditModal = true;
  }

  openDeleteConfirm() {
    if (!this.selectedPage) return;
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
          toast.success('Page deleted');
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









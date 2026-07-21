import { Component, OnInit, ChangeDetectionStrategy, DestroyRef, inject } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { NgIcon, provideIcons } from '@ng-icons/core';
import { lucidePencil, lucidePlus, lucideTrash2, lucideBrain } from '@ng-icons/lucide';
import { toast } from 'ngx-sonner';
import { ApiService, MemoryEntry } from '../../services/api.service';
import { HlmButton } from '@ui/button';
import { HlmInput } from '@ui/input';
import { HlmCard } from '@ui/card';
import { HlmBadge } from '@ui/badge';
import { WmSpinner } from '@ui/spinner';
import { HlmAlert, HlmAlertDescription } from '@ui/alert';
import { BrnDialogImports } from '@spartan-ng/brain/dialog';
import { HlmDialogOverlay, HlmDialogHeader, HlmDialogTitle, HlmDialogFooter } from '@ui/dialog';
import { HlmSelect, HlmSelectTrigger, HlmSelectValue, HlmSelectContent, HlmSelectPortal, HlmSelectItem } from '@ui/select';

@Component({
  selector: 'app-memory-view',
  standalone: true,
  imports: [
    FormsModule,
    HlmButton,
    HlmInput,
    HlmCard,
    HlmBadge,
    WmSpinner,
    HlmAlert,
    HlmAlertDescription,
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
  ],
  providers: [provideIcons({ lucidePlus, lucidePencil, lucideTrash2, lucideBrain })],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="flex flex-col h-full">
      <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
        <h1 class="text-xl sm:text-2xl font-semibold">Memory</h1>
        <div class="flex items-center gap-2 flex-wrap">
          <div hlmSelect [value]="selectedLayer" (valueChange)="selectedLayer = $event ?? ''; loadMemory()" class="inline-block">
            <hlm-select-trigger class="w-44">
              <hlm-select-value />
            </hlm-select-trigger>
            <hlm-select-content *hlmSelectPortal>
              <hlm-select-item value="project">Project Memory</hlm-select-item>
              <hlm-select-item value="session">Session Memory</hlm-select-item>
            </hlm-select-content>
          </div>
          <div hlmSelect [value]="selectedStatus" (valueChange)="selectedStatus = $event ?? ''; loadMemory()" class="inline-block">
            <hlm-select-trigger class="w-44">
              <hlm-select-value />
            </hlm-select-trigger>
            <hlm-select-content *hlmSelectPortal>
              <hlm-select-item value="">All Statuses</hlm-select-item>
              <hlm-select-item value="active">Active</hlm-select-item>
              <hlm-select-item value="stale">Stale</hlm-select-item>
              <hlm-select-item value="archived">Archived</hlm-select-item>
            </hlm-select-content>
          </div>
          <button hlmBtn variant="default" (click)="showForm = true" class="flex items-center gap-1.5">
            <ng-icon name="lucidePlus" size="16" />
            New
          </button>
        </div>
      </header>
      <div class="flex-1 overflow-y-auto">
      <div class="p-6 max-w-4xl mx-auto w-full">

      <brn-dialog [state]="showForm ? 'open' : 'closed'" (stateChanged)="showForm = $event === 'open'; formSubmitted = $event !== 'open' ? false : formSubmitted">
        @if (showForm) {
          <div brnDialogOverlay hlmDialogOverlay (click)="showForm = false"></div>
        }
        <hlm-dialog-content *brnDialogContent>
          <div hlmDialogHeader>
            <h3 hlmDialogTitle>New Memory Entry</h3>
          </div>
          <div class="space-y-3">
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Title</label>
              <input hlmInput [(ngModel)]="newTitle" placeholder="Entry title" required />
              @if (formSubmitted && !newTitle.trim()) {
                <p class="text-xs text-destructive mt-1">Title is required</p>
              }
            </div>
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Content</label>
              <textarea hlmInput [(ngModel)]="newContent" placeholder="What do you want to remember?" rows="4" class="resize-none" required></textarea>
              @if (formSubmitted && !newContent.trim()) {
                <p class="text-xs text-destructive mt-1">Content is required</p>
              }
            </div>
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Tags (comma separated)</label>
              <input hlmInput [(ngModel)]="newTags" placeholder="tag1, tag2" />
            </div>
          </div>
          <div hlmDialogFooter class="flex justify-end gap-2">
            <button hlmBtn variant="ghost" (click)="showForm = false">Cancel</button>
            <button hlmBtn variant="default" (click)="createEntry()">Save</button>
          </div>
        </hlm-dialog-content>
      </brn-dialog>

      <brn-dialog [state]="editEntry !== null ? 'open' : 'closed'" (stateChanged)="editEntry = $event === 'open' ? editEntry : null">
        @if (editEntry !== null) {
          <div brnDialogOverlay hlmDialogOverlay (click)="editEntry = null"></div>
        }
        <hlm-dialog-content *brnDialogContent>
          <div hlmDialogHeader>
            <h3 hlmDialogTitle>Edit Memory Entry</h3>
          </div>
          <div class="space-y-3">
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Title</label>
              <input hlmInput [(ngModel)]="editTitle" placeholder="Entry title" />
              @if (editFormSubmitted && !editTitle.trim()) {
                <p class="text-xs text-destructive mt-1">Title is required</p>
              }
            </div>
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Content</label>
              <textarea hlmInput [(ngModel)]="editContent" placeholder="What do you want to remember?" rows="4" class="resize-none"></textarea>
              @if (editFormSubmitted && !editContent.trim()) {
                <p class="text-xs text-destructive mt-1">Content is required</p>
              }
            </div>
            <div>
              <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Tags (comma separated)</label>
              <input hlmInput [(ngModel)]="editTags" placeholder="tag1, tag2" />
            </div>
          </div>
          <div hlmDialogFooter class="flex justify-end gap-2">
            <button hlmBtn variant="ghost" (click)="editEntry = null">Cancel</button>
            <button hlmBtn variant="default" (click)="updateEntry()">Save</button>
          </div>
        </hlm-dialog-content>
      </brn-dialog>

      <brn-dialog [state]="showDeleteConfirm ? 'open' : 'closed'" (stateChanged)="showDeleteConfirm = $event === 'open'">
        @if (showDeleteConfirm) {
          <div brnDialogOverlay hlmDialogOverlay (click)="showDeleteConfirm = false"></div>
        }
        <hlm-dialog-content *brnDialogContent>
          <div hlmDialogHeader>
            <h3 hlmDialogTitle>Delete Memory Entry</h3>
          </div>
          <p class="text-muted-foreground">
            Are you sure you want to delete <strong>{{ deleteTarget?.title || deleteTarget?.id }}</strong>?
          </p>
          @if (deleteError) {
            <div hlmAlert variant="destructive" class="text-sm">
              <p hlmAlertDescription>{{ deleteError }}</p>
            </div>
          }
          <div hlmDialogFooter class="flex justify-end gap-2">
            <button hlmBtn variant="ghost" (click)="showDeleteConfirm = false">Cancel</button>
            <button hlmBtn variant="destructive" (click)="confirmDelete()">Delete</button>
          </div>
        </hlm-dialog-content>
      </brn-dialog>

      @if (loading) {
        <div class="flex items-center gap-2 text-muted-foreground p-6">
          <wm-spinner size="sm" />
          <span class="text-sm">Loading memory entries...</span>
        </div>
      }
      @if (error) {
        <div hlmAlert variant="destructive" class="text-sm">
          <p hlmAlertDescription>{{ error }}</p>
        </div>
      }
      @if (entries.length > 0) {
        <div class="space-y-2" role="list">
          @for (e of entries; track e.id) {
            <div hlmCard class="p-4 transition-shadow" role="listitem">
              <div class="flex items-center justify-between">
                <span class="font-medium">{{ e.title || e.id }}</span>
                <div class="flex items-center gap-1">
                  <span class="text-xs text-muted-foreground font-mono">{{ e.created_at.substring(0, 10) }}</span>
                  <button hlmBtn variant="ghost" size="sm" (click)="startEdit(e)" class="text-muted-foreground hover:text-foreground" aria-label="Edit entry">
                    <ng-icon name="lucidePencil" size="14" />
                  </button>
                  <button hlmBtn variant="ghost" size="sm" (click)="startDelete(e)" class="text-muted-foreground hover:text-destructive" aria-label="Delete entry">
                    <ng-icon name="lucideTrash2" size="14" />
                  </button>
                </div>
              </div>
              @if (e.tags.length > 0) {
                <div class="flex flex-wrap gap-1.5 mt-2">
                  @for (tag of e.tags; track tag) {
                    <span hlmBadge variant="secondary">{{ tag }}</span>
                  }
                </div>
              }
              <div class="mt-2">
                @if (expanded[e.id]) {
                  <p class="text-sm text-muted-foreground leading-relaxed">{{ e.content }}</p>
                } @else {
                  <p class="text-sm text-muted-foreground leading-relaxed line-clamp-3">{{ e.content }}</p>
                }
                @if (e.content.length > 240) {
                  <button
                    hlmBtn
                    variant="link"
                    size="xs"
                    (click)="expanded[e.id] = !expanded[e.id]"
                    [attr.aria-expanded]="expanded[e.id]"
                    class="mt-1.5"
                  >
                    {{ expanded[e.id] ? 'Show less' : 'Show more' }}
                  </button>
                }
              </div>
            </div>
          }
        </div>
      }
      @if (!loading && !error && entries.length === 0) {
        <div class="flex flex-col items-center justify-center py-16 text-muted-foreground">
          <ng-icon name="lucideBrain" size="32" class="text-muted-foreground/30" />
          <p class="text-lg font-medium mt-4">No memory entries</p>
          <p class="text-xs text-muted-foreground/60 mt-1">Save project knowledge with the New button above.</p>
        </div>
      }
      </div>
      </div>
    </div>
  `,
})
export class MemoryViewComponent implements OnInit {
  selectedLayer = 'project';
  selectedStatus = '';
  entries: MemoryEntry[] = [];
  loading = true;
  error = '';
  showForm = false;
  formSubmitted = false;
  newTitle = '';
  newContent = '';
  newTags = '';
  expanded: Record<string, boolean> = {};
  editEntry: MemoryEntry | null = null;
  editTitle = '';
  editContent = '';
  editTags = '';
  editFormSubmitted = false;
  showDeleteConfirm = false;
  deleteTarget: MemoryEntry | null = null;
  deleteError = '';

  private destroyRef = inject(DestroyRef);

  constructor(private api: ApiService) {}

  ngOnInit() {
    this.loadMemory();
  }

  loadMemory() {
    this.loading = true;
    this.error = '';
    this.api.listMemory(this.selectedLayer, this.selectedStatus).pipe(
      takeUntilDestroyed(this.destroyRef),
    ).subscribe({
      next: (res) => {
        this.entries = res.entries || [];
        this.loading = false;
      },
      error: () => {
        this.error = 'Failed to load memory entries';
        this.loading = false;
      },
    });
  }


  createEntry() {
    this.formSubmitted = true;
    if (!this.newTitle.trim() || !this.newContent.trim()) return;
    const slug = this.newTitle.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
    const tags = this.newTags.split(',').map(t => t.trim()).filter(t => t.length > 0);
    this.api.createPage(`memory/${slug}`, this.newTitle, this.newContent, 'memory', tags).pipe(
      takeUntilDestroyed(this.destroyRef),
    ).subscribe({
      next: () => { this.loadMemory(); this.showForm = false; this.formSubmitted = false; this.newTitle = ''; this.newContent = ''; this.newTags = ''; this.error = ''; toast.success('Memory created'); },
      error: () => { this.error = 'Failed to create memory'; toast.error('Failed to create memory'); }
    });
  }

  startEdit(entry: MemoryEntry) {
    this.editEntry = entry;
    this.editTitle = entry.title;
    this.editContent = entry.content;
    this.editTags = entry.tags.join(', ');
    this.editFormSubmitted = false;
    this.error = '';
  }

  updateEntry() {
    if (!this.editEntry) return;
    this.editFormSubmitted = true;
    if (!this.editTitle.trim() || !this.editContent.trim()) return;
    const tags = this.editTags.split(',').map(t => t.trim()).filter(t => t.length > 0).join(', ');
    this.api.updatePage(this.editEntry.id, { title: this.editTitle, content: this.editContent, tags }).pipe(
      takeUntilDestroyed(this.destroyRef),
    ).subscribe({
      next: () => { this.loadMemory(); this.editEntry = null; this.error = ''; toast.success('Memory updated'); },
      error: () => { this.error = 'Failed to update memory'; toast.error('Failed to update memory'); }
    });
  }

  startDelete(entry: MemoryEntry) {
    this.deleteTarget = entry;
    this.showDeleteConfirm = true;
    this.deleteError = '';
  }

  confirmDelete() {
    if (!this.deleteTarget) return;
    this.api.deletePage(this.deleteTarget.id).pipe(
      takeUntilDestroyed(this.destroyRef),
    ).subscribe({
      next: () => { this.loadMemory(); this.showDeleteConfirm = false; this.deleteTarget = null; this.error = ''; toast.success('Memory deleted'); },
      error: (err) => { this.deleteError = 'Failed to delete memory'; toast.error('Failed to delete memory'); }
    });
  }
}






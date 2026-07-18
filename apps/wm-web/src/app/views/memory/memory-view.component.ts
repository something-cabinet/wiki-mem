import { Component, OnInit, ChangeDetectionStrategy, DestroyRef, inject } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { NgIcon, provideIcons } from '@ng-icons/core';
import { lucidePencil, lucidePlus, lucideTrash2 } from '@ng-icons/lucide';
import { ApiService, MemoryEntry } from '../../services/api.service';
import { WmButton } from '@ui/button';
import { WmInput } from '@ui/input';
import { WmCard } from '@ui/card';
import { WmBadge } from '@ui/badge';
import { WmDialog } from '@ui/dialog';
import { WmSelect } from '@ui/select';
import { WmSpinner } from '@ui/spinner';

@Component({
  selector: 'app-memory-view',
  standalone: true,
  imports: [FormsModule, NgIcon, WmButton, WmInput, WmCard, WmBadge, WmDialog, WmSelect, WmSpinner],
  providers: [provideIcons({ lucidePlus, lucidePencil, lucideTrash2 })],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="flex flex-col h-full">
      <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
        <h1 class="text-xl sm:text-2xl font-bold">Memory</h1>
        <div class="flex items-center gap-2 flex-wrap">
          <wm-select [value]="selectedLayer" (valueChange)="selectedLayer = $event; loadMemory()">
            <option value="project">Project Memory</option>
            <option value="session">Session Memory</option>
          </wm-select>
          <wm-select [value]="selectedStatus" (valueChange)="selectedStatus = $event; loadMemory()">
            <option value="">All Statuses</option>
            <option value="active">Active</option>
            <option value="stale">Stale</option>
            <option value="archived">Archived</option>
          </wm-select>
          <button wmBtn variant="default" (click)="showForm = true" class="flex items-center gap-1.5">
            <ng-icon name="lucidePlus" size="16" />
            New
          </button>
        </div>
      </header>
      <div class="flex-1 p-6 max-w-4xl mx-auto overflow-y-auto">

      <wm-dialog [isOpen]="showForm" (close)="showForm = false; formSubmitted = false">
        <h2 class="text-lg font-bold mb-4">New Memory Entry</h2>
        <div class="space-y-3">
          <div>
            <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Title</label>
            <input wmInput [(ngModel)]="newTitle" placeholder="Entry title" required />
            @if (formSubmitted && !newTitle.trim()) {
              <p class="text-xs text-destructive mt-1">Title is required</p>
            }
          </div>
          <div>
            <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Content</label>
            <textarea wmInput [(ngModel)]="newContent" placeholder="What do you want to remember?" rows="4" class="resize-none" required></textarea>
            @if (formSubmitted && !newContent.trim()) {
              <p class="text-xs text-destructive mt-1">Content is required</p>
            }
          </div>
          <div>
            <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Tags (comma separated)</label>
            <input wmInput [(ngModel)]="newTags" placeholder="tag1, tag2" />
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-5">
          <button wmBtn variant="ghost" (click)="showForm = false">Cancel</button>
          <button wmBtn variant="default" (click)="createEntry()">Save</button>
        </div>
      </wm-dialog>

      <wm-dialog [isOpen]="editEntry !== null" (close)="editEntry = null">
        <h2 class="text-lg font-bold mb-4">Edit Memory Entry</h2>
        <div class="space-y-3">
          <div>
            <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Title</label>
            <input wmInput [(ngModel)]="editTitle" placeholder="Entry title" />
          </div>
          <div>
            <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Content</label>
            <textarea wmInput [(ngModel)]="editContent" placeholder="What do you want to remember?" rows="4" class="resize-none"></textarea>
          </div>
          <div>
            <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Tags (comma separated)</label>
            <input wmInput [(ngModel)]="editTags" placeholder="tag1, tag2" />
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-5">
          <button wmBtn variant="ghost" (click)="editEntry = null">Cancel</button>
          <button wmBtn variant="default" (click)="updateEntry()">Save</button>
        </div>
      </wm-dialog>

      <wm-dialog [isOpen]="showDeleteConfirm" (close)="showDeleteConfirm = false">
        <h2 class="text-lg font-bold mb-4">Delete Memory Entry</h2>
        <p class="text-muted-foreground">
          Are you sure you want to delete <strong>{{ deleteTarget?.title || deleteTarget?.id }}</strong>?
        </p>
        @if (deleteError) {
          <p class="text-destructive text-sm mt-2">{{ deleteError }}</p>
        }
        <div class="flex justify-end gap-2 mt-5">
          <button wmBtn variant="ghost" (click)="showDeleteConfirm = false">Cancel</button>
          <button wmBtn variant="destructive" (click)="confirmDelete()">Delete</button>
        </div>
      </wm-dialog>

      @if (loading) {
        <div class="flex items-center gap-2 text-muted-foreground p-6">
          <wm-spinner size="sm" />
          <span class="text-sm">Loading memory entries...</span>
        </div>
      }
      @if (error) {
        <p class="text-destructive text-sm">{{ error }}</p>
      }
      @if (entries.length > 0) {
        <div class="space-y-2">
          @for (e of entries; track e.id) {
            <div wmCard class="p-4 hover:shadow-md transition-shadow">
              <div class="flex items-center justify-between">
                <span class="font-medium">{{ e.title || e.id }}</span>
                <div class="flex items-center gap-1">
                  <span class="text-xs text-muted-foreground font-mono">{{ e.created_at.substring(0, 10) }}</span>
                  <button wmBtn variant="ghost" size="sm" (click)="startEdit(e)" class="text-muted-foreground hover:text-foreground">
                    <ng-icon name="lucidePencil" size="14" />
                  </button>
                  <button wmBtn variant="ghost" size="sm" (click)="startDelete(e)" class="text-muted-foreground hover:text-red-500">
                    <ng-icon name="lucideTrash2" size="14" />
                  </button>
                </div>
              </div>
              @if (e.tags.length > 0) {
                <div class="flex flex-wrap gap-1.5 mt-2">
                  @for (tag of e.tags; track tag) {
                    <span wmBadge variant="secondary">{{ tag }}</span>
                  }
                </div>
              }
              <div class="mt-2">
                @if (expanded[e.id]) {
                  <p class="text-sm text-muted-foreground leading-relaxed">{{ e.content }}</p>
                } @else {
                  <p class="text-sm text-muted-foreground leading-relaxed line-clamp-3">{{ e.content }}</p>
                }
                @if (e.content.length > 180) {
                  <button
                    (click)="expanded[e.id] = !expanded[e.id]"
                    class="mt-1.5 text-xs text-primary hover:text-primary font-medium transition-colors"
                  >
                    {{ expanded[e.id] ? 'Show less' : 'Show more' }}
                  </button>
                }
              </div>
            </div>
          }
        </div>
      }
      @if (!loading && entries.length === 0) {
        <p class="text-muted-foreground text-center py-8">No memory entries found.</p>
      }
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

  tagColorClass(tag: string): string {
    const colors = [
      'bg-primary/10 text-primary',
      'bg-success/10 text-success',
      'bg-accent/30 text-accent-foreground',
      'bg-secondary/30 text-secondary-foreground',
      'bg-destructive/10 text-destructive',
      'bg-muted/30 text-muted-foreground',
      'bg-card border border-border/50 text-foreground',
    ];
    let hash = 0;
    for (let i = 0; i < tag.length; i++) hash = tag.charCodeAt(i) + ((hash << 5) - hash);
    return colors[Math.abs(hash) % colors.length];
  }

  createEntry() {
    this.formSubmitted = true;
    if (!this.newTitle.trim() || !this.newContent.trim()) return;
    const slug = this.newTitle.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
    const tags = this.newTags.split(',').map(t => t.trim()).filter(t => t.length > 0).join(', ');
    this.api.createPage(`memory/${slug}`, this.newTitle, this.newContent, 'memory', tags).pipe(
      takeUntilDestroyed(this.destroyRef),
    ).subscribe({
      next: () => { this.loadMemory(); this.showForm = false; this.formSubmitted = false; this.newTitle = ''; this.newContent = ''; this.newTags = ''; this.error = ''; },
      error: () => { this.error = 'Failed to create memory'; }
    });
  }

  startEdit(entry: MemoryEntry) {
    this.editEntry = entry;
    this.editTitle = entry.title;
    this.editContent = entry.content;
    this.editTags = entry.tags.join(', ');
    this.error = '';
  }

  updateEntry() {
    if (!this.editEntry) return;
    const tags = this.editTags.split(',').map(t => t.trim()).filter(t => t.length > 0).join(', ');
    this.api.updatePage(this.editEntry.id, { title: this.editTitle, content: this.editContent, tags }).pipe(
      takeUntilDestroyed(this.destroyRef),
    ).subscribe({
      next: () => { this.loadMemory(); this.editEntry = null; this.error = ''; },
      error: () => { this.error = 'Failed to update memory'; }
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
      next: () => { this.loadMemory(); this.showDeleteConfirm = false; this.deleteTarget = null; this.error = ''; },
      error: (err) => { this.deleteError = 'Failed to delete memory'; }
    });
  }
}

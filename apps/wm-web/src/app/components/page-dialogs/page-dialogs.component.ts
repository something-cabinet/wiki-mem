import { Component, input, output, effect } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { HlmButton } from '@ui/button';
import { HlmInput } from '@ui/input';
import { BrnDialogImports } from '@spartan-ng/brain/dialog';
import { HlmDialogOverlay, HlmDialogContent, HlmDialogHeader, HlmDialogTitle, HlmDialogFooter } from '@ui/dialog';
import { HlmAlert, HlmAlertDescription } from '@ui/alert';
import { HlmSelect, HlmSelectTrigger, HlmSelectValue, HlmSelectContent, HlmSelectPortal, HlmSelectItem } from '@ui/select';

export interface PageDialogData {
  id: string;
  title: string;
  type: string;
  content: string;
}

@Component({
  selector: 'app-page-dialogs',
  standalone: true,
  imports: [
    FormsModule, HlmButton, HlmInput,
    BrnDialogImports, HlmDialogOverlay, HlmDialogContent, HlmDialogHeader, HlmDialogTitle, HlmDialogFooter,
    HlmAlert, HlmAlertDescription,
    HlmSelect, HlmSelectTrigger, HlmSelectValue, HlmSelectContent, HlmSelectPortal, HlmSelectItem,
  ],
  template: `
    <!-- Edit dialog -->
    <brn-dialog [state]="showEdit() ? 'open' : 'closed'" (stateChanged)="showEditChange.emit($event === 'open')">
      <div brnDialogOverlay hlmDialogOverlay (click)="showEditChange.emit(false)"></div>
      <hlm-dialog-content *brnDialogContent>
        <div hlmDialogHeader>
          <h3 hlmDialogTitle>Edit Page</h3>
        </div>
        <div class="space-y-3">
          <div>
            <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Path / ID</label>
            <input hlmInput [value]="form.id" disabled class="opacity-60" />
          </div>
          <div>
            <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Title</label>
            <input hlmInput [(ngModel)]="form.title" />
          </div>
          <div>
            <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Content</label>
            <textarea hlmInput [(ngModel)]="form.content" rows="4"></textarea>
          </div>
          <div>
            <label class="block text-xs font-medium text-muted-foreground uppercase tracking-wider mb-1">Type</label>
            <div hlmSelect [(value)]="form.type" class="w-full">
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
          <button hlmBtn variant="ghost" (click)="showEditChange.emit(false)">Cancel</button>
          <button hlmBtn variant="default" (click)="onSave()">Save</button>
        </div>
      </hlm-dialog-content>
    </brn-dialog>

    <!-- Delete dialog -->
    <brn-dialog [state]="showDelete() ? 'open' : 'closed'" (stateChanged)="showDeleteChange.emit($event === 'open')">
      <div brnDialogOverlay hlmDialogOverlay (click)="showDeleteChange.emit(false)"></div>
      <hlm-dialog-content *brnDialogContent>
        <div hlmDialogHeader>
          <h3 hlmDialogTitle>Delete Page</h3>
        </div>
        <p class="text-sm text-muted-foreground">Are you sure you want to delete <strong>{{ data()?.title }}</strong>?</p>
        @if (deleteError()) {
          <div hlmAlert variant="destructive" class="text-sm">
            <p hlmAlertDescription>{{ deleteError() }}</p>
          </div>
        }
        <div hlmDialogFooter class="flex justify-end gap-2">
          <button hlmBtn variant="ghost" (click)="showDeleteChange.emit(false)">Cancel</button>
          <button hlmBtn variant="destructive" (click)="confirmDelete.emit()">Delete</button>
        </div>
      </hlm-dialog-content>
    </brn-dialog>
  `,
})
export class PageDialogsComponent {
  data = input<PageDialogData | null>(null);
  showEdit = input(false);
  showDelete = input(false);
  deleteError = input('');
  showEditChange = output<boolean>();
  showDeleteChange = output<boolean>();
  save = output<PageDialogData>();
  confirmDelete = output<void>();

  form = { id: '', title: '', content: '', type: '' };

  constructor() {
    effect(() => {
      if (this.showEdit() && this.data()) {
        const d = this.data()!;
        this.form = { id: d.id, title: d.title, content: d.content, type: d.type };
      }
    });
  }

  onSave() {
    this.save.emit({ ...this.form });
  }
}

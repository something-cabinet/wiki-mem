import { Component, OnInit, ChangeDetectionStrategy, DestroyRef, inject } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { Router } from '@angular/router';
import { ApiService, TaskBoard, TaskBoardItem } from '../../services/api.service';
import { HlmBadge } from '@ui/badge';
import { HlmAccordion, HlmAccordionItem, HlmAccordionTrigger, HlmAccordionContent } from '@ui/accordion';
import { HlmButton } from '@ui/button';
import { WmSpinner } from '@ui/spinner';
import { HlmAlert, HlmAlertDescription } from '@ui/alert';

@Component({
  selector: 'app-tasks-view',
  standalone: true,
  imports: [HlmBadge, HlmAccordion, HlmAccordionItem, HlmAccordionTrigger, HlmAccordionContent, HlmButton, WmSpinner, HlmAlert, HlmAlertDescription],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="flex flex-col h-full">
      <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
        <h1 class="text-xl sm:text-2xl font-semibold">Task Board</h1>
      </header>
      <div class="flex-1 overflow-y-auto">
      <div class="p-6 max-w-6xl mx-auto w-full">
      @if (loading) {
        <div class="flex items-center gap-2 text-muted-foreground p-6">
          <wm-spinner size="sm" />
          <span class="text-sm">Loading task board...</span>
        </div>
      }
      @if (error) {
        <div hlmAlert variant="destructive" class="text-sm">
          <p hlmAlertDescription>{{ error }}</p>
        </div>
      }
      @if (!loading && !error && !board) {
        <div class="flex flex-col items-center justify-center py-16 text-muted-foreground">
          <p class="text-sm font-medium">No tasks yet</p>
          <p class="text-xs text-muted-foreground/60 mt-1">Create wiki pages with a "task" type to populate the task board.</p>
        </div>
      }
      @if (board) {
        <div class="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
          @for (col of statuses; track col) {
            @let count = board.counts[col] || 0;
            <hlm-accordion class="rounded-lg border border-border overflow-hidden" [class.opacity-75]="count === 0">
              <hlm-accordion-item [isOpened]="count > 0">
                <hlm-accordion-trigger [triggerClass]="'flex items-center justify-between w-full px-3 py-2.5 text-sm font-semibold capitalize transition-colors ' + headerColorClass(col)">
                  <span class="flex items-center gap-1.5">
                    <span class="w-2 h-2 rounded-full" [class]="dotColorClass(col)"></span>
                    {{ col.replace(/-/g, ' ') }}
                  </span>
                  <span hlmBadge variant="secondary">{{ count }}</span>
                </hlm-accordion-trigger>
                <hlm-accordion-content class="bg-muted/20 rounded-b-lg">
                  <div class="p-2.5 space-y-2">
                    @for (item of board.columns[col] || []; track item.id) {
                      <button hlmBtn variant="ghost" type="button" (click)="onTaskClick(item)" class="w-full justify-start text-left h-auto font-normal p-2.5 border-l-4 rounded-lg bg-card hover:bg-accent/50 transition-colors" [class]="taskCardClass(item)">
                        <p class="font-medium truncate leading-snug">{{ item.title }}</p>
                        <p class="text-xs text-muted-foreground mt-1 font-mono">{{ item.id }}</p>
                        @if (item.priority) {
                          <span hlmBadge variant="outline" class="mt-1 text-[10px]">{{ item.priority }}</span>
                        }
                      </button>
                    }
                    @if (count === 0) {
                      <div class="text-xs text-muted-foreground/60 text-center py-4">No tasks</div>
                    }
                  </div>
                </hlm-accordion-content>
              </hlm-accordion-item>
            </hlm-accordion>
          }
        </div>
      }
      </div>
      </div>
    </div>
  `,
})
export class TasksViewComponent implements OnInit {
  board: TaskBoard | null = null;
  loading = true;
  error = '';
  statusOrder = ['draft', 'todo', 'in-progress', 'in-review', 'done', 'blocked', 'on-hold', 'urgent', 'cancelled', 'archived'];
  statuses: string[] = [];
  selectedTask: TaskBoardItem | null = null;
  private router = inject(Router);

  constructor(private api: ApiService, private destroyRef: DestroyRef) {}

  onTaskClick(item: TaskBoardItem) {
    this.router.navigate(['/pages', item.id]);
  }

  ngOnInit() {
    this.api.getTaskBoard().pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        if (res.success) {
          this.board = res;
          this.statuses = this.statusOrder.filter(s => s in (res.counts || {}));
        }
        this.loading = false;
      },
      error: () => {
        this.error = 'Failed to load task board';
        this.loading = false;
      },
    });
  }

  headerColorClass(col: string): string {
    const map: Record<string, string> = {
      draft: 'bg-muted/40 text-muted-foreground hover:bg-muted/60',
      todo: 'bg-muted/60 text-muted-foreground hover:bg-muted/80',
      'in-progress': 'bg-[var(--info)]/10 text-[var(--info)] hover:bg-[var(--info)]/15',
      'in-review': 'bg-[var(--review)]/10 text-[var(--review)] hover:bg-[var(--review)]/15',
      done: 'bg-success/10 text-success hover:bg-success/15',
      blocked: 'bg-destructive/15 text-destructive hover:bg-destructive/25',
      'on-hold': 'bg-[var(--warning)]/10 text-[var(--warning)] hover:bg-[var(--warning)]/15',
      urgent: 'bg-destructive/15 text-destructive hover:bg-destructive/25',
    };
    return map[col] || 'bg-muted/40 text-muted-foreground hover:bg-muted/60';
  }

  dotColorClass(col: string): string {
    const map: Record<string, string> = {
      draft: 'bg-muted-foreground/40',
      todo: 'bg-muted-foreground/60',
      'in-progress': 'bg-[var(--info)]',
      'in-review': 'bg-[var(--review)]',
      done: 'bg-success',
      blocked: 'bg-destructive',
      'on-hold': 'bg-[var(--warning)]',
      urgent: 'bg-destructive',
    };
    return map[col] || 'bg-muted-foreground/40';
  }

  taskCardClass(item: TaskBoardItem): string {
    const priorityBorder: Record<string, string> = {
      high: 'border-l-4 border-l-destructive',
      medium: 'border-l-4 border-l-[var(--warning)]',
      low: 'border-l-4 border-l-success',
    };
    const border = priorityBorder[item.priority] || '';
    return border;
  }

}


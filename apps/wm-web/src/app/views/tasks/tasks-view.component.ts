import { Component, OnInit, ChangeDetectionStrategy, DestroyRef } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { ApiService, TaskBoard, TaskBoardItem } from '../../services/api.service';
import { WmBadge } from '@ui/badge';
import { WmCard } from '@ui/card';
import { WmAccordion } from '@ui/accordion';
import { WmSpinner } from '@ui/spinner';

@Component({
  selector: 'app-tasks-view',
  standalone: true,
  imports: [WmBadge, WmAccordion, WmSpinner],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="flex flex-col h-full">
      <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
        <h1 class="text-xl sm:text-2xl font-bold">Task Board</h1>
      </header>
      <div class="flex-1 p-6 max-w-6xl mx-auto overflow-y-auto">
      @if (loading) {
        <div class="flex items-center gap-2 text-muted-foreground p-6">
          <wm-spinner size="sm" />
          <span class="text-sm">Loading task board...</span>
        </div>
      }
      @if (error) {
        <div class="p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-sm">
          {{ error }}
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
            <wm-accordion
              [expanded]="!collapsed[col]"
              (expandedChange)="collapsed[col] = !$event"
              class="rounded-lg border border-border overflow-hidden"
              [class.opacity-75]="count === 0"
            >
              <div slot="header" class="flex items-center justify-between w-full px-3 py-2.5 text-sm font-semibold capitalize transition-colors" [class]="headerColorClass(col)">
                <span class="flex items-center gap-1.5">
                  <span class="w-2 h-2 rounded-full" [class]="dotColorClass(col)"></span>
                  {{ col }}
                </span>
                <span wmBadge variant="secondary">{{ count }}</span>
              </div>
              <div class="p-2.5 space-y-2 bg-muted/20">
                @for (item of board.columns[col] || []; track item.id) {
                  <button
                    type="button"
                    (click)="onTaskClick(item)"
                    [class]="'w-full text-left p-2.5 text-sm rounded-xl border border-border bg-card text-card-foreground shadow-sm cursor-pointer hover:shadow-md transition-shadow ' + taskCardClass(item)"
                  >
                    <p class="font-medium truncate leading-snug">{{ item.title }}</p>
                    <p class="text-xs text-muted-foreground mt-1 font-mono">{{ item.id }}</p>
                    @if (item.priority) {
                      <span wmBadge [variant]="priorityVariant(item.priority)" class="mt-1 text-[10px]">{{ item.priority }}</span>
                    }
                  </button>
                }
                @if (count === 0) {
                  <div class="text-xs text-muted-foreground/60 text-center py-4">No tasks</div>
                }
              </div>
            </wm-accordion>
          }
        </div>
      }
      </div>
    </div>
  `,
})
export class TasksViewComponent implements OnInit {
  board: TaskBoard | null = null;
  loading = true;
  error = '';
  statuses = ['todo', 'in-progress', 'in-review', 'done', 'blocked', 'on-hold', 'urgent'];
  collapsed: Record<string, boolean> = {};
  selectedTask: TaskBoardItem | null = null;

  constructor(private api: ApiService, private destroyRef: DestroyRef) {}

  onTaskClick(item: TaskBoardItem) {
    this.selectedTask = item;
    // Future: open a detail dialog or side panel
  }

  ngOnInit() {
    this.api.getTaskBoard().pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        if (res.success) this.board = res.board;
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
      todo: 'bg-muted/40 text-muted-foreground hover:bg-muted/60',
      'in-progress': 'bg-primary/10 text-primary hover:bg-primary/15',
      'in-review': 'bg-accent/10 text-accent-foreground hover:bg-accent/15',
      done: 'bg-success/10 text-success hover:bg-success/15',
      blocked: 'bg-destructive/10 text-destructive hover:bg-destructive/15',
      'on-hold': 'bg-secondary/10 text-secondary-foreground hover:bg-secondary/15',
      urgent: 'bg-destructive/15 text-destructive hover:bg-destructive/25',
    };
    return map[col] || 'bg-muted/40 text-muted-foreground hover:bg-muted/60';
  }

  dotColorClass(col: string): string {
    const map: Record<string, string> = {
      todo: 'bg-muted-foreground/40',
      'in-progress': 'bg-primary',
      'in-review': 'bg-accent-foreground/60',
      done: 'bg-success',
      blocked: 'bg-destructive',
      'on-hold': 'bg-secondary-foreground/40',
      urgent: 'bg-destructive',
    };
    return map[col] || 'bg-muted-foreground/40';
  }

  taskCardClass(item: TaskBoardItem): string {
    const priorityBorder: Record<string, string> = {
      high: 'border-l-4 border-l-destructive',
      medium: 'border-l-4 border-l-amber-500',
      low: 'border-l-4 border-l-success',
    };
    const border = priorityBorder[item.priority] || '';
    return `p-2.5 text-sm ${border}`;
  }

  priorityVariant(priority: string): 'default' | 'secondary' | 'outline' {
    const map: Record<string, 'default' | 'secondary' | 'outline'> = {
      high: 'default',
      medium: 'secondary',
      low: 'outline',
    };
    return map[priority] || 'outline';
  }
}

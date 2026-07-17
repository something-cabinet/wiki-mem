import { Component, OnInit, ChangeDetectionStrategy } from '@angular/core';
import { ApiService, TaskBoard, TaskBoardItem } from '../../services/api.service';
import { WmBadge } from '@ui/badge';
import { WmCard } from '@ui/card';
import { WmAccordion } from '@ui/accordion';

@Component({
  selector: 'app-tasks-view',
  standalone: true,
  imports: [WmBadge, WmCard, WmAccordion],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="p-6 max-w-6xl mx-auto">
      <h1 class="text-xl sm:text-2xl font-bold mb-4">Task Board</h1>
      @if (loading) {
        <div class="flex items-center gap-2 text-gray-500">
          <span class="inline-block w-4 h-4 border-2 border-gray-300 border-t-blue-600 rounded-full animate-spin"></span>
          Loading tasks...
        </div>
      }
      @if (board) {
        <div class="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
          @for (col of statuses; track col) {
            @let count = board.counts[col] || 0;
            <wm-accordion
              [expanded]="!collapsed[col]"
              (expandedChange)="collapsed[col] = !$event"
              class="rounded-lg border border-gray-200 overflow-hidden"
              [class.opacity-75]="count === 0"
            >
              <div slot="header" class="flex items-center justify-between w-full px-3 py-2.5 text-sm font-semibold capitalize transition-colors" [class]="headerColorClass(col)">
                <span>{{ col }}</span>
                <span wmBadge variant="secondary">{{ count }}</span>
              </div>
              <div class="p-2.5 space-y-2 bg-gray-50/50">
                @for (item of board.columns[col] || []; track item.id) {
                  <div wmCard [class]="taskCardClass(item)">
                    <p class="font-medium truncate leading-snug">{{ item.title }}</p>
                    <p class="text-xs text-gray-400 mt-1 font-mono">{{ item.id }}</p>
                    @if (item.priority) {
                      <span wmBadge [variant]="priorityVariant(item.priority)" class="mt-1 text-[10px]">{{ item.priority }}</span>
                    }
                  </div>
                }
                @if (count === 0) {
                  <div class="text-xs text-gray-400 text-center py-4 italic">No tasks</div>
                }
              </div>
            </wm-accordion>
          }
        </div>
      }
    </div>
  `,
})
export class TasksViewComponent implements OnInit {
  board: TaskBoard | null = null;
  loading = true;
  statuses = ['todo', 'in-progress', 'in-review', 'done', 'blocked', 'on-hold', 'urgent'];
  collapsed: Record<string, boolean> = {};

  constructor(private api: ApiService) {}

  ngOnInit() {
    this.api.getTaskBoard().subscribe((res) => {
      if (res.success) this.board = res.board;
      this.loading = false;
    });
  }

  headerColorClass(col: string): string {
    const map: Record<string, string> = {
      todo: 'bg-slate-100 text-slate-700 hover:bg-slate-200',
      'in-progress': 'bg-blue-50 text-blue-700 hover:bg-blue-100',
      'in-review': 'bg-violet-50 text-violet-700 hover:bg-violet-100',
      done: 'bg-emerald-50 text-emerald-700 hover:bg-emerald-100',
      blocked: 'bg-red-50 text-red-700 hover:bg-red-100',
      'on-hold': 'bg-amber-50 text-amber-700 hover:bg-amber-100',
      urgent: 'bg-rose-50 text-rose-700 hover:bg-rose-100',
    };
    return map[col] || 'bg-gray-100 text-gray-700 hover:bg-gray-200';
  }

  taskCardClass(item: TaskBoardItem): string {
    const priorityBorder: Record<string, string> = {
      high: 'border-l-4 border-l-red-500',
      medium: 'border-l-4 border-l-amber-500',
      low: 'border-l-4 border-l-emerald-500',
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

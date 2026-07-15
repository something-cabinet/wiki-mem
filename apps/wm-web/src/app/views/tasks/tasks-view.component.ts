import { Component, OnInit } from '@angular/core';
import { NgClass } from '@angular/common';
import { ApiService, TaskBoard, TaskBoardItem } from '../../services/api.service';

@Component({
  selector: 'app-tasks-view',
  standalone: true,
  imports: [NgClass],
  template: `
    <div class="p-6 max-w-6xl mx-auto">
      <h1 class="text-2xl font-bold mb-4">Task Board</h1>
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
            @let isCollapsed = collapsed[col];
            <div class="rounded-lg border border-gray-200 overflow-hidden" [class.opacity-75]="count === 0">
              <button
                (click)="toggleColumn(col)"
                class="w-full flex items-center justify-between px-3 py-2.5 text-sm font-semibold capitalize transition-colors"
                [class]="headerColorClass(col)"
              >
                <span class="flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-3.5 h-3.5 transition-transform" [class.rotate-90]="!isCollapsed">
                    <path stroke-linecap="round" stroke-linejoin="round" d="m8.25 4.5 7.5 7.5-7.5 7.5" />
                  </svg>
                  {{ col }}
                </span>
                <span
                  class="text-xs px-2 py-0.5 rounded-full font-bold"
                  [class]="badgeColorClass(col)"
                >
                  {{ count }}
                </span>
              </button>
              @if (!isCollapsed) {
                <div class="p-2.5 space-y-2 bg-gray-50/50">
                  @for (item of board.columns[col] || []; track item.id) {
                    <div
                      class="p-2.5 bg-white rounded-md shadow-sm border border-gray-200 text-sm hover:shadow-md transition-shadow"
                      [ngClass]="{
                        'border-l-4 border-l-red-500': item.priority === 'high',
                        'border-l-4 border-l-amber-500': item.priority === 'medium',
                        'border-l-4 border-l-emerald-500': item.priority === 'low',
                      }"
                    >
                      <p class="font-medium truncate leading-snug">{{ item.title }}</p>
                      <p class="text-xs text-gray-400 mt-1 font-mono">{{ item.id }}</p>
                    </div>
                  }
                  @if (count === 0) {
                    <div class="text-xs text-gray-400 text-center py-4 italic">No tasks</div>
                  }
                </div>
              }
            </div>
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

  toggleColumn(col: string) {
    this.collapsed[col] = !this.collapsed[col];
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

  badgeColorClass(col: string): string {
    const map: Record<string, string> = {
      todo: 'bg-slate-200 text-slate-700',
      'in-progress': 'bg-blue-200 text-blue-800',
      'in-review': 'bg-violet-200 text-violet-800',
      done: 'bg-emerald-200 text-emerald-800',
      blocked: 'bg-red-200 text-red-800',
      'on-hold': 'bg-amber-200 text-amber-800',
      urgent: 'bg-rose-200 text-rose-800',
    };
    return map[col] || 'bg-gray-200 text-gray-700';
  }
}

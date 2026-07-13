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
        <p class="text-gray-500">Loading tasks...</p>
      }
      @if (board) {
        <div class="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
          @for (col of statuses; track col) {
            <div class="bg-gray-50 rounded-lg p-3">
              <h3 class="font-semibold text-sm mb-2 capitalize flex items-center justify-between">
                {{ col }}
                <span class="text-xs bg-gray-200 px-2 py-0.5 rounded">{{ board.counts[col] || 0 }}</span>
              </h3>
              <div class="space-y-2">
                @for (item of board.columns[col] || []; track item.id) {
                  <div
                    class="p-2 bg-white rounded shadow-sm border border-gray-200 text-sm"
                    [ngClass]="{
                      'border-l-4 border-l-red-500': item.priority === 'high',
                      'border-l-4 border-l-yellow-500': item.priority === 'medium',
                      'border-l-4 border-l-green-500': item.priority === 'low',
                    }"
                  >
                    <p class="font-medium truncate">{{ item.title }}</p>
                    <p class="text-xs text-gray-400 mt-1">{{ item.id }}</p>
                  </div>
                }
              </div>
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

  constructor(private api: ApiService) {}

  ngOnInit() {
    this.api.getTaskBoard().subscribe((res) => {
      if (res.success) this.board = res.board;
      this.loading = false;
    });
  }
}

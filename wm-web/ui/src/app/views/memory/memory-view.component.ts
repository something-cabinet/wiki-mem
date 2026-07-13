import { Component, OnInit } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ApiService, MemoryEntry } from '../../services/api.service';

@Component({
  selector: 'app-memory-view',
  standalone: true,
  imports: [FormsModule],
  template: `
    <div class="p-6 max-w-4xl mx-auto">
      <h1 class="text-2xl font-bold mb-4">Memory</h1>
      <div class="flex gap-2 mb-4">
        <select
          [(ngModel)]="selectedLayer"
          (change)="loadMemory()"
          class="px-3 py-2 border border-gray-300 rounded-lg"
        >
          <option value="project">Project Memory</option>
          <option value="session">Session Memory</option>
        </select>
      </div>
      @if (loading) {
        <p class="text-gray-500">Loading memory...</p>
      }
      @if (entries.length > 0) {
        <div class="space-y-2">
          @for (e of entries; track e.id) {
            <div class="p-3 bg-white rounded-lg shadow-sm border border-gray-200">
              <div class="flex items-center justify-between">
                <span class="font-medium">{{ e.title || e.id }}</span>
                <span class="text-xs text-gray-400">{{ e.created_at.substring(0, 10) }}</span>
              </div>
              @if (e.tags.length > 0) {
                <div class="flex gap-1 mt-1">
                  @for (tag of e.tags; track tag) {
                    <span class="text-xs px-2 py-0.5 rounded bg-blue-50 text-blue-700">{{ tag }}</span>
                  }
                </div>
              }
              <p class="text-sm text-gray-600 mt-1 line-clamp-3">{{ e.content }}</p>
            </div>
          }
        </div>
      }
      @if (!loading && entries.length === 0) {
        <p class="text-gray-500">No memory entries found.</p>
      }
    </div>
  `,
})
export class MemoryViewComponent implements OnInit {
  selectedLayer = 'project';
  entries: MemoryEntry[] = [];
  loading = true;

  constructor(private api: ApiService) {}

  ngOnInit() {
    this.loadMemory();
  }

  loadMemory() {
    this.loading = true;
    this.api.listMemory(this.selectedLayer).subscribe((res) => {
      this.entries = res.entries || [];
      this.loading = false;
    });
  }
}

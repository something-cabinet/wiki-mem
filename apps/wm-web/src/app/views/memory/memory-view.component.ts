import { Component, OnInit } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ApiService, MemoryEntry } from '../../services/api.service';

@Component({
  selector: 'app-memory-view',
  standalone: true,
  imports: [FormsModule],
  template: `
    <div class="p-6 max-w-4xl mx-auto">
      <div class="flex items-center justify-between mb-4">
        <h1 class="text-2xl font-bold">Memory</h1>
        <div class="flex items-center gap-2">
          <select
            [(ngModel)]="selectedLayer"
            (change)="loadMemory()"
            class="px-3 py-1.5 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="project">Project Memory</option>
            <option value="session">Session Memory</option>
          </select>
          <select
            [(ngModel)]="selectedStatus"
            (change)="loadMemory()"
            class="px-3 py-1.5 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="">All Statuses</option>
            <option value="active">Active</option>
            <option value="stale">Stale</option>
            <option value="archived">Archived</option>
          </select>
          <button
            (click)="showNewModal = true"
            class="flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors"
          >
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4"><path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" /></svg>
            New
          </button>
        </div>
      </div>
      @if (showNewModal) {
        <div class="fixed inset-0 bg-black/40 flex items-center justify-center z-50" (click)="showNewModal = false">
          <div class="bg-white rounded-xl shadow-xl p-6 w-full max-w-md mx-4" (click)="$event.stopPropagation()">
            <h2 class="text-lg font-bold mb-4">New Memory Entry</h2>
            <div class="space-y-3">
              <div>
                <label class="block text-xs font-medium text-gray-500 uppercase tracking-wider mb-1">Title</label>
                <input [(ngModel)]="newTitle" placeholder="Entry title" class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm" />
              </div>
              <div>
                <label class="block text-xs font-medium text-gray-500 uppercase tracking-wider mb-1">Content</label>
                <textarea [(ngModel)]="newContent" placeholder="What do you want to remember?" rows="4" class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm resize-none"></textarea>
              </div>
              <div>
                <label class="block text-xs font-medium text-gray-500 uppercase tracking-wider mb-1">Tags (comma separated)</label>
                <input [(ngModel)]="newTags" placeholder="tag1, tag2" class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm" />
              </div>
            </div>
            <div class="flex justify-end gap-2 mt-5">
              <button (click)="showNewModal = false" class="px-3 py-1.5 text-sm text-gray-600 hover:bg-gray-100 rounded-lg transition-colors">Cancel</button>
              <button (click)="createEntry()" class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-medium transition-colors">Save</button>
            </div>
          </div>
        </div>
      }
      @if (loading) {
        <div class="flex items-center gap-2 text-gray-500">
          <span class="inline-block w-4 h-4 border-2 border-gray-300 border-t-blue-600 rounded-full animate-spin"></span>
          Loading memory...
        </div>
      }
      @if (entries.length > 0) {
        <div class="space-y-2">
          @for (e of entries; track e.id) {
            <div class="p-4 bg-white rounded-lg shadow-sm border border-gray-200 hover:shadow-md transition-shadow">
              <div class="flex items-center justify-between">
                <span class="font-medium">{{ e.title || e.id }}</span>
                <span class="text-xs text-gray-400 font-mono">{{ e.created_at.substring(0, 10) }}</span>
              </div>
              @if (e.tags.length > 0) {
                <div class="flex flex-wrap gap-1.5 mt-2">
                  @for (tag of e.tags; track tag) {
                    <span class="text-xs px-2 py-0.5 rounded-full font-medium" [class]="tagColorClass(tag)">{{ tag }}</span>
                  }
                </div>
              }
              <div class="mt-2">
                @if (expanded[e.id]) {
                  <p class="text-sm text-gray-600 leading-relaxed">{{ e.content }}</p>
                } @else {
                  <p class="text-sm text-gray-600 leading-relaxed line-clamp-3">{{ e.content }}</p>
                }
                @if (e.content.length > 180) {
                  <button
                    (click)="expanded[e.id] = !expanded[e.id]"
                    class="mt-1.5 text-xs text-blue-600 hover:text-blue-800 font-medium transition-colors"
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
        <p class="text-gray-500 text-center py-8">No memory entries found.</p>
      }
    </div>
  `,
})
export class MemoryViewComponent implements OnInit {
  selectedLayer = 'project';
  selectedStatus = '';
  entries: MemoryEntry[] = [];
  loading = true;
  error = '';
  showNewModal = false;
  newTitle = '';
  newContent = '';
  newTags = '';
  expanded: Record<string, boolean> = {};

  constructor(private api: ApiService) {}

  ngOnInit() {
    this.loadMemory();
  }

  loadMemory() {
    this.loading = true;
    this.api.listMemory(this.selectedLayer, this.selectedStatus).subscribe((res) => {
      this.entries = res.entries || [];
      this.loading = false;
    });
  }

  tagColorClass(tag: string): string {
    const colors = [
      'bg-blue-50 text-blue-700',
      'bg-emerald-50 text-emerald-700',
      'bg-violet-50 text-violet-700',
      'bg-amber-50 text-amber-700',
      'bg-rose-50 text-rose-700',
      'bg-cyan-50 text-cyan-700',
      'bg-orange-50 text-orange-700',
    ];
    let hash = 0;
    for (let i = 0; i < tag.length; i++) hash = tag.charCodeAt(i) + ((hash << 5) - hash);
    return colors[Math.abs(hash) % colors.length];
  }

  createEntry() {
    const slug = this.newTitle.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
    this.api.createPage(`memory/${slug}`, this.newTitle, this.newContent, 'memory').subscribe({
      next: () => { this.loadMemory(); this.showNewModal = false; this.newTitle = ''; this.newContent = ''; this.newTags = ''; },
      error: () => { this.error = 'Failed to create memory'; }
    });
  }
}

import { Component, OnInit } from '@angular/core';
import { ApiService, InitialState } from '../../services/api.service';

@Component({
  selector: 'app-settings-view',
  standalone: true,
  template: `
    <div class="p-6 max-w-2xl mx-auto">
      <div class="flex items-center justify-between mb-4">
        <h1 class="text-2xl font-bold">Settings</h1>
        <button
          (click)="refresh()"
          class="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium text-gray-600 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 hover:text-gray-800 transition-colors"
          title="Refresh engine status"
        >
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4"><path stroke-linecap="round" stroke-linejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182" /></svg>
          Refresh
        </button>
      </div>
      @if (state) {
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-5">
          <h2 class="font-semibold mb-4 text-sm uppercase tracking-wider text-gray-500">Engine Status</h2>
          <dl class="space-y-3 text-sm">
            <div class="flex justify-between items-center py-1 border-b border-gray-100">
              <dt class="text-gray-500">Graph Nodes</dt>
              <dd class="font-semibold text-slate-800">{{ state.graph_node_count }}</dd>
            </div>
            <div class="flex justify-between items-center py-1 border-b border-gray-100">
              <dt class="text-gray-500">Graph Edges</dt>
              <dd class="font-semibold text-slate-800">{{ state.graph_edge_count }}</dd>
            </div>
            <div class="flex justify-between items-center py-1 border-b border-gray-100">
              <dt class="text-gray-500">Session Memory</dt>
              <dd class="font-semibold text-slate-800">{{ state.session_memory_count }}</dd>
            </div>
            <div class="flex justify-between items-center py-1 border-b border-gray-100">
              <dt class="text-gray-500">Uptime</dt>
              <dd class="font-semibold text-slate-800 font-mono">{{ formatUptime(state.uptime_secs) }}</dd>
            </div>
            <div class="flex justify-between items-center py-1">
              <dt class="text-gray-500">Stale</dt>
              <dd class="font-semibold">
                @if (state.stale) {
                  <span class="inline-flex items-center gap-1 text-red-700">
                    <span class="w-2 h-2 rounded-full bg-red-500 animate-pulse"></span>
                    Yes
                  </span>
                } @else {
                  <span class="inline-flex items-center gap-1 text-emerald-700">
                    <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
                    No
                  </span>
                }
              </dd>
            </div>
          </dl>
        </div>
      } @else {
        <div class="flex items-center gap-2 text-gray-500">
          <span class="inline-block w-4 h-4 border-2 border-gray-300 border-t-blue-600 rounded-full animate-spin"></span>
          Loading...
        </div>
      }
    </div>
  `,
})
export class SettingsViewComponent implements OnInit {
  state: InitialState | null = null;

  constructor(private api: ApiService) {}

  ngOnInit() {
    this.refresh();
  }

  refresh() {
    this.api.getInitial().subscribe((res) => {
      if (res.success) this.state = res;
    });
  }

  formatUptime(seconds: number): string {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    return `${h}h ${m}m ${s}s`;
  }
}

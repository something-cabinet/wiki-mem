import { Component, OnInit } from '@angular/core';
import { ApiService, InitialState } from '../../services/api.service';

@Component({
  selector: 'app-settings-view',
  standalone: true,
  template: `
    <div class="p-6 max-w-2xl mx-auto">
      <h1 class="text-2xl font-bold mb-4">Settings</h1>
      @if (state) {
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
          <h2 class="font-semibold mb-3">Engine Status</h2>
          <dl class="space-y-2 text-sm">
            <div class="flex justify-between py-1 border-b border-gray-100">
              <dt class="text-gray-500">Graph Nodes</dt>
              <dd class="font-medium">{{ state.graph_node_count }}</dd>
            </div>
            <div class="flex justify-between py-1 border-b border-gray-100">
              <dt class="text-gray-500">Graph Edges</dt>
              <dd class="font-medium">{{ state.graph_edge_count }}</dd>
            </div>
            <div class="flex justify-between py-1 border-b border-gray-100">
              <dt class="text-gray-500">Session Memory</dt>
              <dd class="font-medium">{{ state.session_memory_count }}</dd>
            </div>
            <div class="flex justify-between py-1 border-b border-gray-100">
              <dt class="text-gray-500">Uptime</dt>
              <dd class="font-medium">{{ formatUptime(state.uptime_secs) }}</dd>
            </div>
            <div class="flex justify-between py-1">
              <dt class="text-gray-500">Stale</dt>
              <dd class="font-medium">
                <span [class]="state.stale ? 'text-red-600' : 'text-green-600'">
                  {{ state.stale ? 'Yes' : 'No' }}
                </span>
              </dd>
            </div>
          </dl>
        </div>
      } @else {
        <p class="text-gray-500">Loading...</p>
      }
    </div>
  `,
})
export class SettingsViewComponent implements OnInit {
  state: InitialState | null = null;

  constructor(private api: ApiService) {}

  ngOnInit() {
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

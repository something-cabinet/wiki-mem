import { Component } from '@angular/core';
import { RouterOutlet, RouterLink, RouterLinkActive } from '@angular/router';
import { NgClass } from '@angular/common';

@Component({
  selector: 'app-layout',
  standalone: true,
  imports: [RouterOutlet, RouterLink, RouterLinkActive, NgClass],
  template: `
    <div class="flex h-screen">
      <!-- Sidebar -->
      <aside class="w-56 bg-gray-900 text-white flex flex-col shrink-0">
        <div class="p-4 border-b border-gray-700">
          <h1 class="text-lg font-bold">WM Engine</h1>
          <p class="text-xs text-gray-400 mt-1">Wiki Memory Engine</p>
        </div>
        <nav class="flex-1 p-2 space-y-1">
          @for (item of navItems; track item.path) {
            <a
              [routerLink]="item.path"
              routerLinkActive="bg-gray-700 text-white"
              [routerLinkActiveOptions]="{ exact: item.path === '/search' }"
              class="flex items-center gap-3 px-3 py-2 rounded text-sm text-gray-300 hover:bg-gray-800 transition-colors"
            >
              <span class="text-lg">{{ item.icon }}</span>
              <span>{{ item.label }}</span>
            </a>
          }
        </nav>
        <div class="p-3 border-t border-gray-700 text-xs text-gray-500">
          WM Web UI v0.1
        </div>
      </aside>
      <!-- Main content -->
      <main class="flex-1 overflow-auto">
        <router-outlet />
      </main>
    </div>
  `,
})
export class LayoutComponent {
  navItems = [
    { path: '/search', label: 'Search', icon: '🔍' },
    { path: '/graph', label: 'Graph', icon: '🔗' },
    { path: '/tasks', label: 'Tasks', icon: '📋' },
    { path: '/pages', label: 'Pages', icon: '📄' },
    { path: '/memory', label: 'Memory', icon: '🧠' },
    { path: '/settings', label: 'Settings', icon: '⚙️' },
  ];
}

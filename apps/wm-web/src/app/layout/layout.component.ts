import { Component, ChangeDetectionStrategy } from '@angular/core';
import { RouterOutlet, RouterLink, RouterLinkActive } from '@angular/router';
import { NgIcon, provideIcons } from '@ng-icons/core';
import {
  lucideSearch,
  lucideLayoutGrid,
  lucideCheckCircle,
  lucideFileText,
  lucideBrain,
  lucideSettings,
} from '@ng-icons/lucide';

interface NavItem {
  path: string;
  label: string;
  icon: string;
}

@Component({
  selector: 'app-layout',
  standalone: true,
  imports: [RouterOutlet, RouterLink, RouterLinkActive, NgIcon],
  providers: [provideIcons({ lucideSearch, lucideLayoutGrid, lucideCheckCircle, lucideFileText, lucideBrain, lucideSettings })],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
     <div class="flex h-screen bg-background">
      <!-- Hamburger button for mobile -->
      <button (click)="sidebarOpen = !sidebarOpen" class="md:hidden fixed top-3 left-3 z-50 p-2 bg-sidebar text-sidebar-foreground rounded-lg shadow-sm">
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5"><path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" /></svg>
      </button>

      <!-- Sidebar: fixed on mobile, static on desktop -->
      <aside [class.-translate-x-full]="!sidebarOpen"
        class="fixed md:static inset-y-0 left-0 z-40 w-56 bg-sidebar text-sidebar-foreground flex flex-col shrink-0
               transform transition-transform md:transform-none border-r border-sidebar-border">
        <div class="p-4 border-b border-sidebar-border">
          <h1 class="text-lg font-bold tracking-tight">WM Engine</h1>
          <p class="text-xs text-sidebar-foreground/60 mt-1">Wiki Memory Engine</p>
        </div>
        <nav class="flex-1 p-2 space-y-0.5">
          @for (item of navItems; track item.path) {
            <a
              [routerLink]="item.path"
              routerLinkActive="bg-sidebar-accent text-sidebar-accent-foreground"
              [routerLinkActiveOptions]="{ exact: item.path === '/search' }"
              class="group flex items-center gap-3 px-3 py-2 rounded-md text-sm text-sidebar-foreground/70 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground transition-all"
            >
              <ng-icon [name]="item.icon" class="shrink-0 text-sidebar-foreground/50 group-hover:text-sidebar-accent-foreground transition-colors" size="18" />
              <span class="font-medium">{{ item.label }}</span>
            </a>
          }
        </nav>
        <div class="p-3 border-t border-sidebar-border text-xs text-sidebar-foreground/50">
          WM Web UI v0.1
        </div>
      </aside>

      <!-- Overlay backdrop on mobile -->
      @if (sidebarOpen) {
        <div (click)="sidebarOpen = false" class="fixed inset-0 bg-black/50 z-30 md:hidden"></div>
      }

      <!-- Main content -->
      <main class="flex-1 overflow-auto">
        <router-outlet />
      </main>
    </div>
  `,
})
export class LayoutComponent {
  sidebarOpen = false;
  navItems: NavItem[] = [
    { path: '/search', label: 'Search', icon: 'lucideSearch' },
    { path: '/graph', label: 'Graph', icon: 'lucideLayoutGrid' },
    { path: '/tasks', label: 'Tasks', icon: 'lucideCheckCircle' },
    { path: '/pages', label: 'Pages', icon: 'lucideFileText' },
    { path: '/memory', label: 'Memory', icon: 'lucideBrain' },
    { path: '/settings', label: 'Settings', icon: 'lucideSettings' },
  ];
}

import { Component, ChangeDetectionStrategy, OnInit } from '@angular/core';
import { RouterOutlet, RouterLink, RouterLinkActive } from '@angular/router';
import { NgIcon, provideIcons } from '@ng-icons/core';
import {
  lucideSearch,
  lucideLayoutGrid,
  lucideCheckCircle,
  lucideFileText,
  lucideBrain,
  lucideSettings,
  lucideSun,
  lucideMoon,
} from '@ng-icons/lucide';
import {
  HlmSidebar,
  HlmSidebarContent,
  HlmSidebarFooter,
  HlmSidebarGroup,
  HlmSidebarHeader,
  HlmSidebarInset,
  HlmSidebarMenu,
  HlmSidebarMenuButton,
  HlmSidebarMenuItem,
  HlmSidebarTrigger,
  HlmSidebarWrapper,
} from '@ui/sidebar';

interface NavItem {
  path: string;
  label: string;
  icon: string;
}

@Component({
  selector: 'app-layout',
  standalone: true,
  imports: [
    RouterOutlet,
    RouterLink,
    RouterLinkActive,
    NgIcon,
    HlmSidebar,
    HlmSidebarContent,
    HlmSidebarFooter,
    HlmSidebarGroup,
    HlmSidebarHeader,
    HlmSidebarInset,
    HlmSidebarMenu,
    HlmSidebarMenuButton,
    HlmSidebarMenuItem,
    HlmSidebarTrigger,
    HlmSidebarWrapper,
  ],
  providers: [provideIcons({ lucideSearch, lucideLayoutGrid, lucideCheckCircle, lucideFileText, lucideBrain, lucideSettings, lucideSun, lucideMoon })],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <hlm-sidebar-wrapper>
      <hlm-sidebar collapsible="icon" side="left" variant="sidebar">
        <hlm-sidebar-header>
          <div class="p-4 border-b border-sidebar-border">
            <h1 class="text-lg font-bold tracking-tight">WM Engine</h1>
            <p class="text-xs text-sidebar-foreground/60 mt-1">Wiki Memory Engine</p>
          </div>
        </hlm-sidebar-header>
        <hlm-sidebar-content>
          <hlm-sidebar-group>
            <ul hlmSidebarMenu>
              @for (item of navItems; track item.path) {
                <li hlmSidebarMenuItem>
                  <a
                    [routerLink]="item.path"
                    routerLinkActive="bg-sidebar-accent text-sidebar-accent-foreground"
                    [routerLinkActiveOptions]="{ exact: item.path === '/search' }"
                    hlmSidebarMenuButton
                    [isActive]="rla.isActive"
                    #rla="routerLinkActive"
                    [tooltip]="item.label"
                    style="gap: 12px"
                  >
                    <ng-icon [name]="item.icon" />
                    <span class="font-medium">{{ item.label }}</span>
                  </a>
                </li>
              }
            </ul>
          </hlm-sidebar-group>
        </hlm-sidebar-content>
        <hlm-sidebar-footer>
          <div class="p-3 border-t border-sidebar-border space-y-2">
            <p class="text-xs text-sidebar-foreground/60 font-mono">WM Web UI v0.1</p>
            <button
              (click)="toggleDarkMode()"
              class="flex items-center gap-2 w-full px-2 py-1.5 text-xs text-sidebar-foreground/60 hover:text-sidebar-foreground hover:bg-sidebar-accent rounded-md transition-colors"
              [attr.aria-label]="isDarkMode ? 'Switch to light mode' : 'Switch to dark mode'"
            >
              <ng-icon [name]="isDarkMode ? 'lucideSun' : 'lucideMoon'" size="14" />
              <span>{{ isDarkMode ? 'Light Mode' : 'Dark Mode' }}</span>
            </button>
          </div>
        </hlm-sidebar-footer>
      </hlm-sidebar>

      <main hlmSidebarInset class="bg-muted/20">
        <header class="flex h-11 items-center border-b border-border px-3 shrink-0 bg-background">
          <button hlmSidebarTrigger></button>
        </header>
        <div class="flex-1 overflow-auto">
          <router-outlet />
        </div>
      </main>
    </hlm-sidebar-wrapper>
  `,
})
export class LayoutComponent implements OnInit {
  isDarkMode = (() => {
    const stored = localStorage.getItem('wm-dark-mode');
    if (stored !== null) return stored === 'true';
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  })();

  ngOnInit() {
    document.documentElement.classList.toggle('dark', this.isDarkMode);
  }

  toggleDarkMode() {
    this.isDarkMode = !this.isDarkMode;
    document.documentElement.classList.toggle('dark', this.isDarkMode);
    localStorage.setItem('wm-dark-mode', String(this.isDarkMode));
  }

  navItems: NavItem[] = [
    { path: '/search', label: 'Search', icon: 'lucideSearch' },
    { path: '/graph', label: 'Graph', icon: 'lucideLayoutGrid' },
    { path: '/tasks', label: 'Tasks', icon: 'lucideCheckCircle' },
    { path: '/pages', label: 'Pages', icon: 'lucideFileText' },
    { path: '/memory', label: 'Memory', icon: 'lucideBrain' },
    { path: '/settings', label: 'Settings', icon: 'lucideSettings' },
  ];
}

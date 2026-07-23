import { Component, ChangeDetectionStrategy } from '@angular/core';
import { RouterOutlet, RouterLink, RouterLinkActive } from '@angular/router';
import { NgIcon, provideIcons } from '@ng-icons/core';
import {
  lucideSearch,
  lucideLayoutGrid,
  lucideCode,
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
import { HlmSwitch } from '@ui/switch';
import { NgxSonnerToaster } from 'ngx-sonner';
import { ThemeService } from '../services/theme.service';

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
    HlmSwitch,
    NgxSonnerToaster,
  ],
  providers: [provideIcons({ lucideSearch, lucideLayoutGrid, lucideCode, lucideCheckCircle, lucideFileText, lucideBrain, lucideSettings, lucideSun, lucideMoon })],
  changeDetection: ChangeDetectionStrategy.Default,
  template: `
    <hlm-sidebar-wrapper>
      <hlm-sidebar collapsible="icon" side="left" variant="sidebar">
        <hlm-sidebar-header>
          <div class="p-4 border-b border-sidebar-border">
            <h1 class="text-lg font-semibold tracking-tight">WM Engine</h1>
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
                    class="gap-3"
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
            <label class="flex items-center gap-2 w-full px-2 py-1.5 text-xs text-sidebar-foreground/60 hover:text-sidebar-foreground hover:bg-sidebar-accent rounded-md transition-colors cursor-pointer">
              <ng-icon [name]="theme.isDark() ? 'lucideSun' : 'lucideMoon'" size="14" />
              <span>{{ theme.isDark() ? 'Dark Mode' : 'Light Mode' }}</span>
  <hlm-switch class="ml-auto" [checked]="theme.isDark()" (checkedChange)="theme.toggle()"></hlm-switch>
            </label>
          </div>
        </hlm-sidebar-footer>
      </hlm-sidebar>

      <main hlmSidebarInset class="bg-muted/20">
        <a href="#main-content" class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-50 focus:px-4 focus:py-2 focus:bg-background focus:border focus:border-border focus:rounded-md focus:text-sm focus:font-medium">
          Skip to content
        </a>
        <header class="flex h-9 items-center px-3 shrink-0">
          <button hlmSidebarTrigger></button>
        </header>
        <div id="main-content" class="flex-1 overflow-auto">
          <ngx-sonner-toaster position="top-right" richColors />
          <router-outlet />
        </div>
      </main>
    </hlm-sidebar-wrapper>
  `,
})
export class LayoutComponent {
  constructor(protected theme: ThemeService) {}

  navItems: NavItem[] = [
    { path: '/search', label: 'Search', icon: 'lucideSearch' },
    { path: '/graph', label: 'Graph', icon: 'lucideLayoutGrid' },
    { path: '/code', label: 'Code', icon: 'lucideCode' },
    { path: '/tasks', label: 'Tasks', icon: 'lucideCheckCircle' },
    { path: '/pages', label: 'Pages', icon: 'lucideFileText' },
    { path: '/memory', label: 'Memory', icon: 'lucideBrain' },
    { path: '/settings', label: 'Settings', icon: 'lucideSettings' },
  ];
}

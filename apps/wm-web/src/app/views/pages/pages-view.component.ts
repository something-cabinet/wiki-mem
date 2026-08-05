import { Component, OnInit, DestroyRef, ChangeDetectionStrategy, Inject } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { NgIcon, provideIcons } from '@ng-icons/core';
import { lucideFileText } from '@ng-icons/lucide';
import { BackButtonComponent } from '../../components/back-button/back-button.component';
import { HlmBadge } from '@ui/badge';
import { WmSpinner } from '@ui/spinner';
import { HlmAlert, HlmAlertTitle, HlmAlertDescription } from '@ui/alert';
import { HlmCard } from '@ui/card';
import { EnginePort, ENGINE_PORT, Page } from '../../services/engine-port';
import { pageTypeBadgeClass } from '@ui/graph';

@Component({
  selector: 'app-pages-view',
  standalone: true,
  imports: [
    HlmBadge,
    WmSpinner,
    HlmAlert,
    HlmAlertTitle,
    HlmAlertDescription,
    HlmCard,
    NgIcon,
    BackButtonComponent,
  ],
  providers: [provideIcons({ lucideFileText })],
  changeDetection: ChangeDetectionStrategy.Default,
  template: `
    <div class="flex flex-col h-full">
      @if (selectedPage) {
        <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
          <div class="flex items-center gap-2 min-w-0">
            <app-back-button [fallback]="'/pages'" />
            <h1 class="text-xl sm:text-2xl font-semibold truncate">{{ selectedPage.title || selectedPage.id }}</h1>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <span hlmBadge variant="secondary" [class]="pageTypeBadgeClass(selectedPage.type)" class="font-medium">{{ selectedPage.type }}</span>
            <span hlmBadge variant="outline">{{ selectedPage.status }}</span>
          </div>
        </header>
        <div class="flex-1 p-6 max-w-4xl mx-auto overflow-y-auto w-full">
          @if (pageContent) {
            <div class="relative">
              <div class="absolute top-0 right-0 px-2 py-1 text-[10px] text-muted-foreground font-mono uppercase tracking-wider">Content</div>
              <pre class="p-4 pt-6 bg-muted/30 rounded-lg border border-border text-sm overflow-auto max-h-[70vh] font-mono text-muted-foreground leading-relaxed whitespace-pre-wrap break-words">{{ pageContent }}</pre>
            </div>
          } @else {
            <div class="text-center py-16 text-muted-foreground">
              <p class="text-sm">No content for this page.</p>
            </div>
          }
        </div>
      } @else {
        <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
          <h1 class="text-xl sm:text-2xl font-semibold">Pages</h1>
        </header>
        <div class="flex-1 overflow-y-auto">
          <div class="p-6 max-w-4xl mx-auto w-full">
            @if (loading) {
              <div class="flex items-center justify-center gap-2 text-muted-foreground py-16">
                <wm-spinner size="sm" />
                <span class="text-sm">Loading pages...</span>
              </div>
            }
            @if (error) {
              <div hlmAlert variant="destructive" class="p-3 text-sm">
                <span hlmAlertTitle>Error</span>
                <p hlmAlertDescription>{{ error }}</p>
              </div>
            }
            @if (pages.length === 0 && !loading) {
              <div class="flex flex-col items-center justify-center py-16 text-muted-foreground">
                <ng-icon name="lucideFileText" size="32" class="text-muted-foreground/30" />
                <p class="text-lg font-medium mt-4">No pages yet</p>
                <p class="text-xs text-muted-foreground/60 mt-1">Pages appear here once they are created in the wiki.</p>
              </div>
            }
            <div class="grid gap-2">
              @for (p of pages; track p.id) {
                <button
                  hlmCard
                  type="button"
                  (click)="openPage(p.id)"
                  class="w-full text-left p-4 cursor-pointer hover:shadow-md hover:border-foreground/20 transition-all"
                >
                  <div class="flex items-center justify-between">
                    <span class="font-medium text-foreground">{{ p.title || p.id }}</span>
                    <span hlmBadge variant="secondary" [class]="pageTypeBadgeClass(p.type)" class="font-medium">{{ p.type }}</span>
                  </div>
                  <p class="text-xs text-muted-foreground mt-1 font-mono">{{ p.id }}</p>
                </button>
              }
            </div>
          </div>
        </div>
      }
    </div>
  `,
})
export class PagesViewComponent implements OnInit {
  /** Bound to template — Angular templates can't call imported functions directly */
  pageTypeBadgeClass = pageTypeBadgeClass;
  pages: Page[] = [];
  selectedPage: Page | null = null;
  pageContent = '';
  loading = true;
  error = '';

  constructor(
    @Inject(ENGINE_PORT) private api: EnginePort,
    private route: ActivatedRoute,
    private router: Router,
    private destroyRef: DestroyRef,
  ) {}

  ngOnInit() {
    this.route.paramMap.pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (params) => {
        const pageId = params.get('id');
        if (pageId) {
          this.fetchPage(pageId);
        } else {
          this.selectedPage = null;
          this.pageContent = '';
          this.loadList();
        }
      },
    });
  }

  private loadList() {
    this.loading = true;
    this.error = '';
    this.api.listPages().pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        this.pages = res.pages || [];
        this.loading = false;
      },
      error: () => {
        this.error = 'Failed to load pages';
        this.loading = false;
      },
    });
  }

  fetchPage(id: string) {
    this.loading = true;
    this.error = '';
    this.api.getPage(id).pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res: any) => {
        if (res.success) {
          const meta = res.page?.meta;
          this.selectedPage = {
            id: res.page?.id || id,
            title: meta?.title || id,
            type: meta?.type || 'note',
            status: meta?.status || 'draft',
          };
          this.pageContent = res.page?.content || '';
        } else {
          this.error = res.error || 'Page not found';
          this.selectedPage = null;
        }
        this.loading = false;
      },
      error: () => {
        this.error = 'Failed to load page';
        this.loading = false;
        this.selectedPage = null;
        this.loadList();
      },
    });
  }

  openPage(id: string) {
    this.router.navigate(['/pages', id]);
  }
}

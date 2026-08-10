import { Component, ChangeDetectionStrategy, DestroyRef, Inject } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { FormsModule } from '@angular/forms';
import { NgIcon, provideIcons } from '@ng-icons/core';
import { lucideSearch, lucideCode, lucideFileText, lucideChevronLeft, lucideAlertCircle, lucideRefreshCw } from '@ng-icons/lucide';
import { HlmButton } from '@ui/button';
import { HlmInput } from '@ui/input';
import { HlmBadge } from '@ui/badge';
import { WmSpinner } from '@ui/spinner';
import { HlmAlert, HlmAlertDescription } from '@ui/alert';
import { HlmCard } from '@ui/card';
import { CodeIntelPort, CODE_INTEL_PORT, CodeIntelSymbol, CodeIntelDepSet, symbolKindBadgeClass } from '../../services/code-intel-port';

interface LanguageFilter {
  label: string;
  value: string | null;
}

@Component({
  selector: 'app-code-view',
  standalone: true,
  imports: [
    FormsModule,
    HlmButton,
    HlmInput,
    HlmBadge,
    WmSpinner,
    NgIcon,
    HlmAlert,
    HlmAlertDescription,
    HlmCard,
  ],
  providers: [provideIcons({ lucideSearch, lucideCode, lucideFileText, lucideChevronLeft, lucideAlertCircle, lucideRefreshCw })],
  changeDetection: ChangeDetectionStrategy.Default,
  template: `
    <div class="flex flex-col h-full wm-page-enter">
      <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
        <div class="flex items-center gap-2">
          @if (view !== 'search') {
            <button hlmBtn variant="ghost" size="sm" (click)="goBack()" class="-ml-2">
              <ng-icon name="lucideChevronLeft" size="16" />
              Back
            </button>
          }
          <h1 class="text-xl sm:text-2xl font-semibold">Code</h1>
        </div>
        @if (!loading && view === 'search' && groupedResults.size > 0) {
          <span hlmBadge variant="secondary">{{ totalResults }} result{{ totalResults === 1 ? '' : 's' }}</span>
        }
      </header>

      <div class="flex-1 overflow-y-auto">
        <div class="p-6 max-w-5xl mx-auto w-full">

          @if (view === 'search') {
            <div class="flex flex-col gap-3 mb-4">
              <div class="flex gap-2">
                <div class="relative flex-1">
                  <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                    <ng-icon name="lucideSearch" size="16" class="text-muted-foreground" />
                  </div>
                  @if (debouncing) {
                    <span class="absolute inset-y-0 right-0 pr-3 flex items-center">
                      <span class="w-1.5 h-1.5 rounded-full bg-primary animate-pulse" aria-label="Typing..."></span>
                    </span>
                  }
                  <input
                    hlmInput
                    [(ngModel)]="query"
                    (input)="onSearchInput()"
                    (keydown.enter)="doSearch()"
                    placeholder="Search symbols (function, class, variable...)"
                    aria-label="Symbol search query"
                    class="pl-9"
                  />
                </div>
                <button hlmBtn variant="default" (click)="doSearch()">
                  Search
                </button>
              </div>
              <div class="flex items-center gap-2 flex-wrap">
                <span class="text-xs text-muted-foreground font-medium uppercase tracking-wider">Language</span>
                @for (lang of languageFilters; track lang.value) {
                  <button
                    hlmBtn
                    size="sm"
                    [variant]="activeLanguage === lang.value ? 'default' : 'outline'"
                    (click)="setLanguage(lang.value)"
                    class="h-7 text-xs"
                  >
                    {{ lang.label }}
                  </button>
                }
              </div>
            </div>

            @if (loading) {
              <div role="status" aria-live="polite" class="flex items-center gap-2 text-muted-foreground py-8">
                <wm-spinner size="sm" />
                <span class="text-sm">Searching symbols...</span>
              </div>
            }

            @if (error) {
              <div role="alert" hlmAlert variant="destructive" class="p-3 text-sm mb-4">
                <div class="flex items-center gap-2">
                  <ng-icon name="lucideAlertCircle" size="16" />
                  <p hlmAlertDescription>{{ error }}</p>
                </div>
                <button hlmBtn variant="outline" size="sm" class="mt-2" (click)="doSearch()">
                  <ng-icon name="lucideRefreshCw" size="14" />
                  Retry
                </button>
              </div>
            }

            @if (!loading && !error && groupedResults.size > 0) {
              <div role="list" aria-label="Search results" class="space-y-6">
                @for (lang of languageOrder; track lang) {
                  @let group = groupedResults.get(lang);
                  @if (group && group.length > 0) {
                    <section>
                      <h2 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2 sticky top-0 bg-background/95 backdrop-blur py-1 z-10">
                        {{ lang }}
                      </h2>
                      <div class="grid gap-2">
                        @for (sym of group; track sym.name + sym.file + sym.line) {
                          <button
                            hlmCard
                            type="button"
                            (click)="selectSymbol(sym)"
                            class="w-full text-left p-4 cursor-pointer hover:bg-accent/50 transition-colors"
                          >
                            <div class="flex items-center gap-2 flex-wrap">
                              <span class="font-medium text-foreground">{{ sym.name }}</span>
                              <span hlmBadge variant="secondary" [class]="symbolKindBadgeClass(sym.kind)" class="font-medium">{{ sym.kind }}</span>
                              @if (sym.parent_name) {
                                <span class="text-xs text-muted-foreground">in {{ sym.parent_name }}</span>
                              }
                            </div>
                            <div class="flex items-center gap-2 mt-1">
                              <ng-icon name="lucideFileText" size="12" class="text-muted-foreground/60" />
                              <span class="text-xs text-muted-foreground font-mono">{{ sym.file }}:{{ sym.line }}</span>
                            </div>
                            @if (sym.snippet) {
                              <p class="text-xs text-muted-foreground/80 mt-1.5 line-clamp-2 font-mono bg-muted/30 rounded px-2 py-1">{{ sym.snippet }}</p>
                            }
                          </button>
                        }
                      </div>
                    </section>
                  }
                }
              </div>
            }

            @if (!loading && !error && !query && totalResults === 0) {
              <div class="text-center py-16">
                <ng-icon name="lucideCode" size="36" class="text-muted-foreground/30 mx-auto mb-4" />
                <p class="text-lg font-medium text-muted-foreground">Search code symbols across your project</p>
                <p class="text-sm text-muted-foreground/60 mt-1">Type a symbol name and press <kbd class="px-1.5 py-0.5 bg-muted rounded text-xs font-mono">Enter</kbd> to search</p>
              </div>
            }

            @if (!loading && !error && query && totalResults === 0) {
              <div class="flex flex-col items-center justify-center py-16 text-muted-foreground">
                <ng-icon name="lucideSearch" size="32" class="text-muted-foreground/30" />
                <p class="text-lg font-medium mt-4">No symbols found</p>
                <p class="text-sm text-muted-foreground/60 mt-1">Try a different search term or adjust the language filter.</p>
              </div>
            }
          }

          @if (view === 'symbol' && selectedSymbol) {
            <div class="space-y-4">
              <div class="flex items-center gap-2 flex-wrap">
                <span class="text-2xl font-semibold break-words">{{ selectedSymbol.name }}</span>
                <span hlmBadge variant="secondary" [class]="symbolKindBadgeClass(selectedSymbol.kind)" class="font-medium">{{ selectedSymbol.kind }}</span>
              </div>

              <div class="flex items-center gap-2 text-sm text-muted-foreground">
                <ng-icon name="lucideFileText" size="14" />
                <span class="font-mono">{{ selectedSymbol.file }}:{{ selectedSymbol.line }}:{{ selectedSymbol.column }}</span>
              </div>

              @if (selectedSymbol.parent_name) {
                <div class="text-sm text-muted-foreground">
                  Parent: <span class="font-medium text-foreground">{{ selectedSymbol.parent_name }}</span>
                </div>
              }

              @if (selectedSymbol.snippet) {
                <div class="relative">
                  <div class="absolute top-0 right-0 px-2 py-1 text-[10px] text-muted-foreground font-mono uppercase tracking-wider">Snippet</div>
                  <pre class="p-4 pt-6 bg-muted/30 rounded-lg border border-border text-sm overflow-auto max-h-[40vh] font-mono text-muted-foreground leading-relaxed whitespace-pre-wrap break-words">{{ selectedSymbol.snippet }}</pre>
                </div>
              }

              <div class="flex gap-2">
                <button hlmBtn variant="outline" (click)="openFile(selectedSymbol.file)">
                  <ng-icon name="lucideFileText" size="14" />
                  Open file
                </button>
              </div>

              <div class="border-t border-border pt-4 mt-4">
                <h3 class="text-sm font-semibold mb-2">Dependencies</h3>
                @if (depsLoading) {
                  <div class="flex items-center gap-2 text-muted-foreground py-2">
                    <wm-spinner size="sm" />
                    <span class="text-sm">Loading dependencies...</span>
                  </div>
                }
                @if (depsError) {
                  <div role="alert" hlmAlert variant="destructive" class="p-3 text-sm">
                    <div class="flex items-center gap-2">
                      <ng-icon name="lucideAlertCircle" size="16" />
                      <p hlmAlertDescription>{{ depsError }}</p>
                    </div>
                    <button hlmBtn variant="outline" size="sm" class="mt-2" (click)="loadDeps()">
                      <ng-icon name="lucideRefreshCw" size="14" />
                      Retry
                    </button>
                  </div>
                }
                @if (!depsLoading && !depsError) {
                  @if (dependencies.length === 0) {
                    <p class="text-sm text-muted-foreground">No dependencies found for this file.</p>
                  } @else {
                    <div class="space-y-3">
                      @for (depSet of dependencies; track depSet.file) {
                        <div>
                          <p class="text-xs font-mono text-muted-foreground mb-1 break-all">{{ depSet.file }}</p>
                          <div class="flex flex-wrap gap-1.5">
                            @for (dep of depSet.deps; track dep.target + dep.line) {
                              <span hlmBadge variant="outline" class="text-xs font-mono">{{ dep.target }}</span>
                            }
                          </div>
                        </div>
                      }
                    </div>
                  }
                }
              </div>
            </div>
          }

          @if (view === 'file') {
            <div class="space-y-4">
              <div class="flex items-center gap-2 text-sm text-muted-foreground">
                <ng-icon name="lucideFileText" size="14" />
                <span class="font-mono">{{ filePath }}</span>
              </div>

              @if (fileLoading) {
                <div class="flex items-center gap-2 text-muted-foreground py-8">
                  <wm-spinner size="sm" />
                  <span class="text-sm">Loading file...</span>
                </div>
              }

              @if (fileError) {
                <div role="alert" hlmAlert variant="destructive" class="p-3 text-sm">
                  <div class="flex items-center gap-2">
                    <ng-icon name="lucideAlertCircle" size="16" />
                    <p hlmAlertDescription>{{ fileError }}</p>
                  </div>
                  <button hlmBtn variant="outline" size="sm" class="mt-2" (click)="loadFile()">
                    <ng-icon name="lucideRefreshCw" size="14" />
                    Retry
                  </button>
                </div>
              }

              @if (!fileLoading && !fileError) {
                @if (fileContent) {
                  <div class="relative">
                    <div class="absolute top-0 right-0 px-2 py-1 text-[10px] text-muted-foreground font-mono uppercase tracking-wider">{{ fileLanguage }}</div>
                    <pre class="p-4 pt-6 bg-muted/30 rounded-lg border border-border text-sm overflow-auto max-h-[60vh] font-mono text-muted-foreground leading-relaxed whitespace-pre-wrap break-words">{{ fileContent }}</pre>
                  </div>
                } @else {
                  <p class="text-sm text-muted-foreground">File is empty or could not be loaded.</p>
                }
              }

              <div class="border-t border-border pt-4 mt-4">
                <h3 class="text-sm font-semibold mb-2">Symbols in this file</h3>
                @if (fileSymbolsLoading) {
                  <div class="flex items-center gap-2 text-muted-foreground py-2">
                    <wm-spinner size="sm" />
                    <span class="text-sm">Loading symbols...</span>
                  </div>
                }
                @if (fileSymbolsError) {
                  <div role="alert" hlmAlert variant="destructive" class="p-3 text-sm">
                    <p hlmAlertDescription>{{ fileSymbolsError }}</p>
                  </div>
                }
                @if (!fileSymbolsLoading && !fileSymbolsError) {
                  @if (fileSymbols.length === 0) {
                    <p class="text-sm text-muted-foreground">No symbols found in this file.</p>
                  } @else {
                    <div class="grid gap-2">
                      @for (sym of fileSymbols; track sym.name + sym.line) {
                        <button
                          hlmCard
                          type="button"
                          (click)="selectSymbol(sym)"
                          class="w-full text-left p-3 cursor-pointer hover:bg-accent/50 transition-colors"
                        >
                          <div class="flex items-center gap-2">
                            <span class="font-medium text-sm">{{ sym.name }}</span>
                            <span hlmBadge variant="secondary" [class]="symbolKindBadgeClass(sym.kind)" class="font-medium text-xs">{{ sym.kind }}</span>
                            <span class="ml-auto text-xs text-muted-foreground font-mono">L{{ sym.line }}</span>
                          </div>
                        </button>
                      }
                    </div>
                  }
                }
              </div>
            </div>
          }

        </div>
      </div>
    </div>
  `,
})
export class CodeViewComponent {
  /** Bound to template — Angular templates can't call imported functions directly */
  symbolKindBadgeClass = symbolKindBadgeClass;

  query = '';
  activeLanguage: string | null = null;
  results: CodeIntelSymbol[] = [];
  loading = false;
  error = '';
  debouncing = false;
  private searchTimeout: ReturnType<typeof setTimeout> | null = null;

  view: 'search' | 'symbol' | 'file' = 'search';
  selectedSymbol: CodeIntelSymbol | null = null;

  dependencies: CodeIntelDepSet[] = [];
  depsLoading = false;
  depsError = '';

  filePath = '';
  fileContent = '';
  fileLanguage = 'text';
  fileLoading = false;
  fileError = '';
  fileSymbols: CodeIntelSymbol[] = [];
  fileSymbolsLoading = false;
  fileSymbolsError = '';

  languageFilters: LanguageFilter[] = [
    { label: 'All', value: null },
    { label: 'Rust', value: 'rust' },
    { label: 'TypeScript', value: 'typescript' },
    { label: 'Python', value: 'python' },
    { label: 'Go', value: 'go' },
    { label: 'HTML', value: 'html' },
    { label: 'Svelte', value: 'svelte' },
  ];

  get totalResults(): number {
    return this.results.length;
  }

  get groupedResults(): Map<string, CodeIntelSymbol[]> {
    const map = new Map<string, CodeIntelSymbol[]>();
    for (const sym of this.results) {
      const lang = sym.language || 'Unknown';
      if (!map.has(lang)) {
        map.set(lang, []);
      }
      map.get(lang)!.push(sym);
    }
    return map;
  }

  get languageOrder(): string[] {
    return Array.from(this.groupedResults.keys()).sort();
  }

  constructor(@Inject(CODE_INTEL_PORT) private api: CodeIntelPort, private destroyRef: DestroyRef) {}

  onSearchInput() {
    if (this.searchTimeout) {
      clearTimeout(this.searchTimeout);
    }
    this.error = '';
    this.debouncing = true;
    this.searchTimeout = setTimeout(() => {
      this.debouncing = false;
      this.doSearch();
    }, 300);
  }

  setLanguage(lang: string | null) {
    this.activeLanguage = lang;
    if (this.query.trim()) {
      this.doSearch();
    }
  }

  doSearch() {
    if (!this.query.trim()) {
      this.results = [];
      return;
    }
    this.loading = true;
    this.error = '';
    this.api
      .searchSymbols({
        name: this.query,
        language: this.activeLanguage ?? undefined,
        max_results: 50,
      })
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: (res) => {
          this.results = res.symbols || [];
          this.loading = false;
        },
        error: () => {
          this.error = 'Symbol search failed. Check that the server is running.';
          this.loading = false;
        },
      });
  }

  selectSymbol(sym: CodeIntelSymbol) {
    this.selectedSymbol = sym;
    this.view = 'symbol';
    this.dependencies = [];
    this.depsError = '';
    this.loadDeps();
  }

  loadDeps() {
    if (!this.selectedSymbol) return;
    this.depsLoading = true;
    this.depsError = '';
    this.api
      .getDeps({ file: this.selectedSymbol.file })
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: (res) => {
          this.dependencies = res.dependencies || [];
          this.depsLoading = false;
        },
        error: () => {
          this.depsError = 'Failed to load dependencies.';
          this.depsLoading = false;
        },
      });
  }

  openFile(path: string) {
    this.filePath = path;
    this.view = 'file';
    this.fileContent = '';
    this.fileLanguage = 'text';
    this.fileError = '';
    this.fileSymbols = [];
    this.fileSymbolsError = '';
    this.loadFile();
    this.loadFileSymbols();
  }

  loadFile() {
    this.fileLoading = true;
    this.fileError = '';
    this.api
      .getFile({ path: this.filePath })
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: (res) => {
          this.fileContent = res.content || '';
          this.fileLanguage = res.language || 'text';
          this.fileLoading = false;
        },
        error: () => {
          this.fileError = 'Failed to load file content.';
          this.fileLoading = false;
        },
      });
  }

  loadFileSymbols() {
    this.fileSymbolsLoading = true;
    this.fileSymbolsError = '';
    this.api
      .searchSymbols({ file: this.filePath, max_results: 100 })
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe({
        next: (res) => {
          this.fileSymbols = res.symbols || [];
          this.fileSymbolsLoading = false;
        },
        error: () => {
          this.fileSymbolsError = 'Failed to load file symbols.';
          this.fileSymbolsLoading = false;
        },
      });
  }

  goBack() {
    if (this.view === 'symbol') {
      this.view = 'search';
      this.selectedSymbol = null;
    } else if (this.view === 'file') {
      if (this.selectedSymbol) {
        this.view = 'symbol';
      } else {
        this.view = 'search';
      }
    }
  }
}

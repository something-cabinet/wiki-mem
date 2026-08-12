import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { provideIcons } from '@ng-icons/core';
import { lucideSearch } from '@ng-icons/lucide';
import { of } from 'rxjs';
import { SearchViewComponent } from './search-view.component';
import { ENGINE_PORT } from '../../services/engine-port';
import { MockEngineService } from '../../services/mock-engine.service';

describe('SearchViewComponent', () => {
  let api: MockEngineService;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [SearchViewComponent],
      providers: [
        provideRouter([]),
        provideIcons({ lucideSearch }),
        { provide: ENGINE_PORT, useClass: MockEngineService },
      ],
    }).compileComponents();
    api = TestBed.inject(ENGINE_PORT) as MockEngineService;
  });

  it('renders search results returned by the engine port', () => {
    spyOn(api, 'searchQuery').and.returnValue(
      of({
        success: true,
        results: [
          {
            id: 'wiki:concepts:graph-architecture',
            score: 0.42,
            type: 'concept',
            page_type: 'concept',
            snippet: 'graph model internals',
          },
        ],
      }),
    );
    const fixture = TestBed.createComponent(SearchViewComponent);
    fixture.componentInstance.query.set('graph');
    fixture.componentInstance.doSearch();
    fixture.detectChanges();

    const list = fixture.nativeElement.querySelector('[aria-label="Search results"]');
    expect(list).toBeTruthy();
    expect(list.textContent).toContain('graph-architecture');
    expect(list.textContent).toContain('score 0.42');
    expect(list.textContent).toContain('graph model internals');
  });

  it('shows the empty state when a query returns no results', () => {
    spyOn(api, 'searchQuery').and.returnValue(of({ success: true, results: [] }));
    const fixture = TestBed.createComponent(SearchViewComponent);
    fixture.componentInstance.query.set('nothing');
    fixture.componentInstance.doSearch();
    fixture.detectChanges();

    expect(fixture.nativeElement.textContent).toContain('No results found');
  });

  it('surfaces a failed search as an error state', () => {
    spyOn(api, 'searchQuery').and.returnValue(of({ success: false, error: 'boom', results: [] }));
    const fixture = TestBed.createComponent(SearchViewComponent);
    fixture.componentInstance.query.set('x');
    fixture.componentInstance.doSearch();
    fixture.detectChanges();

    expect(fixture.nativeElement.textContent).toContain('boom');
  });
});

import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { Observable, of } from 'rxjs';
import { GraphViewComponent } from './graph-view.component';
import { ENGINE_PORT, GraphFullResponse } from '../../services/engine-port';
import { MockEngineService } from '../../services/mock-engine.service';

describe('GraphViewComponent', () => {
  let api: MockEngineService;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [GraphViewComponent],
      providers: [provideRouter([]), { provide: ENGINE_PORT, useClass: MockEngineService }],
    }).compileComponents();
    api = TestBed.inject(ENGINE_PORT) as MockEngineService;
  });

  it('renders the graph view', () => {
    const fixture = TestBed.createComponent(GraphViewComponent);
    fixture.detectChanges();
    expect(fixture.componentInstance).toBeTruthy();
  });

  it('loads graph edges that omit the provenance field without breaking', () => {
    (spyOn(api, 'getGraphFull') as jasmine.Spy<() => Observable<GraphFullResponse>>).and.returnValue(
      of({
        success: true,
        node_count: 2,
        edge_count: 1,
        nodes: [
          { id: 'a', title: 'A', page_type: 'concept', degree: 1 },
          { id: 'b', title: 'B', page_type: 'spec', degree: 1 },
        ],
        edges: [{ source: 'a', target: 'b', edge_type: 'relates_to' }],
      }),
    );

    const fixture = TestBed.createComponent(GraphViewComponent);
    fixture.detectChanges();

    expect(fixture.componentInstance.graphEdges().length).toBe(1);
    expect(fixture.componentInstance.graphEdges()[0].provenance).toBeUndefined();
  });

  it('shows provenance entries in the legend', () => {
    const fixture = TestBed.createComponent(GraphViewComponent);
    fixture.componentInstance.showLegend.set(true);
    fixture.detectChanges();

    const legend = fixture.nativeElement.querySelector('.bg-popover\\/95');
    expect(legend).toBeTruthy();
    expect(legend.textContent).toContain('explicit');
    expect(legend.textContent).toContain('derived');
    expect(legend.textContent).toContain('ambiguous');
  });
});

import { Injectable } from '@angular/core';
import { Observable, of } from 'rxjs';
import { CodeIntelPort, CodeIntelSymbol, CodeIntelDepSet } from './code-intel-port';

@Injectable()
export class MockCodeIntelService implements CodeIntelPort {
  searchSymbols(params: {
    name?: string;
    kind?: string;
    language?: string;
    file?: string;
    max_results?: number;
  }): Observable<{ symbols: CodeIntelSymbol[] }> {
    return of({ symbols: [] as CodeIntelSymbol[] });
  }

  getDeps(params: { path?: string; reverse?: boolean }): Observable<{ dependencies: CodeIntelDepSet[] }> {
    return of({ dependencies: [] as CodeIntelDepSet[] });
  }

  getFile(params: { path: string }): Observable<{ content: string; language: string }> {
    return of({ content: '', language: 'text' });
  }
}

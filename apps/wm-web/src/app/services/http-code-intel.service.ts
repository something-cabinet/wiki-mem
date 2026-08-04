import { Injectable } from '@angular/core';
import { Observable, from } from 'rxjs';
import { CodeIntelPort, CodeIntelSymbol, CodeIntelDepSet } from './code-intel-port';

@Injectable({ providedIn: 'root' })
export class HttpCodeIntelService implements CodeIntelPort {
  private base = '/api';
  private token = (document.querySelector('meta[name="wm-token"]') as HTMLMetaElement | null)?.content ?? '';

  private async httpCall<T>(action: string, body?: unknown): Promise<T> {
    const res = await fetch(`${this.base}/${action}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'x-wm-token': this.token },
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!res.ok) throw new Error(await res.text());
    const json = await res.json() as { success: boolean; data?: T; error?: string };
    if (!json.success) throw new Error(json.error || 'Request failed');
    return json.data as T;
  }

  private observe<T>(p: Promise<T>): Observable<T> {
    return from(p);
  }

  searchSymbols(params: {
    name?: string;
    kind?: string;
    language?: string;
    file?: string;
    max_results?: number;
  }): Observable<{ symbols: CodeIntelSymbol[] }> {
    return this.observe(this.httpCall<{ symbols: CodeIntelSymbol[] }>('tools/wm_code.symbols', params));
  }

  getDeps(params: { file?: string; reverse?: boolean }): Observable<{ dependencies: CodeIntelDepSet[] }> {
    return this.observe(this.httpCall<{ dependencies: CodeIntelDepSet[] }>('tools/wm_code.deps', params));
  }

  getFile(params: { path: string }): Observable<{ content: string; language: string }> {
    return this.observe(this.httpCall<{ content: string; language: string }>('tools/wm_code.file', params));
  }
}

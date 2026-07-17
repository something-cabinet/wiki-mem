import type { ParsedStub } from './types';
import { matchStub } from './matcher';

export class MockRegistry {
  private defaults: ParsedStub[] = [];
  private scenarios: ParsedStub[] = [];
  private dynamic: ParsedStub[] = [];

  setDefaults(stubs: ParsedStub[]): void { this.defaults = stubs; }
  activateScenario(stubs: ParsedStub[]): void { this.scenarios = stubs; }
  addDynamic(stub: ParsedStub): void { this.dynamic.push(stub); }
  clearScenarios(): void { this.scenarios = []; }
  reset(): void { this.scenarios = []; this.dynamic = []; }

  private get all(): ParsedStub[] {
    return [...this.dynamic, ...this.scenarios, ...this.defaults];
  }

  find(method: string, urlPath: string, query?: Record<string, string>): ParsedStub | undefined {
    const params = query
      ? Object.entries(query).reduce((p, [k, v]) => { p.set(k, v); return p; }, new URLSearchParams())
      : new URLSearchParams();
    return matchStub(method, urlPath, params, this.all);
  }
}

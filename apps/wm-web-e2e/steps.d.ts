/// <reference types='codeceptjs' />

type navigation = typeof import('./pages/navigation');
type search = typeof import('./pages/search');
type graph = typeof import('./pages/graph');
type tasks = typeof import('./pages/tasks');
type pages = typeof import('./pages/pages');
type memory = typeof import('./pages/memory');
type settings = typeof import('./pages/settings');

declare namespace CodeceptJS {
  interface SupportObject {
    I: I;
    current: any;
    navigation: navigation;
    search: search;
    graph: graph;
    tasks: tasks;
    pages: pages;
    memory: memory;
    settings: settings;
  }
  interface Methods extends Playwright {
    resetScenarios(): Promise<void>;
    setupSessionFor(name: string): Promise<void>;
    resetMocks(): Promise<void>;
    activateScenario(name: string): Promise<void>;
    waitForNetworkIdle(timeout?: number): Promise<void>;
    clearBrowserState(): Promise<void>;
    mockRoute(urlPattern: string, responseBody: object, status?: number): Promise<void>;
    unmockAllRoutes(): Promise<void>;
  }
  interface I extends WithTranslation<Methods> {}
  namespace Translation {
    interface Actions {}
  }
}

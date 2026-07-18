import type { Options } from '@wdio/types';
import { loadMockMappings } from './e2e/mock-helper';

export const config: Options.Testrunner = {
  runner: 'local',
  specs: ['./e2e/**/*.test.ts'],
  exclude: [],
  maxInstances: 1,
  /**
   * Before all workers start, load mock IPC mappings from the mock server mapping files.
   * Each test file can also call loadMockMappings() individually in its before() hook.
   */
  before: async () => {
    try {
      await loadMockMappings();
    } catch {
      // Mappings directory might not exist in CI — tests will register their own mocks
    }
  },
  capabilities: [
    {
      browserName: 'chrome',
      'goog:chromeOptions': {
        args: ['--headless', '--no-sandbox'],
      },
    },
  ],
  logLevel: 'info',
  bail: 0,
  baseUrl: 'http://localhost:4200',
  waitforTimeout: 10000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },
  // Tauri browser mode — runs frontend in Chrome, mocks IPC
  services: [
    [
      'tauri',
      {
        appBinaryPath: undefined, // browser mode — no binary needed
        driverProvider: 'embedded',
      },
    ],
  ],
};

export const config: CodeceptJS.MainConfig = {
  tests: "./user-journeys/**/*.journey.ts",
  output: "./output",
  helpers: {
    Playwright: {
      url: "http://localhost:4200",
      browser: "chromium",
      show: !process.env.HEADLESS,
      windowSize: "1280x720",
    },
    MockManager: {
      require: "./helpers/mock-manager_helper.js",
      url: "http://localhost:8081",
    },
  },
  include: {
    navigation: "./pages/navigation.page.ts",
    search: "./pages/search.page.ts",
    graph: "./pages/graph.page.ts",
    tasks: "./pages/tasks.page.ts",
    pages: "./pages/pages.page.ts",
    memory: "./pages/memory.page.ts",
    settings: "./pages/settings.page.ts",
  },
  plugins: {
    retryFailedStep: { enabled: true },
    screenshotOnFail: { enabled: true },
  },
  name: "wm-web-e2e",
};

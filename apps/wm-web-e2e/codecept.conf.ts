export const config: CodeceptJS.MainConfig = {
  tests: "./*/*.journey.ts",
  output: "./output",
  helpers: {
    Playwright: {
      url: "http://localhost:4200",
      browser: "chromium",
      show: !process.env.HEADLESS,
      windowSize: "1280x720",
    },
    MockManager: {
      require: "./helpers/mock-manager_helper.ts",
      url: "http://localhost:8081",
    },
  },
  include: {
    navigation: "./pages/navigation.ts",
    search: "./pages/search.ts",
    graph: "./pages/graph.ts",
    tasks: "./pages/tasks.ts",
    pages: "./pages/pages.ts",
    memory: "./pages/memory.ts",
    settings: "./pages/settings.ts",
  },
  plugins: {
    retryFailedStep: { enabled: true },
    screenshotOnFail: { enabled: true },
  },
  name: "wm-web-e2e",
};

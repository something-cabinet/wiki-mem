Feature("Search");

Before(async ({ I }) => {
  I.amOnPage("/search");
});

Scenario("Search input and all type filters render with correct labels", async ({ I }) => {
  I.seeElement('[aria-label="Search query"]');
  I.seeElement("kbd");
  I.see("TYPE");
  I.seeElement("#brn-tabs-label-all");
  I.seeElement("#brn-tabs-label-page");
  I.seeElement("#brn-tabs-label-task");
  I.seeElement("#brn-tabs-label-memory");
  I.see("All");
  I.see("Pages");
  I.see("Tasks");
  I.see("Memory");
});

Scenario("Search with query returns results with score and breakdown on hover", async ({ I }) => {
  I.fillField('[aria-label="Search query"]', "test");
  I.pressKey("Enter");
  I.waitForText("score", 5);
  I.see("results");
  I.seeElement('[aria-label="Search results"]');
  I.seeElement('[role="listitem"]');
  I.seeElement(".cursor-help");
  I.see("score");
  I.see("page");
  I.moveCursorTo(".cursor-help");
  I.waitForText("BM25", 2);
  I.see("RRF");
  I.see("RRF");
  I.see("Semantic");
  I.see("Title");
  I.see("Exact title");
  I.see("Tags");
  I.see("Exact ID");
  I.see("Recency");
  I.see("Final");
  I.seeElement('[role="table"]');
});

Scenario("Search type filter cycles through all options and back", async ({ I }) => {
  I.fillField('[aria-label="Search query"]', "test");
  I.pressKey("Enter");
  I.waitForText("results", 3);
  I.click("#brn-tabs-label-page");
  I.wait(1);
  I.see("results");
  I.click("#brn-tabs-label-task");
  I.wait(1);
  I.see("results");
  I.click("#brn-tabs-label-memory");
  I.wait(1);
  I.see("results");
  I.click("#brn-tabs-label-all");
  I.wait(1);
  I.see("results");
});

Scenario("Empty search shows placeholder with keyboard hint", async ({ I }) => {
  I.dontSeeElement('[role="list"]');
  I.see("Type a query above");
  I.see("Enter");
  I.seeElement("ng-icon");
});

Scenario("Clicking a result navigates to page detail", async ({ I }) => {
  I.fillField('[aria-label="Search query"]', "test");
  I.pressKey("Enter");
  I.waitForText("score", 3);
  I.click(locate('[role="listitem"]').first());
  I.wait(2);
  I.seeInCurrentUrl("/pages/");
  I.seeElement("h1");
});

Scenario("Sequential searches work correctly", async ({ I }) => {
  I.fillField('[aria-label="Search query"]', "test");
  I.pressKey("Enter");
  I.waitForText("results", 3);
  I.seeElement('[role="listitem"]');
  I.fillField('[aria-label="Search query"]', "graph");
  I.pressKey("Enter");
  I.waitForText("results", 3);
  I.seeElement('[role="listitem"]');
  I.fillField('[aria-label="Search query"]', "wiki");
  I.pressKey("Enter");
  I.waitForText("results", 3);
  I.seeElement('[role="listitem"]');
});

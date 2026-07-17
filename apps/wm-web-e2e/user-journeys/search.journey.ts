Feature("Search");

Before(async ({ I }) => {
  await I.resetScenarios();
  I.amOnPage("/");
  I.waitForElement("h1", 10);
});

Scenario("User can search and see results", async ({ I, search }) => {
  search.seeHeading();
  search.searchFor("mcp");
  search.seeResults();
});

Scenario("User can filter search by type", async ({ I, search }) => {
  search.searchFor("mcp");
  search.seeResults();
  search.filterByType("Pages");
  I.wait(1);
  I.seeInCurrentUrl("/search");
});

Scenario("User sees empty state for no results", async ({ I, search }) => {
  await I.setupSessionFor("vpp");
  search.searchFor("zzzznonexistent");
  search.seeNoResults();
});

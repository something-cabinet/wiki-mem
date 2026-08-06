Feature("Pages");

Before(async ({ I }) => {
  I.amOnPage("/pages");
});

Scenario("Pages page shows heading and page list", async ({ I }) => {
  I.see("Pages");
  I.waitForElement(".grid.gap-2", 3);
});

Scenario("Pages page lists wiki entries with type badges", async ({ I }) => {
  I.waitForElement(".grid.gap-2", 3);
  I.see("Set up CI pipeline");
  I.see("Graph Architecture");
  I.see("tasks");
  I.see("specs");
  I.see("concepts");
  I.see("task");
  I.see("spec");
  I.see("concept");
  I.see("memory");
});

Scenario("Clicking a page opens its detail view", async ({ I }) => {
  I.waitForElement(".grid.gap-2", 3);
  I.click("Graph Architecture");
  I.waitForElement("h1", 3);
  I.seeInCurrentUrl("/pages/");
  I.see("Graph Architecture");
  I.see("concept");
  I.see("active");
});

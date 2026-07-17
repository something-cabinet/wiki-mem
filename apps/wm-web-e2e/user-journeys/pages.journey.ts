Feature("Pages");

Before(async ({ I }) => {
  await I.resetScenarios();
  I.amOnPage("/pages");
  I.waitForElement("h1", 10);
});

Scenario("User can see pages list", async ({ I, pages }) => {
  pages.seeList();
  I.see("Knowns Reference");
  I.see("Graph Architecture");
});

Scenario("User can open create page modal", async ({ I, pages }) => {
  pages.clickCreate();
  I.seeElement('[role="dialog"]');
  pages.cancelCreate();
});

Scenario("User can create a new page", async ({ I, pages }) => {
  pages.clickCreate();
  pages.fillCreateForm({ path: "concepts/test-page", title: "Test Page" });
  pages.submitCreate();
  I.wait(1);
});

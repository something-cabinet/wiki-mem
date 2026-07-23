Feature("Pages");

Before(async ({ I }) => {
  I.amOnPage("/pages");
});

Scenario("Pages page shows heading and create button", async ({ I }) => {
  I.see("Pages");
  I.see("Create Page");
});

Scenario("Pages page lists wiki entries with badges", async ({ I }) => {
  I.waitForText("Create Page", 3);
  I.see("tasks");
  I.see("specs");
  I.see("concepts");
  I.seeElement(".grid.gap-2");
});

Scenario("Create Page dialog opens has form fields and closes on cancel", async ({ I }) => {
  I.click("Create Page");
  I.wait(1);
  I.seeElement('input[placeholder="e.g. projects/my-page"]');
  I.seeElement('input[placeholder="Page title"]');
  I.click("Cancel");
  I.wait(1);
  I.dontSee('input[placeholder="e.g. projects/my-page"]');
});

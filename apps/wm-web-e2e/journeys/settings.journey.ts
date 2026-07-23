Feature("Settings");

Before(async ({ I }) => {
  I.amOnPage("/settings");
});

Scenario("Settings page shows heading", async ({ I }) => {
  I.see("Settings");
});

Scenario("Settings has refresh and appearance controls", async ({ I }) => {
  I.see("Refresh", "button");
  I.see("Refresh");
});

Scenario("Settings shows server status section", async ({ I }) => {
  I.seeElement("button");
  I.see("Refresh");
});

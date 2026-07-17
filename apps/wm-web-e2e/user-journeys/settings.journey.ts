Feature("Settings");

Before(async ({ I }) => {
  await I.resetScenarios();
  I.amOnPage("/settings");
  I.waitForElement("h1", 10);
});

Scenario("User can see engine status", async ({ I, settings }) => {
  settings.seeHeading();
  settings.seeEngineStatus();
  I.see("177");
  I.see("13");
});

Scenario("User can refresh engine status", async ({ I, settings }) => {
  settings.clickRefresh();
  I.wait(1);
  settings.seeEngineStatus();
});

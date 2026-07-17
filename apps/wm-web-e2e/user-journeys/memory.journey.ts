Feature("Memory");

Before(async ({ I }) => {
  await I.resetScenarios();
  I.amOnPage("/memory");
  I.waitForElement("h1", 10);
});

Scenario("User can see memory entries", async ({ I, memory }) => {
  memory.seeEntries();
  I.see("Sync Writes Pattern");
});

Scenario("User can open new memory modal", async ({ I, memory }) => {
  memory.clickNew();
  I.seeElement('[role="dialog"]');
});

Scenario("User can create a memory entry", async ({ I, memory }) => {
  memory.clickNew();
  memory.fillNewForm({
    title: "Test Memory",
    content: "This is a test memory entry for E2E testing.",
    tags: "test, e2e",
  });
  memory.submitNew();
  I.wait(1);
});

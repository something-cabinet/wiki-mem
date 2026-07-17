Feature("Tasks");

Before(async ({ I }) => {
  await I.resetScenarios();
  I.amOnPage("/tasks");
  I.waitForElement("h1", 10);
});

Scenario("User sees task board columns", async ({ I, tasks }) => {
  tasks.seeHeading();
  tasks.seeColumn("Todo");
  tasks.seeColumn("Done");
  tasks.seeTaskCount("Todo", 4);
});

Scenario("User sees their 3 new tasks in Todo", async ({ I }) => {
  I.see("Add ServerHandler impl");
  I.see("Refactor wm-cli");
  I.see("Refactor wm-server");
});

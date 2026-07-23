Feature("Tasks");

Before(async ({ I }) => {
  I.amOnPage("/tasks");
});

Scenario("Tasks page shows task board", async ({ I }) => {
  I.see("Task Board");
  I.seeElement('[id^="brn-accordion-trigger-"]');
});

Scenario("Tasks page shows status columns with counts", async ({ I }) => {
  I.waitForText("todo", 3);
  I.waitForText("done", 3);
});

Scenario("Tasks accordion expands and collapses", async ({ I }) => {
  I.waitForElement('[id^="brn-accordion-trigger-"]', 3);
  I.click('[id^="brn-accordion-trigger-"]');
  I.wait(1);
  I.click('[id^="brn-accordion-trigger-"]');
});

Feature("Memory");

Before(async ({ I }) => {
  I.amOnPage("/memory");
});

Scenario("Memory page loads with heading", async ({ I }) => {
  I.waitForText("Memory", 3);
});

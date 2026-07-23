Feature("Graph");

Before(async ({ I }) => {
  I.amOnPage("/graph");
});

Scenario("Graph page shows nodes and edges counts", async ({ I }) => {
  I.waitForElement("canvas", 5);
  I.see("nodes");
  I.see("edges");
});

Scenario("Graph has zoom controls and spacing slider", async ({ I }) => {
  I.waitForElement("canvas", 3);
  I.seeElement('[aria-label="Zoom in"]');
  I.seeElement('[aria-label="Zoom out"]');
  I.seeElement('[aria-label="Fit to view"]');
  I.seeElement('[aria-label="Graph node spacing"]');
});

Scenario("Graph legend shows all page types", async ({ I }) => {
  I.waitForElement("canvas", 3);
  I.click("Legend");
  I.see("Concept");
  I.see("Spec");
  I.see("Task");
  I.see("Memory");
  I.see("Pattern");
  I.see("Decision");
  I.see("How-to");
  I.see("Reference");
  I.click("Hide");
});

Scenario("Graph legend can be toggled on and off", async ({ I }) => {
  I.waitForElement("canvas", 3);
  I.click("Legend");
  I.see("Concept");
  I.click("Hide");
  I.dontSee("Concept");
});

Scenario("Graph zoom buttons are functional", async ({ I }) => {
  I.waitForElement("canvas", 3);
  I.click('[aria-label="Zoom in"]');
  I.click('[aria-label="Zoom out"]');
  I.click('[aria-label="Fit to view"]');
});

Scenario("Graph canvas is interactive", async ({ I }) => {
  I.waitForElement("canvas", 3);
  I.seeElement("canvas");
});

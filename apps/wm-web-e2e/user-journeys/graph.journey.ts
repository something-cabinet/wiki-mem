Feature("Graph");

Before(async ({ I }) => {
  await I.resetScenarios();
  I.amOnPage("/graph");
  I.waitForElement("h1", 10);
});

Scenario("User sees graph stats", async ({ I, graph }) => {
  graph.seeHeading();
  graph.seeStats();
});

Scenario("User can explore a node", async ({ I, graph }) => {
  graph.exploreNode("wiki:concepts:graph-architecture");
  I.wait(1);
  graph.seeNeighbors();
});

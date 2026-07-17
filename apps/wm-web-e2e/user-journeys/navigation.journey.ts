Feature("Navigation");

Before(async ({ I }) => {
  await I.resetScenarios();
  I.amOnPage("/");
  I.waitForElement("h1", 10);
});

Scenario("User can navigate between pages using sidebar", async ({ I, navigation }) => {
  navigation.goToSearch();
  I.seeInCurrentUrl("/search");

  navigation.goToGraph();
  I.seeInCurrentUrl("/graph");

  navigation.goToTasks();
  I.seeInCurrentUrl("/tasks");

  navigation.goToPages();
  I.seeInCurrentUrl("/pages");

  navigation.goToMemory();
  I.seeInCurrentUrl("/memory");

  navigation.goToSettings();
  I.seeInCurrentUrl("/settings");
});

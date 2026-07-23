Feature("Navigation");

Before(async ({ I }) => {
  I.amOnPage("/");
});

Scenario("App shell loads with all branding elements", async ({ I }) => {
  I.seeElement("header");
  I.seeElement("main");
  I.see("WM Engine");
  I.see("Wiki Memory Engine");
});

Scenario("Sidebar lists all navigation sections", async ({ I }) => {
  I.see("Search");
  I.see("Graph");
  I.see("Tasks");
  I.see("Pages");
  I.see("Memory");
  I.see("Settings");
});

Scenario("Dark mode toggle is present and togglable", async ({ I }) => {
  I.seeElement('[role="switch"]');
  I.click('[role="switch"]');
  I.wait(1);
  I.click('[role="switch"]');
});

Scenario("Each sidebar link navigates to the correct page", async ({ I, navigation }) => {
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

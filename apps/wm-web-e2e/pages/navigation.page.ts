const { I } = inject();

module.exports = {
  goToSearch() {
    I.click('role=link[name="Search"]');
    I.waitInUrl("/search", 5);
  },
  goToGraph() {
    I.click('role=link[name="Graph"]');
    I.waitInUrl("/graph", 5);
  },
  goToTasks() {
    I.click('role=link[name="Tasks"]');
    I.waitInUrl("/tasks", 5);
  },
  goToPages() {
    I.click('role=link[name="Pages"]');
    I.waitInUrl("/pages", 5);
  },
  goToMemory() {
    I.click('role=link[name="Memory"]');
    I.waitInUrl("/memory", 5);
  },
  goToSettings() {
    I.click('role=link[name="Settings"]');
    I.waitInUrl("/settings", 5);
  },
};

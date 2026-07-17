const { I } = inject();

module.exports = {
  seeHeading() {
    I.see("Search", "h1");
  },
  searchFor(query) {
    I.fillField("role=textbox[name='Search query']", query);
    I.click("role=button[name='Search']");
    I.waitForElement('[role="list"]', 10);
  },
  seeResults() {
    I.seeElement('[role="list"]');
  },
  seeNoResults() {
    I.see("No results found");
  },
  filterByType(type) {
    I.click(`role=button[name="${type}"]`);
  },
};

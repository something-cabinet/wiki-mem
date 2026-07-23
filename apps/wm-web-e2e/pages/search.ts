const { I } = inject();

const search = {
  /**
   * Execute a search query. Brief wait after to let results load.
   */
  search(query: string) {
    I.fillField('[aria-label="Search query"]', query);
    I.pressKey("Enter");
    I.wait(1);
  },

  /**
   * Assert a result text appears.
   */
  seeResult(text: string) {
    I.see(text);
  },

  /**
   * Assert no results state.
   */
  seeNoResults() {
    I.see('No results');
  },

  /**
   * Filter search results by type.
   */
  filterBy(type: string) {
    I.click(type, 'button');
    I.wait(1);
  },

  /**
   * Open a search result by its index in the list.
   * Each result is an <a> with role="listitem" directly.
   */
  openResult(index = 0) {
    I.click(locate('[role="listitem"]').at(index + 1));
    I.wait(2);
  },
};

export = search;

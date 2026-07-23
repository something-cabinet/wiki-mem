const { I } = inject();

const memory = {
  /**
   * Navigate to the memory page.
   */
  open() {
    I.amOnPage('/memory');
    I.see('Memory', 'h1');
  },

  /**
   * Create a new memory entry.
   */
  createEntry(title: string, content: string) {
    I.click('New');
    I.waitForText('New Memory Entry', 3, 'h3');
    I.fillField('[placeholder="Entry title"]', title);
    I.fillField('[placeholder="What do you want to remember?"]', content);
    I.click('Save', 'button');
    I.wait(1);
  },

  /**
   * Assert a memory entry is visible.
   */
  seeEntry(text: string) {
    I.see(text);
  },

  /**
   * Assert no memory entries.
   */
  seeNoEntries() {
    I.see('No memory entries');
  },

  /**
   * Edit the first memory entry.
   */
  editEntry(newTitle: string, newContent: string) {
    I.click('[aria-label="Edit entry"]');
    I.waitForText('Edit Memory Entry', 3, 'h3');
    I.fillField('[placeholder="Entry title"]', newTitle);
    I.fillField('[placeholder="What do you want to remember?"]', newContent);
    I.click('Save', 'button');
    I.wait(1);
  },

  /**
   * Delete the first memory entry.
   */
  deleteEntry() {
    I.click('[aria-label="Delete entry"]');
    I.waitForText('Delete Memory Entry', 3, 'h3');
    I.click('Delete', 'button');
    I.wait(1);
  },
};

export = memory;

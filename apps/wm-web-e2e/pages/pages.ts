const { I } = inject();

const pages = {
  /**
   * Create a new wiki page.
   */
  createPage(title: string, content = '') {
    I.click('Create Page');
    I.waitForText('Create Page', 3, 'h3');
    I.fillField('[placeholder="Page title"]', title);
    if (content) {
      I.fillField('[placeholder="Page body content"]', content);
    }
    I.click('Create', 'button');
    I.waitForInvisible('h3:has-text("Create Page")', 3);
  },

  /**
   * Open an existing page by its name in the list.
   */
  openPage(title: string) {
    I.click(title, 'button');
    I.wait(1);
  },

  /**
   * Assert page content is visible.
   */
  seeContent(text: string) {
    I.see(text);
  },

  /**
   * Edit the current page content.
   */
  editContent(newContent: string) {
    I.click('Edit', 'button');
    I.waitForText('Edit Page', 3, 'h3');
    I.fillField('[placeholder="Page title"]', newContent);
    I.fillField('[placeholder="Page body content"]', newContent);
    I.click('Save', 'button');
    I.wait(1);
  },

  /**
   * Delete the current page.
   */
  deletePage() {
    I.click('Delete', 'button');
    I.waitForText('Delete Page', 3, 'h3');
    I.click('Delete', 'button');
    I.wait(1);
  },
};

export = pages;

const { I } = inject();

const settings = {
  /**
   * Navigate to the settings page.
   */
  open() {
    I.amOnPage('/settings');
    I.see('Settings', 'h1');
  },

  /**
   * Toggle dark mode.
   */
  toggleDarkMode() {
    I.click('[aria-label="Toggle dark mode"]');
    I.wait(1);
  },

  /**
   * Assert engine status section is visible.
   */
  seeEngineStatus() {
    I.see('Graph Nodes');
  },

  /**
   * Assert appearance section is visible.
   */
  seeAppearance() {
    I.see('Appearance');
  },
};

export = settings;

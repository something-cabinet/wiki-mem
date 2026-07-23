const { I } = inject();

const tasks = {
  /**
   * Assert the task board is visible.
   */
  seeBoard() {
    I.see('Task Board', 'h1');
  },

  /**
   * See a task with given title in the task board.
   */
  seeTask(title: string) {
    I.see(title);
  },
};

export = tasks;

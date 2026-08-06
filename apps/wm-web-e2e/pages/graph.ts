const { I } = inject();

const graph = {
  /**
   * Assert the graph canvas is visible.
   */
  seeGraph() {
    I.seeElement('canvas');
  },

  /**
   * Zoom in using the graph controls.
   */
  zoomIn() {
    I.click('[aria-label="Zoom in"]');
    I.wait(1);
  },

  /**
   * Zoom out using the graph controls.
   */
  zoomOut() {
    I.click('[aria-label="Zoom out"]');
    I.wait(1);
  },

  /**
   * Fit the graph to view.
   */
  resetView() {
    I.click('[aria-label="Fit to view"]');
    I.wait(1);
  },
};

export default graph;
